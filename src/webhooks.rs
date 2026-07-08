use crate::errors::{InvoqSignatureVerificationError, SignatureVerificationErrorCode};
use crate::types::{InvoicePaidEvent, InvoqWebhookEvent};
use hmac::{Hmac, Mac};
use http::HeaderMap;
use serde_json::Value;
use sha2::Sha256;
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_TOLERANCE_SECONDS: i128 = 300;

type HmacSha256 = Hmac<Sha256>;

/// Header sources accepted by [`verify_webhook`].
pub trait WebhookHeaders {
    fn invoq_signature(&self) -> Option<String>;
}

impl<T: WebhookHeaders + ?Sized> WebhookHeaders for &T {
    fn invoq_signature(&self) -> Option<String> {
        (*self).invoq_signature()
    }
}

impl<T: WebhookHeaders> WebhookHeaders for Option<T> {
    fn invoq_signature(&self) -> Option<String> {
        self.as_ref().and_then(WebhookHeaders::invoq_signature)
    }
}

impl WebhookHeaders for str {
    fn invoq_signature(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl WebhookHeaders for String {
    fn invoq_signature(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl WebhookHeaders for HeaderMap {
    fn invoq_signature(&self) -> Option<String> {
        let values = self
            .get_all("invoq-signature")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .collect::<Vec<_>>();

        if values.is_empty() {
            None
        } else {
            Some(values.join(","))
        }
    }
}

impl WebhookHeaders for HashMap<String, String> {
    fn invoq_signature(&self) -> Option<String> {
        find_signature_header(
            self.iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        )
    }
}

impl WebhookHeaders for HashMap<String, Vec<String>> {
    fn invoq_signature(&self) -> Option<String> {
        self.iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("invoq-signature"))
            .map(|(_, values)| values.join(","))
    }
}

impl WebhookHeaders for BTreeMap<String, String> {
    fn invoq_signature(&self) -> Option<String> {
        find_signature_header(
            self.iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        )
    }
}

impl WebhookHeaders for BTreeMap<String, Vec<String>> {
    fn invoq_signature(&self) -> Option<String> {
        self.iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("invoq-signature"))
            .map(|(_, values)| values.join(","))
    }
}

/// Verify an invoq webhook and return the decoded event.
pub fn verify_webhook<B, H>(
    raw_body: B,
    headers: H,
    webhook_secret: &str,
) -> std::result::Result<InvoqWebhookEvent, InvoqSignatureVerificationError>
where
    B: AsRef<[u8]>,
    H: WebhookHeaders,
{
    verify_webhook_with_now(raw_body.as_ref(), headers, webhook_secret, now_seconds())
}

/// Return whether a verified webhook event matches the invoice.paid shape.
pub fn is_invoice_paid(event: &InvoqWebhookEvent) -> bool {
    invoice_paid_event(event).is_some()
}

/// Decode a verified invoice.paid webhook event.
pub fn invoice_paid_event(event: &InvoqWebhookEvent) -> Option<InvoicePaidEvent> {
    let invoice = event
        .as_object()?
        .get("data")?
        .as_object()?
        .get("invoice")?
        .as_object()?;

    if !invoice.contains_key("reference_id") || !invoice.contains_key("fully_paid_at") {
        return None;
    }

    let parsed: InvoicePaidEvent = serde_json::from_value(event.clone()).ok()?;

    if parsed.event_type == "invoice.paid" {
        Some(parsed)
    } else {
        None
    }
}

fn verify_webhook_with_now<H>(
    raw_body: &[u8],
    headers: H,
    webhook_secret: &str,
    now_seconds: i128,
) -> std::result::Result<InvoqWebhookEvent, InvoqSignatureVerificationError>
where
    H: WebhookHeaders,
{
    let signature_header = headers.invoq_signature().filter(|value| !value.is_empty());

    let Some(signature_header) = signature_header else {
        return Err(signature_error(
            SignatureVerificationErrorCode::MissingSignature,
            "Missing invoq-signature header.",
        ));
    };

    if webhook_secret.is_empty() {
        return Err(signature_error(
            SignatureVerificationErrorCode::InvalidSignatureHeader,
            "Webhook secret must be a non-empty string.",
        ));
    }

    let parsed = parse_signature_header(&signature_header)?;

    if (now_seconds - parsed.timestamp_seconds).abs() > DEFAULT_TOLERANCE_SECONDS {
        return Err(signature_error(
            SignatureVerificationErrorCode::TimestampOutsideTolerance,
            "Webhook timestamp is outside the allowed tolerance.",
        ));
    }

    let expected_signature = hmac_sha256_hex(webhook_secret, &parsed.timestamp, raw_body);

    if !constant_time_equal(expected_signature.as_bytes(), parsed.signature.as_bytes()) {
        return Err(signature_error(
            SignatureVerificationErrorCode::SignatureMismatch,
            "Webhook signature mismatch.",
        ));
    }

    let payload: Value = serde_json::from_slice(raw_body).map_err(|_| {
        signature_error(
            SignatureVerificationErrorCode::InvalidPayload,
            "Webhook payload is not valid JSON.",
        )
    })?;

    if !payload.is_object()
        || !payload
            .as_object()
            .and_then(|object| object.get("type"))
            .is_some_and(Value::is_string)
    {
        return Err(signature_error(
            SignatureVerificationErrorCode::InvalidPayload,
            "Webhook payload must be an object with a string type.",
        ));
    }

    Ok(payload)
}

struct ParsedSignatureHeader {
    timestamp: String,
    timestamp_seconds: i128,
    signature: String,
}

fn parse_signature_header(
    signature_header: &str,
) -> std::result::Result<ParsedSignatureHeader, InvoqSignatureVerificationError> {
    let mut timestamp = None;
    let mut signature = None;

    for part in signature_header.split(',') {
        let Some(separator_index) = part.find('=') else {
            return Err(signature_error(
                SignatureVerificationErrorCode::InvalidSignatureHeader,
                "Invalid invoq-signature header.",
            ));
        };

        let key = part[..separator_index].trim();
        let value = part[separator_index + 1..].trim();

        if key.is_empty() || value.is_empty() {
            continue;
        }

        match key {
            "t" => timestamp = Some(value.to_string()),
            "v1" => signature = Some(value.to_ascii_lowercase()),
            _ => {}
        }
    }

    let Some(timestamp) = timestamp else {
        return Err(signature_error(
            SignatureVerificationErrorCode::InvalidSignatureHeader,
            "Invalid invoq-signature header.",
        ));
    };
    let Some(signature) = signature else {
        return Err(signature_error(
            SignatureVerificationErrorCode::InvalidSignatureHeader,
            "Invalid invoq-signature header.",
        ));
    };

    if !timestamp.chars().all(|value| value.is_ascii_digit()) {
        return Err(signature_error(
            SignatureVerificationErrorCode::InvalidSignatureHeader,
            "Invalid invoq-signature header.",
        ));
    }

    if !is_signature_hex(&signature) {
        return Err(signature_error(
            SignatureVerificationErrorCode::InvalidSignatureHeader,
            "Invalid invoq-signature signature.",
        ));
    }

    let timestamp_seconds = timestamp.parse::<i128>().map_err(|_| {
        signature_error(
            SignatureVerificationErrorCode::InvalidSignatureHeader,
            "Invalid invoq-signature header.",
        )
    })?;

    Ok(ParsedSignatureHeader {
        timestamp,
        timestamp_seconds,
        signature,
    })
}

fn hmac_sha256_hex(secret: &str, timestamp: &str, raw_body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts secret keys of any size");
    mac.update(timestamp.as_bytes());
    mac.update(b".");
    mac.update(raw_body);
    hex_lower(&mac.finalize().into_bytes())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn constant_time_equal(left: &[u8], right: &[u8]) -> bool {
    let max_len = left.len().max(right.len()).max(1);
    let mut diff = left.len() ^ right.len();

    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(left_byte ^ right_byte);
    }

    diff == 0
}

fn find_signature_header<'a>(headers: impl Iterator<Item = (&'a str, &'a str)>) -> Option<String> {
    headers
        .into_iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("invoq-signature"))
        .map(|(_, value)| value.to_string())
}

fn is_signature_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn now_seconds() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i128::from(duration.as_secs()))
        .unwrap_or(0)
}

fn signature_error(
    code: SignatureVerificationErrorCode,
    message: &'static str,
) -> InvoqSignatureVerificationError {
    InvoqSignatureVerificationError::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::{hmac_sha256_hex, invoice_paid_event, verify_webhook_with_now};
    use crate::errors::SignatureVerificationErrorCode;
    use crate::webhooks::{is_invoice_paid, verify_webhook};
    use http::HeaderMap;
    use serde_json::json;
    use std::collections::HashMap;

    const SECRET: &str = "whsec_test_123";
    const TIMESTAMP: i128 = 1_710_000_000;
    const BODY: &str =
        r#"{"id":"evt_test","type":"webhook.ping","data":{"project":{"id":"proj_test"}}}"#;
    const HEADER: &str =
        "t=1710000000,v1=eeafd628acb4e854f5fd942644490b313220dcc7906303d0c8572050ee7795ff";

    #[test]
    fn verifies_string_payload_signatures() {
        let event = verify_webhook_with_now(BODY.as_bytes(), HEADER, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["id"], "evt_test");
        assert_eq!(event["type"], "webhook.ping");
    }

    #[test]
    fn verifies_byte_payloads_and_header_maps() {
        let bytes = hex_to_bytes(
            "7b226964223a226576745f6279746573222c2274797065223a22776562686f6f6b2e70696e67222c2264617461223a7b2270726f6a656374223a7b226964223a2270726f6a5f6279746573227d7d7d",
        );
        let header =
            "t=1710000001,v1=1ee237dd9e509e515eca754c3a34da3536e8c76cfc8ce1fd0a4e74d1366d20e2";
        let mut headers = HeaderMap::new();
        headers.insert("invoq-signature", header.parse().unwrap());

        let event = verify_webhook_with_now(&bytes, &headers, SECRET, 1_710_000_001).unwrap();

        assert_eq!(event["id"], "evt_bytes");
    }

    #[test]
    fn accepts_multi_value_header_maps() {
        let mut headers = HeaderMap::new();
        headers.append("invoq-signature", "t=1710000000".parse().unwrap());
        headers.append(
            "invoq-signature",
            "v1=eeafd628acb4e854f5fd942644490b313220dcc7906303d0c8572050ee7795ff"
                .parse()
                .unwrap(),
        );

        let event = verify_webhook_with_now(BODY.as_bytes(), &headers, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["id"], "evt_test");
    }

    #[test]
    fn uses_last_v1_signature() {
        let header = format!(
            "t=1710000000,v1={},v1=eeafd628acb4e854f5fd942644490b313220dcc7906303d0c8572050ee7795ff",
            "0".repeat(64)
        );

        let event = verify_webhook_with_now(BODY.as_bytes(), header, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["id"], "evt_test");

        let header = format!(
            "t=1710000000,v1=eeafd628acb4e854f5fd942644490b313220dcc7906303d0c8572050ee7795ff,v1={}",
            "0".repeat(64)
        );

        assert_signature_error(
            verify_webhook_with_now(BODY.as_bytes(), header, SECRET, TIMESTAMP).unwrap_err(),
            SignatureVerificationErrorCode::SignatureMismatch,
        );
    }

    #[test]
    fn accepts_case_insensitive_map_headers() {
        let mut headers = HashMap::new();
        headers.insert("Invoq-Signature".to_string(), HEADER.to_string());

        let event = verify_webhook_with_now(BODY.as_bytes(), headers, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["type"], "webhook.ping");
    }

    #[test]
    fn rejects_invalid_signature_inputs() {
        assert_signature_error(
            verify_webhook_with_now(BODY.as_bytes(), Option::<&str>::None, SECRET, TIMESTAMP)
                .unwrap_err(),
            SignatureVerificationErrorCode::MissingSignature,
        );
        assert_signature_error(
            verify_webhook_with_now(BODY.as_bytes(), "v1=abc", SECRET, TIMESTAMP).unwrap_err(),
            SignatureVerificationErrorCode::InvalidSignatureHeader,
        );
        assert_signature_error(
            verify_webhook_with_now(BODY.as_bytes(), HEADER, SECRET, TIMESTAMP + 301).unwrap_err(),
            SignatureVerificationErrorCode::TimestampOutsideTolerance,
        );
        assert_signature_error(
            verify_webhook_with_now(BODY.as_bytes(), HEADER, "wrong", TIMESTAMP).unwrap_err(),
            SignatureVerificationErrorCode::SignatureMismatch,
        );
    }

    #[test]
    fn rejects_invalid_payloads_after_signature() {
        let header = format!(
            "t=1710000000,v1={}",
            hmac_sha256_hex(SECRET, "1710000000", b"not json")
        );

        assert_signature_error(
            verify_webhook_with_now(b"not json", header, SECRET, TIMESTAMP).unwrap_err(),
            SignatureVerificationErrorCode::InvalidPayload,
        );
    }

    #[test]
    fn checks_invoice_paid_shape_before_decoding() {
        let event = json!({
            "id": "evt_paid",
            "type": "invoice.paid",
            "mode": "test",
            "created_at": "2026-06-15T00:00:00.000Z",
            "data": {
                "invoice": {
                    "id": "inv_test",
                    "mode": "test",
                    "status": "paid",
                    "amount": "149",
                    "currency": "USD",
                    "amount_paid": "149",
                    "reference_id": "order_123",
                    "fully_paid_at": "2026-06-15T00:00:00.000Z"
                }
            }
        });

        assert!(is_invoice_paid(&event));
        assert_eq!(
            invoice_paid_event(&event)
                .unwrap()
                .data
                .invoice
                .reference_id,
            Some("order_123".to_string())
        );

        let nullable_optionals = json!({
            "id": "evt_paid",
            "type": "invoice.paid",
            "mode": "test",
            "created_at": "2026-06-15T00:00:00.000Z",
            "data": {
                "invoice": {
                    "id": "inv_test",
                    "mode": "test",
                    "status": "paid",
                    "amount": "149",
                    "currency": "USD",
                    "amount_paid": "149",
                    "reference_id": null,
                    "fully_paid_at": null
                }
            }
        });

        assert!(is_invoice_paid(&nullable_optionals));
        let parsed = invoice_paid_event(&nullable_optionals).unwrap();
        assert_eq!(parsed.data.invoice.reference_id, None);
        assert_eq!(parsed.data.invoice.fully_paid_at, None);

        for status in ["settling", "settled"] {
            let paid_like = json!({
                "id": "evt_paid",
                "type": "invoice.paid",
                "mode": "test",
                "created_at": "2026-06-15T00:00:00.000Z",
                "data": {
                    "invoice": {
                        "id": "inv_test",
                        "mode": "test",
                        "status": status,
                        "amount": "149",
                        "currency": "USD",
                        "amount_paid": "149",
                        "reference_id": "order_123",
                        "fully_paid_at": "2026-06-15T00:00:00.000Z"
                    }
                }
            });

            assert!(is_invoice_paid(&paid_like));
        }

        let review_required = json!({
            "id": "evt_paid",
            "type": "invoice.paid",
            "mode": "test",
            "created_at": "2026-06-15T00:00:00.000Z",
            "data": {
                "invoice": {
                    "id": "inv_test",
                    "mode": "test",
                    "status": "review_required",
                    "amount": "149",
                    "currency": "USD",
                    "amount_paid": "149",
                    "reference_id": "order_123",
                    "fully_paid_at": null
                }
            }
        });

        assert!(!is_invoice_paid(&review_required));

        let missing_amount_paid = json!({
            "id": "evt_paid",
            "type": "invoice.paid",
            "mode": "test",
            "created_at": "2026-06-15T00:00:00.000Z",
            "data": {
                "invoice": {
                    "id": "inv_test",
                    "mode": "test",
                    "status": "paid",
                    "amount": "149",
                    "currency": "USD",
                    "reference_id": "order_123",
                    "fully_paid_at": "2026-06-15T00:00:00.000Z"
                }
            }
        });

        assert!(!is_invoice_paid(&missing_amount_paid));

        let missing_reference_id = json!({
            "id": "evt_paid",
            "type": "invoice.paid",
            "mode": "test",
            "created_at": "2026-06-15T00:00:00.000Z",
            "data": {
                "invoice": {
                    "id": "inv_test",
                    "mode": "test",
                    "status": "paid",
                    "amount": "149",
                    "currency": "USD",
                    "amount_paid": "149",
                    "fully_paid_at": "2026-06-15T00:00:00.000Z"
                }
            }
        });

        assert!(!is_invoice_paid(&missing_reference_id));
    }

    #[test]
    fn public_verify_webhook_accepts_signature_strings() {
        let event = verify_webhook(BODY, HEADER, SECRET).unwrap_err();

        assert_eq!(
            event.code,
            SignatureVerificationErrorCode::TimestampOutsideTolerance
        );
    }

    fn assert_signature_error(
        error: crate::errors::InvoqSignatureVerificationError,
        code: SignatureVerificationErrorCode,
    ) {
        assert_eq!(error.code, code);
    }

    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks(2)
            .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
            .collect()
    }
}
