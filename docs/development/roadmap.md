# Development Roadmap

## Completed

### v0.1.0 (2026-04-04) - Rust

- [x] Core ArkPackageManager engine
- [x] CLI argument parser for all commands
- [x] Plan-based execution model (InstallPlan / InstallStep)
- [x] Package group installation (--group flag)
- [x] TransactionLog with JSONL persistence and crash recovery
- [x] PackageDb with file tracking, ownership, and integrity checking
- [x] Topological dependency resolution
- [x] ArkOutput structured formatting
- [x] Serde roundtrip tests for all types
- [x] Criterion benchmarks
- [x] Full documentation suite
- [x] P(-1) scaffold hardening pass
- [x] Binary entrypoint with clap
- [x] TOML configuration file support
- [x] PackageDb persistence to disk (JSON, atomic write)
- [x] Package hold/unhold
- [x] ANSI color output
- [x] File-level integrity checking (SHA-256)
- [x] Transaction history command
- [x] Interactive confirmation prompts
- [x] `--no-color` CLI flag

### v0.8.0 (2026-04-16) - Cyrius Port

- [x] Full port from Rust to Cyrius (4363 -> 1943 lines)
- [x] Cyrius 5.1.7 toolchain, cc5 compiler, DCE enabled
- [x] Accessor-function pattern (load64/store64) for all 13 struct types
- [x] sigil integration for SHA-256 via dist/sigil.cyr
- [x] `pkg_db_check_integrity_full` with file existence + SHA-256 verification
- [x] IntegrityIssue types: NoManifest, MissingFile, ChecksumMismatch
- [x] `pkg_db_save` full JSON serialization of all entries
- [x] `json_escape_str` with batched safe-character runs
- [x] Transaction log: locked writes via `file_append_locked()`
- [x] Transaction log: O(1) hashmap index for lookups
- [x] Package name validation (reject traversal, special chars)
- [x] Case-insensitive package search
- [x] `parse_args` --no-color offset fix for all subcommands
- [x] `txn_log_recent` underflow guard
- [x] `pkg_db_unregister` uses `map_delete`
- [x] `pkg_db_list` uses `map_values` (decoupled from hashmap internals)
- [x] Heap-allocated I/O buffers (moved off stack)
- [x] P(-1) security audit: 27 internal + 15 external findings
- [x] 147 test assertions (9 groups including security suite)
- [x] 9 benchmarks baselined
- [x] CI/CD workflows updated (lint, test, bench, DCE)
- [x] `#ifdef ARK_MAIN` guard for test/bench inclusion

### v0.8.1 (2026-06-16) - Toolchain & dependency modernization

- [x] Cyrius toolchain pin `5.1.10` → `6.2.12` (drift resolved)
- [x] nous `1.1.1` → `1.2.6`, consumed via single-file `dist/nous.cyr` artifact
- [x] sigil `2.1.2` → `3.7.16`
- [x] stdlib realigned for 6.2.x: `json`/`bigint`/`toml` → `bayan`; added `ct`/`random`/`keccak`/`thread_local`/`slice`/`result`/`fnptr`/`bench`
- [x] Fixed recipe SIGSEGV (`cyml_parse` → `nous_cyml_parse`) and crypto SIGILL (missing stdlib crypto modules)
- [x] Removed 17 stale vendored lib files (14 `nous_*.cyr` + `json`/`bigint`/`toml`)
- [x] 172 tests pass; lint/fmt/benchmarks clean

### v0.8.2 (2026-06-16) - Roadmap reconciliation & shakti execution review

- [x] Roadmap reconciled against the actual codebase (backlog had drifted — many shipped features were still marked open)
- [x] shakti reviewed for execution capability; integration contract captured in [ADR 0001](../adr/0001-shakti-execution-backend.md)
- [x] Confirmed the execution backend is the one true gap: ark builds typed `InstallPlan`s but no step is ever run (`process.cyr` pulled in, never called)

### Capability inventory (verified against code 2026-06-16)

These were on the backlog but are implemented and tested in the tree today:

- [x] Recipe (zugot) parsing **and** validation — `recipe_parse` / `recipe_validate` (`src/recipe.cyr`), green against `curl.cyml`
- [x] Package signing & verification via sigil Ed25519 — `sign_data` / `verify_data` / `generate_keypair` (`src/db.cyr`)
- [x] Per-file checksums — `pdbe_set_file_hash` / `pdbe_get_file_hash` (replaces single per-package checksum)
- [x] Package pinning + version/source locking — `ark_pin` / `ark_unpin`, `pkg_db_pin_version`, `pkg_db_pin_source`
- [x] Source pinning per package (anti silent-source-switch) — `pkg_db_pin_source`
- [x] Rollback **plan** generation — `ark_rollback` builds the reverse plan (execution gated on the backend below)
- [x] Backup & restore — `ark_backup` / `ark_restore`
- [x] Shell completions — `completions/ark.{bash,zsh,fish}`
- [x] Database migration framework — `schema_version` read + migrate-on-load (`src/db.cyr`)
- [x] Secure temp file handling — random suffix + `O_CREAT|O_EXCL|O_WRONLY` 0600 (`src/db.cyr`)
- [x] `fsync` on transaction-log writes — locked append + fsync (`src/transaction.cyr`)
- [x] Privilege-aware config loading — skips user-writable configs when `euid == 0` (`src/cli.cyr`)
- [x] Bazaar local catalog browse — `bazaar_db_load` / search / list-by-category (`src/bazaar.cyr`); download/verify still open

## Backlog

### Execution backend (the critical path)
- [ ] **Step executor: plan → shakti → system** — run each `InstallStep`; gate privileged plans (`iplan_root == 1`) through the setuid `shakti` binary via `process.cyr`; capture exit code, record transaction, verify integrity post-step (see [ADR 0001](../adr/0001-shakti-execution-backend.md))
- [ ] Rollback **execution** (reverse plan exists; needs the executor to run it)
- [ ] Plan signing for shakti verification (sign the plan ark hands off)
- [ ] Dry-run / `--simulate` mode for the executor

### Marketplace & community
- [ ] Marketplace package download + SHA-256/signature verification (no HTTP/fetch path yet)
- [ ] Bazaar install path (catalog browse done; wire to download + executor)
- [ ] Mirror support
- [ ] Package rating & reviews integration
- [ ] Typosquatting detection (Levenshtein distance)

### Resolution (mostly nous-side)
- [ ] Dependency conflict resolution UI
- [ ] Namespace scoping for dependency-confusion defense (nous)

### CLI
- [ ] Progress bar / spinner during operations

### Testing & quality
- [ ] Integration tests with the real nous resolver
- [ ] Property-based testing for the recipe parser
- [ ] Fuzz harness for the JSONL transaction-log parser
- [ ] Fuzz harness for package-name validation
- [ ] End-to-end test harness (requires the executor)

## Future

- Plugin system for custom sources
- Remote management API
- Metrics and telemetry (opt-in)
- Offline mode with cached packages

## v1.0 Criteria

- [ ] All backlog items complete
- [ ] 90%+ test coverage
- [ ] Benchmarks stable across releases
- [ ] Security audit passed (P(-1) done, formal audit pending)
- [ ] Documentation complete with examples and guides
- [ ] Recipe parsing validated against full zugot corpus (single-recipe `curl.cyml` passes today)
- [ ] Package signing verified end-to-end (primitives in place; needs the executor to exercise the full path)
- [ ] Integration tested on AGNOS target hardware
- [x] nous ported to Cyrius and integrated (nous 1.2.6, consumed via `dist/nous.cyr`)
- [ ] Execution backend live (plan → shakti → system)
