//! Implementation of the "exact" payment scheme for EVM-compatible chains.
//!
//! This scheme uses EIP-3009 `transferWithAuthorization` for gasless ERC-20 transfers.
//! The payer signs an authorization that allows the facilitator to execute the transfer
//! on their behalf without requiring the payer to have ETH for gas.

use crate::errors::{Result, X402Error};
use crate::schemes::Scheme;
use crate::types::{PaymentPayload, PaymentRequirements, TransferAuthorization, X402_VERSION};
use crate::utils::{current_timestamp, generate_nonce, parse_address, string_to_u256};
use async_trait::async_trait;
use ethers::abi::Token;
use ethers::contract::abigen;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::utils::keccak256;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{transaction::eip712::Eip712, Signature, H256, U256};
use serde_json::json;
use std::sync::Arc;

// Define the EIP-3009 domain and types for EIP-712 signing
const EIP712_DOMAIN_NAME: &str = "USD Coin";
const EIP712_DOMAIN_VERSION: &str = "2";

// ABI for EIP-3009 compliant ERC-20 token
abigen!(
    EIP3009Token,
    r#"[
        function transferWithAuthorization(address from, address to, uint256 value, uint256 validAfter, uint256 validBefore, bytes32 nonce, uint8 v, bytes32 r, bytes32 s) external
        function authorizationState(address authorizer, bytes32 nonce) external view returns (bool)
        function decimals() external view returns (uint8)
        function name() external view returns (string)
        function version() external view returns (string)
    ]"#
);

/// Implementation of the "exact" scheme for EVM chains.
///
/// This scheme requires the payer to pay exactly the `maxAmountRequired` using
/// EIP-3009 signed authorization.
pub struct ExactEvm;

impl ExactEvm {
    /// Creates a new instance of the ExactEvm scheme.
    pub fn new() -> Self {
        Self
    }

    /// Creates the EIP-712 typed data hash for the transfer authorization.
    fn create_authorization_hash(
        from: Address,
        to: Address,
        value: U256,
        valid_after: U256,
        valid_before: U256,
        nonce: H256,
        domain_separator: H256,
    ) -> H256 {
        // EIP-712 type hash for TransferWithAuthorization
        let type_hash = keccak256(
            b"TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)"
        );

        // Encode the struct data
        let struct_hash = keccak256(
            &ethers::abi::encode(&[
                Token::FixedBytes(type_hash.to_vec()),
                Token::Address(from),
                Token::Address(to),
                Token::Uint(value),
                Token::Uint(valid_after),
                Token::Uint(valid_before),
                Token::FixedBytes(nonce.as_bytes().to_vec()),
            ])
        );

        // EIP-712 final hash: "\x19\x01" ‖ domainSeparator ‖ hashStruct(message)
        let mut message = Vec::new();
        message.extend_from_slice(b"\x19\x01");
        message.extend_from_slice(domain_separator.as_bytes());
        message.extend_from_slice(&struct_hash);

        H256::from(keccak256(&message))
    }

    /// Creates the domain separator for EIP-712.
    fn create_domain_separator(
        token_address: Address,
        chain_id: U256,
        name: &str,
        version: &str,
    ) -> H256 {
        let type_hash = keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        );

        H256::from(keccak256(
            &ethers::abi::encode(&[
                Token::FixedBytes(type_hash.to_vec()),
                Token::FixedBytes(keccak256(name.as_bytes()).to_vec()),
                Token::FixedBytes(keccak256(version.as_bytes()).to_vec()),
                Token::Uint(chain_id),
                Token::Address(token_address),
            ])
        ))
    }
}

impl Default for ExactEvm {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Scheme for ExactEvm {
    fn name(&self) -> &str {
        "exact"
    }

    async fn generate_payload(
        &self,
        requirements: &PaymentRequirements,
        private_key: &str,
        rpc_url: &str,
    ) -> Result<PaymentPayload> {
        // Parse addresses and amounts
        let to = parse_address(&requirements.pay_to)?;
        let value = string_to_u256(&requirements.max_amount_required)?;
        let asset = parse_address(&requirements.asset)?;

        // Create wallet from private key
        let wallet = private_key
            .parse::<LocalWallet>()
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid private key: {}", e)))?;
        let from = wallet.address();

        // Connect to provider to get chain ID
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let chain_id = provider.get_chainid().await?;

        // Generate nonce and timestamps
        let nonce_bytes: [u8; 32] = {
            let nonce_str = generate_nonce();
            let nonce_hex = nonce_str.trim_start_matches("0x");
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(nonce_hex, &mut bytes)
                .map_err(|e| X402Error::InvalidPayload(format!("Invalid nonce: {}", e)))?;
            bytes
        };
        let nonce = H256::from(nonce_bytes);

        let now = current_timestamp();
        let valid_after = U256::from(now);
        let valid_before = U256::from(now + requirements.max_timeout_seconds);

        // Get token name and version from extra field or use defaults
        let (token_name, token_version) = if let Some(extra) = &requirements.extra {
            let name = extra
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(EIP712_DOMAIN_NAME);
            let version = extra
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or(EIP712_DOMAIN_VERSION);
            (name.to_string(), version.to_string())
        } else {
            (EIP712_DOMAIN_NAME.to_string(), EIP712_DOMAIN_VERSION.to_string())
        };

        // Create domain separator and authorization hash
        let domain_separator = Self::create_domain_separator(
            asset,
            chain_id,
            &token_name,
            &token_version,
        );

        let message_hash = Self::create_authorization_hash(
            from,
            to,
            value,
            valid_after,
            valid_before,
            nonce,
            domain_separator,
        );

        // Sign the hash
        let signature = wallet.sign_hash(message_hash)
            .map_err(|e| X402Error::SignatureError(e.to_string()))?;

        // Create the authorization object
        // Convert r and s from U256 to [u8; 32]
        let mut r_bytes = [0u8; 32];
        signature.r.to_big_endian(&mut r_bytes);
        let mut s_bytes = [0u8; 32];
        signature.s.to_big_endian(&mut s_bytes);
        
        let mut sig_bytes = Vec::with_capacity(65);
        sig_bytes.extend_from_slice(&r_bytes);
        sig_bytes.extend_from_slice(&s_bytes);
        sig_bytes.push(signature.v as u8);
        
        let authorization = TransferAuthorization {
            from: format!("{:?}", from),
            to: format!("{:?}", to),
            value: value.to_string(),
            valid_after: valid_after.to_string(),
            valid_before: valid_before.to_string(),
            nonce: format!("0x{}", hex::encode(nonce_bytes)),
            signature: format!("0x{}", hex::encode(sig_bytes)),
        };

        Ok(PaymentPayload {
            x402_version: X402_VERSION,
            scheme: self.name().to_string(),
            network: requirements.network.clone(),
            payload: json!(authorization),
        })
    }

    async fn verify(
        &self,
        payload: &PaymentPayload,
        requirements: &PaymentRequirements,
        rpc_url: &str,
    ) -> Result<bool> {
        // Parse the authorization from payload
        let auth: TransferAuthorization = serde_json::from_value(payload.payload.clone())
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid authorization: {}", e)))?;

        // Verify scheme and network match
        if payload.scheme != self.name() {
            return Ok(false);
        }
        if payload.network != requirements.network {
            return Ok(false);
        }

        // Parse addresses and values
        let from = parse_address(&auth.from)?;
        let to = parse_address(&auth.to)?;
        let value = string_to_u256(&auth.value)?;
        let expected_to = parse_address(&requirements.pay_to)?;
        let expected_value = string_to_u256(&requirements.max_amount_required)?;
        let asset = parse_address(&requirements.asset)?;

        // Verify to and value match requirements
        if to != expected_to {
            return Ok(false);
        }
        if value != expected_value {
            return Ok(false);
        }

        // Verify timestamps
        let valid_after = string_to_u256(&auth.valid_after)?;
        let valid_before = string_to_u256(&auth.valid_before)?;
        let now = U256::from(current_timestamp());

        if now < valid_after || now > valid_before {
            return Ok(false);
        }

        // Connect to provider
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let chain_id = provider.get_chainid().await?;

        // Get token name and version
        let (token_name, token_version) = if let Some(extra) = &requirements.extra {
            let name = extra
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(EIP712_DOMAIN_NAME);
            let version = extra
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or(EIP712_DOMAIN_VERSION);
            (name, version)
        } else {
            (EIP712_DOMAIN_NAME, EIP712_DOMAIN_VERSION)
        };

        // Parse nonce
        let nonce_hex = auth.nonce.trim_start_matches("0x");
        let mut nonce_bytes = [0u8; 32];
        hex::decode_to_slice(nonce_hex, &mut nonce_bytes)
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid nonce: {}", e)))?;
        let nonce = H256::from(nonce_bytes);

        // Check if nonce was already used on-chain
        let token_contract = EIP3009Token::new(asset, Arc::new(provider.clone()));
        let is_used = token_contract
            .authorization_state(from, nonce.into())
            .call()
            .await
            .unwrap_or(true); // Assume used if call fails

        if is_used {
            return Err(X402Error::NonceUsed(auth.nonce.clone()));
        }

        // Verify signature
        let domain_separator = Self::create_domain_separator(
            asset,
            chain_id,
            token_name,
            token_version,
        );

        let message_hash = Self::create_authorization_hash(
            from,
            to,
            value,
            valid_after,
            valid_before,
            nonce,
            domain_separator,
        );

        // Parse signature
        let sig_hex = auth.signature.trim_start_matches("0x");
        if sig_hex.len() != 130 {
            // 65 bytes * 2 hex chars
            return Ok(false);
        }

        let sig_bytes = hex::decode(sig_hex)
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid signature: {}", e)))?;

        let signature = Signature::try_from(sig_bytes.as_slice())
            .map_err(|e| X402Error::SignatureError(e.to_string()))?;

        // Recover signer from signature
        let recovered = signature.recover(message_hash)?;

        Ok(recovered == from)
    }

    async fn settle(
        &self,
        payload: &PaymentPayload,
        requirements: &PaymentRequirements,
        rpc_url: &str,
        facilitator_key: &str,
    ) -> Result<String> {
        // Parse the authorization
        let auth: TransferAuthorization = serde_json::from_value(payload.payload.clone())
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid authorization: {}", e)))?;

        // Parse signature components
        let sig_hex = auth.signature.trim_start_matches("0x");
        let sig_bytes = hex::decode(sig_hex)
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid signature: {}", e)))?;

        let r = H256::from_slice(&sig_bytes[0..32]);
        let s = H256::from_slice(&sig_bytes[32..64]);
        let v = sig_bytes[64];

        // Parse addresses and values
        let from = parse_address(&auth.from)?;
        let to = parse_address(&auth.to)?;
        let value = string_to_u256(&auth.value)?;
        let asset = parse_address(&requirements.asset)?;

        let nonce_hex = auth.nonce.trim_start_matches("0x");
        let mut nonce_bytes = [0u8; 32];
        hex::decode_to_slice(nonce_hex, &mut nonce_bytes)
            .map_err(|e| X402Error::InvalidPayload(format!("Invalid nonce: {}", e)))?;
        let nonce = H256::from(nonce_bytes);

        let valid_after = string_to_u256(&auth.valid_after)?;
        let valid_before = string_to_u256(&auth.valid_before)?;

        // Create wallet and provider
        let wallet = facilitator_key
            .parse::<LocalWallet>()
            .map_err(|e| X402Error::ConfigError(format!("Invalid facilitator key: {}", e)))?;
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let chain_id = provider.get_chainid().await?;
        let client = SignerMiddleware::new(provider, wallet.with_chain_id(chain_id.as_u64()));
        let client = Arc::new(client);

        // Create contract instance
        let token_contract = EIP3009Token::new(asset, client);

        // Call transferWithAuthorization and get pending transaction
        let call = token_contract.transfer_with_authorization(
            from,
            to,
            value,
            valid_after,
            valid_before,
            nonce.into(),
            v,
            r.into(),
            s.into(),
        );

        let pending_tx = call
            .send()
            .await
            .map_err(|e| X402Error::SettlementError(format!("Transaction failed: {}", e)))?;

        // Wait for confirmation
        let receipt = pending_tx
            .await
            .map_err(|e| X402Error::SettlementError(format!("Receipt error: {}", e)))?
            .ok_or_else(|| X402Error::SettlementError("No receipt".to_string()))?;

        Ok(format!("{:?}", receipt.transaction_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_evm_name() {
        let scheme = ExactEvm::new();
        assert_eq!(scheme.name(), "exact");
    }

    #[test]
    fn test_domain_separator() {
        let token = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".parse().unwrap();
        let chain_id = U256::from(8453u64);
        
        let domain = ExactEvm::create_domain_separator(
            token,
            chain_id,
            "USD Coin",
            "2",
        );
        
        assert_ne!(domain, H256::zero());
    }
}

