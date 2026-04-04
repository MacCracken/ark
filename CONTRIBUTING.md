# Contributing to Ark

Thank you for your interest in contributing to Ark, the AGNOS package manager.

## Getting Started

1. Clone the repository and its sibling crate `nous`:
   ```bash
   git clone https://github.com/MacCracken/ark.git
   git clone https://github.com/MacCracken/nous.git
   ```

2. Ensure you have Rust 1.89+ installed:
   ```bash
   rustup update stable
   ```

3. Run the test suite:
   ```bash
   cargo test
   ```

4. Run the full cleanliness check:
   ```bash
   cargo fmt --check
   cargo clippy --all-features --all-targets -- -D warnings
   cargo audit
   cargo deny check
   RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
   ```

## Development Process

See [CLAUDE.md](CLAUDE.md) for the full development process, including the Work Loop and P(-1) Scaffold Hardening procedures.

### Key Rules

- Every public enum must have `#[non_exhaustive]`
- Every pure function must have `#[must_use]`
- Every type must be `Serialize + Deserialize` with a roundtrip test
- Zero `unwrap`/`panic` in library code
- Never skip benchmarks before claiming performance improvements
- Dependency resolution belongs in `nous`, not in `ark`

## Submitting Changes

1. Create a feature branch from `main`
2. Follow the Work Loop (see CLAUDE.md)
3. Ensure all cleanliness checks pass
4. Run benchmarks and include numbers for performance-related changes
5. Update CHANGELOG.md
6. Open a pull request with a clear description

## Reporting Issues

Please open an issue on GitHub with:
- A clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Rust version and platform

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
