# GitHub Actions Workflows

This directory contains the CI/CD pipelines for the x402-rs project.

## Workflows

### 🔨 CI (`ci.yml`)
**Triggers:** Push and PR to `main` and `develop` branches

**Jobs:**
- **Test Suite**: Runs on Ubuntu, macOS, and Windows with stable and beta Rust
  - Format checking (`cargo fmt`)
  - Linting (`cargo clippy`)
  - Build verification
  - Unit tests
  - Doc tests
  - Example builds
- **Coverage**: Generates code coverage reports and uploads to Codecov
- **Documentation**: Builds Rustdoc to ensure docs compile
- **Security Audit**: Checks for known security vulnerabilities
- **Dependencies Check**: Monitors outdated dependencies
- **Release Build**: Tests optimized release builds

### 🎨 Lint (`lint.yml`)
**Triggers:** Push and PR to `main` and `develop` branches

**Jobs:**
- **Format Check**: Ensures consistent formatting with `rustfmt`
- **Clippy**: Catches common mistakes and anti-patterns
- **Typos**: Checks for spelling errors in code and docs
- **Markdown Lint**: Validates markdown file formatting

### 🚀 Release (`release.yml`)
**Triggers:** Git tags matching `v*.*.*` pattern

**Jobs:**
- **Create Release**: Creates a GitHub release
- **Publish Crate**: Publishes to crates.io (requires `CARGO_REGISTRY_TOKEN` secret)
- **Build Docs**: Deploys documentation to GitHub Pages

### 🌙 Nightly (`nightly.yml`)
**Triggers:** Daily at 2 AM UTC, or manual dispatch

**Jobs:**
- **Nightly Test**: Tests with Rust nightly (non-blocking)
- **Minimal Versions**: Tests with minimal dependency versions
- **Cross Compilation**: Tests cross-compilation to various targets

### 📊 Benchmark (`benchmark.yml`)
**Triggers:** Push and PR to `main` branch

**Jobs:**
- **Benchmark**: Runs performance benchmarks (placeholder for future criterion benches)

## Required Secrets

To enable all features, configure these secrets in your GitHub repository:

### For crates.io Publishing
- `CARGO_REGISTRY_TOKEN`: Your crates.io API token
  - Get it from: https://crates.io/me
  - Settings → Secrets and variables → Actions → New repository secret

### For Code Coverage (Optional)
- `CODECOV_TOKEN`: Your Codecov upload token
  - Get it from: https://codecov.io
  - Add repository → Copy token

## Status Badges

Add these to your README.md:

```markdown
[![CI](https://github.com/niklabh/x402-rs/workflows/CI/badge.svg)](https://github.com/niklabh/x402-rs/actions/workflows/ci.yml)
[![Security Audit](https://github.com/niklabh/x402-rs/workflows/Security%20Audit/badge.svg)](https://github.com/niklabh/x402-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/niklabh/x402-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/niklabh/x402-rs)
```

## Caching Strategy

All workflows use GitHub Actions cache to speed up builds:
- Cargo registry cache
- Cargo git index cache
- Build artifacts cache

This significantly reduces CI run times after the first run.

## Branch Protection

Recommended branch protection rules for `main`:

1. Require status checks to pass:
   - Test Suite (ubuntu-latest, stable)
   - Clippy
   - Format Check
2. Require branches to be up to date
3. Require review from code owners
4. Dismiss stale reviews

## Local Testing

Before pushing, run these commands locally to catch issues early:

```bash
# Format check
cargo fmt -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --verbose

# Build examples
cargo build --examples

# Documentation
cargo doc --no-deps
```

## Dependabot

Dependabot is configured in `dependabot.yml` to automatically:
- Update Cargo dependencies weekly
- Update GitHub Actions weekly
- Create PRs with proper labels

## Troubleshooting

### CI Failures

1. **Format Check Failed**
   ```bash
   cargo fmt
   git add .
   git commit -m "fix: format code"
   ```

2. **Clippy Warnings**
   ```bash
   cargo clippy --fix --allow-dirty
   git add .
   git commit -m "fix: clippy warnings"
   ```

3. **Test Failures**
   ```bash
   cargo test
   # Fix failing tests
   ```

4. **Build Failures**
   - Check Cargo.lock is committed
   - Verify all dependencies are available
   - Check for platform-specific issues

### Release Workflow Not Triggering

1. Ensure tag matches pattern `v*.*.*`:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. Verify GitHub Actions are enabled in repository settings

3. Check that `CARGO_REGISTRY_TOKEN` secret is set

## Performance

Typical CI run times (with cache):
- Test Suite: ~3-5 minutes per platform
- Lint: ~1-2 minutes
- Coverage: ~5-7 minutes
- Documentation: ~2-3 minutes

First run (no cache): ~10-15 minutes

## Contributing

When adding new workflows:
1. Test locally with [act](https://github.com/nektos/act) if possible
2. Start with `workflow_dispatch` trigger for testing
3. Add appropriate caching
4. Document any new secrets required
5. Update this README

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/actions-rs)
- [Workflow Syntax](https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions)

