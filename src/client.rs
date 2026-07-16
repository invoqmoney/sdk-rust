use crate::errors::{InvoqError, Result};
use crate::request::{request_json, RequestClientOptions};
use crate::types::{
    CreateInvoiceInput, CreateTestPaymentInput, Invoice, PublicInvoice, TestPaymentInvoice,
};
use reqwest::{Client, Method, Url};
use std::time::Duration;

const DEFAULT_API_ORIGIN: &str = "https://api.invoq.money";
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
const MAX_TIMEOUT_MS: u64 = u32::MAX as u64;

/// Client for invoq server APIs.
#[derive(Clone)]
pub struct Invoq {
    pub invoices: Invoices,
}

impl Invoq {
    /// Create a client using the production invoq API.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_options(api_key, InvoqOptions::default())
    }

    /// Create a client with custom options.
    pub fn with_options(api_key: impl Into<String>, options: InvoqOptions) -> Result<Self> {
        let api_key = api_key.into();

        if api_key.trim().is_empty() {
            return Err(InvoqError::configuration(
                "invoq API key must be a non-empty string.",
            ));
        }

        let api_origin = normalize_api_origin(&options.api_origin)?;
        let timeout = normalize_timeout_ms(options.timeout_ms)?;
        let http_client = options.http_client.unwrap_or_else(|| {
            Client::builder()
                .build()
                .expect("default reqwest client configuration is valid")
        });

        let client_options = RequestClientOptions {
            api_key,
            api_origin,
            http_client,
            timeout,
        };

        Ok(Self {
            invoices: Invoices { client_options },
        })
    }
}

impl std::fmt::Debug for Invoq {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Invoq")
            .field(
                "api_origin",
                &self.invoices.client_options.api_origin.as_str(),
            )
            .finish()
    }
}

/// Configuration for an [`Invoq`] client.
#[derive(Clone)]
pub struct InvoqOptions {
    api_origin: String,
    timeout_ms: u64,
    http_client: Option<Client>,
}

impl InvoqOptions {
    /// Use a custom API origin.
    pub fn api_origin(mut self, api_origin: impl Into<String>) -> Self {
        self.api_origin = api_origin.into();
        self
    }

    /// Use a custom request timeout, in milliseconds.
    ///
    /// The value must be between 1 and 4,294,967,295 milliseconds.
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Use a custom reqwest HTTP client.
    ///
    /// This replaces the default client transport. Request timeout is still
    /// controlled by [`InvoqOptions::timeout_ms`].
    pub fn http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }
}

impl Default for InvoqOptions {
    fn default() -> Self {
        Self {
            api_origin: DEFAULT_API_ORIGIN.to_string(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            http_client: None,
        }
    }
}

impl std::fmt::Debug for InvoqOptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InvoqOptions")
            .field("api_origin", &self.api_origin)
            .field("timeout_ms", &self.timeout_ms)
            .finish_non_exhaustive()
    }
}

/// Invoice API operations.
#[derive(Clone)]
pub struct Invoices {
    client_options: RequestClientOptions,
}

impl Invoices {
    /// Create an invoice.
    pub async fn create(&self, input: CreateInvoiceInput) -> Result<Invoice> {
        require_non_empty(&input.amount, "amount")?;

        request_json(
            &self.client_options,
            Method::POST,
            &["v1", "invoices"],
            Some(&input),
        )
        .await
    }

    /// Get a public invoice by id.
    pub async fn get(&self, invoice_id: impl AsRef<str>) -> Result<PublicInvoice> {
        let invoice_id = invoice_id.as_ref();
        require_non_empty(invoice_id, "invoiceId")?;

        request_json::<PublicInvoice, ()>(
            &self.client_options,
            Method::GET,
            &["v1", "invoices", invoice_id],
            None,
        )
        .await
    }

    /// Create a test payment for a test invoice.
    pub async fn create_test_payment(
        &self,
        invoice_id: impl AsRef<str>,
        input: CreateTestPaymentInput,
    ) -> Result<TestPaymentInvoice> {
        let invoice_id = invoice_id.as_ref();
        require_non_empty(invoice_id, "invoiceId")?;
        require_non_empty(&input.amount, "amount")?;

        request_json(
            &self.client_options,
            Method::POST,
            &["v1", "invoices", invoice_id, "test-payments"],
            Some(&input),
        )
        .await
    }
}

impl std::fmt::Debug for Invoices {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Invoices")
            .field("api_origin", &self.client_options.api_origin.as_str())
            .finish()
    }
}

fn normalize_api_origin(value: &str) -> Result<Url> {
    let mut url = Url::parse(value).map_err(|_| {
        InvoqError::configuration("api_origin must be an absolute http or https origin.")
    })?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(InvoqError::configuration(
            "api_origin must be an absolute http or https origin.",
        ));
    }

    if url.host_str().is_none()
        || url.cannot_be_a_base()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(InvoqError::configuration(
            "api_origin must be an absolute http or https origin.",
        ));
    }

    if url.query().is_some() || url.fragment().is_some() {
        return Err(InvoqError::configuration(
            "api_origin must not include query or hash parts.",
        ));
    }

    let pathname = url.path().trim_end_matches('/');
    let pathname = if pathname.is_empty() { "/" } else { pathname };

    if pathname != "/" {
        return Err(InvoqError::configuration(
            "api_origin must not include a path.",
        ));
    }

    url.set_path("/");
    Ok(url)
}

fn normalize_timeout_ms(value: u64) -> Result<Duration> {
    if value == 0 || value > MAX_TIMEOUT_MS {
        return Err(InvoqError::configuration(format!(
            "timeout_ms must be a positive integer of at most {MAX_TIMEOUT_MS}."
        )));
    }

    Ok(Duration::from_millis(value))
}

fn require_non_empty(value: &str, field_name: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(InvoqError::invalid_request(format!(
            "{field_name} must be a non-empty string."
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_api_origin, Invoq, InvoqOptions};
    use crate::errors::{ApiErrorPayload, InvoqError};
    use crate::types::{CreateInvoiceInput, CreateTestPaymentInput, InvoiceStatus};
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn validates_api_keys_and_api_origin() {
        assert!(Invoq::new("").is_err());
        assert!(Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin("ftp://api.test")
        )
        .is_err());
        assert!(Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin("https://api.test/api")
        )
        .is_err());
        assert!(Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin("https://api.test?debug=1")
        )
        .is_err());
        assert!(Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin("https://user:pass@api.test")
        )
        .is_err());
        assert!(Invoq::with_options("sk_test_123", InvoqOptions::default().timeout_ms(0)).is_err());
        assert!(Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().timeout_ms(u64::from(u32::MAX) + 1)
        )
        .is_err());
    }

    #[test]
    fn normalizes_api_origin() {
        assert_eq!(
            normalize_api_origin("https://api.test/").unwrap().as_str(),
            "https://api.test/"
        );
        assert!(normalize_api_origin("https://api.test/v1").is_err());
    }

    #[test]
    fn default_request_timeout_is_documented_value() {
        assert_eq!(InvoqOptions::default().timeout_ms, 10_000);
        assert_eq!(super::DEFAULT_TIMEOUT_MS, 10_000);
    }

    #[tokio::test]
    async fn creates_invoices_with_native_json_and_authorization_headers() {
        let invoice = invoice_json("unpaid");
        let server = TestServer::spawn(201, &format!(r#"{{"data":{invoice}}}"#));

        let client = Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin(server.url()),
        )
        .unwrap();
        let result = client
            .invoices
            .create(
                CreateInvoiceInput::new("149")
                    .currency(crate::types::InvoiceCurrency::Usd)
                    .description("Test order")
                    .reference_id("order_123")
                    .return_url("https://merchant.test/thanks"),
            )
            .await
            .unwrap();
        let request = server.join();

        assert_eq!(result.id, "inv_test_123");
        assert_eq!(result.amount_overpaid, "0.000000000000000000");
        assert_eq!(result.monitoring_status, None);
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/v1/invoices");
        assert_eq!(
            request.headers.get("authorization").unwrap(),
            "Bearer sk_test_123"
        );
        assert_eq!(request.headers.get("accept").unwrap(), "application/json");
        assert_eq!(
            request.headers.get("content-type").unwrap(),
            "application/json"
        );
        assert!(request
            .headers
            .get("user-agent")
            .is_some_and(|value| value.starts_with("invoq-rust/")));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
            serde_json::json!({
                "amount": "149",
                "currency": "USD",
                "description": "Test order",
                "reference_id": "order_123",
                "return_url": "https://merchant.test/thanks"
            })
        );
    }

    #[tokio::test]
    async fn gets_invoices_by_id() {
        let invoice = public_invoice_json("unpaid");
        let server = TestServer::spawn(200, &format!(r#"{{"data":{invoice}}}"#));

        let client = Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin(server.url()),
        )
        .unwrap();
        let result = client.invoices.get("inv/test 123").await.unwrap();
        let request = server.join();

        assert_eq!(result.id, "inv_test_123");
        assert_eq!(
            result.payment_status,
            crate::types::InvoicePaymentStatus::Unpaid
        );
        assert_eq!(result.project.name.as_deref(), Some("Test project"));
        assert_eq!(result.amount_overpaid, "0.000000000000000000");
        assert_eq!(result.monitoring_status, None);
        assert!(result.transfers.is_empty());
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/v1/invoices/inv%2Ftest%20123");
        assert!(!request.headers.contains_key("content-type"));
        assert!(request.body.is_empty());
    }

    #[tokio::test]
    async fn creates_test_payments_and_returns_only_data_envelope() {
        let invoice = test_payment_invoice_json("paid");
        let server = TestServer::spawn(
            201,
            &format!(r#"{{"data":{invoice},"meta":{{"result":"created"}}}}"#),
        );

        let client = Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin(server.url()),
        )
        .unwrap();
        let result = client
            .invoices
            .create_test_payment(
                "inv_test_123",
                CreateTestPaymentInput::new("149").reference_id("test_payment_001"),
            )
            .await
            .unwrap();
        let request = server.join();

        assert_eq!(result.status, InvoiceStatus::Paid);
        assert_eq!(result.amount_paid, "149");
        assert_eq!(result.amount_overpaid, "0.000000000000000000");
        assert_eq!(result.monitoring_status, None);
        assert_eq!(request.path, "/v1/invoices/inv_test_123/test-payments");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&request.body).unwrap(),
            serde_json::json!({
                "amount": "149",
                "reference_id": "test_payment_001"
            })
        );
    }

    #[tokio::test]
    async fn rejects_blank_required_request_strings_before_sending() {
        let client = Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin("https://api.test"),
        )
        .unwrap();

        assert!(matches!(
            client.invoices.create(CreateInvoiceInput::new(" ")).await,
            Err(InvoqError::InvalidRequest(_))
        ));
        let get_error = client.invoices.get("").await.unwrap_err();
        assert_eq!(
            get_error.to_string(),
            "invoiceId must be a non-empty string."
        );
        let test_payment_invoice_error = client
            .invoices
            .create_test_payment("", CreateTestPaymentInput::new("1"))
            .await
            .unwrap_err();
        assert_eq!(
            test_payment_invoice_error.to_string(),
            "invoiceId must be a non-empty string."
        );
        assert!(matches!(
            client
                .invoices
                .create_test_payment("inv_test_123", CreateTestPaymentInput::new(""))
                .await,
            Err(InvoqError::InvalidRequest(_))
        ));
    }

    #[tokio::test]
    async fn maps_api_error_envelopes_to_invoq_api_error() {
        let server = TestServer::spawn(
            400,
            r#"{"code":"invalid_request","message":"Invalid request.","fields":[{"location":"body","field":"amount","code":"required","message":"Required."}],"meta":{"request_id":"req_test"}}"#,
        );

        let client = Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin(server.url()),
        )
        .unwrap();
        let error = client
            .invoices
            .create(CreateInvoiceInput::new("1"))
            .await
            .unwrap_err();
        let _request = server.join();

        let InvoqError::Api(error) = error else {
            panic!("expected API error");
        };

        assert_eq!(error.status, 400);
        assert_eq!(error.code.as_deref(), Some("invalid_request"));
        assert_eq!(error.fields.unwrap()[0].field, "amount");
        assert_eq!(error.meta.unwrap()["request_id"], "req_test");
    }

    #[tokio::test]
    async fn maps_non_json_http_errors_to_invoq_api_error() {
        let server = TestServer::spawn(502, "<html>bad gateway</html>");

        let client = Invoq::with_options(
            "sk_test_123",
            InvoqOptions::default().api_origin(server.url()),
        )
        .unwrap();
        let error = client
            .invoices
            .create(CreateInvoiceInput::new("1"))
            .await
            .unwrap_err();
        let _request = server.join();

        let InvoqError::Api(error) = error else {
            panic!("expected API error");
        };

        assert_eq!(error.status, 502);
        assert_eq!(
            error.payload,
            ApiErrorPayload::Text("<html>bad gateway</html>".to_string())
        );
    }

    fn invoice_json(status: &str) -> String {
        format!(
            r#"{{
                "id":"inv_test_123",
                "mode":"test",
                "amount":"149",
                "currency":"USD",
                "reference_id":"order_123",
                "description":"Test order",
                "return_url":"https://merchant.test/thanks",
                "deposit_address":null,
                "status":"{status}",
                "amount_due":"149.000000000000000000",
                "amount_overpaid":"0.000000000000000000",
                "monitoring_ends_at":null,
                "monitoring_status":null,
                "direct_onchain_rails":[]
            }}"#
        )
    }

    fn public_invoice_json(status: &str) -> String {
        format!(
            r#"{{
                "id":"inv_test_123",
                "mode":"test",
                "amount":"149",
                "currency":"USD",
                "description":"Test order",
                "return_url":null,
                "deposit_address":null,
                "status":"{status}",
                "amount_due":"149.000000000000000000",
                "amount_overpaid":"0.000000000000000000",
                "monitoring_ends_at":null,
                "monitoring_status":null,
                "direct_onchain_rails":[],
                "amount_paid":"0",
                "payment_status":"unpaid",
                "project":{{
                    "id":"proj_test_123",
                    "name":"Test project",
                    "logo_url":null
                }},
                "transfers":[]
            }}"#
        )
    }

    fn test_payment_invoice_json(status: &str) -> String {
        format!(
            r#"{{
                "id":"inv_test_123",
                "mode":"test",
                "amount":"149",
                "currency":"USD",
                "reference_id":"order_123",
                "description":"Test order",
                "return_url":"https://merchant.test/thanks",
                "deposit_address":null,
                "status":"{status}",
                "amount_due":"0.000000000000000000",
                "amount_overpaid":"0.000000000000000000",
                "monitoring_ends_at":null,
                "monitoring_status":null,
                "direct_onchain_rails":[],
                "amount_paid":"149",
                "fully_paid_at":"2026-06-15T00:00:00.000Z"
            }}"#
        )
    }

    struct TestServer {
        url: String,
        handle: thread::JoinHandle<ReceivedRequest>,
    }

    impl TestServer {
        fn spawn(status: u16, body: &str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let body = body.to_string();
            let handle = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_request(&mut stream);
                write!(
                    stream,
                    "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
                request
            });

            Self { url, handle }
        }

        fn url(&self) -> String {
            self.url.clone()
        }

        fn join(self) -> ReceivedRequest {
            self.handle.join().unwrap()
        }
    }

    #[derive(Debug)]
    struct ReceivedRequest {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        body: String,
    }

    fn read_request(stream: &mut std::net::TcpStream) -> ReceivedRequest {
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 1024];
        let header_end;

        loop {
            let read = stream.read(&mut buffer).unwrap();
            assert_ne!(read, 0, "request ended before headers");
            bytes.extend_from_slice(&buffer[..read]);

            if let Some(index) = find_subsequence(&bytes, b"\r\n\r\n") {
                header_end = index + 4;
                break;
            }
        }

        let headers_text = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
        let mut lines = headers_text.split("\r\n");
        let request_line = lines.next().unwrap();
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().unwrap().to_string();
        let path = request_parts.next().unwrap().to_string();
        let mut headers = HashMap::new();

        for line in lines.filter(|line| !line.is_empty()) {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };

            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }

        let content_length = headers
            .get("content-length")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        let mut body = bytes[header_end..].to_vec();

        while body.len() < content_length {
            let read = stream.read(&mut buffer).unwrap();
            assert_ne!(read, 0, "request ended before body");
            body.extend_from_slice(&buffer[..read]);
        }

        body.truncate(content_length);

        ReceivedRequest {
            method,
            path,
            headers,
            body: String::from_utf8(body).unwrap(),
        }
    }

    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
