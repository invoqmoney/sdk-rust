use crate::errors::{ApiErrorPayload, InvoqApiError, InvoqError, Result};
use crate::types::{ApiErrorField, ApiErrorLocation};
use reqwest::{Client, Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

const USER_AGENT: &str = concat!("invoq-rust/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub(crate) struct RequestClientOptions {
    pub api_key: String,
    pub api_origin: Url,
    pub http_client: Client,
    pub timeout: Duration,
}

pub(crate) async fn request_json<T, B>(
    client_options: &RequestClientOptions,
    method: Method,
    path_segments: &[&str],
    body: Option<&B>,
) -> Result<T>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let mut url = client_options.api_origin.clone();
    url.path_segments_mut()
        .expect("normalized api base URL is hierarchical")
        .extend(path_segments);

    let mut request = client_options
        .http_client
        .request(method, url)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .bearer_auth(&client_options.api_key)
        .timeout(client_options.timeout);

    if let Some(body) = body {
        request = request
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(body);
    }

    let response = request.send().await.map_err(map_connect_error)?;
    let status = response.status();
    let response_text = response.text().await.map_err(map_read_response_error)?;

    let payload = match serde_json::from_str::<Value>(&response_text) {
        Ok(payload) => payload,
        Err(error) => {
            if !status.is_success() {
                return Err(api_error_from_response(
                    status.as_u16(),
                    ApiErrorPayload::Text(response_text),
                )
                .into());
            }

            return Err(InvoqError::ParseResponse(error));
        }
    };

    if !status.is_success() {
        return Err(
            api_error_from_response(status.as_u16(), ApiErrorPayload::Json(payload)).into(),
        );
    }

    let data = payload
        .get("data")
        .ok_or_else(|| InvoqError::MissingDataEnvelope {
            payload: payload.clone(),
        })?;

    serde_json::from_value(data.clone()).map_err(InvoqError::ParseResponse)
}

fn api_error_from_response(status: u16, payload: ApiErrorPayload) -> InvoqApiError {
    let error = match &payload {
        ApiErrorPayload::Json(Value::Object(object)) => Some(object),
        _ => None,
    };

    let code = error
        .and_then(|object| object.get("code"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let message = error
        .and_then(|object| object.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("invoq API request failed.")
        .to_string();
    let fields = error
        .and_then(|object| object.get("fields"))
        .and_then(parse_fields);
    let meta = error
        .and_then(|object| object.get("meta"))
        .and_then(|value| value.as_object().map(|_| value.clone()));

    InvoqApiError {
        message,
        status,
        code,
        fields,
        meta,
        payload,
    }
}

fn parse_fields(value: &Value) -> Option<Vec<ApiErrorField>> {
    let fields = value.as_array()?;
    Some(fields.iter().filter_map(parse_field).collect())
}

fn parse_field(value: &Value) -> Option<ApiErrorField> {
    let object = value.as_object()?;
    let location = match object.get("location")?.as_str()? {
        "query" => ApiErrorLocation::Query,
        "path" => ApiErrorLocation::Path,
        "body" => ApiErrorLocation::Body,
        "header" => ApiErrorLocation::Header,
        _ => return None,
    };

    Some(ApiErrorField {
        field: object.get("field")?.as_str()?.to_string(),
        location,
        code: object.get("code")?.as_str()?.to_string(),
        message: object.get("message")?.as_str()?.to_string(),
    })
}

fn map_connect_error(error: reqwest::Error) -> InvoqError {
    if error.is_timeout() {
        InvoqError::Timeout(error)
    } else {
        InvoqError::Connect(error)
    }
}

fn map_read_response_error(error: reqwest::Error) -> InvoqError {
    if error.is_timeout() {
        InvoqError::Timeout(error)
    } else {
        InvoqError::ReadResponse(error)
    }
}

#[cfg(test)]
mod tests {
    use super::api_error_from_response;
    use crate::errors::ApiErrorPayload;
    use serde_json::json;

    #[test]
    fn parses_api_error_fields_as_a_complete_contract() {
        let error = api_error_from_response(
            400,
            ApiErrorPayload::Json(json!({
                "message": "Invalid request.",
                "fields": [
                    {
                        "location": "body",
                        "field": "amount",
                        "code": "required",
                        "message": "Required."
                    }
                ]
            })),
        );

        let fields = error.fields.unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field, "amount");
    }

    #[test]
    fn ignores_invalid_api_error_fields() {
        let payload = json!({
            "message": "Invalid request.",
            "fields": [
                {
                    "location": "body",
                    "field": "amount",
                    "code": "required",
                    "message": "Required."
                },
                {
                    "location": "unexpected",
                    "field": "currency",
                    "code": "invalid",
                    "message": "Invalid."
                }
            ]
        });
        let error = api_error_from_response(400, ApiErrorPayload::Json(payload.clone()));

        let fields = error.fields.unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].field, "amount");
        assert_eq!(error.payload, ApiErrorPayload::Json(payload));
    }

    #[test]
    fn preserves_empty_api_error_fields_array() {
        let error = api_error_from_response(
            400,
            ApiErrorPayload::Json(json!({
                "message": "Invalid request.",
                "fields": []
            })),
        );

        assert_eq!(error.fields.unwrap(), Vec::new());
    }
}
