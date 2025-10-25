# x402-rs

[![Crates.io](https://img.shields.io/crates/v/x402-rs.svg)](https://crates.io/crates/x402-rs)
[![Documentation](https://docs.rs/x402-rs/badge.svg)](https://docs.rs/x402-rs)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A complete, production-ready Rust implementation of the [x402 protocol](https://x402.org) for blockchain-based micropayments over HTTP.

## Overview

The x402 protocol revitalizes the HTTP 402 "Payment Required" status code to enable seamless, instant payments for web resources like APIs, content, or files. It provides:

- 🚀 **Instant payments** - ~2 second settlements
- 🔗 **Chain-agnostic** - Support for multiple blockchain networks
- 💰 **Zero protocol fees** - Direct peer-to-peer payments
- 🔒 **Trust-minimized** - Cryptographic signatures, no custody
- 🎯 **Low friction** - No accounts, subscriptions, or complex setups

## Features

- **Client Library**: Automatic handling of 402 responses and payment generation
- **Server Middleware**: Easy integration with web frameworks (Axum support included)
- **Facilitator Service**: Run your own payment verification and settlement service
- **EVM Support**: Full support for EVM-compatible chains using EIP-3009
- **Extensible**: Easy to add new payment schemes and blockchain networks

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
x402-rs = "0.1.0"
```

For async runtime:

```toml
[dependencies]
x402-rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Client Usage

Make requests to x402-enabled APIs with automatic payment handling:

```rust
use x402_rs::client::{X402ClientConfig, get};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure client with your private key and RPC endpoint
    let config = X402ClientConfig::new(
        "0xYOUR_PRIVATE_KEY",
        "https://mainnet.base.org"
    );

    // Make a request - payment is handled automatically
    let response = get(&config, "https://api.example.com/weather").await?;
    
    println!("Response: {}", response.text().await?);
    Ok(())
}
```

### Server Usage

Protect your endpoints with payment requirements:

```rust
use x402_rs::server::PaymentConfig;
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // Configure payment requirements
    let payment_config = PaymentConfig::new(
        "0xYOUR_ADDRESS",                           // Recipient address
        "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
        6,                                           // Token decimals
        "8453",                                      // Base mainnet
        "exact",                                     // Payment scheme
        0.01,                                        // $0.01 USD
        "API access fee",
        "https://facilitator.example.com",
    );

    // Your endpoint handler would check for X-PAYMENT header
    // and verify/settle payments using the config
    
    // See examples/server.rs for complete implementation
}
```

### Running a Facilitator

A facilitator verifies and settles payments on behalf of servers:

```rust
use x402_rs::facilitator::FacilitatorConfig;

#[tokio::main]
async fn main() {
    let config = FacilitatorConfig::new(
        "0xFACILITATOR_PRIVATE_KEY",  // For paying gas
        "https://mainnet.base.org"
    );
    
    // Implement /verify, /settle, /supported endpoints
    // See examples/facilitator.rs for complete implementation
}
```

## Protocol Flow

1. **Client requests resource**: Standard HTTP GET/POST
2. **Server responds with 402**: If payment needed, returns payment requirements
3. **Client generates payment**: Creates signed EIP-3009 authorization
4. **Client retries with payment**: Includes `X-PAYMENT` header
5. **Server verifies**: Checks signature and amount via facilitator
6. **Server settles**: Executes transaction on-chain
7. **Server responds with 200**: Returns resource with optional `X-PAYMENT-RESPONSE` header

## Supported Networks

Currently supports EVM-compatible chains with EIP-3009:

- **Base** - Mainnet (8453), Sepolia (84532)
- **Ethereum** - Mainnet (1), Sepolia (11155111)
- **Polygon** - Mainnet (137), Mumbai (80001)
- **Optimism** - Mainnet (10), Sepolia (11155420)
- **Arbitrum** - One (42161), Sepolia (421614)

Any EVM chain with EIP-3009 compatible tokens (USDC, etc.) is supported.

## Payment Schemes

### Exact Scheme

The "exact" scheme requires payment of exactly the specified amount using EIP-3009 `transferWithAuthorization`. Key features:

- **Gasless for payers**: No ETH needed, facilitator pays gas
- **Time-bounded**: Authorizations have validity windows
- **Replay protection**: Unique nonces prevent reuse
- **EIP-712 signatures**: Standard Ethereum signed messages

## Examples

The repository includes three complete examples:

### 1. Server Example

```bash
# Set environment variables
export PAY_TO="0xYourAddress"
export FACILITATOR_URL="http://localhost:3001"

# Run the server
cargo run --example server
```

### 2. Client Example

```bash
# Set environment variables
export PRIVATE_KEY="0xYourPrivateKey"
export RPC_URL="https://mainnet.base.org"
export API_URL="http://localhost:3000/weather"

# Run the client
cargo run --example client
```

### 3. Facilitator Example

```bash
# Set environment variables
export FACILITATOR_KEY="0xFacilitatorPrivateKey"
export RPC_URL="https://mainnet.base.org"

# Run the facilitator
cargo run --example facilitator
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with logs:

```bash
RUST_LOG=debug cargo test
```

## Security Considerations

- **Private Keys**: Never commit private keys. Use environment variables or secure key management.
- **RPC Endpoints**: Use reliable RPC providers (Alchemy, Infura, etc.)
- **Nonce Management**: Facilitators must track used nonces to prevent replay attacks
- **Amount Validation**: Always verify payment amounts match requirements
- **Timeout Handling**: Respect `maxTimeoutSeconds` to prevent stale authorizations

## Architecture

```
x402-rs/
├── src/
│   ├── lib.rs           # Main library entry point
│   ├── types.rs         # Protocol type definitions
│   ├── errors.rs        # Error types
│   ├── utils.rs         # Utility functions
│   ├── client.rs        # Client implementation
│   ├── server.rs        # Server middleware
│   ├── facilitator.rs   # Facilitator service
│   └── schemes/
│       ├── mod.rs       # Scheme trait
│       └── exact_evm.rs # EIP-3009 implementation
├── examples/
│   ├── server.rs        # Example API server
│   ├── client.rs        # Example client
│   └── facilitator.rs   # Example facilitator
└── tests/
    └── integration.rs   # Integration tests
```

## API Documentation

Generate and view the full API documentation:

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Run clippy: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Submit a pull request

## Roadmap

- [ ] Additional payment schemes ("upto", "tiered")
- [ ] Solana support
- [ ] Lightning Network integration
- [ ] Rate limiting and quota management
- [ ] Payment analytics and reporting
- [ ] Multi-token support per endpoint
- [ ] Subscription and recurring payment models

## References

- [x402 Protocol Specification](https://github.com/coinbase/x402) - Official Coinbase implementation
- [x402.org](https://x402.org) - Protocol overview and documentation
- [EIP-3009](https://eips.ethereum.org/EIPS/eip-3009) - Transfer With Authorization
- [EIP-712](https://eips.ethereum.org/EIPS/eip-712) - Typed structured data hashing and signing

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [Coinbase](https://www.coinbase.com) for creating the x402 protocol
- The Rust community for excellent blockchain tooling
- [ethers-rs](https://github.com/gakonst/ethers-rs) for Ethereum functionality

## Support

- 📖 [Documentation](https://docs.rs/x402-rs)
- 🐛 [Issue Tracker](https://github.com/yourusername/x402-rs/issues)
- 💬 [Discussions](https://github.com/yourusername/x402-rs/discussions)

---

**Note**: This library is under active development. The API may change before 1.0 release.

