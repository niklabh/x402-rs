//! Client-side functionality for the x402 protocol.
//!
//! This module provides functions for making HTTP requests that handle 402 Payment Required
//! responses, generate payment payloads, and retry requests with payment.

use crate::errors::{Result, X402Error};
use crate::schemes::{exact_evm::ExactEvm, Scheme};
use crate::types::{PaymentPayload, PaymentRequiredResponse};
use crate::utils::{decode_payment_header, encode_payment_header};
use reqwest::{Client, Method, Response, StatusCode};
use serde_json::Value;
use std::sync::Arc;

/// Configuration for x402 client requests.
#[derive(Clone)]
pub struct X402ClientConfig {
    /// Private key of the payer (for signing authorizations)
    pub private_key: String,
    
    /// RPC URL for blockchain interactions
    pub rpc_url: String,
    
    /// HTTP client to use for requests
    pub http_client: Client,
    
    /// Preferred payment scheme (e.g., "exact")
    pub preferred_scheme: Option<String>,
    
    /// Preferred network (e.g., "8453" for Base mainnet)
    pub preferred_network: Option<String>,
}

impl X402ClientConfig {
    /// Creates a new client configuration.
    ///
    /// # Arguments
    ///
    /// * `private_key` - The payer's private key (with or without 0x prefix)
    /// * `rpc_url` - RPC endpoint URL
    ///
    /// # Examples
    ///
    /// ```
    /// use x402_rs::client::X402ClientConfig;
    ///
    /// let config = X402ClientConfig::new(
    ///     "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    ///     "https://mainnet.base.org"
    /// );
    /// ```
    pub fn new(private_key: impl Into<String>, rpc_url: impl Into<String>) -> Self {
        Self {
            private_key: private_key.into(),
            rpc_url: rpc_url.into(),
            http_client: Client::new(),
            preferred_scheme: Some("exact".to_string()),
            preferred_network: None,
        }
    }

    /// Sets the preferred payment scheme.
    pub fn with_scheme(mut self, scheme: impl Into<String>) -> Self {
        self.preferred_scheme = Some(scheme.into());
        self
    }

    /// Sets the preferred network.
    pub fn with_network(mut self, network: impl Into<String>) -> Self {
        self.preferred_network = Some(network.into());
        self
    }

    /// Sets a custom HTTP client.
    pub fn with_client(mut self, client: Client) -> Self {
        self.http_client = client;
        self
    }
}

/// Makes an HTTP request with automatic x402 payment handling.
///
/// If the server responds with 402 Payment Required, this function will:
/// 1. Parse the payment requirements
/// 2. Generate a payment payload
/// 3. Retry the request with the X-PAYMENT header
///
/// # Arguments
///
/// * `config` - Client configuration with keys and preferences
/// * `method` - HTTP method (GET, POST, etc.)
/// * `url` - Target URL
/// * `body` - Optional request body (for POST, PUT, etc.)
///
/// # Examples
///
/// ```no_run
/// use x402_rs::client::{X402ClientConfig, request_with_payment};
/// use reqwest::Method;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = X402ClientConfig::new(
///     "0xprivatekey",
///     "https://mainnet.base.org"
/// );
///
/// let response = request_with_payment(
///     &config,
///     Method::GET,
///     "https://api.example.com/weather",
///     None,
/// ).await?;
///
/// println!("Response: {}", response.text().await?);
/// # Ok(())
/// # }
/// ```
pub async fn request_with_payment(
    config: &X402ClientConfig,
    method: Method,
    url: &str,
    body: Option<Value>,
) -> Result<Response> {
    // Build initial request
    let mut request = config.http_client.request(method.clone(), url);

    if let Some(body) = &body {
        request = request.json(body);
    }

    // Send initial request
    let response = request.send().await?;

    // Check if payment is required
    if response.status() == StatusCode::PAYMENT_REQUIRED {
        // Parse 402 response
        let payment_info: PaymentRequiredResponse = response.json().await?;

        // Select a suitable payment requirement
        let requirement = select_requirement(&payment_info, config)?;

        // Generate payment payload
        let payload = generate_payment_payload(requirement, config).await?;

        // Encode payload as Base64
        let payment_header = encode_payment_header(&payload)?;

        // Retry request with payment header
        let mut retry_request = config.http_client.request(method, url);
        retry_request = retry_request.header("X-PAYMENT", payment_header);

        if let Some(body) = body {
            retry_request = retry_request.json(&body);
        }

        let retry_response = retry_request.send().await?;

        // Check for payment response header
        if let Some(payment_response) = retry_response.headers().get("X-PAYMENT-RESPONSE") {
            if let Ok(encoded) = payment_response.to_str() {
                if let Ok(_decoded) = decode_payment_header(encoded) {
                    // Payment response received
                    #[cfg(feature = "tracing")]
                    tracing::debug!("Payment response: {:?}", _decoded);
                }
            }
        }

        Ok(retry_response)
    } else {
        // No payment required, return original response
        Ok(response)
    }
}

/// Selects an appropriate payment requirement from the server's offers.
fn select_requirement<'a>(
    response: &'a PaymentRequiredResponse,
    config: &X402ClientConfig,
) -> Result<&'a crate::types::PaymentRequirements> {
    // Filter by preferred scheme and network if specified
    let mut candidates: Vec<_> = response.accepts.iter().collect();

    if let Some(scheme) = &config.preferred_scheme {
        candidates.retain(|r| &r.scheme == scheme);
    }

    if let Some(network) = &config.preferred_network {
        candidates.retain(|r| &r.network == network);
    }

    // Return first matching requirement
    candidates
        .first()
        .copied()
        .ok_or(X402Error::NoSuitableRequirement)
}

/// Generates a payment payload for the selected requirement.
async fn generate_payment_payload(
    requirement: &crate::types::PaymentRequirements,
    config: &X402ClientConfig,
) -> Result<PaymentPayload> {
    // Match the scheme and generate appropriate payload
    let scheme: Arc<dyn Scheme> = match requirement.scheme.as_str() {
        "exact" => Arc::new(ExactEvm::new()),
        _ => return Err(X402Error::UnsupportedScheme(requirement.scheme.clone())),
    };

    scheme
        .generate_payload(requirement, &config.private_key, &config.rpc_url)
        .await
}

/// A simpler convenience function for GET requests.
///
/// # Examples
///
/// ```no_run
/// use x402_rs::client::{X402ClientConfig, get};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = X402ClientConfig::new(
///     "0xprivatekey",
///     "https://mainnet.base.org"
/// );
///
/// let response = get(&config, "https://api.example.com/data").await?;
/// println!("{}", response.text().await?);
/// # Ok(())
/// # }
/// ```
pub async fn get(config: &X402ClientConfig, url: &str) -> Result<Response> {
    request_with_payment(config, Method::GET, url, None).await
}

/// A simpler convenience function for POST requests.
///
/// # Examples
///
/// ```no_run
/// use x402_rs::client::{X402ClientConfig, post};
/// use serde_json::json;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = X402ClientConfig::new(
///     "0xprivatekey",
///     "https://mainnet.base.org"
/// );
///
/// let body = json!({"query": "temperature"});
/// let response = post(&config, "https://api.example.com/query", body).await?;
/// # Ok(())
/// # }
/// ```
pub async fn post(config: &X402ClientConfig, url: &str, body: Value) -> Result<Response> {
    request_with_payment(config, Method::POST, url, Some(body)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PaymentRequirements;

    #[test]
    fn test_client_config_creation() {
        let config = X402ClientConfig::new("0xkey", "https://rpc.url");
        assert_eq!(config.private_key, "0xkey");
        assert_eq!(config.rpc_url, "https://rpc.url");
        assert_eq!(config.preferred_scheme, Some("exact".to_string()));
    }

    #[test]
    fn test_config_builders() {
        let config = X402ClientConfig::new("0xkey", "https://rpc.url")
            .with_scheme("upto")
            .with_network("8453");

        assert_eq!(config.preferred_scheme, Some("upto".to_string()));
        assert_eq!(config.preferred_network, Some("8453".to_string()));
    }

    #[test]
    fn test_select_requirement() {
        let response = PaymentRequiredResponse {
            x402_version: 1,
            accepts: vec![
                PaymentRequirements {
                    scheme: "exact".to_string(),
                    network: "8453".to_string(),
                    max_amount_required: "10000".to_string(),
                    resource: "/api/test".to_string(),
                    description: None,
                    mime_type: None,
                    output_schema: None,
                    pay_to: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
                    max_timeout_seconds: 300,
                    asset: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
                    extra: None,
                },
            ],
            error: None,
        };

        let config = X402ClientConfig::new("0xkey", "https://rpc.url");
        let requirement = select_requirement(&response, &config).unwrap();
        assert_eq!(requirement.scheme, "exact");
    }
}

