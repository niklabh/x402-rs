//! # x402-rs
//!
//! A complete Rust implementation of the x402 protocol for blockchain-based micropayments over HTTP.
//!
//! The x402 protocol revitalizes the HTTP 402 "Payment Required" status code to enable seamless,
//! instant payments for web resources like APIs, content, or files. It is chain-agnostic, supports
//! multiple payment schemes, and emphasizes low friction, zero protocol fees, and fast settlements.
//!
//! ## Features
//!
//! - **Client Support**: Automatic handling of 402 responses and payment generation
//! - **Server Support**: Middleware for protecting endpoints with payment requirements
//! - **Facilitator Support**: Run a facilitator service for verification and settlement
//! - **EVM Chains**: Full support for EVM-compatible chains using EIP-3009
//! - **Extensible**: Easy to add new payment schemes and blockchain networks
//!
//! ## Quick Start
//!
//! ### Client Example
//!
//! ```rust,no_run
//! use x402_rs::client::{X402ClientConfig, get};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = X402ClientConfig::new(
//!     "0xYOUR_PRIVATE_KEY",
//!     "https://mainnet.base.org"
//! );
//!
//! let response = get(&config, "https://api.example.com/weather").await?;
//! println!("Response: {}", response.text().await?);
//! # Ok(())
//! # }
//! ```
//!
//! ### Server Example
//!
//! ```rust,no_run
//! use x402_rs::server::PaymentConfig;
//!
//! let config = PaymentConfig::new(
//!     "0xYOUR_ADDRESS",
//!     "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
//!     6,        // decimals
//!     "8453",   // Base mainnet
//!     "exact",  // scheme
//!     0.01,     // $0.01 USD
//!     "API access fee",
//!     "https://facilitator.example.com",
//! );
//! ```
//!
//! ## Protocol Overview
//!
//! The x402 protocol follows this flow:
//!
//! 1. **Client requests resource**: Standard HTTP request
//! 2. **Server responds with 402**: If payment needed, returns payment requirements
//! 3. **Client generates payment**: Creates signed authorization payload
//! 4. **Client retries with payment**: Includes X-PAYMENT header
//! 5. **Server verifies and settles**: Via facilitator or directly
//! 6. **Server responds with 200**: Returns the resource with optional X-PAYMENT-RESPONSE header
//!
//! ## Supported Networks
//!
//! - Base (mainnet: 8453, testnet: 84532)
//! - Ethereum (mainnet: 1, testnets)
//! - Polygon (mainnet: 137, testnet: 80001)
//! - Any EVM-compatible chain with EIP-3009 support
//!
//! ## Payment Schemes
//!
//! ### Exact Scheme
//!
//! The "exact" scheme requires the payer to authorize exactly the amount specified in
//! `maxAmountRequired`. It uses EIP-3009 `transferWithAuthorization` for gasless transfers,
//! meaning the payer doesn't need ETH for gas fees.
//!
//! ## Security
//!
//! - **Trust-minimized**: Payers sign authorizations; facilitators cannot move funds beyond authorization
//! - **Replay protection**: Unique nonces prevent transaction replays
//! - **Time-bounded**: Authorizations have validity windows
//! - **On-chain verification**: All signatures verified using EIP-712 standards
//!
//! ## References
//!
//! - [x402 Specification](https://github.com/coinbase/x402)
//! - [x402.org](https://x402.org)
//! - [EIP-3009](https://eips.ethereum.org/EIPS/eip-3009)

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod client;
pub mod errors;
pub mod facilitator;
pub mod schemes;
pub mod server;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use errors::{Result, X402Error};
pub use types::{
    PaymentPayload, PaymentRequiredResponse, PaymentRequirements, SettlementRequest,
    SettlementResponse, SupportedKind, SupportedResponse, TransferAuthorization,
    VerificationRequest, VerificationResponse, X402_VERSION,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert_eq!(X402_VERSION, 1);
    }

    #[test]
    fn test_module_accessibility() {
        // Ensure all modules are accessible
        let _ = client::X402ClientConfig::new("key", "url");
        let _ = server::PaymentConfig::new(
            "addr",
            "asset",
            6,
            "network",
            "scheme",
            1.0,
            "desc",
            "facilitator",
        );
        let _ = facilitator::FacilitatorConfig::new("key", "url");
    }
}

