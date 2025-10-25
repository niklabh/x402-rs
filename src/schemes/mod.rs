//! Payment scheme implementations.
//!
//! This module contains the trait definition for payment schemes and concrete
//! implementations for different blockchain networks.

pub mod exact_evm;

use crate::errors::Result;
use crate::types::{PaymentPayload, PaymentRequirements};
use async_trait::async_trait;

/// Trait for implementing different payment schemes.
///
/// Each scheme (e.g., "exact", "upto") must implement this trait to handle
/// payload generation, verification, and settlement.
#[async_trait]
pub trait Scheme: Send + Sync {
    /// Returns the name of this scheme (e.g., "exact").
    fn name(&self) -> &str;

    /// Generates a payment payload for the given requirements.
    ///
    /// # Arguments
    ///
    /// * `requirements` - The payment requirements from the server
    /// * `private_key` - The payer's private key for signing
    /// * `rpc_url` - RPC endpoint for the blockchain network
    ///
    /// # Returns
    ///
    /// A `PaymentPayload` ready to be encoded in the X-PAYMENT header
    async fn generate_payload(
        &self,
        requirements: &PaymentRequirements,
        private_key: &str,
        rpc_url: &str,
    ) -> Result<PaymentPayload>;

    /// Verifies a payment payload against requirements.
    ///
    /// # Arguments
    ///
    /// * `payload` - The payment payload to verify
    /// * `requirements` - The expected payment requirements
    /// * `rpc_url` - RPC endpoint for blockchain queries
    ///
    /// # Returns
    ///
    /// `Ok(true)` if valid, `Ok(false)` or `Err` if invalid
    async fn verify(
        &self,
        payload: &PaymentPayload,
        requirements: &PaymentRequirements,
        rpc_url: &str,
    ) -> Result<bool>;

    /// Settles a payment on-chain.
    ///
    /// # Arguments
    ///
    /// * `payload` - The verified payment payload
    /// * `requirements` - The payment requirements
    /// * `rpc_url` - RPC endpoint for submitting transactions
    /// * `facilitator_key` - Private key of the facilitator (to pay gas)
    ///
    /// # Returns
    ///
    /// Transaction hash of the settlement
    async fn settle(
        &self,
        payload: &PaymentPayload,
        requirements: &PaymentRequirements,
        rpc_url: &str,
        facilitator_key: &str,
    ) -> Result<String>;
}

