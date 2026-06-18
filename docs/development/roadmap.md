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

### v0.8.3 (2026-06-16) - Execution backend (bites 1–2) + install-crash fix

- [x] Executor module `src/exec.cyr`: step lowering (system → `apt-get`), shakti wrapping (`shakti -- …`), `exec_plan` (real + dry-run), transaction recording
- [x] CLI wiring: `--apply` / `--dry-run`; confirmation gated on real apply; `ark_install`/`ark_remove`/`ark_rollback` dispatch via `ark_apply_or_describe`
- [x] `ArkConfig` gains `shakti_path` + `apply` mode (`APPLY_NONE`/`APPLY_REAL`/`APPLY_DRY`)
- [x] Fixed `ark install` SIGSEGV on hosts without a provisioned marketplace (nous `1.2.6` → `1.2.7`: `registry_new` tolerates a missing dir)
- [x] 204 tests pass (+32 exec assertions); lint/fmt clean
- [ ] Still open: real apply needs apt+shakti on target; marketplace (takumi) / Flutter (agpkg) lowering; rollback execution; dry-run end-to-end

### v0.8.4 (2026-06-17) - `.ark` package reader

- [x] `src/ark_package.cyr`: reader for takumi's `.ark` v1 format — root-hash + ed25519 signature verify, manifest TOML parse/unescape, file index, DEFLATE inflate, per-file hash re-verification
- [x] stdlib deps `sankoch` (deflate) + `sync` (mutex)
- [x] 28 tests (232 total) against real takumi fixtures (`tests/fixtures/sample-{signed,unsigned}.ark`); tamper detection covered
- [ ] Next: install from `.ark` — extract payloads to disk + record to `PackageDb` (consumes the reader)

### v0.8.5 (2026-06-17) - Toolchain pin + Flutter lowering

- [x] Toolchain pin `6.2.12` → `6.2.18` (drift cleared); lib re-synced, deps re-resolved, 238 tests green
- [x] Flutter (agpkg) step lowering — `agpkg install/remove <pkg>`, unprivileged
- [x] Marketplace lowering re-scoped: native `.ark` installer, not a shell step (review outcome)

### v0.8.6 (2026-06-17) - Install from `.ark`

- [x] `ark_pkg_install` — read+verify `.ark`, materialize files under a configurable root (dirs/regular/config/symlink, recursive `mkdir -p`), register to `PackageDb` (version/files/per-file hashes/signature), record transaction
- [x] 13 tests (251 total) against the signed fixture into a temp root
- [ ] Next: wire behind `STEP_MARKETPLACE_INSTALL` + `ark install ./pkg.ark` CLI

### v0.8.7 (2026-06-17) - CLI local `.ark` install

- [x] `ark install ./pkg.ark` routes to the native installer (`ark_install_local`); `--root <dir>` staging flag
- [x] 6 tests (257 total): path detection, `ark_execute` routing, materialize + register
- [ ] Next functional gap: marketplace **download** (fetch a `.ark` to install), then route `STEP_MARKETPLACE_INSTALL`

### v0.8.8 (2026-06-17) - Toolchain + dependency refresh

- [x] Toolchain pin `6.2.18` → `6.2.20`; lib re-synced, deps re-resolved
- [x] sigil `3.7.16` → `3.8.0` (nous already latest 1.2.7); 257 tests green
- [x] Integrated **mela 0.9.3** (+ agnostik/sandhi + net/tls/async stdlib): mela 0.9.3 namespaced its API (`mela_*`), clearing the nous/sigil symbol collisions; landed marketplace download→install (`ark install name@version --marketplace <url>`)

### v0.8.10 (2026-06-18) - Marketplace download via mela's client

- [x] mela `0.9.4` → `0.9.5` (adds format-agnostic `mela_fetch_artifact`)
- [x] ark's marketplace download now drives mela's `registry_client_new` + `mela_fetch_artifact` (removed ark's own `ark_http_get_to_file`/direct sandhi). ark consumes mela's transport + guards.
- [x] 263 tests green

### v0.8.9 (2026-06-18) - Toolchain 6.2.21 + mela 0.9.4

- [x] Toolchain pin `6.2.20` → `6.2.21` (drift cleared); lib re-synced, deps re-resolved
- [x] mela `0.9.3` → `0.9.4` (deferred seams closed: real download/publish/uninstall/keyring-load); collisions still 0; 263 tests green
- [ ] Reconcile package-format split: ark installs `.ark` (takumi); mela serves `.agnos-agent` (agents). Decide whether/how ark consumes mela's agent bundles vs. its own `.ark` marketplace download

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
- [x] **Step executor: plan → shakti → system** (0.8.3, bites 1–2) — `src/exec.cyr` lowers steps, wraps privileged ones as `shakti -- …`, runs via `process.cyr`, records each to the transaction log; `--apply`/`--dry-run` wired into the CLI with confirm gating (see [ADR 0001](../adr/0001-shakti-execution-backend.md))
- [x] **Flutter (agpkg) lowering** (0.8.5) — `STEP_FLUTTER_*` → `agpkg install/remove <pkg>`, unprivileged (no shakti). Marketplace is *not* a shell lowering: takumi builds `.ark`, ark installs it natively (see "Install from `.ark`")
- [x] **Install from `.ark`** (0.8.6, `ark_pkg_install`) — materialize verified payloads under a configurable root (dirs/regular/config/symlink) + register to `PackageDb` (version, files, per-file hashes, signature) + transaction record
- [x] **CLI: `ark install ./pkg.ark`** (0.8.7) — local `.ark` install with `--root <dir>` staging; routed in `ark_execute` → `ark_install_local`
- [ ] Wire install-from-`.ark` behind `STEP_MARKETPLACE_INSTALL` (blocked on the marketplace download path — no `.ark` to fetch yet)
- [ ] **Real `--apply` on target** — exercise the full apt+shakti path on a host that has both (blocked on a Debian/AGNOS target or a stubbed-shakti e2e harness)
- [ ] Rollback **execution** — `ark_rollback` is wired through the executor; needs a test exercising it
- [ ] **System backend seam** ([ADR 0002](../adr/0002-package-source-model.md)) — `system_backend` mode (`apt` / `apt-agnos` / `native`); gates the `apt-agnos` bridge
- [ ] **`apt-agnos` bridge** — route apt through the AGNOS syscall wrapper so AGNOS can use ark before the v2 native backend lands
- [ ] Plan signing for shakti verification (sign the plan ark hands off)

### `.ark` package format — consume takumi output
- [x] **`.ark` v1 reader + integrity/signature verification** (0.8.4, `src/ark_package.cyr`) — header, `[[manifest]]` TOML (bayan + symmetric un-escape), file index, DEFLATE inflate (`sankoch`), SHA-256 root-hash + ed25519 verify, per-file hash re-check. Conformance-tested against real takumi fixtures incl. tamper detection.
- [x] **Unpack + register** (0.8.6) — materialize files honoring type, register in `PackageDb` with per-file hashes + signature (`ark_pkg_install`)

### Marketplace & community
- **mela** is consumed at **0.9.3** (`dist/mela.cyr`) — 0.9.3 namespaced its public API (`mela_*`), clearing the `registry_new`/`manifest_new`/`keyring_new` collisions with nous/sigil. Pulls the sandhi TLS/HTTP/async/ws stack (transitive).
- [x] **Marketplace download → install** (0.8.8, `src/marketplace.cyr`) — `ark install name@version --marketplace <url>`: mela validates + builds the URL, sandhi transports, the 0.8.6 installer verifies + materializes + registers. Cache-hit skips the network.
- [ ] Resolve "latest" (no explicit version) via mela's manifest/latest endpoints
- [ ] Verify the artifact against mela's keyring + transparency log (beyond the `.ark`'s own embedded signature)
- [ ] Wire `STEP_MARKETPLACE_INSTALL` (resolved marketplace packages) to the marketplace installer, so `ark install <name>` (no URL) uses a configured marketplace
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

## v2.0 — Native, apt-independent package management

**Goal: AGNOS owns its package layer end to end — ark installs, removes, and
resolves without depending on Debian's apt/dpkg toolchain being present.**

Today the `SOURCE_SYSTEM` path is apt: nous's `sysdb` shells out to
`apt-cache`/`dpkg-query` for resolution and the executor lowers system steps
to `apt-get`. That ties ark to a Debian base and to apt's availability,
correctness, and repos. v2 makes apt **optional** — a compatibility shim, not
the foundation. The native path (zugot recipes → takumi builds → a native
package store, fetched as signed prebuilt artifacts) becomes the primary
system source.

### Native package store (replaces dpkg as source of truth)
- [ ] Promote ark's `PackageDb` to the authoritative installed-package store
      (files, owners, versions, checksums) — no `dpkg-query` dependency
- [ ] Native package artifact format (build on the existing `.ark` packaging),
      with SHA-256 + signature verified on install (sigil Ed25519, already in tree)
- [ ] Native install/remove that unpacks/links into the system and records to
      the store + transaction log — executor lowering target alongside apt

### Native system resolver (nous-side)
- [ ] New `SOURCE_NATIVE` backend in nous: resolve against a native repo index
      + the local store, independent of `apt-cache`
- [ ] Strategy update: `STRAT_SYSTEM_FIRST` resolves native-first, apt-fallback
      only when the apt compat shim is enabled
- [ ] Keep `sysdb`/apt behind a capability flag for Debian-base interop during
      the transition

### Native binary repository + distribution
- [ ] Native repo/mirror protocol: signed index + content-addressed artifacts
      (folds in the v1 "marketplace download + verification" + "mirror" items)
- [ ] takumi as the primary build backend for source recipes; prebuilt native
      artifacts as the fast path
- [ ] Base-system bootstrap provided natively (so a fresh AGNOS install needs
      no apt at all)

### Compatibility & migration
- [ ] apt support demoted to an optional `--source apt` / compat strategy;
      ark fully functional with it absent
- [ ] One-time import of an existing dpkg-installed set into the native store
      (migration aid for Debian-based hosts)
- [x] ADR for the native-vs-apt source model and the cutover plan ([ADR 0002](../adr/0002-package-source-model.md))
- [ ] Interim: `apt-agnos` backend mode — apt fronted by the AGNOS syscall wrapper, so AGNOS can use ark before native lands (see ADR 0002)

Cross-repo: this is co-designed with **nous** (native resolver backend),
**takumi** (build backend), **zugot** (recipe corpus), and **shakti**
(privileged native install). ark drives; resolution stays in nous.
