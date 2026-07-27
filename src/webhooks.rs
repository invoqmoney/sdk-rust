use crate::errors::{InvoqSignatureVerificationError, SignatureVerificationErrorCode};
use crate::types::{InvoicePaidEvent, InvoicePaymentReversedEvent, InvoqWebhookEvent};
use hmac::{Hmac, Mac};
use http::HeaderMap;
use serde::de::DeserializeOwned;
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
///
/// Paid-equivalent invoice statuses only: `review_required` has money against it
/// but is not cleared for fulfillment, so it does not decode. This guard fails
/// closed — an event it cannot recognize is never fulfilled.
pub fn invoice_paid_event(event: &InvoqWebhookEvent) -> Option<InvoicePaidEvent> {
    lifecycle_event(event, "invoice.paid")
}

/// Return whether a verified webhook event matches the
/// invoice.payment_reversed shape.
pub fn is_invoice_payment_reversed(event: &InvoqWebhookEvent) -> bool {
    invoice_payment_reversed_event(event).is_some()
}

/// Decode a verified invoice.payment_reversed webhook event.
///
/// Unlike [`invoice_paid_event`] this applies no status rule, and
/// [`crate::InvoicePaymentReversedEventInvoice`] keeps `status` as a plain
/// string on purpose: rejecting an unrecognized status would drop the event and
/// leave an order fulfilled on a payment that no longer exists.
pub fn invoice_payment_reversed_event(
    event: &InvoqWebhookEvent,
) -> Option<InvoicePaymentReversedEvent> {
    lifecycle_event(event, "invoice.payment_reversed")
}

/// Shape check shared by both invoice lifecycle events.
///
/// `reference_id` and `fully_paid_at` are nullable but always present, and an
/// `Option` field would also accept them missing, so they are checked here.
/// Every other field — including `payment_revision`, which must be an integer —
/// is enforced by the target type.
fn lifecycle_event<T>(event: &InvoqWebhookEvent, event_type: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    let object = event.as_object()?;

    if object.get("type")?.as_str()? != event_type {
        return None;
    }

    let invoice = object
        .get("data")?
        .as_object()?
        .get("invoice")?
        .as_object()?;

    if !invoice.contains_key("reference_id") || !invoice.contains_key("fully_paid_at") {
        return None;
    }

    serde_json::from_value(event.clone()).ok()
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
    use crate::types::InvoqWebhookEvent;
    use crate::webhooks::{
        invoice_payment_reversed_event, is_invoice_paid, is_invoice_payment_reversed,
        verify_webhook,
    };
    use http::HeaderMap;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    const SECRET: &str = "whsec_test_123";
    const TIMESTAMP: i128 = 1_710_000_000;
    // An event type this version does not model: verification is shape-agnostic,
    // so a new backend event never fails on an older SDK.
    const BODY: &str =
        r#"{"id":"evt_test","type":"invoice.future_event","data":{"invoice":{"id":"inv_test"}}}"#;
    const HEADER: &str =
        "t=1710000000,v1=7882995406911f86ee0e8a85feba7e21befe10ead08701ff7ff066738ca4c28e";

    #[test]
    fn verifies_string_payload_signatures() {
        let event = verify_webhook_with_now(BODY.as_bytes(), HEADER, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["id"], "evt_test");
        assert_eq!(event["type"], "invoice.future_event");
    }

    #[test]
    fn verifies_byte_payloads_and_header_maps() {
        let bytes = hex_to_bytes(
            "7b226964223a226576745f6279746573222c2274797065223a22696e766f6963652e6675747572655f6576656e74222c2264617461223a7b22696e766f696365223a7b226964223a22696e765f6279746573227d7d7d",
        );
        let header =
            "t=1710000001,v1=fa0fde1c5d73fe059235b19dc1d7785e1d3c695e055dfcfa8f69a1202bacee37";
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
            "v1=7882995406911f86ee0e8a85feba7e21befe10ead08701ff7ff066738ca4c28e"
                .parse()
                .unwrap(),
        );

        let event = verify_webhook_with_now(BODY.as_bytes(), &headers, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["id"], "evt_test");
    }

    #[test]
    fn uses_last_v1_signature() {
        let header = format!(
            "t=1710000000,v1={},v1=7882995406911f86ee0e8a85feba7e21befe10ead08701ff7ff066738ca4c28e",
            "0".repeat(64)
        );

        let event = verify_webhook_with_now(BODY.as_bytes(), header, SECRET, TIMESTAMP).unwrap();

        assert_eq!(event["id"], "evt_test");

        let header = format!(
            "t=1710000000,v1=7882995406911f86ee0e8a85feba7e21befe10ead08701ff7ff066738ca4c28e,v1={}",
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

        assert_eq!(event["type"], "invoice.future_event");
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
    fn accepts_every_paid_equivalent_status() {
        for status in ["paid", "settling", "settled"] {
            let mut invoice = paid_invoice();
            invoice["status"] = json!(status);

            assert!(is_invoice_paid(&lifecycle_event("invoice.paid", invoice)));
        }
    }

    #[test]
    fn checks_the_full_invoice_shape_before_decoding() {
        for field in INVOICE_FIELDS {
            let invoice = without_field(paid_invoice(), field);

            assert!(
                !is_invoice_paid(&lifecycle_event("invoice.paid", invoice)),
                "expected a missing {field} to be rejected"
            );
        }

        for payment_revision in [json!("1"), json!(1.5), json!(-1)] {
            let mut invoice = paid_invoice();
            invoice["payment_revision"] = payment_revision;

            assert!(!is_invoice_paid(&lifecycle_event("invoice.paid", invoice)));
        }
    }

    #[test]
    fn rejects_a_mangled_envelope_around_a_valid_invoice() {
        for field in ["id", "mode", "created_at", "data"] {
            let event = without_field(lifecycle_event("invoice.paid", paid_invoice()), field);

            assert!(
                !is_invoice_paid(&event),
                "expected a missing envelope {field} to be rejected"
            );
        }
    }

    #[test]
    fn rejects_statuses_that_are_not_cleared_for_fulfillment() {
        for status in ["review_required", "partially_paid", "unexpected"] {
            let mut invoice = paid_invoice();
            invoice["status"] = json!(status);

            assert!(!is_invoice_paid(&lifecycle_event("invoice.paid", invoice)));
        }
    }

    #[test]
    fn rejects_a_reversal_whatever_it_reverted_the_invoice_to() {
        assert!(!is_invoice_paid(&lifecycle_event(
            "invoice.payment_reversed",
            reversed_invoice()
        )));

        let mut invoice = reversed_invoice();
        invoice["status"] = json!("paid");

        assert!(!is_invoice_paid(&lifecycle_event(
            "invoice.payment_reversed",
            invoice
        )));
    }

    #[test]
    fn accepts_a_reversal_in_any_canonical_status() {
        for status in [
            "unpaid",
            "partially_paid",
            "review_required",
            "paid",
            "settling",
            "settled",
        ] {
            let mut invoice = reversed_invoice();
            invoice["status"] = json!(status);

            assert!(is_invoice_payment_reversed(&lifecycle_event(
                "invoice.payment_reversed",
                invoice
            )));
        }
    }

    #[test]
    fn checks_the_same_shared_invoice_shape_for_reversals() {
        for field in INVOICE_FIELDS {
            let invoice = without_field(reversed_invoice(), field);

            assert!(
                !is_invoice_payment_reversed(&lifecycle_event("invoice.payment_reversed", invoice)),
                "expected a missing {field} to be rejected"
            );
        }

        let mut invoice = reversed_invoice();
        invoice["payment_revision"] = json!("2");

        assert!(!is_invoice_payment_reversed(&lifecycle_event(
            "invoice.payment_reversed",
            invoice
        )));
    }

    /// The reversal guard must not fail closed like the paid guard: dropping a
    /// reversal leaves an order fulfilled on a payment that no longer exists.
    #[test]
    fn accepts_a_reversal_status_this_version_does_not_know() {
        let mut invoice = reversed_invoice();
        invoice["status"] = json!("unexpected");

        assert!(is_invoice_payment_reversed(&lifecycle_event(
            "invoice.payment_reversed",
            invoice
        )));
    }

    #[test]
    fn rejects_a_paid_event_as_a_reversal() {
        assert!(!is_invoice_payment_reversed(&lifecycle_event(
            "invoice.paid",
            paid_invoice()
        )));
    }

    /// The documented path: verify, then branch on the event type.
    #[test]
    fn carries_signed_lifecycle_events_through_to_their_typed_fields() {
        let paid_body = lifecycle_event("invoice.paid", paid_invoice()).to_string();
        let paid = verify_webhook_with_now(
            paid_body.as_bytes(),
            signature_header(&paid_body),
            SECRET,
            TIMESTAMP,
        )
        .unwrap();
        let paid = invoice_paid_event(&paid).unwrap();

        assert_eq!(paid.event_type, "invoice.paid");
        assert_eq!(paid.data.invoice.status, crate::InvoicePaidStatus::Paid);
        assert_eq!(paid.data.invoice.reference_id.as_deref(), Some("order_123"));
        assert_eq!(paid.data.invoice.payment_revision, 1);

        let reversed_body =
            lifecycle_event("invoice.payment_reversed", reversed_invoice()).to_string();
        let reversed = verify_webhook_with_now(
            reversed_body.as_bytes(),
            signature_header(&reversed_body),
            SECRET,
            TIMESTAMP,
        )
        .unwrap();

        assert!(!is_invoice_paid(&reversed));

        let reversed = invoice_payment_reversed_event(&reversed).unwrap();

        assert_eq!(reversed.data.invoice.status, "partially_paid");
        assert_eq!(reversed.data.invoice.payment_revision, 2);
        assert_eq!(reversed.data.invoice.fully_paid_at, None);
    }

    #[test]
    fn decodes_nullable_lifecycle_invoice_fields_as_none() {
        let mut invoice = paid_invoice();
        invoice["reference_id"] = json!(null);
        invoice["fully_paid_at"] = json!(null);

        let event = lifecycle_event("invoice.paid", invoice);
        let parsed = invoice_paid_event(&event).unwrap();

        assert_eq!(parsed.data.invoice.reference_id, None);
        assert_eq!(parsed.data.invoice.fully_paid_at, None);
    }

    #[test]
    fn public_verify_webhook_accepts_signature_strings() {
        let event = verify_webhook(BODY, HEADER, SECRET).unwrap_err();

        assert_eq!(
            event.code,
            SignatureVerificationErrorCode::TimestampOutsideTolerance
        );
    }

    // Every field the shared envelope check requires of data.invoice.
    const INVOICE_FIELDS: [&str; 9] = [
        "id",
        "mode",
        "status",
        "amount",
        "currency",
        "amount_paid",
        "reference_id",
        "payment_revision",
        "fully_paid_at",
    ];

    // The snapshot at the moment the invoice was first fully paid.
    fn paid_invoice() -> Value {
        json!({
            "id": "inv_test",
            "mode": "test",
            "status": "paid",
            "amount": "149.0000",
            "currency": "USD",
            "amount_paid": "149.000000000000000000",
            "reference_id": "order_123",
            "payment_revision": 1,
            "fully_paid_at": "2026-06-15T00:00:00.000Z"
        })
    }

    // The same invoice after a credited transfer was reversed.
    fn reversed_invoice() -> Value {
        let mut invoice = paid_invoice();
        invoice["status"] = json!("partially_paid");
        invoice["amount_paid"] = json!("20.000000000000000000");
        invoice["payment_revision"] = json!(2);
        invoice["fully_paid_at"] = json!(null);
        invoice
    }

    fn lifecycle_event(event_type: &str, invoice: Value) -> InvoqWebhookEvent {
        json!({
            "id": "wdel_test",
            "type": event_type,
            "mode": "test",
            "created_at": "2026-06-15T00:00:00.000Z",
            "data": {
                "invoice": invoice
            }
        })
    }

    fn without_field(mut value: Value, field: &str) -> Value {
        value
            .as_object_mut()
            .expect("test payloads are JSON objects")
            .remove(field);
        value
    }

    fn signature_header(body: &str) -> String {
        format!(
            "t=1710000000,v1={}",
            hmac_sha256_hex(SECRET, "1710000000", body.as_bytes())
        )
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
