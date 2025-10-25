//! Facilitator service implementation for the x402 protocol.
//!
//! A facilitator is an optional intermediary service that verifies payment payloads
//! and settles transactions on-chain. This module provides the server endpoints
//! needed to run a facilitator service.

use crate::errors::{Result, X402Error};
use crate::schemes::{exact_evm::ExactEvm, Scheme};
use crate::types::{
    SettlementRequest, SettlementResponse, SupportedKind, SupportedResponse, VerificationRequest,
    VerificationResponse,
};
use std::collections::HashSet;
use std::sync::Arc;

/// Configuration for a facilitator service.
#[derive(Clone)]
pub struct FacilitatorConfig {
    /// Private key for the facilitator (to pay gas for settlements)
    pub private_key: String,
    
    /// RPC URL for blockchain interactions
    pub rpc_url: String,
    
    /// List of supported (scheme, network) combinations
    pub supported: Vec<(String, String)>,
    
    /// Set of used nonces to prevent replay attacks
    pub used_nonces: Arc<tokio::sync::RwLock<HashSet<String>>>,
}

impl FacilitatorConfig {
    /// Creates a new facilitator configuration.
    ///
    /// # Arguments
    ///
    /// * `private_key` - Facilitator's private key (for paying gas)
    /// * `rpc_url` - RPC endpoint URL
    ///
    /// # Examples
    ///
    /// ```
    /// use x402_rs::facilitator::FacilitatorConfig;
    ///
    /// let config = FacilitatorConfig::new(
    ///     "0xfacilitator_private_key",
    ///     "https://mainnet.base.org"
    /// );
    /// ```
    pub fn new(private_key: impl Into<String>, rpc_url: impl Into<String>) -> Self {
        Self {
            private_key: private_key.into(),
            rpc_url: rpc_url.into(),
            supported: vec![("exact".to_string(), "8453".to_string())],
            used_nonces: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        }
    }

    /// Adds a supported (scheme, network) combination.
    pub fn add_supported(&mut self, scheme: impl Into<String>, network: impl Into<String>) {
        self.supported.push((scheme.into(), network.into()));
    }

    /// Checks if a (scheme, network) combination is supported.
    pub fn is_supported(&self, scheme: &str, network: &str) -> bool {
        self.supported.iter().any(|(s, n)| s == scheme && n == network)
    }
}

/// Handles the `/verify` endpoint.
///
/// Verifies a payment payload without executing it on-chain.
///
/// # Arguments
///
/// * `request` - Verification request with payment header and requirements
/// * `config` - Facilitator configuration
///
/// # Returns
///
/// `VerificationResponse` indicating if the payment is valid
pub async fn handle_verify(
    request: VerificationRequest,
    config: &FacilitatorConfig,
) -> Result<VerificationResponse> {
    // Decode payment header
    let payload = match crate::utils::decode_payment_header(&request.payment_header) {
        Ok(p) => p,
        Err(e) => {
            return Ok(VerificationResponse {
                is_valid: false,
                invalid_reason: Some(format!("Invalid payment header: {}", e)),
            });
        }
    };

    // Check if scheme/network is supported
    if !config.is_supported(&payload.scheme, &payload.network) {
        return Ok(VerificationResponse {
            is_valid: false,
            invalid_reason: Some(format!(
                "Unsupported scheme/network: {}/{}",
                payload.scheme, payload.network
            )),
        });
    }

    // Get the appropriate scheme implementation
    let scheme: Arc<dyn Scheme> = match payload.scheme.as_str() {
        "exact" => Arc::new(ExactEvm::new()),
        _ => {
            return Ok(VerificationResponse {
                is_valid: false,
                invalid_reason: Some(format!("Unsupported scheme: {}", payload.scheme)),
            });
        }
    };

    // Verify the payload
    match scheme
        .verify(&payload, &request.payment_requirements, &config.rpc_url)
        .await
    {
        Ok(true) => {
            // Extract and check nonce to prevent replay
            if let Ok(auth) = serde_json::from_value::<crate::types::TransferAuthorization>(
                payload.payload.clone(),
            ) {
                let mut nonces = config.used_nonces.write().await;
                if nonces.contains(&auth.nonce) {
                    return Ok(VerificationResponse {
                        is_valid: false,
                        invalid_reason: Some("Nonce already used".to_string()),
                    });
                }
            }

            Ok(VerificationResponse {
                is_valid: true,
                invalid_reason: None,
            })
        }
        Ok(false) => Ok(VerificationResponse {
            is_valid: false,
            invalid_reason: Some("Verification failed".to_string()),
        }),
        Err(e) => Ok(VerificationResponse {
            is_valid: false,
            invalid_reason: Some(e.to_string()),
        }),
    }
}

/// Handles the `/settle` endpoint.
///
/// Verifies and executes a payment on-chain.
///
/// # Arguments
///
/// * `request` - Settlement request with payment header and requirements
/// * `config` - Facilitator configuration
///
/// # Returns
///
/// `SettlementResponse` with transaction hash if successful
pub async fn handle_settle(
    request: SettlementRequest,
    config: &FacilitatorConfig,
) -> Result<SettlementResponse> {
    // First verify the payment
    let verify_request = VerificationRequest {
        payment_header: request.payment_header.clone(),
        payment_requirements: request.payment_requirements.clone(),
    };

    let verification = handle_verify(verify_request, config).await?;

    if !verification.is_valid {
        return Ok(SettlementResponse {
            tx_hash: String::new(),
            block_number: None,
            error: verification.invalid_reason,
        });
    }

    // Decode payload
    let payload = crate::utils::decode_payment_header(&request.payment_header)?;

    // Get the scheme implementation
    let scheme: Arc<dyn Scheme> = match payload.scheme.as_str() {
        "exact" => Arc::new(ExactEvm::new()),
        _ => {
            return Ok(SettlementResponse {
                tx_hash: String::new(),
                block_number: None,
                error: Some(format!("Unsupported scheme: {}", payload.scheme)),
            });
        }
    };

    // Mark nonce as used
    if let Ok(auth) =
        serde_json::from_value::<crate::types::TransferAuthorization>(payload.payload.clone())
    {
        let mut nonces = config.used_nonces.write().await;
        nonces.insert(auth.nonce.clone());
    }

    // Settle the payment
    match scheme
        .settle(
            &payload,
            &request.payment_requirements,
            &config.rpc_url,
            &config.private_key,
        )
        .await
    {
        Ok(tx_hash) => Ok(SettlementResponse {
            tx_hash,
            block_number: None,
            error: None,
        }),
        Err(e) => Ok(SettlementResponse {
            tx_hash: String::new(),
            block_number: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Handles the `/supported` endpoint.
///
/// Returns the list of supported (scheme, network) combinations.
///
/// # Arguments
///
/// * `config` - Facilitator configuration
///
/// # Returns
///
/// `SupportedResponse` with the list of supported payment kinds
pub async fn handle_supported(config: &FacilitatorConfig) -> Result<SupportedResponse> {
    let supported = config
        .supported
        .iter()
        .map(|(scheme, network)| SupportedKind {
            scheme: scheme.clone(),
            network: network.clone(),
            assets: None, // Can be extended to list specific assets
        })
        .collect();

    Ok(SupportedResponse { supported })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facilitator_config() {
        let config = FacilitatorConfig::new("0xkey", "https://rpc.url");
        assert_eq!(config.private_key, "0xkey");
        assert_eq!(config.rpc_url, "https://rpc.url");
        assert!(config.is_supported("exact", "8453"));
        assert!(!config.is_supported("upto", "8453"));
    }

    #[test]
    fn test_add_supported() {
        let mut config = FacilitatorConfig::new("0xkey", "https://rpc.url");
        config.add_supported("upto", "137"); // Polygon
        assert!(config.is_supported("upto", "137"));
    }

    #[tokio::test]
    async fn test_handle_supported() {
        let mut config = FacilitatorConfig::new("0xkey", "https://rpc.url");
        config.add_supported("upto", "137");

        let response = handle_supported(&config).await.unwrap();
        assert_eq!(response.supported.len(), 2);
    }
}

