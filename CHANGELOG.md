# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **CLI binary** (`src/main.rs`) with clap derive — all commands accessible via `ark <command>`
- **TOML config file support** (`src/config.rs`, feature-gated `config`) with search order: `$ARK_CONFIG`, `./ark.toml`, `~/.config/ark/ark.toml`, `/etc/agnos/ark.toml`, defaults
- **PackageDb persistence** — `load(path)` / `save()` with atomic write (temp + rename) to JSON
- **Package hold/unhold** — `ark hold <pkg>` prevents upgrades, `ark unhold <pkg>` re-enables them; `held: bool` field on `PackageDbEntry` with `#[serde(default)]` for backward compat
- **ANSI color output** — `to_colored_string()` / `render(color: bool)` on `ArkOutput` using `anstyle`; headers bold, success green, error red, warning yellow, package names cyan
- **File-level integrity checking** — `check_integrity_full()` reads files on disk, computes SHA-256, compares to stored checksums; `ark verify` / `ark verify <pkg>`
- **Transaction history** — `ark history [count]` displays recent transactions from the log
- **Interactive confirmation** — `confirm.rs` with testable `confirm_with()`, used by CLI for install/remove when config flags are set
- **`--no-color` flag** on CLI to disable colored output
- `ArkPackageManager` now owns `PackageDb` and `TransactionLog`, loaded from config paths on construction
- `ArkConfig` gains `package_db_path` and `transaction_log_path` fields
- `PartialEq` / `Eq` derives on `ArkConfig`
- `DEFAULT_PACKAGE_DB_PATH` constant
- 32 new tests (114 total), covering persistence, hold/unhold, color output, integrity checking, CLI parsing, config loading, confirmation prompts, serde backward compat

### Changed

- `ArkPackageManager::execute()` now takes `&mut self` (needed for hold/unhold to modify PackageDb)
- `ARK_VERSION` uses `env!("CARGO_PKG_VERSION")` instead of hardcoded string

### Dependencies

- Added `anstyle` (1.x) — ANSI styling (already transitive from clap)
- Added `clap` (4.x, derive feature, optional `cli` feature)
- Added `toml` (0.8.x, optional `config` feature)
- Added `tracing-subscriber` (0.3.x, optional `cli` feature)
- Fixed `nous` missing `anyhow` dependency declaration

## [0.1.0] - 2026-04-04

### Added

- Core `ArkPackageManager` engine with `ArkConfig` configuration
- CLI argument parser (`parse_args`) supporting install, remove, search, list, info, update, upgrade, and status commands
- Plan-based execution model: `InstallPlan` with `InstallStep` variants for system (apt), marketplace, and Flutter sources
- Package group installation via `--group` flag (desktop, ai/ml, shell, edge/iot)
- `TransactionLog` with JSONL append-only persistence for crash recovery and audit
- Transaction lifecycle: begin, add_op, mark_op_complete, mark_op_failed, commit, rollback, fail
- `PackageDb` unified package registry with file tracking, ownership queries, and integrity checking
- Topological dependency resolution via `resolve_install_order`
- SHA-256 integrity verification infrastructure
- `ArkOutput` structured output formatting with Header, Package, Info, Separator, Success, Error, Warning lines
- `#[non_exhaustive]` on all public enums for forward compatibility
- `#[must_use]` on all pure functions
- Serde roundtrip tests for all public types
- Criterion benchmarks for parse_args, serde, PackageDb, TransactionLog, format_plan, and output display
- `scripts/bench-history.sh` for benchmark tracking

### Notes

- Ark generates execution plans but does not execute them directly (security by design)
- Dependency resolution is delegated to `nous` -- ark never reimplements it
- 82 tests, 0 failures, 2 ignored (pending nous alignment)

[Unreleased]: https://github.com/MacCracken/ark/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/MacCracken/ark/releases/tag/v0.1.0
