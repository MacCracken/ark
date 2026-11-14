# Development Roadmap

> Per-release detail lives in [CHANGELOG.md](../../CHANGELOG.md). This roadmap
> tracks the *shape*: what has shipped, what's open, and the forward AGNOS /
> sovereignty track. Current release: **1.4.3**.

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
- **1.4.3 (2026-11-13) — cyrius 6.6.20.** sigil + sandhi become stdlib leaves; nous /
  mela / agnostik on their last 6.6.20-pinned tags; `path` overrides dropped. The
  6.6.20 warnings led to three live bugs: config-file bools and `system_backend`
  ignored (unterminated TOML slices), the transaction log never reloading, and a
  heap overflow on any config / log / backup file past 64 KiB.

**Deps (1.4.3):** cyrius 6.6.20; nous 1.4.1, mela 1.0.5, agnostik 1.7.1 (git);
sigil 3.13.10, sandhi 1.10.8, sankoch 2.8.1, bayan 1.5.12 (cyrius stdlib).

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

## Moving the cyrius pin to 6.6.5 — pin moved to 6.6.20 at 1.4.3

This section lists only what 6.6.5 itself changes.

- [ ] 6.6.5 honours the `--check` in `cyrius doc src/main.cyr --check` (the
  cleanliness step at `CLAUDE.md:26` and `:39`). `src/main.cyr` defines no
  fns, so the step still checks nothing. The 12 other modules in `src/` carry
  305 undocumented fns. Point the step at the modules (`doc` takes 1..N files
  from 6.6.5), and document them or record a baseline. See the cyrius
  CHANGELOG [6.6.5] entry "The whole CLI took flags as file names, dropped
  flags written after the operand, and dropped extra operands".
  **Baseline at 1.4.3 (6.6.20):** `cyrius doc src/*.cyr --check` reports 91
  documented / 305 undocumented and exits non-zero; `src/main.cyr` alone is
  still 0 / 0.
- [x] At the bump, re-run `cyrius deps` — the aarch64 syscall peer moved
  SYS_UNLINKAT 35 → 263, so an un-re-vendored peer's sys_unlink would run
  nanosleep. Done at 1.4.3 (`cyrius lib sync --full --relock` + `cyrius deps`).

## Open

### Found at 1.4.3
- [ ] **Persist transaction ops** so `ark rollback` works across processes.
  `txn_log_persist` writes the op *count* only; a reloaded transaction has no
  ops, and since 1.4.3 (when reload started working) `ark rollback` refuses one
  rather than reporting a zero-step success. Needs each op's type / package /
  version / source / from-version in the JSONL record, plus a reload test.
- [ ] **zugot `marketplace/ark.cyml` is at 0.8.0** — never synced through 1.x.
  Needs the 1.4.3 release asset's sha256, so it follows the tag.

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

## Moving the cyrius pin to 6.6.6 — pin moved to 6.6.20 at 1.4.3

✅ Verified at 1.4.3: the suite passes (434 / 0, exit 0), and two real
`ark install --apply --root <dir>` runs leave two records in the root's
`transaction.log` (journal appends, ids continue `txn-000001` → `txn-000002`).
The rollback leg of the cycle below cannot run across processes yet — see
"Found at 1.4.3" under Open.

The transaction log is the only part of ark that touches the 6.6.6 Windows fix.
`src/transaction.cyr:198` opens it with
`file_open(lpath, O_WRONLY | O_CREAT | O_APPEND, 420)`, then `file_lock` / `file_write` /
`ark_fsync` / `file_unlock` — a journal, which is the worst case for the pre-6.6.6 PE bug
where `O_APPEND` overwrote from offset 0 instead of appending. **ark is not exposed**: no PE
target (`CYRIUS_TARGET_AGNOS` is the only `CYRIUS_TARGET_*` in `src/`), CI is `ubuntu-latest`
only, and `release.yml` produces no `windows-*` job, `.exe` asset or `--win` build. So this is
a forward-looking note rather than a fix: if ark is ever built for Windows, 6.6.6 is the
floor, and the transaction journal is the first thing to re-verify.

Otherwise this is a plain pin bump — nothing in ark needs to change first. What was checked:

- `O_APPEND` / `O_TRUNC` appear nowhere else outside the vendored `lib/`; the transaction log
  above is the single site.
- ark has the largest `file_exists` / `file_read_all` surface in this group (48 call sites),
  but 6.6.6's "no longer request write access, so they succeed on read-only files and volumes"
  change is PE-side only, so it produces no behaviour change on the targets ark ships.
- None of item 3's new compile errors have sites: no `async fn`, no `operator` fn, no
  `ret2`/`rethi`, no SIMD intrinsics, no struct declarations in `src/` at all, and therefore
  no struct/vector copies, pair returns or struct-valued calls at top level. Item 5's
  by-value-struct-param deep copy has nothing to act on either — there is not one
  struct- or vector-typed parameter or `var` declaration in the repo's own sources.
- No top-level bare `{` blocks (item 4). No duplicated global declarations (item 6). No
  `regression_*` call sites of its own — the two `include` hits are `lib/regression.cyr`
  pulling `lib/regression_agnos.cyr` (item 8). No own `vec_*` function colliding with the 14
  names `lib/vec.cyr` exports, so the new transitive `lib/assert.cyr` → `lib/vec.cyr` include
  is inert (item 9).

After bumping, verify: the full `.tcyr` suite passes per-file (exit code, not just the grep
summary), and one `ark install` → `ark upgrade` → rollback cycle to confirm the transaction
journal still accumulates records rather than replacing them.

## Recorded by cyrius 6.7.6 (2026-10-08) — for the next cyrius pin move

⛔ Needs cyrius >= 6.7.6 — nothing to do until cyrius 6.7.6 is tagged; then at your pin bump:

- **Git first: a `path` beside `git` / `tag` becomes a dev override** (here: `nous`, `mela`,
  `agnostik`; sigil and sandhi are stdlib leaves since 1.4.3). ark 1.4.3 already dropped every
  `path` line (under 6.6.x it won locally), so at 6.7.6 there is nothing to remove; add `path`
  back only if you want local mode.
  From 6.7.6 it is read only in local mode (`CYRIUS_LOCAL=1`, `CYRIUS_LOCAL=<dep,dep>` or `--local`).
  With no switch, `cyrius deps` / `build` resolve the TAG (commit pin checked) and print one `hint:`
  line naming the checkout not used, so a local build and CI compile the same code.
- Local mode prints a `local:` line per override and builds from `build/local-deps/` without writing
  `lib/` or `cyrius.lock`; a hand-written `include "lib/<dep>.cyr"` sees the override there too (the
  compiler's `CYRIUS_LIB_OVERLAY`, set by `cyrius`).
- The first default resolve adds commit pins for these deps to `cyrius.lock` — a one-time diff.
- CI lock guards and `sed`-the-path-out steps can become `cyrius deps --locked`.
- Reference: the cyrius guide's *Git first; local development is a switch (6.7.6)*.

## Moved from cyrius's roadmap (2026-10-08) — for the next cyrius pin move

cyrius no longer carries this repo's follow-ups; the items below were moved here from its roadmap. Each
applies from the cyrius version it names. Everything else that changes between this repo's pin and cyrius
6.7.6 is in cyrius's `CHANGELOG.md` and `docs/development/ecosystem-migration.md`.

- [x] **(cyrius ≥ 6.6.17)** `[build] defines` is read by every `cyrius build`, so the `-D` flags on this repo's
  `cyrius build` CI lines are redundant. Dropped at 1.4.3 (binary byte-identical either way).

## Adopting cmdit (CLI parsing)

Surveyed 2026-10-14 against cmdit 1.2.6 (1.3.0 has the same API). Demand-gated, not a forced
sweep, but ark is the case cmdit's single-pass `cmdit_dispatch` was built for: cmdit's README
and ADR 0002 name ark's double scan as the bug to fix. All argv handling is in `src/cli.cyr`; the
`getenv` calls (`cli.cyr:280,287`, `engine.cyr:16`, `exec.cyr:20`) read config, not flags.

**Hand-rolled today**

- `parse_args` (`cli.cyr:5-228`): a `streq` chain over 18 verbs + 4 aliases (`uninstall`, `ls`,
  `show`, `baz`), each with its own `argv(i)` loop. Its `av` parameter is unused (`cli.cyr:389`).
- **The double scan.** Pass 1 skips only a *leading* `--no-color` to find the verb
  (`cli.cyr:11-12`); pass 2 in `main` re-walks all of argv for `--no-color` / `--apply` /
  `--dry-run` / `--system-backend <v>` (`cli.cyr:392-411`). The passes disagree:
  `ark --dry-run install x` is "Unknown command"; `ark search x --no-color` and
  `ark list --dry-run` are "Unknown flag" (`cli.cyr:83,98`) while install/remove/upgrade carry
  no-op arms (`cli.cyr:29-30,61-62,117`); verbs that skip `-` tokens keep a global's *value* as a
  positional (`ark hold foo --system-backend native` also holds `native`, `cli.cyr:128`;
  `rollback` targets txn `native`, `cli.cyr:189-192`); `bazaar` takes every token as its query
  while pass 2 still applies globals inside it (`cli.cyr:207-218`); `--apply --dry-run` is
  last-wins (`cli.cyr:395-396`). Audit M4 (`docs/audit/2026-04-16-p-1-security-audit.md:169`)
  was a bug in the same leading-skip.
- No `--help`, `--version` or `--`: `ark --help` prints "Unknown command. Available: …", exit 1.
  Parse errors go to **stdout** (`println`), exit 1 (`cli.cyr:390`). A lone `-` is a flag.
- `ARK_VERSION_STR = "1.4.1"` (`types.cyr:19`) has drifted from `VERSION` (1.4.2); fix it before
  `--version` prints it.

**Site → cmdit**

- Globals on the top handle, consumed wherever they appear: `cmdit_bool` × 3 (`no-color`,
  `apply`, `dry-run`) and `cmdit_enum(h, 0, "system-backend", 0, "apt|apt-agnos|aptagnos|native", …)`
  mapped to `BACKEND_*` caller-side, overriding config only when `cmdit_seen`. Deletes pass 2 and
  the no-op arms.
- 18 `cmdit_verb` rows in today's order + `cmdit_verb_alias` × 4; switch on `cmdit_verb_matched`
  (stiva's append-only `enum` pattern, `stiva/src/main.cyr:46-127`).
- Per-verb handles via `cmdit_parse_verb`: install `-f/--force`, `-g/--group`, `--root`,
  `--marketplace <url>`, `--dir`, `-y/--no-confirm`; remove `--purge`; search `-s/--source` as a
  `cmdit_enum` over every `parse_source_str` spelling (`types.cyr:50-58`); list `--marketplace` /
  `--system` / `--flutter`; pin `--source`. Scopes keep install's string `--marketplace` apart
  from list's bool.
- `cmdit_require_positionals(vh, 1)` for remove/hold/unhold/pin/unpin/info/restore. Caller-side
  still: install's "≥1 package unless `--group`/`--dir`" (`cli.cyr:49-52`), search's query join,
  `history [count]`'s `str_to_int`.
- `bazaar`: keep `<subcmd> [query…]` as positionals and `ark_bazaar`'s own sub-dispatch
  (`bazaar.cyr:129-241`); nested re-entry would turn today's exit-0 usage line for an unknown
  sub-command into a cmdit error.
- `cmdit_help`, `cmdit_version(h, ARK_VERSION_STR)`; `cmdit_print_error` lists commands from the
  table on an unknown verb (replaces `cli.cyr:224-226`).

**Gaps (none block)**

- No long-name flag alias: list's `--market` / `--apt` (`cli.cyr:95-96`) need second
  `cmdit_bool` rows OR'd caller-side, both shown in help. Candidate append-only cmdit addition if
  another repo needs it.
- Presence, not order: `--apply`/`--dry-run` and list's three source bools lose last-wins. Pick
  a rule (reject both, or dry-run wins); mutex flags are cmdit v2 backlog.
- `cmdit_completions` sees only the top handle (verbs + globals), not the per-verb flags
  `completions/ark.bash:17-33` completes by hand. Those scripts are stale anyway (12 of 18 verbs,
  none of `--apply`/`--dry-run`/`--system-backend`/`--root`/`--marketplace`/`--dir`/`-y`).

**Migration**

- [ ] `cyrius.cyml`: no stdlib `flags` to drop; `args` + `io` already present. No pin bump
  (cmdit's dist compiles on 6.6.2). 1.3.0 is untagged as of 2026-10-14; 1.2.6 is the same API.
  ```
  [deps.cmdit]
  git = "https://github.com/MacCracken/cmdit.git"
  tag = "1.3.0"
  modules = ["dist/cmdit.cyr"]
  ```
- [ ] Rebuild `parse_args` + `main`'s pass on `cmdit_dispatch_argv` (pure) so tests drive
  synthetic argv; `main` calls `cmdit_dispatch`.
- [ ] Tests: `test_parse_args` (`tests/ark.tcyr:34`) only builds `ArkCommand`s and never calls
  `parse_args` — ark has **no** CLI parse test. Add synthetic-argv cases for every double-scan
  row above. `fuzz/ark.fcyr:3` claims "CLI argument parsing" but `fuzz_source_parse` (`:77-83`)
  fuzzes `parse_source_str` only — add a dispatch target or fix the header. Regenerate or
  hand-update `completions/`.
- [ ] Output identity: there is no golden, so capture one first — stdout, stderr and exit code
  from `build/ark` (`ci.yml:48`) for every verb × each global before/after/absent, plus every
  error path. Success paths stay byte-identical; error-path changes (stdout → stderr, exit 1 → 2,
  message text) are a decision recorded in the CHANGELOG, or preserved by switching on
  `cmdit_error(h)`.

References: cmdit `README.md` (API reference) and `docs/adr/0002-verb-dispatch.md`
(identify-and-return, nesting as re-entry); stiva `src/main.cyr` is the worked verb tool
(36 verbs, per-verb handles, completions).

**Update — cmdit 1.3.x / 1.4.0 (2026-10-14).** Two of the gaps above are closed:

- **Long-name flag alias (1.3.6):** `cmdit_long_alias(vh, idx, "market")` gives list's `--marketplace` a second
  name, and the same for `--system` / `--apt`. Both parse cores match it exactly, help shows
  `--marketplace, --market`, and bash completion offers both. No second `cmdit_bool` row is needed.
- **Per-verb flags in completions (1.3.7):** `cmdit_verb_flags(h, verb_id, vh)` attaches each verb's handle.
  bash, zsh and fish then complete that verb's flags once it is typed, so `completions/ark.bash:17-33` and the
  zsh/fish scripts can be generated from the handles that parse them. Aliases (`uninstall`, `ls`) select their
  verb's list. `cmdit_verb_flags_at(h, cmdit_verb_matched(h))` returns the matched verb's handle after dispatch,
  so the per-verb handles can be built once, up front.

Not generated: the value lists `completions/ark.bash` completes after `--source|-s` and `--group|-g`, and
global flags in zsh and fish. Those stay hand-written, or become a cmdit request.

Also since the survey: `--bool=value` is refused with `CMDIT_ERR_UNEXPECTED_VALUE` (1.3.1); `cmdit_print_error`
names the offending token (1.3.3); `cmdit_help_fd(h, 1)` prints help on stdout (1.3.5). 1.4.0 changes no API or
output; it makes a parse about 2× faster and `--help` over 10× faster. Presence-not-order (`--apply` / `--dry-run`)
is unchanged; mutex flags are still on cmdit's v2 backlog. Pin `tag = "1.4.0"` instead of `"1.3.0"` in the
block above.
