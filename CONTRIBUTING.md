# Contributing to x402-rs

Thank you for your interest in contributing to x402-rs! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please be respectful and considerate in all interactions.

## How to Contribute

### Reporting Issues

- Check if the issue already exists in the issue tracker
- Use the issue template if provided
- Include as much detail as possible:
  - Version of x402-rs
  - Rust version
  - Operating system
  - Steps to reproduce
  - Expected vs actual behavior

### Submitting Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following the coding standards below
3. **Add tests** for any new functionality
4. **Update documentation** including:
   - Rustdoc comments for public APIs
   - README if adding new features
   - CHANGELOG for notable changes
5. **Ensure all tests pass**: `cargo test`
6. **Run the linter**: `cargo clippy -- -D warnings`
7. **Format your code**: `cargo fmt`
8. **Commit with clear messages** following conventional commits
9. **Push to your fork** and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/x402-rs.git
cd x402-rs

# Build the project
cargo build

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### Running Examples

```bash
# Terminal 1: Start facilitator
cargo run --example facilitator

# Terminal 2: Start server
cargo run --example server

# Terminal 3: Run client
cargo run --example client
```

## Coding Standards

### Style Guide

- Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` to catch common mistakes

### Documentation

- All public items must have doc comments
- Include examples in doc comments where applicable
- Use proper markdown formatting
- Link to related items with `[Item]`

Example:
```rust
/// Handles payment verification.
///
/// # Arguments
///
/// * `request` - The verification request
/// * `config` - Facilitator configuration
///
/// # Examples
///
/// ```
/// use x402_rs::facilitator::{handle_verify, FacilitatorConfig};
///
/// # async fn example() {
/// let config = FacilitatorConfig::new("key", "rpc");
/// // ...
/// # }
/// ```
pub async fn handle_verify(request: VerificationRequest, config: &FacilitatorConfig) -> Result<VerificationResponse> {
    // implementation
}
```

### Testing

- Write unit tests for individual functions
- Write integration tests for end-to-end flows
- Aim for high test coverage (80%+)
- Test both success and error cases

### Error Handling

- Use the `Result` type for fallible operations
- Define specific error variants in `X402Error`
- Provide meaningful error messages
- Use `?` operator for error propagation

### Async Code

- Use `async/await` for I/O operations
- Use `tokio` as the async runtime
- Mark traits with `#[async_trait]` when needed
- Avoid blocking operations in async functions

## Project Structure

```
x402-rs/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── types.rs         # Type definitions
│   ├── errors.rs        # Error types
│   ├── utils.rs         # Utility functions
│   ├── client.rs        # Client implementation
│   ├── server.rs        # Server middleware
│   ├── facilitator.rs   # Facilitator service
│   └── schemes/         # Payment schemes
│       ├── mod.rs
│       └── exact_evm.rs
├── examples/            # Usage examples
├── tests/               # Integration tests
└── benches/             # Benchmarks (if any)
```

## Adding New Features

### Adding a New Payment Scheme

1. Create a new file in `src/schemes/`
2. Implement the `Scheme` trait
3. Add scheme to the match statements in client/facilitator
4. Add tests for the new scheme
5. Update documentation and README

### Adding Support for New Blockchains

1. Implement blockchain-specific logic in schemes
2. Add network identifier handling
3. Add tests with the new network
4. Update supported networks in documentation

## Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Build process or auxiliary tool changes

Examples:
```
feat(client): add support for upto payment scheme

fix(server): correct amount validation in payment requirements

docs(readme): add installation instructions for Windows
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with notable changes
3. Run full test suite
4. Create a git tag: `git tag v0.1.0`
5. Push tag: `git push origin v0.1.0`
6. Publish to crates.io: `cargo publish`

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing documentation and examples

## License

By contributing, you agree that your contributions will be licensed under both MIT and Apache-2.0 licenses.

