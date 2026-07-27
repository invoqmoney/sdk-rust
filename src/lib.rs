#![doc = include_str!("../README.md")]

mod client;
mod errors;
mod request;
mod types;
mod webhooks;

pub use client::{Invoices, Invoq, InvoqOptions};
pub use errors::{
    ApiErrorPayload, InvoqApiError, InvoqError, InvoqSignatureVerificationError, Result,
    SignatureVerificationErrorCode,
};
pub use types::{
    ApiErrorField, ApiErrorLocation, ChainNamespace, CheckoutStatus, CreateInvoiceInput,
    CreateTestPaymentInput, Invoice, InvoiceCurrency, InvoiceMode, InvoicePaidEvent,
    InvoicePaidEventData, InvoicePaidEventInvoice, InvoicePaidStatus, InvoicePaymentReversedEvent,
    InvoicePaymentReversedEventData, InvoicePaymentReversedEventInvoice, InvoiceStatus,
    InvoqWebhookEvent, PaymentInstructions, PaymentOption, PaymentOptionCollectionMethod,
    PaymentOptionStatus, PublicInvoice, PublicInvoiceProject, PublicInvoiceTransfer,
    TestPaymentInvoice,
};
pub use webhooks::{
    invoice_paid_event, invoice_payment_reversed_event, is_invoice_paid,
    is_invoice_payment_reversed, verify_webhook, WebhookHeaders,
};

/// Current crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn exposes_package_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
