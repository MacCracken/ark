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

## Backlog

### Package Management
- [ ] Recipe (zugot) parsing and validation
- [ ] Package signing and verification via sigil (sigil dep in place)
- [ ] Actual execution backend (plan -> shakti -> system)
- [ ] Package pinning and version locking
- [ ] Dependency conflict resolution UI
- [ ] Rollback execution (undo a committed transaction)
- [ ] Namespace scoping for dependency confusion defense (nous)
- [ ] Source pinning per package (prevent silent source switching)

### CLI
- [ ] Progress bar / spinner during operations
- [ ] Shell completions (bash, zsh, fish)

### Database & Persistence
- [ ] Database migration framework
- [ ] Backup and restore
- [ ] Per-file checksums (replace single checksum per package)
- [ ] Secure temp file handling (O_NOFOLLOW, random names)

### Marketplace & Community
- [ ] Marketplace download and verification
- [ ] Bazaar (community package) support
- [ ] Package rating and reviews integration
- [ ] Mirror support
- [ ] Typosquatting detection (Levenshtein distance)

### Security
- [ ] Privilege-aware config loading (ignore CWD config when root)
- [ ] Cryptographic package signing via sigil Ed25519
- [ ] fsync on transaction log writes
- [ ] Plan signing for shakti verification

### Testing & Quality
- [ ] Integration tests with real nous resolver
- [ ] Property-based testing for parser
- [ ] Fuzzing for JSONL transaction log parser
- [ ] End-to-end test harness
- [ ] Fuzz harness for package name validation

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
- [ ] Recipe parsing validated against zugot corpus
- [ ] Package signing verified end-to-end
- [ ] Integration tested on AGNOS target hardware
- [ ] nous ported to Cyrius and integrated
