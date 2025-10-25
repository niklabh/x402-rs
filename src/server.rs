//! Server-side functionality for the x402 protocol.
//!
//! This module provides middleware and helpers for integrating x402 payment requirements
//! into web servers, particularly with the Axum framework.

use crate::errors::{Result, X402Error};
use crate::types::{PaymentRequiredResponse, PaymentRequirements, SettlementRequest, VerificationRequest};
use crate::utils::{decode_payment_header, dollar_to_token_amount};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

/// Configuration for payment requirements on a server endpoint.
#[derive(Clone, Debug)]
pub struct PaymentConfig {
    /// Address to receive payments
    pub pay_to: String,
    
    /// Token contract address (e.g., USDC)
    pub asset: String,
    
    /// Token decimals (e.g., 6 for USDC)
    pub decimals: u8,
    
    /// Network identifier (e.g., "8453" for Base mainnet)
    pub network: String,
    
    /// Payment scheme (e.g., "exact")
    pub scheme: String,
    
    /// Price in USD
    pub price_usd: f64,
    
    /// Description of what the payment is for
    pub description: String,
    
    /// Facilitator URL for verification and settlement
    pub facilitator_url: String,
    
    /// Maximum timeout in seconds for payment validity
    pub max_timeout_seconds: u64,
    
    /// Token name and version for EIP-712 (optional)
    pub token_name: Option<String>,
    pub token_version: Option<String>,
}

impl PaymentConfig {
    /// Creates a new payment configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use x402_rs::server::PaymentConfig;
    ///
    /// let config = PaymentConfig::new(
    ///     "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    ///     "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
    ///     6,
    ///     "8453", // Base mainnet
    ///     "exact",
    ///     0.01, // $0.01
    ///     "Weather API access",
    ///     "https://facilitator.example.com",
    /// );
    /// ```
    pub fn new(
        pay_to: impl Into<String>,
        asset: impl Into<String>,
        decimals: u8,
        network: impl Into<String>,
        scheme: impl Into<String>,
        price_usd: f64,
        description: impl Into<String>,
        facilitator_url: impl Into<String>,
    ) -> Self {
        Self {
            pay_to: pay_to.into(),
            asset: asset.into(),
            decimals,
            network: network.into(),
            scheme: scheme.into(),
            price_usd,
            description: description.into(),
            facilitator_url: facilitator_url.into(),
            max_timeout_seconds: 300,
            token_name: None,
            token_version: None,
        }
    }

    /// Sets the timeout for payment validity.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.max_timeout_seconds = seconds;
        self
    }

    /// Sets token metadata for EIP-712.
    pub fn with_token_metadata(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.token_name = Some(name.into());
        self.token_version = Some(version.into());
        self
    }

    /// Converts the configuration to payment requirements.
    pub fn to_requirements(&self, resource: &str) -> Result<PaymentRequirements> {
        let amount_str = dollar_to_token_amount(self.price_usd, self.decimals, 1.0)?;

        let mut extra = json!({});
        if let Some(name) = &self.token_name {
            extra["name"] = json!(name);
        }
        if let Some(version) = &self.token_version {
            extra["version"] = json!(version);
        }

        Ok(PaymentRequirements {
            scheme: self.scheme.clone(),
            network: self.network.clone(),
            max_amount_required: amount_str,
            resource: resource.to_string(),
            description: Some(self.description.clone()),
            mime_type: Some("application/json".to_string()),
            output_schema: None,
            pay_to: self.pay_to.clone(),
            max_timeout_seconds: self.max_timeout_seconds,
            asset: self.asset.clone(),
            extra: if extra.as_object().unwrap().is_empty() {
                None
            } else {
                Some(extra)
            },
        })
    }
}

/// Checks if a request has a valid payment header.
///
/// # Arguments
///
/// * `payment_header` - The X-PAYMENT header value (Base64 encoded)
/// * `config` - Payment configuration
/// * `resource` - The requested resource path
///
/// # Returns
///
/// `Ok(tx_hash)` if payment is valid and settled, `Err` otherwise
pub async fn verify_and_settle_payment(
    payment_header: &str,
    config: &PaymentConfig,
    resource: &str,
) -> Result<String> {
    let requirements = config.to_requirements(resource)?;

    // Verify payment with facilitator
    let client = Client::new();
    let verify_request = VerificationRequest {
        payment_header: payment_header.to_string(),
        payment_requirements: requirements.clone(),
    };

    let verify_url = format!("{}/verify", config.facilitator_url);
    let verify_response = client
        .post(&verify_url)
        .json(&verify_request)
        .send()
        .await?;

    if !verify_response.status().is_success() {
        return Err(X402Error::VerificationFailed(
            "Facilitator verification failed".to_string(),
        ));
    }

    let verification: crate::types::VerificationResponse = verify_response.json().await?;

    if !verification.is_valid {
        return Err(X402Error::VerificationFailed(
            verification
                .invalid_reason
                .unwrap_or_else(|| "Unknown reason".to_string()),
        ));
    }

    // Settle payment with facilitator
    let settle_request = SettlementRequest {
        payment_header: payment_header.to_string(),
        payment_requirements: requirements,
    };

    let settle_url = format!("{}/settle", config.facilitator_url);
    let settle_response = client
        .post(&settle_url)
        .json(&settle_request)
        .send()
        .await?;

    if !settle_response.status().is_success() {
        return Err(X402Error::SettlementError(
            "Facilitator settlement failed".to_string(),
        ));
    }

    let settlement: crate::types::SettlementResponse = settle_response.json().await?;

    if let Some(error) = settlement.error {
        return Err(X402Error::SettlementError(error));
    }

    Ok(settlement.tx_hash)
}

/// Creates a 402 Payment Required response.
///
/// # Arguments
///
/// * `configs` - Map of payment configurations (can support multiple payment options)
/// * `resource` - The requested resource path
///
/// # Examples
///
/// ```
/// use x402_rs::server::{PaymentConfig, create_payment_required_response};
/// use std::collections::HashMap;
///
/// let mut configs = HashMap::new();
/// configs.insert("usdc".to_string(), PaymentConfig::new(
///     "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbb",
///     "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
///     6,
///     "8453",
///     "exact",
///     0.01,
///     "API access",
///     "https://facilitator.example.com",
/// ));
///
/// let response = create_payment_required_response(&configs, "/api/weather").unwrap();
/// assert_eq!(response.accepts.len(), 1);
/// ```
pub fn create_payment_required_response(
    configs: &HashMap<String, PaymentConfig>,
    resource: &str,
) -> Result<PaymentRequiredResponse> {
    let accepts: Result<Vec<_>> = configs
        .values()
        .map(|config| config.to_requirements(resource))
        .collect();

    Ok(PaymentRequiredResponse {
        x402_version: 1,
        accepts: accepts?,
        error: None,
    })
}

/// Helper to create a simple single-payment configuration.
///
/// # Examples
///
/// ```
/// use x402_rs::server::create_simple_config;
///
/// let config = create_simple_config(
///     "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
///     0.01,
///     "API access",
///     "https://facilitator.example.com",
/// );
/// ```
pub fn create_simple_config(
    pay_to: &str,
    price_usd: f64,
    description: &str,
    facilitator_url: &str,
) -> PaymentConfig {
    // Default to Base mainnet USDC
    PaymentConfig::new(
        pay_to,
        "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
        6,
        "8453", // Base mainnet
        "exact",
        price_usd,
        description,
        facilitator_url,
    )
    .with_token_metadata("USD Coin", "2")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_config_creation() {
        let config = PaymentConfig::new(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            6,
            "8453",
            "exact",
            0.01,
            "Test payment",
            "https://facilitator.test",
        );

        assert_eq!(config.price_usd, 0.01);
        assert_eq!(config.decimals, 6);
    }

    #[test]
    fn test_to_requirements() {
        let config = PaymentConfig::new(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            6,
            "8453",
            "exact",
            0.01,
            "Test payment",
            "https://facilitator.test",
        );

        let requirements = config.to_requirements("/api/test").unwrap();
        assert_eq!(requirements.scheme, "exact");
        assert_eq!(requirements.max_amount_required, "10000"); // $0.01 in USDC (6 decimals)
    }

    #[test]
    fn test_create_payment_required_response() {
        let mut configs = HashMap::new();
        configs.insert(
            "usdc".to_string(),
            PaymentConfig::new(
                "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
                "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                6,
                "8453",
                "exact",
                0.01,
                "Test",
                "https://facilitator.test",
            ),
        );

        let response = create_payment_required_response(&configs, "/test").unwrap();
        assert_eq!(response.x402_version, 1);
        assert_eq!(response.accepts.len(), 1);
    }
}

