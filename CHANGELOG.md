# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.8.14] - 2026-06-18

Fourth chunk of the **v0.9.0 quality-gate milestone** — security audit + hardening.

### Security

- **Pre-v1.0 security audit** of the post-P(-1) surfaces (`.ark` reader/installer, marketplace download, executor), grounded in current package-manager 0-day/CVE classes (zip-slip, signature fail-open / no-trust-anchor, decompression bombs, OOB reads). Findings doc: `docs/audit/2026-06-18-pre-v1-audit.md`. Three exploitable issues fixed:

### Fixed

- **Zip-slip / path traversal (CWE-22, HIGH)** in `ark_pkg_install` — entry paths were joined to the install root and written unchecked; a malicious `.ark` could write outside the root (arbitrary file write). New `apkg_path_safe()` (leading `/`, no `..` component) + a **pre-scan that aborts before any write** if any entry path is unsafe. Regression-tested (`test_pkg_security`, +8 assertions).
- **Decompression bomb (CWE-409, MED)** — `ark_pkg_read` now rejects a declared uncompressed size > 256 MiB (`ARK_MAX_DATA`) before allocating, and requires the compressed bytes to fit the verified prefix.
- **Out-of-bounds reads (CWE-125, MED)** — every variable-length read in `ark_pkg_read` (manifest, per-entry path/target, data block) is now bounded by the SHA-256-verified prefix length; the per-file hash loop checks each payload lies within the inflated buffer. Malformed/truncated `.ark` is rejected, not crashed.

### Notes

- Two trust-model findings remain **open and documented** (feed the v1.0 "package signing verified end-to-end" criterion): the `.ark` signature is verified against the package's *own embedded pubkey* (no trust anchor) and unsigned packages are accepted; and symlink-entry targets aren't constrained. Recommendation: verify against mela's keyring / a configured trust store and require signatures for untrusted sources, failing closed.

## [0.8.13] - 2026-06-18

Third chunk of the **v0.9.0 quality-gate milestone** — documentation.

### Added

- **`docs/guides/getting-started.md`** — usage guide for the shipped CLI: the plan-first execution model (`--dry-run` / `--apply`), local `.ark` install (`--root`), marketplace install (`name@version --marketplace <url>`), remove, and the full inspect/hold/pin/rollback/backup/bazaar command set.
- **`docs/examples/install-a-package.md`** — worked end-to-end example (takumi `.ark` → ark verify → materialize → register), covering local + marketplace install and post-install `verify`, using the test fixtures.

### Changed

- **README** commands section refreshed to the shipped surface (was missing `--apply`/`--dry-run`/`--root`/`--marketplace`/local-`.ark`/verify/history/pin/hold/rollback/backup/bazaar); links the new guide + example.

## [0.8.12] - 2026-06-18

Second chunk of the **v0.9.0 quality-gate milestone**.

### Added

- **Integration tests against the real nous resolver** (`test_nous_integration`, `tests/ark.tcyr`) — loads the zugot recipe corpus, builds a resolver, and exercises recipe-backed `resolver_resolve_all_with_recipes` (curl + transitive deps), `resolver_search`, and the unknown-package clean-miss path (the nous 1.2.7 SIGSEGV regression). Gated on the corpus being present. +9 assertions (272 total).
- **Recipe-parser fuzz target** (`fuzz_recipe` in `fuzz/ark.fcyr`) — feeds malformed/truncated/binary input to `recipe_parse`; the parser must return `Err`, never crash. Covers the "property-based testing for the recipe parser" gate (fuzzing form). `cyrius fuzz`: all seeds ok.

## [0.8.11] - 2026-06-18

First chunk of the **v0.9.0 quality-gate milestone**.

### Fixed

- **Heap overflow in `recipe_parse_file`** (`src/recipe.cyr`) — it `alloc`'d a 64 KiB buffer but read with a 512 KiB cap, corrupting the heap for any recipe > 64 KiB. Buffer now matches the read cap.

### Added

- **Zugot corpus validation harness** (`tests/zugot_corpus.cyr`) — parses every recipe in the zugot corpus with ark's own parser. **563/563 parse, 0 failures** (44 carry expected validation warnings — meta/marketplace packages with no source tarball). Satisfies the v1.0 criterion "recipe parsing validated against the full zugot corpus." (Reads a generated path list; see the file header.)
- **Fuzz harness now discoverable** — moved `tests/ark.fcyr` → `fuzz/ark.fcyr` so `cyrius fuzz` finds and runs it (txn-log JSONL, package-name, CLI args, JSON escape). `cyrius fuzz`: all seeds ok, PASS.

### Changed

- **mela 0.9.5 → 1.0.0** — API-compatible bump (mela froze its public API at 1.0.0; `mela_fetch_artifact` unchanged). 263 tests green.

## [0.8.10] - 2026-06-18

### Changed

- **Marketplace download now goes through mela's client** (`src/marketplace.cyr`). mela 0.9.5 added a format-agnostic `mela_fetch_artifact(client, name, version, dest)` (writes the artifact verbatim to a caller-chosen path, no `.agnos-agent` assumption). ark now builds a mela `registry_client_new` and calls it to fetch the `.ark` into its cache, replacing ark's hand-rolled `ark_http_get_to_file` (removed) and its direct `sandhi_http_*` calls. ark now consumes mela's transport + download guards instead of duplicating them.
- **mela 0.9.4 → 0.9.5** (the dep providing the new fetch primitive).
- 263 tests still green; build/lint/fmt clean. Cache-hit install path unchanged.

## [0.8.9] - 2026-06-18

### Changed

- **Toolchain pin 6.2.20 → 6.2.21** (`.cyrius-toolchain`, `cyrius.cyml`); lib re-synced, deps re-resolved. Drift cleared (matches the wrapper). 263-test suite green, build/lint clean, nous resolution intact.
- **mela 0.9.3 → 0.9.4** — mela closed its remaining deferred seams (real `rc_download` fetch+write, `rc_publish` upload, `rc_check_updates`, on-disk uninstall, keyring disk-load). Namespacing held — mela ∩ {nous, sigil, agnostik, sankoch} symbol collisions remain **0**. mela's transitive dep tags (agnostik 1.3.1, sigil 3.8.0, sankoch 2.4.3, sandhi 1.6.7) match ark's pins.

### Notes

- mela 0.9.4's `rc_download` writes **`.agnos-agent`** artifacts (mela's agent-bundle format, extracted by mela's own pipeline) — **not** takumi's `.ark`. So ark's `.ark` marketplace path keeps doing its own fetch (it can't feed a `.agnos-agent` into `ark_pkg_install`). The two package formats/pipelines are distinct: ark = system/`.ark` (takumi), mela = app+agent/`.agnos-agent`. Reconciling ark's marketplace download with mela's artifact type (and whether ark should install agent bundles at all) is a design follow-up, not a drop-in delegation.

## [0.8.8] - 2026-06-17

### Changed

- **Toolchain pin 6.2.18 → 6.2.20** (`.cyrius-toolchain`, `cyrius.cyml`); lib re-synced, deps re-resolved. 257-test suite green, build/lint clean.
- **sigil 3.7.16 → 3.8.0** — crypto crate bump; signed-`.ark` verification and the security suite (both exercise sigil's Ed25519 + SHA-256) confirm API compatibility. nous already at latest (1.2.7).
- **Added mela 0.9.3 + transitive deps** (`agnostik` 1.3.1, `sandhi` 1.6.7) and the supporting stdlib stack (`trait`, `sakshi`, `thread`, `net`, `async`, `atomic`, `mmap`, `dynlib`, `fdlopen`, `regression`, `http`, `tls`, `ws`). mela 0.9.3 namespaces its public API (`mela_*`), resolving the `registry_new`/`manifest_new`/`keyring_new` collisions with nous/sigil that blocked the 0.9.2 attempt — nous's resolver is verified intact.

### Added

- **Marketplace download → install** (`src/marketplace.cyr`): `ark install name@version --marketplace <url>` fetches the package's `.ark` from a mela marketplace and installs it natively. Division of labour — **mela** validates the name/version + builds the artifact URL, **sandhi** provides the HTTP/TLS transport, the **0.8.6 installer** verifies + materializes + registers. A cache hit (artifact already under the cache dir) skips the network. `--root` staging supported.
  - 6 new tests (263 total): cache-path construction (mela `sanitize_filename`), offline cache-hit install end-to-end, invalid-segment rejection (mela validator), and `name@version` arg parsing. Live network fetch needs a marketplace server; the offline-failure path is graceful.

### Notes

- mela's own artifact fetch+write is deferred (mela ADR-0006), so ark performs the GET via sandhi (the same client mela uses) using mela's URL/validation helpers. Resolving "latest" (no explicit version) and signature/transparency-log verification against mela's keyring are follow-ups.

## [0.8.7] - 2026-06-17

### Added

- **`ark install ./pkg.ark` — local `.ark` install from the CLI.** A single install argument ending in `.ark` is routed (in `ark_execute`) to the native installer (`ark_install_local` → `ark_pkg_install`) instead of nous resolution. Verified end-to-end: `ark install <file>.ark` reads, verifies, materializes, and registers the package.
- **`--root <dir>` flag** on `install` — stages the install under a directory instead of the real root (`ArkCommand.root`). Unprivileged-friendly for testing/inspection; the default (real root) needs appropriate privilege.
- 6 new tests (257 total): `.ark` path detection (`apkg_is_ark_path`, suffix-exact), `ark_execute` routing of a `.ark` arg, and materialization + `PackageDb` registration via the command path.

### Notes

- Routing `STEP_MARKETPLACE_INSTALL` through the installer remains blocked on the marketplace **download** path (no `.ark` to install until fetch exists) — local install is the working entry point today.

## [0.8.6] - 2026-06-17

### Added

- **Install from `.ark`** (`ark_pkg_install`, `src/ark_package.cyr`) — the native install path that consumes the 0.8.4 reader. Reads + fully verifies a `.ark` (root hash, signature, per-file hashes), then:
  - **Materializes** every entry under a configurable install root (`""` = real root, or a staging dir for tests), honoring file type — directories (recursive `mkdir -p`), regular/config files (writes the verified payload bytes), and symlinks; parent directories are created as needed.
  - **Registers** the package in `PackageDb` with version, the owned file list, per-file SHA-256 hashes (`pdbe_set_file_hash`), installed size, and the package signature.
  - **Records** an install transaction (begin → op → commit).
  - Returns a clean failure result (no partial install attempt) when the `.ark` is missing or fails verification.
- 13 new tests (251 total): installs the signed fixture into a temp root and asserts files on disk, payload content, `PackageDb` registration (version, file count, per-file hash, signature), and clean failure on a missing package.

This is the native-install execution target for ADR 0002's `native` mode and the install half of the marketplace/community path. Wiring it behind the executor's `STEP_MARKETPLACE_INSTALL` (and a CLI `ark install ./pkg.ark`) is a follow-up.

## [0.8.5] - 2026-06-17

### Changed

- **Toolchain pin 6.2.12 → 6.2.18** (`.cyrius-toolchain`, `cyrius.cyml`) — matches the installed wrapper, clears the pin-drift warning. lib re-synced, deps re-resolved; 238-test suite green, build/lint/fmt clean.
- **Marketplace lowering re-scoped (review outcome).** `STEP_MARKETPLACE_INSTALL`/`REMOVE` are *not* shell-command steps: takumi is a build-time engine (`recipe → takumi → .ark`), not an installer, so marketplace/community packages install via ark's own native `.ark` installer (extract + register), not via `step_to_argv`. The executor's sentinel message and code comments now reflect this; the work is tracked as "Install from `.ark`", not a lowering gap.

### Added

- **Flutter (agpkg) step lowering** (`src/exec.cyr`) — `STEP_FLUTTER_INSTALL`/`REMOVE` lower to `agpkg install <pkg>` / `agpkg remove <pkg>`, run unprivileged (no shakti wrap), matching ark's established display convention. 6 new tests (238 total) covering the argv and privilege classification. (agpkg's version-pin syntax is left TBD until its CLI is fixed.)

## [0.8.4] - 2026-06-17

### Added

- **`.ark` v1 package reader** (`src/ark_package.cyr`) — ark can now read the binary package format takumi produces (takumi `docs/adr/0001-ark-binary-format.md`). `ark_pkg_read(path, max_len)`:
  - Verifies the SHA-256 **root hash** over the file prefix and, when present, the **ed25519 signature** over that root hash; returns 0 (not a crash) on any mismatch, unknown version, or malformed/missing file.
  - Parses the embedded **manifest** TOML via `bayan`, with symmetric un-escaping (bayan delimits but doesn't un-escape) — quotes/newlines/tabs roundtrip losslessly.
  - Parses the **file index** (type, flags, path, size, sha256, symlink target, data offset) and inflates the **DEFLATE** data block (`sankoch`), then re-verifies every per-file content hash against the index.
  - Exposes manifest (`apkg_man_*`), file entries (`apkg_afe_*`), and the inflated payload (`apkg_data`/`apkg_afe_data_offset`) for extraction.
  - Added stdlib deps `sankoch` (deflate) and `sync` (mutex, pulled by sankoch).
  - 28 new tests (232 total) against **real takumi-produced fixtures** (`tests/fixtures/sample-{signed,unsigned}.ark`): full verify, manifest/escape roundtrip, file index, payload extraction, signature presence, and tamper detection (flip a byte → read returns 0).

Reader only — installing from a `.ark` (laying down files, recording to `PackageDb`) wires up in a later bite.

## [0.8.3] - 2026-06-16

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

### Changed

- **nous `1.2.6` → `1.2.7`** — picks up the fix for the resolution SIGSEGV below.

### Fixed

- **`ark install` SIGSEGV on systems without a provisioned marketplace.** Root cause was in nous: `registry_new` hard-errored when `/var/lib/agnos/marketplace` was absent, so `nous_resolver_new` returned a null sentinel and `nous_resolve_all` then handed back a non-error payload that the caller dereferenced. nous 1.2.7 treats a missing marketplace dir as an empty registry; `ark install`/`--dry-run` now resolve cleanly (0-package plan / "nothing to do") instead of crashing. Surfaced by the bite-2 confirm-gating change, which stopped masking it behind the cancel-on-no-stdin prompt.

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
