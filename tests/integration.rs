//! Integration tests for the x402-rs library.
//!
//! These tests verify the end-to-end functionality of the protocol,
//! including client, server, and facilitator interactions.

use serde_json::json;
use std::collections::HashMap;
use x402_rs::{
    client::{X402ClientConfig, request_with_payment},
    facilitator::{FacilitatorConfig, handle_supported, handle_verify},
    server::{PaymentConfig, create_payment_required_response},
    types::{PaymentRequiredResponse, VerificationRequest},
    utils::{encode_payment_header, decode_payment_header, dollar_to_token_amount},
};

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
    assert_eq!(config.scheme, "exact");
}

#[test]
fn test_payment_requirements_generation() {
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
    assert_eq!(requirements.network, "8453");
    assert_eq!(requirements.resource, "/api/test");
    assert_eq!(requirements.max_amount_required, "10000"); // $0.01 in USDC
}

#[test]
fn test_payment_required_response_creation() {
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
    assert_eq!(response.accepts[0].scheme, "exact");
}

#[test]
fn test_payment_required_response_serialization() {
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
    let json = serde_json::to_string(&response).unwrap();
    
    // Deserialize and verify
    let deserialized: PaymentRequiredResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.x402_version, 1);
    assert_eq!(deserialized.accepts.len(), 1);
}

#[test]
fn test_client_config_creation() {
    let config = X402ClientConfig::new(
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        "https://mainnet.base.org"
    );

    assert!(!config.private_key.is_empty());
    assert!(!config.rpc_url.is_empty());
    assert_eq!(config.preferred_scheme, Some("exact".to_string()));
}

#[test]
fn test_client_config_builders() {
    let config = X402ClientConfig::new("0xkey", "https://rpc.url")
        .with_scheme("upto")
        .with_network("137");

    assert_eq!(config.preferred_scheme, Some("upto".to_string()));
    assert_eq!(config.preferred_network, Some("137".to_string()));
}

#[test]
fn test_facilitator_config_creation() {
    let config = FacilitatorConfig::new(
        "0xfacilitator_key",
        "https://mainnet.base.org"
    );

    assert!(!config.private_key.is_empty());
    assert!(!config.rpc_url.is_empty());
    assert!(config.is_supported("exact", "8453"));
}

#[test]
fn test_facilitator_add_supported() {
    let mut config = FacilitatorConfig::new("0xkey", "https://rpc.url");
    config.add_supported("upto", "137");

    assert!(config.is_supported("exact", "8453")); // default
    assert!(config.is_supported("upto", "137")); // added
    assert!(!config.is_supported("exact", "137")); // not added
}

#[tokio::test]
async fn test_facilitator_supported_endpoint() {
    let mut config = FacilitatorConfig::new("0xkey", "https://rpc.url");
    config.add_supported("exact", "84532"); // Base Sepolia

    let response = handle_supported(&config).await.unwrap();

    assert_eq!(response.supported.len(), 2); // default + added
    assert!(response.supported.iter().any(|s| s.network == "8453"));
    assert!(response.supported.iter().any(|s| s.network == "84532"));
}

#[test]
fn test_dollar_to_token_conversion() {
    // Test USDC (6 decimals)
    let amount = dollar_to_token_amount(0.01, 6, 1.0).unwrap();
    assert_eq!(amount, "10000");

    let amount = dollar_to_token_amount(1.0, 6, 1.0).unwrap();
    assert_eq!(amount, "1000000");

    // Test 18 decimal token
    let amount = dollar_to_token_amount(0.01, 18, 1.0).unwrap();
    assert_eq!(amount, "10000000000000000");
}

#[test]
fn test_payment_header_encoding_decoding() {
    use x402_rs::types::PaymentPayload;

    let payload = PaymentPayload {
        x402_version: 1,
        scheme: "exact".to_string(),
        network: "8453".to_string(),
        payload: json!({"test": "data"}),
    };

    let encoded = encode_payment_header(&payload).unwrap();
    assert!(!encoded.is_empty());

    let decoded = decode_payment_header(&encoded).unwrap();
    assert_eq!(decoded.scheme, "exact");
    assert_eq!(decoded.network, "8453");
}

#[test]
fn test_multiple_payment_options() {
    let mut configs = HashMap::new();
    
    // Add USDC option
    configs.insert(
        "usdc".to_string(),
        PaymentConfig::new(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            6,
            "8453",
            "exact",
            0.01,
            "USDC payment",
            "https://facilitator.test",
        ),
    );

    // Add another token option
    configs.insert(
        "usdt".to_string(),
        PaymentConfig::new(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            6,
            "8453",
            "exact",
            0.01,
            "USDT payment",
            "https://facilitator.test",
        ),
    );

    let response = create_payment_required_response(&configs, "/test").unwrap();
    assert_eq!(response.accepts.len(), 2);
}

#[test]
fn test_address_parsing() {
    use x402_rs::utils::parse_address;

    // Valid address (40 hex characters)
    let addr = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbb");
    assert!(addr.is_ok());

    // Invalid address
    let addr = parse_address("invalid");
    assert!(addr.is_err());

    // Address without 0x prefix should also work
    let addr = parse_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEbb");
    assert!(addr.is_ok());
}

#[test]
fn test_u256_conversions() {
    use x402_rs::utils::{string_to_u256, u256_to_string};
    use ethers::types::U256;

    // Decimal string
    let value = string_to_u256("1000000").unwrap();
    assert_eq!(value, U256::from(1000000u64));

    // Hex string
    let value = string_to_u256("0x0f4240").unwrap();
    assert_eq!(value, U256::from(1000000u64));

    // Round trip
    let original = U256::from(123456789u64);
    let string = u256_to_string(original);
    let parsed = string_to_u256(&string).unwrap();
    assert_eq!(original, parsed);
}

#[test]
fn test_nonce_generation() {
    use x402_rs::utils::generate_nonce;

    let nonce1 = generate_nonce();
    let nonce2 = generate_nonce();

    assert_eq!(nonce1.len(), 66); // 0x + 64 hex chars
    assert!(nonce1.starts_with("0x"));
    assert_ne!(nonce1, nonce2); // Should be different
}

#[test]
fn test_timestamp_validation() {
    use x402_rs::utils::{current_timestamp, is_timestamp_valid};

    let now = current_timestamp();
    
    // Valid: current time is between after and before
    assert!(is_timestamp_valid(now - 60, now + 300));
    
    // Invalid: current time is before valid_after
    assert!(!is_timestamp_valid(now + 60, now + 300));
    
    // Invalid: current time is after valid_before
    assert!(!is_timestamp_valid(now - 300, now - 60));
}

#[test]
fn test_error_types() {
    use x402_rs::errors::X402Error;

    let err = X402Error::InvalidPayload("test".to_string());
    assert_eq!(err.to_string(), "Invalid payment payload: test");

    let err = X402Error::UnsupportedScheme("unknown".to_string());
    assert_eq!(err.to_string(), "Unsupported scheme: unknown");

    let err = X402Error::TimeoutExceeded;
    assert_eq!(err.to_string(), "Timeout exceeded");
}

#[test]
fn test_type_serialization() {
    use x402_rs::types::*;

    // Test PaymentRequirements serialization
    let req = PaymentRequirements {
        scheme: "exact".to_string(),
        network: "8453".to_string(),
        max_amount_required: "10000".to_string(),
        resource: "/test".to_string(),
        description: Some("Test".to_string()),
        mime_type: Some("application/json".to_string()),
        output_schema: None,
        pay_to: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        max_timeout_seconds: 300,
        asset: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
        extra: Some(json!({"name": "USDC", "version": "2"})),
    };

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: PaymentRequirements = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.scheme, "exact");

    // Test TransferAuthorization serialization
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

