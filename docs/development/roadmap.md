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

### v0.9.0 (2026-06-18) — Quality-gate milestone released

Caps 0.8.11–0.8.15: corpus validation (563/563), fuzz (txn/pkg/recipe), real-nous
integration, docs (guides + examples), CVE-grounded security audit + 4 bug fixes
(recipe heap-overflow, zip-slip, decompression bomb, OOB reads), and
trust-anchored signing.

- [x] Trust set (`ark_trust_load` / `acfg_trust_keys`) + `require_signed` fail-closed gate in `ark_pkg_install`; signer pubkey exposed (`apkg_pubkey`); +9 assertions (289 tests); closes audit finding #4 core
- Carried to v1.0 (non-blocking): coverage-metric tooling; default require_signed on for marketplace; source trust from mela's keyring; symlink-target constraint (audit #5)

### v0.9.1 (2026-06-18) — Security hardening follow-ups

- [x] Symlink two-pass materialization (audit #5, CWE-22)
- [x] Marketplace forces `require_signed` (untrusted → fail closed)
- [x] Trust set sourced from mela's keyring (`ark_trust_from_keyring` / `ark_trust_load_keyring`)
- [x] +3 assertions (292 tests); audit findings #4 and #5 resolved

### v1.0.0 (2026-06-18) — Cyrius port complete 🎉

- [x] First stable release: resolve → plan → execute; native `.ark` read/verify/install; trust-anchored marketplace download
- [x] Quality-gated (0.9.0) + security-hardened (0.9.1); docs/README reconciled to the shipped surface
- Deferred: marketplace enhancements + progress UI (post-1.0); AGNOS native/apply track (1.1.x)

### v1.1.0 (2026-06-29) — AGNOS compatibility: the host-side AGNOS track (M0–M3)

Brings up ark's share of the 1.1.x AGNOS track end to end on the host: the
ADR-0002 **system-backend seam** (M0), the **native `.ark` installer wired into
the executor** (M1), the **native backend + authoritative store** (M2 ark-share),
and **syscall portability** so the agnos cross-build compiles (M3). Toolchain
moved to **cyrius 6.3.5**. The remaining AGNOS-track work (M4+, the nous
`SOURCE_NATIVE` resolver, on-hardware QEMU validation) stays future — see below.
The **1.0.x marketplace/UX enhancement arc is pushed back behind 1.1.0** (still
planned, not dropped).

- [x] **Toolchain → cyrius 6.3.5** (pin `6.2.21` → `6.3.5`): stdlib re-sync (picks up arena-aware `tls_accept_*` for sandhi), deps re-resolved. 6.3.5 hardened reachable-undefined into an error — fixed the bench + fuzz harness include sets (they referenced `cli.cyr` symbols defined in non-included files) and the continuation-line fmt drift on 3 files.
- [x] **M0 — `system_backend` seam (ADR 0002).** `ArkConfig.system_backend` (`apt`/`apt-agnos`/`native`) + configurable `apt_wrapper`, threaded into `exec.cyr` `step_to_argv_be(step, backend, wrapper)`; `exec_plan` honors it. `--system-backend` CLI flag + `system_backend`/`apt_wrapper` TOML keys. Acceptance met: **apt mode byte-identical** (1-arg `step_to_argv` retained), full suite green, **`apt-agnos` wrapper-fronts apt-get**, **`native` selectable**.
- [x] **M1 — resolved steps → native `.ark` installer.** `exec_plan` routes marketplace/community (and native system) steps to ark's installer instead of failing loud; install/remove cores (`ark_pkg_install_inner` / `ark_pkg_remove_inner`) record the op in the **plan-wide transaction** so rollback reverses it; `ark_rollback` is now **source-aware** (native installs roll back via the native remover, not apt). Resolve-**"latest"** composed from mela's `build_latest_url`→`manifest_from_json`→`man_version` (mela 1.0.0, no new dep). Acceptance met: a plan installs from a **local `.ark` via `--apply`, rollback-able, no apt** (`test_native_apply`).
- [x] **M2 — native backend + authoritative store (ark's share).** Under `BACKEND_NATIVE`, `SOURCE_SYSTEM` steps install from ark's local `.ark` store into the **`PackageDb` (authoritative — ark never shells `dpkg-query`)**; default flips to `native` on agnos (`#ifdef CYRIUS_TARGET_AGNOS`). Acceptance met for ark's share (`test_native_backend`). **Producer gate (nous):** there is no `SOURCE_NATIVE` resolver / signed native index / lockfile in nous 1.2.7 (HEAD == 1.2.7, verified per-tag) — ark is the wired-and-ready *consumer*; the resolver half lands when nous ships it.
- [x] **M3 — syscall portability triage.** New `src/portable.cyr` `#ifdef`-guarded shims (`ark_now_secs`/`ark_fsync`/`ark_rename`/`ark_unlink`/`ark_symlink`) replace the Linux-host raw literals (`time 201`, `fsync 74`, `rename 82`, `O_* 193`) and the agnos-absent `sys_symlink`/4-arg ABIs; `confirm()` 128-byte over-read into a 16-byte buffer fixed. Acceptance (host-side, **compiling ≠ working**): the **agnos cross-build now compiles** (`cyrius build --agnos`; previously failed on undefined `sys_symlink`) and the host path stays **byte-identical** (337 tests green). **Deferred:** on-agnos QEMU+iron runtime validation; symlink-bearing `.ark` is rejected on agnos until the symlink syscall lands (agnos 1.51.x).
- [x] 346 tests (+54: M0 +26, M1 +9, M2 +8, M3 +5, review-fix regressions +6), bench + fuzz green; host **and** agnos builds clean. Adversarially reviewed; 7 correctness findings fixed (source attribution, durable saves, full-source rollback, version-carrying remove, portable open, CLI/validation).
- [ ] **Follow-up (pre-existing, discovered in review): `PackageDb` does not reload from disk.** `pkg_db_save` writes correct JSON but `pkg_db_load` never reconstructs the `packages` object (reads only `schema_version`), so the installed-set doesn't survive across CLI processes. Predates 1.1.0 (affected pin/hold too). Blocked on a working object parser — the bundled DOM parser `json_v_parse` returns 0 in-build; only the flat key→string parser works. Needed before the native store is truly authoritative across processes / on-agnos.

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

## Deferred behind 1.1.0: 1.x — marketplace & UX enhancements

> **Pushed back (2026-06-29):** this marketplace/UX enhancement arc is deferred
> behind the 1.1.0 AGNOS-compatibility work (M0–M3). Still planned, not dropped —
> it resumes after the AGNOS track settles. (Two of its items were absorbed by
> M1: resolve-"latest" and wiring resolved steps to the native installer.)

Non-blocking enhancements layered on the shipped 1.0 core (resolve → plan →
execute, `.ark` read/verify/install, trust-anchored marketplace download — see
Completed). Group sizes are a guide, not a contract.

### marketplace resolution UX
- [x] Resolve "latest" (no explicit version) via mela's manifest/latest endpoints — **shipped in 1.1.0 (M1)** (`ark_marketplace_resolve_latest`)
- [x] Wire `STEP_MARKETPLACE_INSTALL` (resolved packages) to the installer, so a resolved plan installs from local `.ark` without a `--marketplace` URL — **shipped in 1.1.0 (M1)** (`exec_native_step`). *(Still open: auto-**download** for a resolved-but-uncached package from a configured default marketplace.)*

### supply-chain trust depth
- [ ] Verify the artifact against mela's **transparency log** (beyond the trusted-signer check)
- [ ] Typosquatting detection (Levenshtein distance) on resolution

### distribution breadth
- [ ] Mirror support (fall over between configured marketplace mirrors)
- [ ] Bazaar install path (catalog browse done; wire to download + the installer)

### UX & resolution polish
- [ ] Progress bar / spinner during downloads + apply
- [ ] Package rating & reviews integration
- [ ] Dependency conflict-resolution UI
- [ ] Namespace scoping for dependency-confusion defense (mostly nous-side)

> The system-backend seam, the native-installer wiring, and resolve-"latest"
> from this arc **shipped in 1.1.0** (M0/M1). The remaining AGNOS-gated execution
> work (real `--apply` on hardware, the `apt-agnos` bridge, plan-signing-for-
> shakti) is the 1.1.x AGNOS track below.

## v0.9.0 — Quality gates (released 2026-06-18)

The hardening milestone before 1.0 — prove the shipped surface, not add
features. **Cut as 0.9.0** (see Completed). Substantive gates met; the coverage
**metric** stays a tooling caveat. (AGNOS-gated execution work is **not** here;
it's the 1.1.x arc.)

- [ ] 90%+ test coverage
- [ ] Benchmarks stable across releases (`cyrius bench tests/ark.bcyr`)
- [x] Formal security audit (CVE-grounded) of the post-P(-1) surfaces — `docs/audit/2026-06-18-pre-v1-audit.md`; fixed zip-slip + decompression-bomb + OOB-read (0.8.14). Trust-anchor / require-signed findings feed the signing criterion below.
- [x] Documentation complete — examples + guides current with the shipped CLI (`docs/guides/getting-started.md`, `docs/examples/install-a-package.md`, README refreshed; 0.8.13)
- [x] Recipe parsing validated against the full zugot corpus — **563/563 parse** (`tests/zugot_corpus.cyr`, 0.8.11)
- [x] Integration tests against the real nous resolver — `test_nous_integration` (recipe-backed resolve_all + search + unknown-miss; 0.8.12)
- [x] Property-based testing for the recipe parser — `fuzz_recipe` fuzz target (0.8.12)
- [x] Fuzz harnesses — JSONL transaction-log parser + package-name validation (`fuzz/ark.fcyr`, discovered by `cyrius fuzz`, 0.8.11)
- [ ] End-to-end test harness (build `.ark` → install → verify; fixtures + a stubbed shakti)

## v1.0 Criteria — met (released 1.0.0, 2026-06-18)

- [x] Core package manager complete — resolve → plan → execute; `.ark`
      read/verify/install; trust-anchored marketplace download (the
      *enhancement* backlog above — resolve-latest, mirror, ratings,
      typosquatting, conflict UI, progress bar — is **post-1.0**, not a blocker)
- [x] v0.9.0 quality gates green (corpus, fuzz, integration, property, docs, audit)
- [x] nous ported to Cyrius and integrated (1.2.7, via `dist/nous.cyr`)
- [x] Execution backend live — plan → shakti → system (0.8.3)
- [x] `.ark` read / verify / install + marketplace download (0.8.4–0.8.10)
- [x] Package signing trust-anchored + fail-closed (0.8.15 / 0.9.1); audit #4/#5 resolved
- [x] Documentation reconciled to the shipped surface (1.0.0)

> The full apt+shakti **apply** path and AGNOS-hardware integration are **not
> v1.0 blockers** — they're the 1.1.x AGNOS track below. Coverage is evidenced
> by the test suite (292 tests + fuzz); the `cyrius coverage` *metric* tool
> stays a caveat.

## Post-v1: 1.1.x — AGNOS track

AGNOS-gated work, deferred past 1.0 (waits on AGNOS kernel + shakti maturing).
ark is the pivot; the path is already captured in code + ADRs.

### 1.1.x — live privileged execution
- [ ] Real `--apply` on a target with apt + setuid shakti (end-to-end), incl. rollback execution
- [ ] **`apt-agnos` bridge** ([ADR 0002](../adr/0002-package-source-model.md)) — apt fronted by the AGNOS syscall wrapper, so AGNOS can use ark before native lands
- [ ] **System backend seam** — `system_backend` mode (`apt` / `apt-agnos` / `native`) threaded through `ArkConfig` + executor lowering + nous source selection
- [ ] Plan signing for shakti verification
- [ ] Integration tested on AGNOS target hardware

### 1.1.x → native, apt-independent package management (the "ark v2 sovereignty path")
**Goal: AGNOS owns its package layer end to end — no dependency on Debian's
apt/dpkg.** apt becomes an optional compat shim; the native path (zugot →
takumi → signed native `.ark` → nous → ark) becomes the primary system source.
See [ADR 0002](../adr/0002-package-source-model.md).

> **Why this is now a sequenced arc, not a wishlist**: **agnova (the native
> installer) can't install anything *right* without a sovereign package manager
> beneath it** — on a no-apt box, `agnova install <x>` is meaningless unless this
> chain resolves/fetches/verifies/materializes natively. So this is on the
> **critical path** (`sovereign ark → agnova → server-stage exit`), not parallel
> ecosystem work. Full cross-repo orchestration spine + the adversarially-verified
> milestone analysis (apt-disposition, gates, stage split, the agnos-surface gaps
> filed as agnos 1.51.x): [`agnosticos/docs/development/planning/ark-v2-sovereignty-path.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/ark-v2-sovereignty-path.md).
>
> **⚠ Scope correction**: M0–M2 below are **host-side Linux refactor work and are
> NOT AGNOS-gated** (despite this section sitting under the "AGNOS track" header) —
> they're unblocked *today*. Only M3+ (on-agnos) waits on kernel/shakti surface.
>
> **apt disposition (verified)**: **KEEP apt behind the `system_backend` seam — do
> NOT delete it.** It's already dead-by-construction on agnos (`nous/sysdb.cyr`
> gates the whole system leg on `which_exists("apt-cache") && which_exists("dpkg-query")`;
> agnos has neither → unreachable). The sovereign cutover is a **default-flip**, not
> a removal; deleting breaks the Debian dev/CI loop for zero gain. (ADR 0002.)

ark's share of the arc (ark drives; resolution stays in nous; build in takumi):

- [x] **M0 — `system_backend` seam** — **shipped in 1.1.0** (see Completed). ADR 0002 seam in `ArkConfig` + `exec.cyr step_to_argv_be`; apt byte-identical, apt-agnos/native selectable.
- [x] **M1 — wire resolved steps → the native `.ark` installer** — **shipped in 1.1.0**. `exec_plan` routes marketplace/native steps to ark's installer/remover, plan-txn rollback, source-aware reverse, resolve-"latest" via mela. *(Still open: auto-**download** of a resolved-but-uncached package from a configured default marketplace.)*
- [x] **M2 — native backend + authoritative store (ark's share)** — **shipped in 1.1.0**. `BACKEND_NATIVE` lowering + `PackageDb` as the authoritative store; agnos defaults to native. **Producer gate (nous, still open):** the `SOURCE_NATIVE` resolver + signed index + lockfile do not exist in nous 1.2.7 — ark is the wired-and-ready consumer; the resolver half lands when nous ships it (and the store stays empty until takumi builds a real zugot recipe → `.ark` and indexes it).
- [x] **M3 — syscall portability triage (host-side)** — **shipped in 1.1.0**. `src/portable.cyr` `#ifdef` shims; the **agnos cross-build now compiles**, host byte-identical. **Still open (deferred):** on-agnos QEMU+iron runtime validation (compiling ≠ working); symlink-bearing `.ark` is rejected on agnos until the symlink syscall lands (agnos 1.51.x (a)).
- [ ] **M4 — AGNOS-side update mechanism (ark's share; server).** Turn `ark_update`/`ark_upgrade` from plan-display into real **apply-on-native** that re-materializes from verified `.ark` with a rollback-able transaction (rides the present crash-safe ext2/4 + ark txn log + `ark_rollback`). **Gated on an agnos/gnoboot boot-slot or atomic-image-swap primitive (absent — filed agnos 1.51.x (b))**; the A/B-slot-vs-in-place model is an open design call.
- [ ] **Compat & migration**: apt demoted to `--source apt`; one-time dpkg-set import into the native store (Debian-host convenience).

> **Not ark's milestones** (tracked in their repos, listed here for the chain):
> **M5** sovereign build-step executor + **M6** self-host "build agnos on agnos" → **takumi**;
> the `SOURCE_NATIVE` resolver / signed index / lockfile → **nous**; recipe portability +
> native index format → **zugot**. The blocking `apt-agnos` bridge + shakti privileged
> execution stay in the *1.1.x live privileged execution* subsection above.

## Future (speculative)

- Plugin system for custom sources
- Remote management API
- Metrics and telemetry (opt-in)
- Offline mode with cached packages
