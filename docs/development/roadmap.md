# Development Roadmap

> Per-release detail lives in [CHANGELOG.md](../../CHANGELOG.md). This roadmap
> tracks the *shape*: what has shipped, what's open, and the forward AGNOS /
> sovereignty track. Current release: **1.1.3**.

## Shipped

### Release arc

- **0.1.0 (Rust, 2026-04-04) → 0.8.x (Cyrius port).** The original Rust package
  manager, then a full port to Cyrius (accessor-pattern structs, sigil crypto,
  the `.ark` reader/installer, mela marketplace download). Bite-level history is
  in the CHANGELOG.
- **0.9.0 / 0.9.1 (2026-06-18) — quality + security gates.** Full zugot-corpus
  parse (563/563), fuzz harnesses, real-nous integration tests, guides+examples,
  a CVE-grounded security audit + fixes (recipe heap-overflow, zip-slip,
  decompression bomb, OOB reads), and trust-anchored signing (`require_signed`
  fail-closed, trust set from mela's keyring, symlink two-pass materialization).
- **1.0.0 (2026-06-18) — Cyrius port complete.** First stable release: resolve →
  plan → execute; native `.ark` read/verify/install; trust-anchored marketplace
  download. Quality-gated + security-hardened; docs reconciled to the shipped CLI.
- **1.1.0 (2026-06-29) — AGNOS host-side track (M0–M3).** Toolchain → **cyrius
  6.3.5**. The ADR-0002 **system-backend seam** (M0: `apt`/`apt-agnos`/`native`,
  threaded through `step_to_argv_be`); the **native `.ark` installer wired into
  the executor** (M1: `exec_native_step`, source-aware rollback, resolve-"latest");
  the **native backend + authoritative `PackageDb` store** (M2, agnos default);
  and **syscall portability** (M3: `src/portable.cyr` — the agnos cross-build
  compiles). Adversarially reviewed; 7 correctness fixes.
- **1.1.1 — real `ark upgrade`.** Builds a backend-aware plan from available
  updates, drops held + pin-violating packages, honors a package filter, and
  applies through the executor in a rollback-able transaction.
- **1.1.2 — `PackageDb` persistent round-trip (the reload fix).** `pkg_db_load`
  now reconstructs entries (the `json_v_parse(Str)` 6.3.5 `_str` mis-dispatch —
  use `bayan_json_v_parse_str(str_data, str_len)`) and `pkg_db_save` persists the
  full entry (pins, owned files, per-file hashes; schema v3). The native store is
  now authoritative across processes and ark-managed holds/pins persist.
- **1.1.3 — `ark upgrade` detects marketplace updates.** nous `1.2.7` → `1.3.0`
  (its pluggable `resolver_check_updates`); `ark_check_updates` feeds nous the
  installed set from `PackageDb` + an `avail_fn` backed by mela's `/latest`
  (`marketplace_url` config). apt-only with no URL configured.

**Deps (1.1.3):** cyrius 6.3.5; nous 1.3.0, sigil 3.9.7, mela 1.0.1, sandhi 1.7.0,
agnostik 1.3.1, sankoch 2.4.6.

### Capabilities (verified against code, 2026-06-29)

What ark does today, end to end:

- **Resolve → plan → execute.** nous resolves (system/marketplace/community/
  flutter); ark builds a typed `InstallPlan`; `--dry-run` shows the concrete
  commands, `--apply` runs them, plan-only (default) just describes. Privileged
  steps escalate through **shakti** (ADR 0001); ark never holds privilege.
- **System backend (ADR 0002).** `apt` / `apt-agnos` (wrapper-fronted) / `native`
  via `ArkConfig.system_backend` (`--system-backend`, TOML); the mode picks the
  inner command, shakti still escalates. agnos defaults to `native`.
- **Native `.ark` packages.** Read + verify (SHA-256 root hash + Ed25519 sig +
  per-file hashes), materialize (zip-slip/symlink-hardened), register in
  `PackageDb`. Local file (`ark install ./pkg.ark`), marketplace
  (`name[@version] --marketplace <url>`, resolve-"latest"), or a resolved plan —
  all route to the same installer/remover, rollback-able.
- **Authoritative, persistent store.** `PackageDb` is the installed-set of
  record (ark never shells `dpkg-query`); it round-trips to disk (held/pins/
  files/hashes/provenance survive across processes; a truncated/corrupt load
  refuses to clobber).
- **`ark upgrade` / `ark update`.** Multi-source detection (apt via nous sysdb;
  marketplace via mela `/latest`), held/pin-respecting, applied + rollback-able.
- **Transactions** (begin/commit/rollback, source-aware reverse), **integrity
  checking**, **hold/unhold**, **pin/unpin** (version + source), **backup/
  restore**, **history**, **bazaar** local catalog browse, **shell completions**.
- **Cross-target.** Builds for x86_64-linux and (cross-compiles for) agnos; the
  syscall-divergent paths route through `src/portable.cyr`.

## Open

### Quality-gate caveats (carried)
- [ ] Coverage **metric** — `cyrius coverage` is non-actionable; coverage is
  evidenced by the suite (399 tests + fuzz), not a percentage.
- [ ] End-to-end harness — build `.ark` → install → verify with a stubbed shakti
  (install→verify is covered; build-via-takumi is CI/hardware).
- [ ] Benchmark trend stability across releases (`cyrius bench tests/ark.bcyr`).

### Marketplace & UX enhancements
Non-blocking polish on the shipped core. (resolve-"latest", resolved-step
install wiring, and marketplace update detection already shipped — 1.1.0/1.1.3.)
- [ ] Auto-**download** of a resolved-but-uncached package from a configured
  default marketplace (the executor's native-install path needs the `.ark` in
  the cache today).
- [ ] Verify the artifact against mela's **transparency log** (beyond the
  trusted-signer check).
- [ ] Typosquatting detection (Levenshtein) on resolution.
- [ ] **Mirror** support (fall over between configured marketplace mirrors).
- [ ] **Bazaar install path** (catalog browse done; wire to download + installer).
- [ ] Progress bar / spinner during downloads + apply.
- [ ] Package rating & reviews; dependency conflict-resolution UI; namespace
  scoping for dependency-confusion defense (mostly nous-side).

## Post-v1: the AGNOS / sovereignty track

AGNOS-gated work. ark is the pivot; the path is captured in code + ADRs. The M0–M3
host-side seam shipped in 1.1.0 (see Shipped); what remains is on-hardware
execution and the native-source producer chain.

### Live privileged execution (on AGNOS hardware)
- [ ] Real `--apply` on a target with apt + setuid shakti, end to end, incl.
  rollback execution.
- [ ] **`apt-agnos` bridge** ([ADR 0002](../adr/0002-package-source-model.md)) —
  apt fronted by the AGNOS syscall wrapper, so AGNOS can use ark before native
  lands. (ark routes apt through the configurable wrapper; AGNOS owns the shim.)
- [ ] Plan signing for shakti verification.
- [ ] On-agnos runtime validation (QEMU + iron) of `.ark` fetch+install
  (M3 acceptance — *compiling ≠ working*). Blocked for symlink-bearing `.ark` by
  the absent agnos symlink syscall (filed agnos 1.51.x (a)); scope to
  symlink-free fixtures until it lands.

### Native, apt-independent package management ("ark v2 sovereignty path")
**Goal: AGNOS owns its package layer end to end — no dependency on Debian's
apt/dpkg.** apt becomes an optional compat shim; the native path (zugot →
takumi → signed native `.ark` → nous → ark) becomes the primary system source.
See [ADR 0002](../adr/0002-package-source-model.md).

> **Why this is a sequenced arc, not a wishlist:** agnova (the native installer)
> can't install anything *right* without a sovereign package manager beneath it —
> on a no-apt box, `agnova install <x>` is meaningless unless this chain
> resolves/fetches/verifies/materializes natively. It's on the **critical path**
> (`sovereign ark → agnova → server-stage exit`). Full cross-repo orchestration
> spine + the milestone analysis:
> [`agnosticos/.../ark-v2-sovereignty-path.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/ark-v2-sovereignty-path.md).
>
> **apt disposition (verified):** **KEEP apt behind the `system_backend` seam — do
> NOT delete it.** It's dead-by-construction on agnos (`nous/sysdb.cyr` gates the
> system leg on `which_exists("apt-cache") && which_exists("dpkg-query")`; agnos
> has neither). The sovereign cutover is a **default-flip**, not a removal;
> deleting breaks the Debian dev/CI loop for zero gain. (ADR 0002.)

ark's remaining share (resolution stays in nous; build in takumi):
- [ ] **M4 — AGNOS-side update mechanism.** *Host slice shipped (1.1.1–1.1.3):*
  `ark upgrade` is real apply-on-native with apt + marketplace detection.
  *Open:* the **whole-system atomic image swap** (re-materialize the system
  image), gated on an agnos/gnoboot boot-slot or atomic-image-swap primitive
  (absent — filed agnos 1.51.x (b)); A/B-slot-vs-in-place is an open design call.
- [ ] **Native-source update detection** — plugs into the same `avail_fn` seam
  (1.1.3) once nous ships the `SOURCE_NATIVE` index.
- [ ] **Downgrade-on-rollback** for upgrades (vs the current remove) — the
  `OP_UPGRADE` + `from_version` txn slots exist; needs the prior artifact cached.
- [ ] **Compat & migration** — apt demoted to `--source apt`; one-time dpkg-set
  import into the native store (Debian-host convenience).

> **Producer gates (not ark's — tracked in their repos):** the `SOURCE_NATIVE`
> resolver / signed index / lockfile → **nous**; recipe portability + native
> index format → **zugot**; the sovereign build-step executor (M5) + self-host
> "build agnos on agnos" (M6) → **takumi**. ark is the wired-and-ready consumer:
> the native store stays empty until takumi builds a real zugot recipe → `.ark`
> and nous resolves it as `SOURCE_NATIVE`.

## Future (speculative)

- Plugin system for custom sources
- Remote management API
- Metrics and telemetry (opt-in)
- Offline mode with cached packages
