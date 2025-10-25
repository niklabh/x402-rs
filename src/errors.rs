//! Error types for the x402-rs library.
//!
//! This module defines all error types that can occur during x402 protocol operations.

use thiserror::Error;

/// Main error type for x402 operations.
#[derive(Error, Debug)]
pub enum X402Error {
    /// Error during HTTP request/response handling
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Error during JSON serialization/deserialization
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error during Base64 encoding/decoding
    #[error("Base64 error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    /// Error during blockchain operations
    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    /// Invalid payment payload
    #[error("Invalid payment payload: {0}")]
    InvalidPayload(String),

    /// Payment verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Payment settlement failed
    #[error("Settlement failed: {0}")]
    SettlementError(String),

    /// Unsupported payment scheme
    #[error("Unsupported scheme: {0}")]
    UnsupportedScheme(String),

    /// Unsupported network
    #[error("Unsupported network: {0}")]
    UnsupportedNetwork(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid amount
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    /// Timeout exceeded
    #[error("Timeout exceeded")]
    TimeoutExceeded,

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureError(String),

    /// Nonce already used (replay attack prevention)
    #[error("Nonce already used: {0}")]
    NonceUsed(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// No suitable payment requirement found
    #[error("No suitable payment requirement found")]
    NoSuitableRequirement,

    /// The response was not a 402 Payment Required
    #[error("Expected 402 Payment Required, got status: {0}")]
    Not402Response(u16),

    /// Error parsing URL
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Generic error with custom message
    #[error("{0}")]
    Other(String),
}

/// Result type alias for x402 operations.
pub type Result<T> = std::result::Result<T, X402Error>;

impl From<ethers::core::types::SignatureError> for X402Error {
    fn from(err: ethers::core::types::SignatureError) -> Self {
        X402Error::SignatureError(err.to_string())
    }
}

impl From<ethers::providers::ProviderError> for X402Error {
    fn from(err: ethers::providers::ProviderError) -> Self {
        X402Error::BlockchainError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = X402Error::InvalidPayload("test error".to_string());
        assert_eq!(err.to_string(), "Invalid payment payload: test error");
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let x402_err: X402Error = json_err.into();
        assert!(matches!(x402_err, X402Error::JsonError(_)));
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }
        
        assert_eq!(returns_result().unwrap(), 42);
    }
}

