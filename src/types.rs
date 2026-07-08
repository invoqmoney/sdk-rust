use serde::{Deserialize, Serialize};

/// Invoice environment.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceMode {
    Test,
    Live,
}

/// Invoice currency.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum InvoiceCurrency {
    #[serde(rename = "USD")]
    #[default]
    Usd,
}

/// Canonical invoice status.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Unpaid,
    PartiallyPaid,
    Paid,
    Settling,
    Settled,
    ReviewRequired,
}

/// Checkout-facing payment status.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoicePaymentStatus {
    Unpaid,
    Confirming,
    PartiallyPaid,
    Paid,
    Settling,
    Settled,
    ReviewRequired,
}

/// Paid-equivalent statuses emitted by `invoice.paid` webhooks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoicePaidStatus {
    Paid,
    Settling,
    Settled,
}

/// Field-level validation error returned by the invoq API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiErrorField {
    pub field: String,
    pub location: ApiErrorLocation,
    pub code: String,
    pub message: String,
}

/// Location for a field-level API error.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiErrorLocation {
    Query,
    Path,
    Body,
    Header,
}

/// Public direct-onchain payment rail returned with invoice payment instructions.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DirectOnchainRail {
    pub chain_namespace: String,
    pub chain_reference: String,
    pub token_address: String,
    pub network_label: String,
    pub display_symbol: String,
    pub logo_url: Option<String>,
    pub chain_logo_url: Option<String>,
    pub network_fee_usd: String,
    pub eta_seconds: u64,
}

/// Payer-visible project branding returned by public invoice reads.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicInvoiceProject {
    pub id: String,
    pub name: Option<String>,
    pub logo_url: Option<String>,
}

/// Invoice returned by invoice creation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Invoice {
    pub id: String,
    pub mode: InvoiceMode,
    pub amount: String,
    pub currency: InvoiceCurrency,
    pub reference_id: Option<String>,
    pub description: Option<String>,
    pub return_url: Option<String>,
    pub deposit_address: Option<String>,
    pub status: InvoiceStatus,
    pub amount_due: String,
    pub monitoring_ends_at: Option<String>,
    pub direct_onchain_rails: Vec<DirectOnchainRail>,
}

/// Invoice returned by public invoice reads.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PublicInvoice {
    pub id: String,
    pub mode: InvoiceMode,
    pub amount: String,
    pub currency: InvoiceCurrency,
    pub description: Option<String>,
    pub return_url: Option<String>,
    pub deposit_address: Option<String>,
    pub status: InvoiceStatus,
    pub amount_due: String,
    pub monitoring_ends_at: Option<String>,
    pub direct_onchain_rails: Vec<DirectOnchainRail>,
    pub amount_paid: String,
    pub payment_status: InvoicePaymentStatus,
    pub project: PublicInvoiceProject,
}

/// Invoice returned after simulating payment on a test invoice.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TestPaymentInvoice {
    pub id: String,
    pub mode: InvoiceMode,
    pub amount: String,
    pub currency: InvoiceCurrency,
    pub reference_id: Option<String>,
    pub description: Option<String>,
    pub return_url: Option<String>,
    pub deposit_address: Option<String>,
    pub status: InvoiceStatus,
    pub amount_due: String,
    pub monitoring_ends_at: Option<String>,
    pub direct_onchain_rails: Vec<DirectOnchainRail>,
    pub amount_paid: String,
    pub fully_paid_at: Option<String>,
}

/// Input for creating an invoice.
///
/// Optional request fields are omitted when unset. Use
/// [`CreateInvoiceInput::without_return_url`] to send `return_url: null`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreateInvoiceInput {
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<InvoiceCurrency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<Option<String>>,
}

impl CreateInvoiceInput {
    /// Create invoice input with a required decimal amount string.
    pub fn new(amount: impl Into<String>) -> Self {
        Self {
            amount: amount.into(),
            currency: None,
            description: None,
            reference_id: None,
            return_url: None,
        }
    }

    /// Set the invoice currency. Currently only USD is supported by invoq.
    pub fn currency(mut self, currency: InvoiceCurrency) -> Self {
        self.currency = Some(currency);
        self
    }

    /// Set the payer-visible invoice description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set a caller-side idempotency reference.
    pub fn reference_id(mut self, reference_id: impl Into<String>) -> Self {
        self.reference_id = Some(reference_id.into());
        self
    }

    /// Set the payer-visible return URL shown on successful payment screens.
    pub fn return_url(mut self, return_url: impl Into<String>) -> Self {
        self.return_url = Some(Some(return_url.into()));
        self
    }

    /// Explicitly opt out of the project's default return URL.
    pub fn without_return_url(mut self) -> Self {
        self.return_url = Some(None);
        self
    }
}

/// Input for creating a test payment.
///
/// Optional request fields are omitted when unset; request JSON does not send
/// `null` for optional strings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreateTestPaymentInput {
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

impl CreateTestPaymentInput {
    /// Create test payment input with a required decimal amount string.
    pub fn new(amount: impl Into<String>) -> Self {
        Self {
            amount: amount.into(),
            reference_id: None,
        }
    }

    /// Set a caller-side idempotency reference for this test payment.
    pub fn reference_id(mut self, reference_id: impl Into<String>) -> Self {
        self.reference_id = Some(reference_id.into());
        self
    }
}

/// Known invoice.paid webhook invoice payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaidEventInvoice {
    pub id: String,
    pub mode: InvoiceMode,
    pub status: InvoicePaidStatus,
    pub amount: String,
    pub currency: InvoiceCurrency,
    pub amount_paid: String,
    pub reference_id: Option<String>,
    pub fully_paid_at: Option<String>,
}

/// Known invoice.paid webhook data payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaidEventData {
    pub invoice: InvoicePaidEventInvoice,
}

/// Known invoice.paid webhook event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaidEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub mode: InvoiceMode,
    pub created_at: String,
    pub data: InvoicePaidEventData,
}

/// Verified webhook event payload. Unknown future event types are preserved.
pub type InvoqWebhookEvent = serde_json::Value;

#[cfg(test)]
mod tests {
    use super::{
        CreateInvoiceInput, CreateTestPaymentInput, Invoice, InvoiceCurrency, InvoicePaidStatus,
        InvoicePaymentStatus, InvoiceStatus,
    };

    #[test]
    fn create_invoice_input_omits_unset_optional_strings() {
        let value = serde_json::to_value(CreateInvoiceInput::new("149")).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "amount": "149"
            })
        );
        assert!(value.get("description").is_none());
        assert!(value.get("reference_id").is_none());
        assert!(value.get("return_url").is_none());
    }

    #[test]
    fn create_invoice_input_serializes_set_optional_strings_as_strings() {
        let value = serde_json::to_value(
            CreateInvoiceInput::new("149")
                .currency(InvoiceCurrency::Usd)
                .description("Test order")
                .reference_id("order_123")
                .return_url("https://merchant.test/thanks"),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "amount": "149",
                "currency": "USD",
                "description": "Test order",
                "reference_id": "order_123",
                "return_url": "https://merchant.test/thanks"
            })
        );
    }

    #[test]
    fn create_invoice_input_serializes_without_return_url_as_null() {
        let value =
            serde_json::to_value(CreateInvoiceInput::new("149").without_return_url()).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "amount": "149",
                "return_url": null
            })
        );
    }

    #[test]
    fn create_test_payment_input_omits_unset_reference_id() {
        let value = serde_json::to_value(CreateTestPaymentInput::new("149")).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "amount": "149"
            })
        );
        assert!(value.get("reference_id").is_none());
    }

    #[test]
    fn create_test_payment_input_serializes_set_reference_id_as_string() {
        let value = serde_json::to_value(
            CreateTestPaymentInput::new("149").reference_id("test_payment_001"),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "amount": "149",
                "reference_id": "test_payment_001"
            })
        );
    }

    #[test]
    fn status_enums_use_backend_wire_values() {
        assert_eq!(
            serde_json::to_value(InvoiceStatus::PartiallyPaid).unwrap(),
            serde_json::json!("partially_paid")
        );
        assert_eq!(
            serde_json::to_value(InvoicePaymentStatus::ReviewRequired).unwrap(),
            serde_json::json!("review_required")
        );
        assert_eq!(
            serde_json::from_value::<InvoicePaidStatus>(serde_json::json!("settled")).unwrap(),
            InvoicePaidStatus::Settled
        );
    }

    #[test]
    fn invoice_deserializes_null_optional_strings() {
        let invoice: Invoice = serde_json::from_value(serde_json::json!({
            "id": "inv_test_123",
            "mode": "test",
            "amount": "149",
            "currency": "USD",
            "reference_id": null,
            "description": null,
            "return_url": null,
            "deposit_address": null,
            "status": "unpaid",
            "amount_due": "149.000000000000000000",
            "monitoring_ends_at": null,
            "direct_onchain_rails": []
        }))
        .unwrap();

        assert_eq!(invoice.reference_id, None);
        assert_eq!(invoice.description, None);
        assert_eq!(invoice.return_url, None);
    }
}
