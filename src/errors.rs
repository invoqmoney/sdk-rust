use crate::types::ApiErrorField;
use serde_json::Value;
use thiserror::Error;

/// Result type returned by invoq SDK operations.
pub type Result<T> = std::result::Result<T, InvoqError>;

/// Top-level SDK error.
#[derive(Debug, Error)]
pub enum InvoqError {
    #[error("{0}")]
    Configuration(String),
    #[error("{0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Api(Box<InvoqApiError>),
    #[error(transparent)]
    SignatureVerification(#[from] InvoqSignatureVerificationError),
    #[error("Failed to connect to invoq API.")]
    Connect(#[source] reqwest::Error),
    #[error("invoq API request timed out.")]
    Timeout(#[source] reqwest::Error),
    #[error("Failed to read invoq API response.")]
    ReadResponse(#[source] reqwest::Error),
    #[error("Failed to parse invoq API response.")]
    ParseResponse(#[source] serde_json::Error),
    #[error("invoq API response did not include a data envelope.")]
    MissingDataEnvelope { payload: Value },
}

impl InvoqError {
    pub(crate) fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest(message.into())
    }
}

impl From<InvoqApiError> for InvoqError {
    fn from(error: InvoqApiError) -> Self {
        Self::Api(Box::new(error))
    }
}

/// Error returned by the invoq API.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct InvoqApiError {
    pub message: String,
    pub status: u16,
    pub code: Option<String>,
    pub fields: Option<Vec<ApiErrorField>>,
    pub meta: Option<Value>,
    pub payload: ApiErrorPayload,
}

/// Raw API error response payload.
#[derive(Clone, Debug, PartialEq)]
pub enum ApiErrorPayload {
    Json(Value),
    Text(String),
}

/// Webhook signature verification error code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureVerificationErrorCode {
    MissingSignature,
    InvalidSignatureHeader,
    TimestampOutsideTolerance,
    SignatureMismatch,
    InvalidPayload,
}

impl SignatureVerificationErrorCode {
    /// Stable snake_case code matching invoq API and Node.js SDK errors.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingSignature => "missing_signature",
            Self::InvalidSignatureHeader => "invalid_signature_header",
            Self::TimestampOutsideTolerance => "timestamp_outside_tolerance",
            Self::SignatureMismatch => "signature_mismatch",
            Self::InvalidPayload => "invalid_payload",
        }
    }
}

/// Error returned when webhook verification fails.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct InvoqSignatureVerificationError {
    pub code: SignatureVerificationErrorCode,
    pub message: String,
}

impl InvoqSignatureVerificationError {
    pub(crate) fn new(code: SignatureVerificationErrorCode, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
}
