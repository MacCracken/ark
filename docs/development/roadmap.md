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

### v0.8.9 (2026-06-18) - Toolchain 6.2.21 + mela 0.9.4

- [x] Toolchain pin `6.2.20` → `6.2.21` (drift cleared); lib re-synced, deps re-resolved
- [x] mela `0.9.3` → `0.9.4` (deferred seams closed: real download/publish/uninstall/keyring-load); collisions still 0; 263 tests green
- [x] **Package-format direction decided (2026-06-18):** `.ark` (takumi) is the primary marketplace artifact — mela publishes both but **leans on `.ark`**, which aligns directly with ark's installer. ark fetches `.ark` via mela's `mela_fetch_artifact` (0.8.10) → `ark_pkg_install`; `.agnos-agent` (agent bundles) stays mela's own pipeline. No format bridge needed in ark.

### v0.8.10 (2026-06-18) - Marketplace download via mela's client

- [x] mela `0.9.4` → `0.9.5` (adds format-agnostic `mela_fetch_artifact`)
- [x] ark's marketplace download now drives mela's `registry_client_new` + `mela_fetch_artifact` (removed ark's own `ark_http_get_to_file`/direct sandhi). ark consumes mela's transport + guards.
- [x] 263 tests green

### v0.8.11 (2026-06-18) - v0.9.0 quality gates (chunk 1)

- [x] Fixed `recipe_parse_file` heap overflow (64 KiB buffer vs 512 KiB read cap)
- [x] Zugot corpus validation: 563/563 recipes parse (`tests/zugot_corpus.cyr`)
- [x] Fuzz harness discoverable + passing (`fuzz/ark.fcyr` via `cyrius fuzz`)
- [x] mela 0.9.5 → 1.0.0 (API-compatible)
- Remaining 0.9.0 gates: coverage metric, formal audit, docs, nous integration tests, property-based parser tests, e2e harness

### v0.8.12 (2026-06-18) - v0.9.0 quality gates (chunk 2)

- [x] Integration tests vs the real nous resolver (`test_nous_integration`, zugot-backed)
- [x] Recipe-parser fuzz target (`fuzz_recipe`)
- Remaining 0.9.0 gates: e2e harness (install→verify is covered; build-via-takumi is env/CI), docs (examples + guides), formal audit, coverage metric (tooling)

### v0.8.13 (2026-06-18) - v0.9.0 quality gates (chunk 3: docs)

- [x] `docs/guides/getting-started.md` + `docs/examples/install-a-package.md`; README refreshed to the shipped CLI
- Remaining 0.9.0 gates: formal security audit; coverage metric (tooling). E2e: install→verify tested + documented (build-via-takumi is CI/hardware)

### v0.8.14 (2026-06-18) - v0.9.0 quality gates (chunk 4: security audit)

- [x] CVE-grounded audit of post-P(-1) surfaces (`docs/audit/2026-06-18-pre-v1-audit.md`)
- [x] Fixed zip-slip (CWE-22), decompression bomb (CWE-409), OOB reads (CWE-125) in the `.ark` reader/installer; +8 regression assertions (280 tests)
- Open (→ signing criterion): trust-anchored signature verification + require-signed policy; symlink-target constraint

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

### Execution backend (shipped)
- [x] **Step executor: plan → shakti → system** (0.8.3, bites 1–2) — `src/exec.cyr` lowers steps, wraps privileged ones as `shakti -- …`, runs via `process.cyr`, records each to the transaction log; `--apply`/`--dry-run` wired into the CLI with confirm gating ([ADR 0001](../adr/0001-shakti-execution-backend.md))
- [x] **Flutter (agpkg) lowering** (0.8.5) — `STEP_FLUTTER_*` → `agpkg install/remove <pkg>`, unprivileged
- [x] **Install from `.ark`** (0.8.6, `ark_pkg_install`) — materialize verified payloads under a configurable root (dirs/regular/config/symlink) + register to `PackageDb` + transaction record
- [x] **CLI: `ark install ./pkg.ark`** (0.8.7) — local `.ark` install with `--root <dir>` staging

  > Remaining execution work — real `--apply` on a target, rollback execution, the `apt-agnos` bridge, the system-backend seam, plan-signing-for-shakti — is **AGNOS-gated → the 1.1.x arc** below.

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

(Testing & quality items are the **v0.9.0** milestone below.)

## v0.9.0 — Quality gates (final hardening before v1.0)

The last milestone before cutting 1.0 — prove the shipped surface, not add
features. (AGNOS-gated execution work is **not** here; it's the 1.1.x arc.)

- [ ] 90%+ test coverage
- [ ] Benchmarks stable across releases (`cyrius bench tests/ark.bcyr`)
- [x] Formal security audit (CVE-grounded) of the post-P(-1) surfaces — `docs/audit/2026-06-18-pre-v1-audit.md`; fixed zip-slip + decompression-bomb + OOB-read (0.8.14). Trust-anchor / require-signed findings feed the signing criterion below.
- [x] Documentation complete — examples + guides current with the shipped CLI (`docs/guides/getting-started.md`, `docs/examples/install-a-package.md`, README refreshed; 0.8.13)
- [x] Recipe parsing validated against the full zugot corpus — **563/563 parse** (`tests/zugot_corpus.cyr`, 0.8.11)
- [x] Integration tests against the real nous resolver — `test_nous_integration` (recipe-backed resolve_all + search + unknown-miss; 0.8.12)
- [x] Property-based testing for the recipe parser — `fuzz_recipe` fuzz target (0.8.12)
- [x] Fuzz harnesses — JSONL transaction-log parser + package-name validation (`fuzz/ark.fcyr`, discovered by `cyrius fuzz`, 0.8.11)
- [ ] End-to-end test harness (build `.ark` → install → verify; fixtures + a stubbed shakti)

## v1.0 Criteria

- [ ] Backlog features (above) complete
- [ ] v0.9.0 quality gates all green
- [x] nous ported to Cyrius and integrated (1.2.7, via `dist/nous.cyr`)
- [x] Execution backend live — plan → shakti → system (0.8.3)
- [x] `.ark` read / verify / install + marketplace download (0.8.4–0.8.10)
- [~] Package signing: `.ark` Ed25519 + per-file hashes verified on install — **but against the package's own embedded pubkey (no trust anchor), and unsigned is accepted** (audit finding #4). Trust-anchored verification (mela keyring / configured store) + require-signed policy is the remaining signing work before installing from untrusted marketplaces

> The full apt+shakti **apply** path and AGNOS-hardware integration are **not
> v1.0 blockers** — they're the 1.1.x AGNOS track below.

## Post-v1: 1.1.x — AGNOS track

AGNOS-gated work, deferred past 1.0 (waits on AGNOS kernel + shakti maturing).
ark is the pivot; the path is already captured in code + ADRs.

### 1.1.x — live privileged execution
- [ ] Real `--apply` on a target with apt + setuid shakti (end-to-end), incl. rollback execution
- [ ] **`apt-agnos` bridge** ([ADR 0002](../adr/0002-package-source-model.md)) — apt fronted by the AGNOS syscall wrapper, so AGNOS can use ark before native lands
- [ ] **System backend seam** — `system_backend` mode (`apt` / `apt-agnos` / `native`) threaded through `ArkConfig` + executor lowering + nous source selection
- [ ] Plan signing for shakti verification
- [ ] Integration tested on AGNOS target hardware

### 1.1.x → native, apt-independent package management
**Goal: AGNOS owns its package layer end to end — no dependency on Debian's
apt/dpkg.** apt becomes an optional compat shim; the native path (zugot →
takumi → native store, signed prebuilt artifacts) becomes the primary system
source. See [ADR 0002](../adr/0002-package-source-model.md).

- [ ] Native package store: promote `PackageDb` to the authoritative installed-package store (no `dpkg-query`); native artifact format on `.ark`, SHA-256 + Ed25519 verified
- [ ] Native install/remove (unpack/link + record) — executor lowering target alongside apt
- [ ] Native system resolver (nous): `SOURCE_NATIVE` against a native index + local store; `STRAT_SYSTEM_FIRST` native-first, apt-fallback behind a capability flag
- [ ] Native repo/mirror protocol: signed index + content-addressed artifacts (folds in mirror support); base-system bootstrap with no apt
- [ ] Compat & migration: apt demoted to `--source apt`; one-time dpkg-set import into the native store

Cross-repo: co-designed with **nous** (native resolver), **takumi** (build
backend), **zugot** (recipes), and **shakti** (privileged install). ark drives;
resolution stays in nous.

## Future (speculative)

- Plugin system for custom sources
- Remote management API
- Metrics and telemetry (opt-in)
- Offline mode with cached packages
