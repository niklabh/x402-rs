//! Core type definitions for the x402 protocol.
//!
//! This module contains all the data structures used in the x402 protocol,
//! including payment requirements, payloads, verification, and settlement types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Version of the x402 protocol.
pub const X402_VERSION: u32 = 1;

/// Response returned by a server when payment is required (HTTP 402).
///
/// Contains the list of accepted payment requirements that the client can choose from.
///
/// # Examples
///
/// ```
/// use x402_rs::types::PaymentRequiredResponse;
///
/// let response = PaymentRequiredResponse {
///     x402_version: 1,
///     accepts: vec![],
///     error: None,
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentRequiredResponse {
    /// Protocol version (currently 1)
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    
    /// List of accepted payment requirements
    pub accepts: Vec<PaymentRequirements>,
    
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Describes the payment requirements for a specific resource.
///
/// Each requirement specifies the payment scheme, network, amount, recipient address,
/// and other metadata necessary for the payment.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentRequirements {
    /// Payment scheme (e.g., "exact", "upto")
    pub scheme: String,
    
    /// Network identifier (e.g., "base", "8453" for Base mainnet, "84532" for Base Sepolia)
    pub network: String,
    
    /// Maximum amount required in the smallest unit (e.g., wei for ETH, smallest token unit)
    /// Represented as a string to handle uint256
    #[serde(rename = "maxAmountRequired")]
    pub max_amount_required: String,
    
    /// The resource URL or identifier
    pub resource: String,
    
    /// Human-readable description of what the payment is for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// MIME type of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    
    /// JSON schema describing the output format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    
    /// Recipient address (EVM address for EVM chains)
    #[serde(rename = "payTo")]
    pub pay_to: String,
    
    /// Maximum time in seconds that the payment is valid
    #[serde(rename = "maxTimeoutSeconds")]
    pub max_timeout_seconds: u64,
    
    /// Token contract address (e.g., USDC contract address)
    pub asset: String,
    
    /// Scheme-specific extra data (e.g., {"name": "USDC", "version": "2"} for EIP-3009)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

/// Payment payload sent by the client in the X-PAYMENT header.
///
/// This contains the scheme-specific payment data, encoded as Base64 JSON.
///
/// # Examples
///
/// ```
/// use x402_rs::types::PaymentPayload;
/// use serde_json::json;
///
/// let payload = PaymentPayload {
///     x402_version: 1,
///     scheme: "exact".to_string(),
///     network: "8453".to_string(),
///     payload: json!({"from": "0x...", "to": "0x..."}),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentPayload {
    /// Protocol version
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    
    /// Payment scheme used
    pub scheme: String,
    
    /// Network identifier
    pub network: String,
    
    /// Scheme-specific payload data
    pub payload: Value,
}

/// EIP-3009 transferWithAuthorization parameters for the "exact" scheme on EVM.
///
/// This struct represents the authorization data needed to execute a gasless ERC-20 transfer.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferAuthorization {
    /// Address of the payer (token holder)
    pub from: String,
    
    /// Address of the payee
    pub to: String,
    
    /// Amount to transfer (uint256 as string)
    pub value: String,
    
    /// Timestamp after which the authorization is valid
    #[serde(rename = "validAfter")]
    pub valid_after: String,
    
    /// Timestamp before which the authorization is valid
    #[serde(rename = "validBefore")]
    pub valid_before: String,
    
    /// Unique nonce for replay protection (32 bytes as hex string)
    pub nonce: String,
    
    /// EIP-712 signature (v, r, s concatenated as hex string)
    pub signature: String,
}

/// Request to verify a payment without settling it on-chain.
///
/// Sent from the server to a facilitator's `/verify` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerificationRequest {
    /// The X-PAYMENT header value (Base64 encoded PaymentPayload)
    #[serde(rename = "paymentHeader")]
    pub payment_header: String,
    
    /// The payment requirements that the server expects
    #[serde(rename = "paymentRequirements")]
    pub payment_requirements: PaymentRequirements,
}

/// Response from the facilitator's `/verify` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerificationResponse {
    /// Whether the payment payload is valid
    #[serde(rename = "isValid")]
    pub is_valid: bool,
    
    /// Optional reason if invalid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}

/// Request to settle a payment on-chain.
///
/// Sent from the server to a facilitator's `/settle` endpoint after verification.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettlementRequest {
    /// The X-PAYMENT header value (Base64 encoded PaymentPayload)
    #[serde(rename = "paymentHeader")]
    pub payment_header: String,
    
    /// The payment requirements
    #[serde(rename = "paymentRequirements")]
    pub payment_requirements: PaymentRequirements,
}

/// Response from the facilitator's `/settle` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettlementResponse {
    /// Transaction hash of the settlement
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    
    /// Block number where the transaction was included (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    
    /// Optional error message if settlement failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Information returned in the X-PAYMENT-RESPONSE header.
///
/// Sent by the server to the client after successful settlement.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentResponse {
    /// Transaction hash of the settlement
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    
    /// Timestamp of settlement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_at: Option<String>,
    
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Represents a supported payment kind (scheme + network combination).
///
/// Returned by the facilitator's `/supported` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupportedKind {
    /// Payment scheme
    pub scheme: String,
    
    /// Network identifier
    pub network: String,
    
    /// Optional list of supported assets on this network
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<String>>,
}

/// Response from the facilitator's `/supported` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupportedResponse {
    /// List of supported payment kinds
    pub supported: Vec<SupportedKind>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_payment_required_response_serialization() {
        let response = PaymentRequiredResponse {
            x402_version: 1,
            accepts: vec![PaymentRequirements {
                scheme: "exact".to_string(),
                network: "8453".to_string(),
                max_amount_required: "10000".to_string(),
                resource: "/api/weather".to_string(),
                description: Some("Weather API access".to_string()),
                mime_type: Some("application/json".to_string()),
                output_schema: None,
                pay_to: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
                max_timeout_seconds: 300,
                asset: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
                extra: Some(json!({"name": "USDC", "version": "2"})),
            }],
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: PaymentRequiredResponse = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.x402_version, 1);
        assert_eq!(deserialized.accepts.len(), 1);
        assert_eq!(deserialized.accepts[0].scheme, "exact");
    }

    #[test]
    fn test_payment_payload_serialization() {
        let payload = PaymentPayload {
            x402_version: 1,
            scheme: "exact".to_string(),
            network: "8453".to_string(),
            payload: json!({
                "from": "0x123",
                "to": "0x456",
                "value": "10000"
            }),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: PaymentPayload = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.scheme, "exact");
        assert_eq!(deserialized.network, "8453");
    }

    #[test]
    fn test_transfer_authorization() {
        let auth = TransferAuthorization {
            from: "0xFrom".to_string(),
            to: "0xTo".to_string(),
            value: "1000000".to_string(),
            valid_after: "0".to_string(),
            valid_before: "9999999999".to_string(),
            nonce: "0x1234".to_string(),
            signature: "0xabcd".to_string(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("validAfter"));
        assert!(json.contains("validBefore"));
    }
}

