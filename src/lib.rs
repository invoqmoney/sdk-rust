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
    ApiErrorField, ApiErrorLocation, CreateInvoiceInput, CreateTestPaymentInput, DirectOnchainRail,
    Invoice, InvoiceCurrency, InvoiceMode, InvoicePaidEvent, InvoicePaidEventData,
    InvoicePaidEventInvoice, InvoicePaidStatus, InvoicePaymentStatus, InvoiceStatus,
    InvoqWebhookEvent, MonitoringStatus, PublicInvoice, PublicInvoiceProject,
    PublicInvoiceTransfer, TestPaymentInvoice,
};
pub use webhooks::{invoice_paid_event, is_invoice_paid, verify_webhook, WebhookHeaders};

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
