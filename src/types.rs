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

/// Canonical accounting status.
///
/// `paid`, `settling`, and `settled` all mean the buyer paid and differ only in
/// how far the funds have moved to your wallet; `review_required` is not a paid
/// state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Unpaid,
    PartiallyPaid,
    Paid,
    Settling,
    Settled,
    ReviewRequired,
    /// A value this SDK version does not know.
    ///
    /// The backend can add one without treating it as breaking. Without this
    /// arm the whole response would fail to deserialize the day that ships.
    /// `match` still forces you to handle it.
    #[serde(untagged)]
    Unknown(String),
}

/// Payer-facing checkout state, derived on every response.
///
/// Evaluated in this order: `paid`, then `confirming` (on-chain evidence has
/// arrived and is not yet confirmed), `expired` (past `monitoring_ends_at`),
/// `open` (at least one payment option is ready), then `unavailable`. It never
/// authorizes fulfillment — use the `invoice.paid` webhook for that.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckoutStatus {
    Paid,
    Confirming,
    Expired,
    Open,
    Unavailable,
    /// A value this SDK version does not know.
    ///
    /// The backend can add one without treating it as breaking. Without this
    /// arm the whole response would fail to deserialize the day that ships.
    /// `match` still forces you to handle it.
    #[serde(untagged)]
    Unknown(String),
}

/// Paid-equivalent statuses emitted by `invoice.paid` webhooks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoicePaidStatus {
    Paid,
    Settling,
    Settled,
}

/// Chain namespace of a payment option or a confirmed transfer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainNamespace {
    Eip155,
    Solana,
    Tron,
    /// A value this SDK version does not know.
    ///
    /// The backend can add one without treating it as breaking. Without this
    /// arm the whole response would fail to deserialize the day that ships.
    /// `match` still forces you to handle it.
    #[serde(untagged)]
    Unknown(String),
}

/// How a payment option collects funds.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentOptionCollectionMethod {
    EvmDeposit,
    DirectExact,
    /// A value this SDK version does not know.
    ///
    /// The backend can add one without treating it as breaking. Without this
    /// arm the whole response would fail to deserialize the day that ships.
    /// `match` still forces you to handle it.
    #[serde(untagged)]
    Unknown(String),
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
    /// A value this SDK version does not know.
    ///
    /// The backend can add one without treating it as breaking. Without this
    /// arm the whole response would fail to deserialize the day that ships.
    /// `match` still forces you to handle it.
    #[serde(untagged)]
    Unknown(String),
}

/// Payable instructions carried by a `ready` payment option.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PaymentInstructions {
    /// An address owned by this invoice alone; any on-time transfer to it
    /// credits the invoice by its amount.
    ///
    /// `suggested_amount` is guidance, not a match requirement: it is
    /// `max(0, amount_due - pending)` rounded up to the rail's decimals, so it
    /// can exceed `amount_due` by one token unit.
    EvmDeposit {
        deposit_address: String,
        suggested_amount: String,
    },
    /// The merchant's own address plus an exact amount.
    ///
    /// The buyer must send exactly `exact_amount` (`invoice_amount` plus
    /// `matching_increment`) in a single transfer. The increment is how the
    /// payment is attributed to this invoice; it reaches the merchant but is
    /// never invoice credit. All three carry exactly `token_decimals`
    /// fractional digits.
    DirectExact {
        recipient_address: String,
        invoice_amount: String,
        matching_increment: String,
        exact_amount: String,
    },
}

/// Whether a payment option can be paid right now.
///
/// This is the only part of an option re-evaluated on every response. An
/// `Unavailable` option is out of service — under review, a blocked address or
/// rail, a paused chain, an elapsed payment window — and must not be offered to
/// the buyer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PaymentOptionStatus {
    Ready(PaymentInstructions),
    Unavailable,
    /// A status this SDK version does not know.
    ///
    /// Treated as not payable, which is the safe direction: an option we cannot
    /// interpret must never be offered to a buyer. Unlike the other enums this
    /// one cannot carry the raw value — it is the internally tagged
    /// discriminator — so an unknown status does not round-trip.
    #[serde(other)]
    Unknown,
}

/// One way to pay an invoice, fixed when the invoice is created.
///
/// A receiving address or rail configured later never rewrites an issued
/// option. Identify an option by (`chain_namespace`, `chain_reference`,
/// `token_address`) — never by its position in `payment_options`, and never by
/// `network_label`, `display_symbol`, `logo_url`, or `chain_logo_url`, which are
/// display metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PaymentOption {
    pub collection_method: PaymentOptionCollectionMethod,
    pub chain_namespace: ChainNamespace,
    pub chain_reference: String,
    pub currency: InvoiceCurrency,
    pub token_address: String,
    pub token_decimals: u32,
    pub network_label: String,
    pub display_symbol: String,
    pub logo_url: Option<String>,
    pub chain_logo_url: Option<String>,
    #[serde(flatten)]
    pub status: PaymentOptionStatus,
}

/// Payer-visible project branding returned by public invoice reads.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicInvoiceProject {
    pub id: String,
    pub name: Option<String>,
    pub logo_url: Option<String>,
}

/// One confirmed inbound transfer credited to the invoice — the payer-facing
/// receipt trail.
///
/// `amount` is in invoice-currency units at the same 18-decimal scale as
/// `amount_paid`, and for a `direct_exact` option it excludes the matching
/// increment. `transaction_id` is not unique on its own: one transaction can
/// carry several credits, which `event_index` separates.
/// `explorer_transaction_url` is `None` when the chain has no configured
/// explorer.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PublicInvoiceTransfer {
    pub chain_namespace: ChainNamespace,
    pub chain_reference: String,
    pub transaction_id: String,
    pub event_index: u64,
    pub amount: String,
    pub explorer_transaction_url: Option<String>,
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
    pub status: InvoiceStatus,
    pub checkout_status: CheckoutStatus,
    /// Increments whenever the confirmed payment set changes; settlement alone
    /// does not move it. Use it to discard a snapshot older than one you hold.
    pub payment_revision: u64,
    /// `max(amount - amount_paid, 0)` and `max(amount_paid - amount, 0)`, both
    /// at the 18-decimal scale of `amount_paid`. Read these instead of
    /// subtracting money yourself.
    pub amount_due: String,
    pub amount_overpaid: String,
    /// One day after creation, and the only payment window. `None` in test mode.
    pub monitoring_ends_at: Option<String>,
    /// The only place payment instructions live. Empty in test mode.
    pub payment_options: Vec<PaymentOption>,
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
    pub project: PublicInvoiceProject,
    pub status: InvoiceStatus,
    pub checkout_status: CheckoutStatus,
    pub payment_revision: u64,
    pub amount_paid: String,
    pub amount_due: String,
    pub amount_overpaid: String,
    /// Confirmed receipts, at most the 20 largest, largest first. Empty in test
    /// mode.
    pub transfers: Vec<PublicInvoiceTransfer>,
    pub monitoring_ends_at: Option<String>,
    pub payment_options: Vec<PaymentOption>,
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
    pub status: InvoiceStatus,
    pub checkout_status: CheckoutStatus,
    pub payment_revision: u64,
    pub amount_due: String,
    pub amount_overpaid: String,
    pub monitoring_ends_at: Option<String>,
    pub payment_options: Vec<PaymentOption>,
    pub amount_paid: String,
    pub fully_paid_at: Option<String>,
}

/// Input for creating an invoice.
///
/// These four fields are the whole request body: currency (always USD) and mode
/// (from the key) are not request fields, and the API rejects unknown body keys.
/// Optional request fields are omitted when unset. Use
/// [`CreateInvoiceInput::without_return_url`] to send `return_url: null`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreateInvoiceInput {
    pub amount: String,
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
            description: None,
            reference_id: None,
            return_url: None,
        }
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
///
/// `status` is typed to the paid-equivalent statuses alone, so an invoice that
/// is not cleared for fulfillment never decodes as an `invoice.paid` event.
/// Payment instructions and `return_url` are absent by design: reconcile by
/// invoice id plus `reference_id`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaidEventInvoice {
    pub id: String,
    pub mode: InvoiceMode,
    pub status: InvoicePaidStatus,
    pub amount: String,
    pub currency: InvoiceCurrency,
    pub amount_paid: String,
    pub reference_id: Option<String>,
    pub payment_revision: u64,
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

/// Known invoice.payment_reversed webhook invoice payload.
///
/// `status` is deliberately untyped, unlike [`InvoicePaidEventInvoice`]: a
/// status this SDK version does not model must still decode, because dropping a
/// reversal leaves an order fulfilled on a payment that no longer exists.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaymentReversedEventInvoice {
    pub id: String,
    pub mode: InvoiceMode,
    pub status: String,
    pub amount: String,
    pub currency: InvoiceCurrency,
    pub amount_paid: String,
    pub reference_id: Option<String>,
    pub payment_revision: u64,
    pub fully_paid_at: Option<String>,
}

/// Known invoice.payment_reversed webhook data payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaymentReversedEventData {
    pub invoice: InvoicePaymentReversedEventInvoice,
}

/// Known invoice.payment_reversed webhook event.
///
/// A paid invoice dropped back below its amount — a reorg removing a credited
/// transfer, say. It carries a higher `payment_revision` and
/// `fully_paid_at: None`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InvoicePaymentReversedEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub mode: InvoiceMode,
    pub created_at: String,
    pub data: InvoicePaymentReversedEventData,
}

/// Verified webhook event payload. Unknown future event types are preserved.
pub type InvoqWebhookEvent = serde_json::Value;

#[cfg(test)]
mod tests {
    use super::{
        ApiErrorLocation, ChainNamespace, CheckoutStatus, CreateInvoiceInput,
        CreateTestPaymentInput, Invoice, InvoicePaidStatus, InvoiceStatus, PaymentInstructions,
        PaymentOption, PaymentOptionCollectionMethod, PaymentOptionStatus, PublicInvoice,
        TestPaymentInvoice,
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
                .description("Test order")
                .reference_id("order_123")
                .return_url("https://merchant.test/thanks"),
        )
        .unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "amount": "149",
                "description": "Test order",
                "reference_id": "order_123",
                "return_url": "https://merchant.test/thanks"
            })
        );
    }

    /// The create schema is strict. One extra key — `currency` above all, since
    /// it is returned in responses and reads like a request field — fails the
    /// whole call with `400 invalid_request` and `fields[].code:
    /// "unknown_field"`. The body must stay exactly these four keys.
    #[test]
    fn create_invoice_input_body_is_exactly_the_four_request_fields() {
        let body = serde_json::to_string(
            &CreateInvoiceInput::new("149")
                .description("Test order")
                .reference_id("order_123")
                .return_url("https://merchant.test/thanks"),
        )
        .unwrap();

        assert_eq!(
            body,
            r#"{"amount":"149","description":"Test order","reference_id":"order_123","return_url":"https://merchant.test/thanks"}"#
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
            serde_json::to_value(CheckoutStatus::Unavailable).unwrap(),
            serde_json::json!("unavailable")
        );
        assert_eq!(
            serde_json::from_value::<CheckoutStatus>(serde_json::json!("confirming")).unwrap(),
            CheckoutStatus::Confirming
        );
        assert_eq!(
            serde_json::from_value::<InvoicePaidStatus>(serde_json::json!("settled")).unwrap(),
            InvoicePaidStatus::Settled
        );
        assert_eq!(
            serde_json::to_value(ChainNamespace::Eip155).unwrap(),
            serde_json::json!("eip155")
        );
        assert_eq!(
            serde_json::to_value(PaymentOptionCollectionMethod::EvmDeposit).unwrap(),
            serde_json::json!("evm_deposit")
        );
    }

    /// The create fixture, byte for byte: three issued options covering both
    /// collection methods.
    #[test]
    fn invoice_deserializes_the_create_wire_shape() {
        let invoice: Invoice = serde_json::from_value(serde_json::json!({
            "id": "inv_0123456789abcdefghjk",
            "mode": "live",
            "amount": "12.3400",
            "currency": "USD",
            "reference_id": "order_10086",
            "description": "Website audit for June",
            "return_url": "https://example.com/orders/order_10086",
            "status": "unpaid",
            "checkout_status": "open",
            "payment_revision": 0,
            "amount_due": "12.340000000000000000",
            "amount_overpaid": "0.000000000000000000",
            "monitoring_ends_at": "2026-07-26T10:00:00.000Z",
            "payment_options": [
                {
                    "collection_method": "evm_deposit",
                    "chain_namespace": "eip155",
                    "chain_reference": "8453",
                    "currency": "USD",
                    "token_address": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
                    "token_decimals": 6,
                    "network_label": "Base",
                    "display_symbol": "USDC",
                    "logo_url": null,
                    "chain_logo_url": null,
                    "status": "ready",
                    "deposit_address": "0x20c124f3919bb502c6126cda5bd6e5287859d5ca",
                    "suggested_amount": "12.340000"
                },
                {
                    "collection_method": "direct_exact",
                    "chain_namespace": "solana",
                    "chain_reference": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                    "currency": "USD",
                    "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "token_decimals": 6,
                    "network_label": "Solana",
                    "display_symbol": "USDC",
                    "logo_url": null,
                    "chain_logo_url": null,
                    "status": "ready",
                    "recipient_address": "GmaDrppBC7P5ARKV8g3djiwP89vz1jLK23V2GBjuAEGB",
                    "invoice_amount": "12.340000",
                    "matching_increment": "0.000123",
                    "exact_amount": "12.340123"
                },
                {
                    "collection_method": "direct_exact",
                    "chain_namespace": "tron",
                    "chain_reference": "0x2b6653dc",
                    "currency": "USD",
                    "token_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
                    "token_decimals": 6,
                    "network_label": "TRON",
                    "display_symbol": "USDT",
                    "logo_url": null,
                    "chain_logo_url": null,
                    "status": "ready",
                    "recipient_address": "TJRabPrwbZy45sbavfcjinPJC18kjpRTv8",
                    "invoice_amount": "12.340000",
                    "matching_increment": "0.009999",
                    "exact_amount": "12.349999"
                }
            ]
        }))
        .unwrap();

        assert_eq!(invoice.checkout_status, CheckoutStatus::Open);
        assert_eq!(invoice.payment_revision, 0);
        assert_eq!(invoice.payment_options.len(), 3);
        assert_eq!(invoice.payment_options[0].token_decimals, 6);
        assert_eq!(
            invoice.payment_options[0].chain_namespace,
            ChainNamespace::Eip155
        );
        assert_eq!(invoice.payment_options[0].logo_url, None);

        let PaymentOptionStatus::Ready(PaymentInstructions::EvmDeposit {
            deposit_address,
            suggested_amount,
        }) = &invoice.payment_options[0].status
        else {
            panic!("expected a ready evm_deposit option");
        };

        assert_eq!(
            deposit_address,
            "0x20c124f3919bb502c6126cda5bd6e5287859d5ca"
        );
        assert_eq!(suggested_amount, "12.340000");

        let PaymentOptionStatus::Ready(PaymentInstructions::DirectExact {
            recipient_address,
            invoice_amount,
            matching_increment,
            exact_amount,
        }) = &invoice.payment_options[2].status
        else {
            panic!("expected a ready direct_exact option");
        };

        assert_eq!(recipient_address, "TJRabPrwbZy45sbavfcjinPJC18kjpRTv8");
        assert_eq!(invoice_amount, "12.340000");
        assert_eq!(matching_increment, "0.009999");
        assert_eq!(exact_amount, "12.349999");
    }

    /// Round-tripping keeps the flattened discriminators flat: `status` and the
    /// payable fields sit beside the common ones, never nested.
    #[test]
    fn payment_options_round_trip_to_the_flat_wire_shape() {
        let unavailable = serde_json::json!({
            "collection_method": "direct_exact",
            "chain_namespace": "solana",
            "chain_reference": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            "currency": "USD",
            "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "token_decimals": 6,
            "network_label": "Solana",
            "display_symbol": "USDC",
            "logo_url": null,
            "chain_logo_url": null,
            "status": "unavailable"
        });
        let option: super::PaymentOption = serde_json::from_value(unavailable.clone()).unwrap();

        assert_eq!(option.status, PaymentOptionStatus::Unavailable);
        assert_eq!(
            option.collection_method,
            PaymentOptionCollectionMethod::DirectExact
        );
        assert_eq!(serde_json::to_value(&option).unwrap(), unavailable);

        let ready = serde_json::json!({
            "collection_method": "evm_deposit",
            "chain_namespace": "eip155",
            "chain_reference": "8453",
            "currency": "USD",
            "token_address": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
            "token_decimals": 6,
            "network_label": "Base",
            "display_symbol": "USDC",
            "logo_url": null,
            "chain_logo_url": null,
            "status": "ready",
            "deposit_address": "0x20c124f3919bb502c6126cda5bd6e5287859d5ca",
            "suggested_amount": "12.340000"
        });
        let option: super::PaymentOption = serde_json::from_value(ready.clone()).unwrap();

        assert_eq!(serde_json::to_value(&option).unwrap(), ready);
    }

    #[test]
    fn public_invoice_deserializes_the_paid_read_wire_shape() {
        let invoice: PublicInvoice = serde_json::from_value(serde_json::json!({
            "id": "inv_0123456789abcdefghjk",
            "mode": "live",
            "amount": "12.3400",
            "currency": "USD",
            "description": "Website audit for June",
            "return_url": "https://example.com/orders/order_10086",
            "project": {
                "id": "proj_0123456789abcdefghjkmnpq",
                "name": "Acme store",
                "logo_url": null
            },
            "status": "settled",
            "checkout_status": "paid",
            "payment_revision": 1,
            "amount_paid": "12.340000000000000000",
            "amount_due": "0.000000000000000000",
            "amount_overpaid": "0.000000000000000000",
            "transfers": [
                {
                    "chain_namespace": "solana",
                    "chain_reference": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                    "transaction_id": "2Ana1pUpv2ZbMVkwF5FXapYeBEjdxDatLn7nvJkhgTSXbs59SyZSx866bXirPgj8QQVB57uxHJBG1YFvkRbFj4T",
                    "event_index": 2,
                    "amount": "12.340000000000000000",
                    "explorer_transaction_url": "https://solscan.io/tx/2Ana1pUpv2ZbMVkwF5FXapYeBEjdxDatLn7nvJkhgTSXbs59SyZSx866bXirPgj8QQVB57uxHJBG1YFvkRbFj4T"
                }
            ],
            "monitoring_ends_at": "2026-07-26T10:00:00.000Z",
            "payment_options": [
                {
                    "collection_method": "direct_exact",
                    "chain_namespace": "solana",
                    "chain_reference": "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
                    "currency": "USD",
                    "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "token_decimals": 6,
                    "network_label": "Solana",
                    "display_symbol": "USDC",
                    "logo_url": null,
                    "chain_logo_url": null,
                    "status": "unavailable"
                }
            ]
        }))
        .unwrap();

        assert_eq!(invoice.status, InvoiceStatus::Settled);
        assert_eq!(invoice.checkout_status, CheckoutStatus::Paid);
        assert_eq!(invoice.payment_revision, 1);
        assert_eq!(invoice.project.name.as_deref(), Some("Acme store"));
        assert_eq!(invoice.transfers.len(), 1);
        assert_eq!(invoice.transfers[0].chain_namespace, ChainNamespace::Solana);
        assert_eq!(invoice.transfers[0].event_index, 2);
        assert_eq!(invoice.transfers[0].amount, invoice.amount_paid);
        assert!(invoice.transfers[0]
            .explorer_transaction_url
            .as_deref()
            .is_some_and(|url| url.starts_with("https://solscan.io/tx/")));
        assert_eq!(
            invoice.payment_options[0].status,
            PaymentOptionStatus::Unavailable
        );
    }

    /// Test invoices carry no payment window, no receipts, and no options.
    #[test]
    fn public_invoice_deserializes_the_test_read_wire_shape() {
        let invoice: PublicInvoice = serde_json::from_value(serde_json::json!({
            "id": "inv_9876543210abcdefghjk",
            "mode": "test",
            "amount": "12.3400",
            "currency": "USD",
            "description": null,
            "return_url": null,
            "project": {
                "id": "proj_0123456789abcdefghjkmnpq",
                "name": "Acme store",
                "logo_url": null
            },
            "status": "partially_paid",
            "checkout_status": "unavailable",
            "payment_revision": 1,
            "amount_paid": "5.000000000000000000",
            "amount_due": "7.340000000000000000",
            "amount_overpaid": "0.000000000000000000",
            "transfers": [],
            "monitoring_ends_at": null,
            "payment_options": []
        }))
        .unwrap();

        assert_eq!(invoice.description, None);
        assert_eq!(invoice.return_url, None);
        assert_eq!(invoice.checkout_status, CheckoutStatus::Unavailable);
        assert_eq!(invoice.monitoring_ends_at, None);
        assert!(invoice.transfers.is_empty());
        assert!(invoice.payment_options.is_empty());
    }

    #[test]
    fn test_payment_invoice_deserializes_the_create_shape_plus_payment_state() {
        let invoice: TestPaymentInvoice = serde_json::from_value(serde_json::json!({
            "id": "inv_9876543210abcdefghjk",
            "mode": "test",
            "amount": "149.0000",
            "currency": "USD",
            "reference_id": null,
            "description": null,
            "return_url": null,
            "status": "paid",
            "checkout_status": "paid",
            "payment_revision": 1,
            "amount_due": "0.000000000000000000",
            "amount_overpaid": "0.000000000000000000",
            "monitoring_ends_at": null,
            "payment_options": [],
            "amount_paid": "149.000000000000000000",
            "fully_paid_at": "2026-06-15T00:00:00.000Z"
        }))
        .unwrap();

        assert_eq!(invoice.reference_id, None);
        assert_eq!(invoice.status, InvoiceStatus::Paid);
        assert_eq!(invoice.amount_paid, "149.000000000000000000");
        assert_eq!(
            invoice.fully_paid_at.as_deref(),
            Some("2026-06-15T00:00:00.000Z")
        );
    }

    // The backend adds enum values without calling it breaking. Before these
    // arms existed, any one of these made the WHOLE response fail to
    // deserialize — every caller broken at once with nothing deployed.
    #[test]
    fn tolerates_enum_values_this_version_does_not_know() {
        let status: InvoiceStatus = serde_json::from_str("\"refunded\"").unwrap();
        assert_eq!(status, InvoiceStatus::Unknown("refunded".to_string()));

        let checkout: CheckoutStatus = serde_json::from_str("\"settling_out\"").unwrap();
        assert_eq!(
            checkout,
            CheckoutStatus::Unknown("settling_out".to_string())
        );

        let chain: ChainNamespace = serde_json::from_str("\"aptos\"").unwrap();
        assert_eq!(chain, ChainNamespace::Unknown("aptos".to_string()));

        let method: PaymentOptionCollectionMethod = serde_json::from_str("\"lightning\"").unwrap();
        assert_eq!(
            method,
            PaymentOptionCollectionMethod::Unknown("lightning".to_string())
        );

        let location: ApiErrorLocation = serde_json::from_str("\"trailer\"").unwrap();
        assert_eq!(location, ApiErrorLocation::Unknown("trailer".to_string()));
    }

    // An unknown value must survive a round trip, or a client that reads and
    // re-sends a snapshot would silently rewrite it.
    #[test]
    fn round_trips_unknown_enum_values_unchanged() {
        for raw in ["\"refunded\"", "\"unpaid\""] {
            let status: InvoiceStatus = serde_json::from_str(raw).unwrap();
            assert_eq!(serde_json::to_string(&status).unwrap(), raw);
        }
    }

    // The one enum that cannot carry its raw value: it is the internally
    // tagged discriminator. Unknown must read as not payable.
    #[test]
    fn treats_an_unknown_payment_option_status_as_not_payable() {
        let option: PaymentOption = serde_json::from_value(serde_json::json!({
            "collection_method": "evm_deposit",
            "chain_namespace": "eip155",
            "chain_reference": "8453",
            "currency": "USD",
            "token_address": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
            "token_decimals": 6,
            "network_label": "Base",
            "display_symbol": "USDC",
            "logo_url": null,
            "chain_logo_url": null,
            "status": "some_future_state"
        }))
        .unwrap();

        assert_eq!(option.status, PaymentOptionStatus::Unknown);
        assert!(matches!(option.status, PaymentOptionStatus::Unknown));
    }

    // Known values must be completely unaffected by the catch-all arms.
    #[test]
    fn keeps_known_enum_values_exact() {
        let status: InvoiceStatus = serde_json::from_str("\"review_required\"").unwrap();
        assert_eq!(status, InvoiceStatus::ReviewRequired);
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            "\"review_required\""
        );

        let chain: ChainNamespace = serde_json::from_str("\"eip155\"").unwrap();
        assert_eq!(chain, ChainNamespace::Eip155);
        assert_eq!(serde_json::to_string(&chain).unwrap(), "\"eip155\"");
    }
}
