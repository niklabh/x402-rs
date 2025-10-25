//! Utility functions for x402 operations.
//!
//! This module provides helper functions for encoding/decoding, conversions,
//! and other common operations used throughout the library.

use crate::errors::{Result, X402Error};
use crate::types::PaymentPayload;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ethers::types::{Address, U256};
use std::str::FromStr;

/// Encodes a PaymentPayload as Base64 JSON for the X-PAYMENT header.
///
/// # Arguments
///
/// * `payload` - The payment payload to encode
///
/// # Examples
///
/// ```
/// use x402_rs::types::PaymentPayload;
/// use x402_rs::utils::encode_payment_header;
/// use serde_json::json;
///
/// let payload = PaymentPayload {
///     x402_version: 1,
///     scheme: "exact".to_string(),
///     network: "8453".to_string(),
///     payload: json!({}),
/// };
///
/// let encoded = encode_payment_header(&payload).unwrap();
/// assert!(!encoded.is_empty());
/// ```
pub fn encode_payment_header(payload: &PaymentPayload) -> Result<String> {
    let json = serde_json::to_string(payload)?;
    Ok(BASE64.encode(json.as_bytes()))
}

/// Decodes a Base64 JSON PaymentPayload from the X-PAYMENT header.
///
/// # Arguments
///
/// * `encoded` - The Base64 encoded payment payload
///
/// # Examples
///
/// ```
/// use x402_rs::types::PaymentPayload;
/// use x402_rs::utils::{encode_payment_header, decode_payment_header};
/// use serde_json::json;
///
/// let payload = PaymentPayload {
///     x402_version: 1,
///     scheme: "exact".to_string(),
///     network: "8453".to_string(),
///     payload: json!({}),
/// };
///
/// let encoded = encode_payment_header(&payload).unwrap();
/// let decoded = decode_payment_header(&encoded).unwrap();
/// assert_eq!(decoded.scheme, "exact");
/// ```
pub fn decode_payment_header(encoded: &str) -> Result<PaymentPayload> {
    let decoded = BASE64.decode(encoded.as_bytes())?;
    let json_str = String::from_utf8(decoded)
        .map_err(|e| X402Error::InvalidPayload(format!("Invalid UTF-8: {}", e)))?;
    let payload: PaymentPayload = serde_json::from_str(&json_str)?;
    Ok(payload)
}

/// Converts a string representation of a uint256 to ethers U256.
///
/// # Arguments
///
/// * `s` - String representation of the number (can be decimal or hex with 0x prefix)
///
/// # Examples
///
/// ```
/// use x402_rs::utils::string_to_u256;
///
/// let value = string_to_u256("1000000").unwrap();
/// assert_eq!(value, 1000000u64.into());
///
/// let hex_value = string_to_u256("0x0f4240").unwrap();
/// assert_eq!(hex_value, 1000000u64.into());
/// ```
pub fn string_to_u256(s: &str) -> Result<U256> {
    // Try decimal first
    if let Ok(value) = U256::from_dec_str(s) {
        return Ok(value);
    }
    
    // Try hex if it has 0x prefix
    if s.starts_with("0x") || s.starts_with("0X") {
        if let Ok(value) = U256::from_str(s) {
            return Ok(value);
        }
    }
    
    Err(X402Error::InvalidAmount(format!("Cannot parse '{}' as U256", s)))
}

/// Converts a U256 to its string representation.
///
/// # Arguments
///
/// * `value` - The U256 value to convert
///
/// # Examples
///
/// ```
/// use x402_rs::utils::u256_to_string;
/// use ethers::types::U256;
///
/// let value = U256::from(1000000u64);
/// let s = u256_to_string(value);
/// assert_eq!(s, "1000000");
/// ```
pub fn u256_to_string(value: U256) -> String {
    value.to_string()
}

/// Validates and parses an Ethereum address.
///
/// # Arguments
///
/// * `addr` - The address string to validate (with or without 0x prefix)
///
/// # Examples
///
/// ```
/// use x402_rs::utils::parse_address;
///
/// let addr = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbb").unwrap();
/// // Verify it parsed successfully
/// assert!(format!("{:?}", addr).len() > 0);
/// ```
pub fn parse_address(addr: &str) -> Result<Address> {
    Address::from_str(addr).map_err(|e| X402Error::InvalidAddress(format!("{}: {}", addr, e)))
}

/// Generates a random 32-byte nonce for EIP-3009 authorization.
///
/// # Examples
///
/// ```
/// use x402_rs::utils::generate_nonce;
///
/// let nonce = generate_nonce();
/// assert_eq!(nonce.len(), 66); // "0x" + 64 hex chars
/// ```
pub fn generate_nonce() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let nonce: [u8; 32] = rng.gen();
    format!("0x{}", hex::encode(nonce))
}

/// Converts a dollar amount to the smallest token unit based on decimals.
///
/// # Arguments
///
/// * `dollar_amount` - Amount in dollars (e.g., 0.01 for 1 cent)
/// * `decimals` - Token decimals (e.g., 6 for USDC, 18 for USDT on some chains)
/// * `token_usd_price` - Current price of 1 token in USD (e.g., 1.0 for stablecoins)
///
/// # Examples
///
/// ```
/// use x402_rs::utils::dollar_to_token_amount;
///
/// // $0.01 in USDC (6 decimals, $1 per USDC)
/// let amount = dollar_to_token_amount(0.01, 6, 1.0).unwrap();
/// assert_eq!(amount, "10000");
/// ```
pub fn dollar_to_token_amount(
    dollar_amount: f64,
    decimals: u8,
    token_usd_price: f64,
) -> Result<String> {
    if token_usd_price <= 0.0 {
        return Err(X402Error::InvalidAmount("Token price must be positive".to_string()));
    }
    
    let token_amount = dollar_amount / token_usd_price;
    let multiplier = 10f64.powi(decimals as i32);
    let smallest_unit = (token_amount * multiplier).round() as u128;
    
    Ok(smallest_unit.to_string())
}

/// Gets the current Unix timestamp in seconds.
///
/// # Examples
///
/// ```
/// use x402_rs::utils::current_timestamp;
///
/// let now = current_timestamp();
/// assert!(now > 1600000000); // After Sept 2020
/// ```
pub fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Checks if a timestamp is within the valid range.
///
/// # Arguments
///
/// * `valid_after` - Start of validity period (Unix timestamp)
/// * `valid_before` - End of validity period (Unix timestamp)
///
/// # Examples
///
/// ```
/// use x402_rs::utils::{current_timestamp, is_timestamp_valid};
///
/// let now = current_timestamp();
/// assert!(is_timestamp_valid(now - 60, now + 300));
/// assert!(!is_timestamp_valid(now + 60, now + 300));
/// ```
pub fn is_timestamp_valid(valid_after: u64, valid_before: u64) -> bool {
    let now = current_timestamp();
    now >= valid_after && now <= valid_before
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encode_decode_payment_header() {
        let payload = PaymentPayload {
            x402_version: 1,
            scheme: "exact".to_string(),
            network: "8453".to_string(),
            payload: json!({"test": "data"}),
        };

        let encoded = encode_payment_header(&payload).unwrap();
        let decoded = decode_payment_header(&encoded).unwrap();

        assert_eq!(decoded.scheme, payload.scheme);
        assert_eq!(decoded.network, payload.network);
    }

    #[test]
    fn test_string_to_u256() {
        assert_eq!(string_to_u256("1000000").unwrap(), U256::from(1000000u64));
        assert_eq!(string_to_u256("0").unwrap(), U256::zero());
        assert_eq!(string_to_u256("0x0f4240").unwrap(), U256::from(1000000u64));
    }

    #[test]
    fn test_u256_to_string() {
        assert_eq!(u256_to_string(U256::from(1000000u64)), "1000000");
        assert_eq!(u256_to_string(U256::zero()), "0");
    }

    #[test]
    fn test_parse_address() {
        // Use a properly formatted Ethereum address (40 hex chars)
        let addr = parse_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbb").unwrap();
        // Just verify it parsed successfully
        assert_eq!(format!("{:?}", addr).len() > 0, true);
        
        // Test address without 0x prefix
        let addr2 = parse_address("742d35Cc6634C0532925a3b844Bc9e7595f0bEbb").unwrap();
        assert_eq!(addr, addr2);
        
        // Test that invalid addresses fail
        let invalid = parse_address("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        
        assert_eq!(nonce1.len(), 66);
        assert!(nonce1.starts_with("0x"));
        assert_ne!(nonce1, nonce2); // Should be different
    }

    #[test]
    fn test_dollar_to_token_amount() {
        // $0.01 in USDC (6 decimals)
        let amount = dollar_to_token_amount(0.01, 6, 1.0).unwrap();
        assert_eq!(amount, "10000");

        // $1.00 in USDC
        let amount = dollar_to_token_amount(1.0, 6, 1.0).unwrap();
        assert_eq!(amount, "1000000");

        // $0.01 in USDT (18 decimals on some chains)
        let amount = dollar_to_token_amount(0.01, 18, 1.0).unwrap();
        assert_eq!(amount, "10000000000000000");
    }

    #[test]
    fn test_timestamp_validation() {
        let now = current_timestamp();
        assert!(is_timestamp_valid(now - 60, now + 300));
        assert!(!is_timestamp_valid(now + 60, now + 300));
        assert!(!is_timestamp_valid(now - 300, now - 60));
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1600000000); // After Sept 2020
        assert!(ts < 2000000000); // Before May 2033
    }
}

