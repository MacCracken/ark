# Development Roadmap

## Completed

### v0.1.0 (2026-04-04)

- [x] Core ArkPackageManager engine
- [x] CLI argument parser for all commands
- [x] Plan-based execution model (InstallPlan / InstallStep)
- [x] Package group installation (--group flag)
- [x] TransactionLog with JSONL persistence and crash recovery
- [x] PackageDb with file tracking, ownership, and integrity checking
- [x] Topological dependency resolution
- [x] ArkOutput structured formatting
- [x] `#[non_exhaustive]` on all public enums
- [x] `#[must_use]` on all pure functions
- [x] Serde roundtrip tests for all types
- [x] Criterion benchmarks
- [x] Full documentation suite (README, CHANGELOG, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT, architecture, roadmap)
- [x] P(-1) scaffold hardening pass

### Work Loop 1 (2026-04-04)

- [x] Binary entrypoint (main.rs) with clap
- [x] TOML configuration file support (feature-gated)
- [x] PackageDb persistence to disk (JSON, atomic write)
- [x] Package hold/unhold (prevent upgrades)
- [x] ANSI color output via anstyle
- [x] File-level integrity checking (SHA-256 hash verification)
- [x] `ark history` command (view transaction log)
- [x] Interactive confirmation prompts
- [x] `--no-color` CLI flag
- [x] 114 tests, 0 failures

## Backlog

### Package Management
- [ ] Recipe (zugot) parsing and validation
- [ ] Package signing and verification via sigil
- [ ] Actual execution backend (plan -> shakti -> system)
- [ ] Package pinning and version locking
- [ ] Dependency conflict resolution UI
- [ ] Rollback execution (undo a committed transaction)

### CLI
- [ ] Progress bar / spinner during operations
- [ ] Shell completions (bash, zsh, fish)

### Database & Persistence
- [ ] Database migration framework
- [ ] Backup and restore

### Marketplace & Community
- [ ] Marketplace download and verification
- [ ] Bazaar (community package) support
- [ ] Package rating and reviews integration
- [ ] Mirror support

### Testing & Quality
- [ ] Integration tests with real nous resolver
- [ ] Property-based testing for parser
- [ ] Fuzzing for JSONL transaction log parser
- [ ] End-to-end test harness

## Future

- Plugin system for custom sources
- Remote management API
- Metrics and telemetry (opt-in)
- Offline mode with cached packages

## v1.0 Criteria

- [ ] All backlog items complete
- [ ] 90%+ test coverage
- [ ] Benchmarks stable across releases
- [ ] Security audit passed
- [ ] Documentation complete with examples and guides
- [ ] Recipe parsing validated against zugot corpus
- [ ] Package signing verified end-to-end
- [ ] Integration tested on AGNOS target hardware
