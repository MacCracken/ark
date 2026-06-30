# Contributing to Ark

Thank you for your interest in contributing to Ark, the AGNOS package manager.

Ark is written in **[Cyrius](https://github.com/MacCracken/cyrius)** — it has
been pure Cyrius since v0.8.0 (no Cargo/Rust). The build, test, lint, and
benchmark tooling is all the `cyrius` CLI.

## Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/MacCracken/ark.git
   ```

2. Install the pinned cyrius toolchain (see `.cyrius-toolchain` / `cyrius.cyml` —
   currently **6.3.5**). Dependencies (sigil, nous, mela, agnostik, sandhi)
   resolve automatically from `cyrius.cyml` / `cyrius.lock` as `dist/*.cyr`
   bundles; you do **not** need to clone them. The `path = "../<dep>"` entries in
   the manifest are optional local-dev overrides.

3. Build and run the test suite:
   ```bash
   cyrius build -D ARK_MAIN src/main.cyr build/ark            # host build
   cyrius build -D ARK_MAIN --agnos src/main.cyr build/ark    # AGNOS cross-build
   cyrius test tests/ark.tcyr
   ```

4. Run the cleanliness checks (or `cyrius audit` for the full project sweep —
   fmt/lint/docs/tests/bench):
   ```bash
   cyrius fmt src/main.cyr --check
   cyrius lint src/main.cyr
   cyrius vet src/main.cyr
   cyrius deny src/main.cyr
   ```

5. Run benchmarks and fuzzers:
   ```bash
   cyrius bench tests/ark.bcyr
   cyrius fuzz                  # runs fuzz/*.fcyr harnesses
   ```

## Development Process

See [CLAUDE.md](CLAUDE.md) for the full development process, including the Work
Loop and P(-1) Scaffold Hardening procedures.

### Key Rules

- Public enums are `#[non_exhaustive]`-equivalent — forward-compatible (new
  variants must not break consumers)
- Every type round-trips through serialization, with a roundtrip test
- Zero `unwrap`/`panic` in library code — failures return a `Result`
- Package integrity is verified (SHA-256 root hash + Ed25519 signature +
  per-file hashes) on **every** install; never bypass it
- Dependency resolution belongs in `nous`, not in `ark`
- Never skip benchmarks before claiming performance improvements

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
- cyrius toolchain version and platform

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
