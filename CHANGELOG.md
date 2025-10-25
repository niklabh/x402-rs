# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive GitHub Actions CI/CD pipeline
  - Multi-platform testing (Linux, macOS, Windows)
  - Multi-version Rust testing (stable, beta)
  - Code coverage with tarpaulin and Codecov
  - Security auditing with RustSec
  - Dependency checking
  - Nightly testing
  - Cross-compilation validation
- Dependabot configuration for automated dependency updates
- GitHub issue and PR templates
- CODEOWNERS file

## [0.1.0] - 2025-01-XX

### Added
- Initial release of x402-rs
- Complete x402 protocol implementation
- Client library for automatic 402 handling
- Server middleware for payment protection
- Facilitator service implementation
- EIP-3009 "exact" payment scheme for EVM chains
- Support for Base, Ethereum, Polygon, and other EVM networks
- Comprehensive documentation and examples
- 66 tests (unit, integration, doc tests)
- Three complete examples (client, server, facilitator)

### Features
- **Client**: Automatic handling of 402 Payment Required responses
- **Server**: Easy integration with web frameworks (Axum support)
- **Facilitator**: Verification and settlement service
- **Security**: Trust-minimized payments with EIP-712 signatures
- **Performance**: ~2 second settlements, async throughout
- **Extensibility**: Trait-based scheme system for future protocols

### Documentation
- Comprehensive README with usage examples
- Full Rustdoc API documentation
- Contributing guidelines
- Example implementations for all components

[Unreleased]: https://github.com/niklabh/x402-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/niklabh/x402-rs/releases/tag/v0.1.0

