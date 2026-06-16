# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Execution backend — foundation** (`src/exec.cyr`), implementing [ADR 0001](docs/adr/0001-shakti-execution-backend.md). First bite of the `plan → shakti → system` executor:
  - `step_to_argv` lowers each `InstallStep` to a concrete system command. System steps map to `apt-get install/remove/purge/update` (with `pkg=ver` version pinning); marketplace (takumi) and Flutter (agpkg) return a "backend not yet wired" sentinel rather than silently skipping — those bites are still pending.
  - Privileged steps are wrapped as `shakti -- <argv…>` (`exec_wrap_argv`); `exec_plan` runs steps via `process.exec_vec_str`, which returns shakti's propagated child exit code, and records each step to the transaction log (commit on success, fail on first error for rollback).
  - `exec_plan` supports a **dry-run** mode that renders the exact commands without forking — fully unit-testable without root/apt.
  - `ArkConfig` gains `shakti_path` (default `/usr/bin/shakti`) and an `apply` flag (default `0` = plan-only, preserving current behavior).
  - 32 new tests (204 total) covering lowering, version pinning, shakti wrapping, privilege classification, transaction-op mapping, and dry-run plan execution.

- **Execution backend — CLI wiring** (bite 2). The executor is now reachable from the live CLI:
  - `--apply` runs the plan via the executor; `--dry-run` renders the concrete `shakti -- …` commands without running them; neither flag preserves the existing plan-only display. Plumbed through `ArkConfig.apply` (`APPLY_NONE`/`APPLY_REAL`/`APPLY_DRY`).
  - `ark_install`, `ark_remove`, and `ark_rollback` dispatch their built plan through `ark_apply_or_describe(mgr, iplan, described)`.
  - Confirmation is now gated on `APPLY_REAL` — plan-only and dry-run never prompt (they mutate nothing); a real apply still prompts per `confirm_system_installs`/`confirm_removals`.
  - `parse_args` accepts `--apply`/`--dry-run` on `install`/`remove` and skips leading flags for `rollback`.
  - Test fix: the `curl.cyml` recipe assertion no longer pins the exact upstream version (zugot bumped curl to 8.20.0); it now asserts the version parses non-empty.

Known issue (pre-existing, surfaced by the confirm-gating change): `nous`'s `resolver_resolve_all_with_recipes` can SIGSEGV nondeterministically when resolving against an unpopulated marketplace (dev environments without `/var/lib/agnos/marketplace` data). Previously masked because `ark install` always prompted and cancelled without stdin. Resolution lives in nous — tracked for a fix there; it blocks live end-to-end install in bare dev environments but not the executor logic, which is covered by dry-run unit tests.

## [0.8.2] - 2026-06-16

### Changed

- **Roadmap reconciled** against the actual codebase. The backlog had drifted badly — recipe parsing+validation, Ed25519 signing/verification, per-file checksums, pinning + version/source locking, backup/restore, shell completions, the DB migration framework, secure temp handling (`O_EXCL`/`O_NOFOLLOW`), `fsync` on the txn log, and privilege-aware config loading were all implemented and tested but still marked open. Moved into a verified "Capability inventory"; backlog now reflects only genuinely-open work.
- Marked the v1.0 "nous ported to Cyrius and integrated" criterion complete.

### Added

- **ADR 0001 — Execution backend via shakti** (`docs/adr/0001-shakti-execution-backend.md`). Reviewed shakti (0.7.0) for execution capability and fixed the integration contract: ark will run privileged steps by invoking the setuid `shakti` CLI as a subprocess (`shakti -- COMMAND`, exit-code propagation) via `process.cyr`, gated on the plan's `iplan_root` flag — not by linking the library. Establishes the path for the still-open executor.

### Notes

- `version`/`VERSION`/`ARK_VERSION_STR` synced to `0.8.2`.
- No functional code change — docs/review release. 172/172 tests still pass; build clean.
- The execution backend (plan → shakti → system) is confirmed as the one true gap: ark builds typed `InstallPlan`s but never runs a step (`process.cyr` is pulled in yet never called).

## [0.8.1] - 2026-06-16

### Changed

- **Toolchain pin** bumped `5.1.10` → `6.2.12` (`.cyrius-toolchain`, `cyrius.cyml`); resolved manifest-pin drift against the installed wrapper.
- **Dependency updates**: `nous` `1.1.1` → `1.2.6`, `sigil` `2.1.2` → `3.7.16`.
- **nous consumption modernized** to the single-file consumer artifact `dist/nous.cyr` (matching sigil's `dist/sigil.cyr`), replacing the 14 individual `src/*.cyr` module includes. Removed the 14 stale vendored `lib/nous_*.cyr` files.
- **stdlib deps realigned for 6.2.x**: `json`, `bigint`, and `toml` were carved out of the stdlib into `bayan` at the 6.2.x pin — replaced all three with `bayan`. Added `ct`, `random`, `keccak`, `thread_local` (sigil crypto primitives), plus `slice`, `result`, `fnptr`, `bench`. Removed stale `lib/json.cyr`, `lib/bigint.cyr`, `lib/toml.cyr`.

### Removed

- **`scripts/bench-history.sh`** — retired the Rust-era `cargo bench` wrapper; benchmarking is now handled by `cyrius bench tests/ark.bcyr` (CLAUDE.md work-loop steps updated to match).

### Fixed

- **Recipe parsing crash (SIGSEGV)**: `recipe_parse` called the bare `cyml_parse(content)`, which now binds to bayan's two-arg `cyml_parse(data, len)` and read garbage as the length. Switched to nous's renamed single-arg `nous_cyml_parse(src)` (`src/recipe.cyr`). nous renamed its parser to avoid clashing with the stdlib-bundled bayan symbol.
- **Crypto crash (SIGILL)**: signing/verification path trapped on undefined `random_bytes`/`ct_*`/`thread_local_*`; resolved by adding the 6.2.12 stdlib crypto modules to `[deps] stdlib`.

### Notes

- `version`/`VERSION`/`ARK_VERSION_STR` synced to `0.8.1`.
- Full suite green: 172/172 tests pass; benchmarks and `cyrius lint`/`fmt --check` clean.

## [0.1.0-dev] - Rust (historical, pre-port development log)

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
