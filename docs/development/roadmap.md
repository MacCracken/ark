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

## Backlog

### Package Management
- [ ] Recipe (zugot) parsing and validation
- [ ] Package signing and verification via sigil
- [ ] Actual execution backend (plan -> shakti -> system)
- [ ] Package pinning and version locking
- [ ] Dependency conflict resolution UI
- [ ] Rollback execution (undo a committed transaction)
- [ ] Package hold/unhold (prevent upgrades)

### CLI
- [ ] Binary entrypoint (main.rs) with clap
- [ ] Interactive confirmation prompts
- [ ] Progress bar / spinner during operations
- [ ] Color output (ANSI formatting)
- [ ] Shell completions (bash, zsh, fish)
- [ ] `ark history` command (view transaction log)

### Database & Persistence
- [ ] PackageDb persistence to disk
- [ ] File-level integrity checking (hash each file on disk)
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

- Configuration file support (ark.toml)
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
