# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed
- **Removed a latent last-def-wins shadowing hazard against the vendored
  `lib/nous.cyr`.** `src/recipe.cyr` defined `parse_toml_array` and
  `recipe_parse_file`, whose names collided with nous's own
  `parse_toml_array(s): i64` / `recipe_parse_file(path): i64` (different
  signatures/bodies — nous returns a raw `i64`, ark returns a `Result`). Cyrius
  resolves duplicate `fn` names last-def-wins with only a warning, so behavior
  was correct only as long as ark's definitions happened to link last; a link-
  order flip would have silently bound ark's call sites to nous's raw-`i64`
  bodies. Renamed ark's copies to `ark_parse_toml_array` / `ark_recipe_parse_file`
  and updated all ark call sites (`src/` + tests); nous's internal calls now
  correctly bind to its own definitions. No behavior change — suite green 408/0.

## [1.4.0] — 2026-07-03 — `$ARK_ZUGOT_PATH` override + cyrius 6.3.38

### Changed
- **Toolchain: migrated to cyrius 6.3.38** (`cyrius.cyml` pin + `.cyrius-toolchain`;
  `cyrius lib sync --full` re-vendored the stdlib snapshot, which also pulls in
  bayan 1.0.4 — TOML `"""` multi-line support). Suite green: 408/0.

### Added
- **`$ARK_ZUGOT_PATH` override for the community RecipeDb path.** The zugot
  recipe tree ark resolves community (`SOURCE_COMMUNITY`) packages from
  defaults to `/usr/share/zugot`; it can now be pointed elsewhere via the
  `ARK_ZUGOT_PATH` environment variable — but **only when unprivileged**
  (euid != 0), the same local-injection guard `load_config` applies to
  `$ARK_CONFIG` (audit C4). A set-but-invalid override (missing directory)
  falls back to the default rather than silently disabling recipe resolution.
  This makes the sovereign install-by-name path (`zugot → takumi signed .ark →
  nous resolve → ark cache install`) testable against an alternate recipe tree
  without mutating system paths. Proven end-to-end on the host: a takumi-built
  signed `.ark` installs via `ark install <name> --apply --root <target>`,
  files + DB record landing in the target root, host `/var` untouched.

## [1.3.0] — 2026-07-02 — sovereign `--root` installs (record + files into the target) + batch `--dir` mode

### Added
- **`ark install --dir <pkgdir>` — batch-install every `.ark` in a directory**
  (+ `--no-confirm` / `-y` to skip the confirm prompt). Each package installs into
  `--root` (files staged under it + recorded in the target's own DB), non-`.ark`
  entries are skipped, and it fails closed (a non-zero exit) if any package fails.
  This is the call **agnova** makes to install a directory of pre-staged base
  packages in one shot: `ark install --apply --no-confirm --root <target> --dir
  /run/agnos/installer/packages/`. New `ark_install_dir` + `acmd_dir` /
  `acmd_no_confirm` command fields. +1 test (`install_dir_batch`, 408 total).

### Fixed
- **`--root` installs now record into the TARGET root's package DB + transaction log**
  (`src/cli.cyr`, `src/db.cyr`, `src/transaction.cyr`, `src/types.cyr`). Previously
  `ark install --root <dir> <.ark>` staged the installed FILES under `<dir>` (via
  `native_root`) but wrote the DB record + txn log to the HOST `/var/lib/agnos/ark`
  (or silently no-op'd on a read-only host `/var`) — so an install into an agnova
  target root left the target with no package DB of its own. The CLI dispatch now
  root-prefixes `acfg_db_path`/`acfg_log_path` via `apkg_join` before `ark_mgr_new`,
  so `<root>/var/lib/agnos/ark/{packages.json,transaction.log}` are written. Verified
  across plain / trailing-slash / deep-nested roots; host `/var` untouched; a 2nd
  install reads the target DB back and appends to its log (real round-trip). New
  `acfg_set_db_path` / `acfg_set_log_path` setters.
- **Name-based `--root` installs now stage their FILES into the target root too**
  (`src/engine.cyr`). `ark install --root <dir> <name>` installs via the resolved-step
  executor, which stages files at `config.native_root` — but nothing set it, so a name
  install ignored `--root` and materialized into the LIVE root (the file-side twin of
  the DB bug above; only the single-`.ark` path honored `--root`). `ark_execute` now
  wires `--root` → `native_root` for the name + group install paths via a new
  `_apply_native_root` helper; the single-`.ark` / marketplace branches pass their
  root explicitly and don't read `native_root`, so they're unaffected. +1 test
  (`install_root_wiring`, 405 total).
- **`pkg_db_save` / `txn_log_persist` no longer silently drop the record when the DB
  dir is absent.** `O_CREAT` makes the file, not its parent; on a fresh (or freshly
  `--root`-ed) system `/var/lib/agnos/ark` doesn't exist, so the open failed `fd<0`
  and `pkg_db_save` `return 0`'d while reporting the package installed. Both writers
  now `apkg_mkdir_parents` the (target-rooted) parent dir first.
- Source hygiene: removed a stray NUL byte from a `src/db.cyr` comment (it made
  `grep` treat the file as binary).

### Known
- `--root` is parsed only by `install`; query commands (`list`, `info`) reject it, so
  a target's DB can't yet be inspected host-side via `ark list --root <dir>` (the
  target's own ark reads it correctly once booted). Making `--root` a global flag is a
  follow-up.

## [1.2.0] — 2026-07-02 — cyrius 6.3.35 toolchain migration + nous 1.3.1

### Changed
- **Toolchain migrated to cyrius `6.3.35`.** `cyrius.cyml` pin `6.3.9` → `6.3.35` and
  `.cyrius-toolchain` `6.3.5` → `6.3.35` (both were stale and mutually inconsistent; CI
  reads `.cyrius-toolchain`). Full gate green on 6.3.35: fmt/vet/capacity clean, the
  seven linted `src/` modules clean, **suite 403 passed / 0 failed**.
- **`nous` dependency `1.3.0` → `1.3.1`** (`cyrius.cyml` [deps.nous]; `cyrius.lock`
  regenerated).

### Fixed
- **Resolver-path SIGSEGV on cyrius ≥ 6.3.13 (via the `nous` dep).** Building ark on
  6.3.13+ detonated a stack-smash in nous's `our_is_dir` (`registry_new → our_is_dir`,
  reached by `resolver_new`): nous sized its `struct stat` buffer `var statbuf[18]`
  (18 u64 slots under the pre-6.3.13 heap-local model, but 18 **bytes** on the stack
  since 6.3.13), so `sys_stat`'s 144-byte write overran it. It crashed ark's
  `nous_integration` test at teardown. Fixed upstream in **nous 1.3.1** (`statbuf[144]`);
  ark re-resolves the corrected `dist/nous.cyr`. No ark source change was required —
  ark's own code carries no undersized stack buffers.

## [1.1.6] — 2026-06-30 — install fails loudly: no more silent no-op success

### Fixed
- **`ark_pkg_install` now fails loudly when a write fails — install can no longer
  report success on a no-op** (closes the robustness gap flagged in 1.1.5). Pass 1
  and pass 2 of `ark_pkg_install_inner` ignored the return codes of `apkg_mkdir_p`
  / `apkg_mkdir_parents` / `file_write_all` / `ark_symlink`, so a fully-failed
  materialize — the pre-fix M3 `ark_mkdir` ABI bug, or simply an unwritable install
  root — still returned `Installed <pkg> (N files)` and exit 0. Each
  directory-create, file-write (must write exactly `size` bytes), and
  symlink-create is now checked; on any failure the install rolls back the files
  already laid down (via the tracked `files` vec, `ark_install_rollback`) and
  returns an `ark_result` failure (`ark: install failed — cannot …`), so the CLI
  prints the error and exits non-zero. `apkg_mkdir_p` / `apkg_mkdir_parents` now
  return 0/-1 (was always 0); since mkdir's return for an existing dir is
  target-specific (Linux `-EEXIST` vs agnos idempotent `0`), a non-zero leaf mkdir
  is confirmed against the directory actually existing (`is_dir`) before being
  treated as failure. New regression test `test_ark_install_failure` points an
  install at an uncreatable root (a path under a regular file → `ENOTDIR`) and
  asserts the result is a failure with nothing registered or materialized.

## [1.1.5] — 2026-06-30 — ark v2 M3 PROVEN: on-agnos `.ark`-with-symlinks install round-trips

The full M3 milestone: a prebuilt `.ark` carrying a symlink installs on agnos and
the symlink lands on the agnos-fs. Verified by `agnos/scripts/ark-install-smoke.sh`
(`ARK_INSTALL_SELFTEST` runs `ark install --root /arkroot <.ark>` in QEMU): the
regular file `/arkroot/lib/libfoo.so.1` (19 B) and the symlink
`/arkroot/lib/libfoo.so` → `libfoo.so.1` both land, `e2fsck -fn` clean. Fixture
built by takumi `tests/mkfixture` (`create_file_list` + `ark_write`).

### Fixed
- **`ark_mkdir` portable shim — install created NO directories on agnos** (the M3
  "install reports OK but nothing lands" bug). `apkg_mkdir_p` called the raw 2-arg
  `sys_mkdir(path, mode)` (host ABI); agnos's `sys_mkdir`#9 is `(path, pathlen)` —
  no mode — so the mode `0755` (=493) was read as `pathlen`, a bogus length → every
  mkdir failed → no install dirs → every file/symlink write under them failed. Same
  ABI-skew class as `ark_symlink`. Added `ark_mkdir(path, mode)` in `portable.cyr`
  (`#ifdef CYRIUS_TARGET_AGNOS` → `sys_mkdir(path, strlen(path))`, dropping mode;
  host unchanged) and routed both `apkg_mkdir_p` call sites through it. With it, the
  on-agnos install lays the directory tree, the file, and the symlink correctly.

### Notes
- **Robustness gap surfaced (not yet fixed): `ark_pkg_install` ignores the return
  codes of `apkg_mkdir_parents` / `file_write_all` / `ark_symlink` (pass 1/2), so it
  reported `Installed (2 files)` + exit 0 even when EVERY write failed** (pre-mkdir-fix).
  A failed install should fail loudly. Tracked as a follow-on hardening.

## [1.1.4] — 2026-06-30 — symlink-create wired to the agnos peer (ark v2 M3 (a))

The agnos `symlink`#63 syscall is now live on both sides (kernel 1.51.0 + cyrius
`sys_symlink` peer 6.3.6), so ark's install pass-2 can create symlinks on agnos —
the ark v2 item (a) the M3 on-agnos `.ark` install needs. The on-agnos `sys_symlink`
round-trip is proven (`agnos/scripts/symlink-smoke.sh`).

### Changed

- **`src/portable.cyr` — `ark_symlink` routes to the 4-arg agnos peer** under
  `CYRIUS_TARGET_AGNOS` (`sys_symlink(target, strlen(target), linkpath,
  strlen(linkpath))`, mirroring `ark_rename`/`ark_unlink`), replacing the
  `return -1` stub. **`ark_symlink_supported()` now returns true on agnos** (was
  0 while the syscall was absent), so a symlink-bearing `.ark` is no longer
  refused before install on agnos.
- **cyrius pin `6.3.5` → `6.3.9`** + `cyrius lib sync` re-vendored the stdlib
  (the vendored `lib/syscalls_x86_64_agnos.cyr` predated 6.3.6 and lacked
  `sys_symlink`, so the agnos cross-build failed `undefined function 'sys_symlink'`
  until the sync). The agnos cross-build now compiles (15.9 MB).

### Notes

- **The 15.9 MB ark binary now LOADS + RUNS in ring 3 on agnos** (agnos
  `ark-run-smoke.sh` PASS — ark parses argv, prints its usage, exits clean). It
  first triple-faulted; the agnos kernel fix relocated the per-proc + per-CPU
  kernel stacks onto the direct-map (a kstack VA-space collision, NOT an ark-size
  problem — ark was not shrunk). See
  `agnos/docs/development/issues/2026-06-30-large-binary-exec-stack-overflow.md`
  (RESOLVED). The remaining ark M3 step is the actual on-agnos `.ark`-with-symlinks
  install round-trip (symlink-bearing fixture via takumi `ark_write` + `ark install`).

## [1.1.3] - 2026-06-29 — `ark upgrade` detects marketplace updates (nous 1.3.0)

Closes the M4 update-**detection** loop on the consumer side: `ark upgrade` now
finds marketplace/community updates, not just apt — built on nous 1.3.0's
pluggable detector and ark's now-persistent `PackageDb` (1.1.2).

### Changed

- **nous `1.2.7` → `1.3.0`** — picks up `resolver_check_updates(r, installed,
  avail_fn)`, the unified multi-source update detector.
- **`ark upgrade` / `ark update` use multi-source detection.** New
  `ark_check_updates(mgr)` hands nous the installed set built from ark's
  `PackageDb` (the marketplace/community packages it actually installed) plus an
  `avail_fn` that resolves each package's latest version from the configured
  marketplace via mela's `/latest` (`ark_avail_lookup` → `ark_marketplace_resolve_latest`).
  nous owns the semver comparison; apt detection stays internal to nous. So
  `ark upgrade` now surfaces a marketplace package whose published version is
  newer than the installed one, and the upgrade applies it through the same
  backend-aware executor + rollback path (1.1.1).

### Added

- **`marketplace_url` config field** (`ArkConfig`, TOML key `marketplace_url`) —
  the default marketplace base URL update detection queries. Empty (default) ⇒
  detection stays apt-only, so existing behavior is unchanged until a marketplace
  is configured.
- +8 tests (399 total): `marketplace_url` default/set-get; `ark_avail_lookup`
  returns 0 for a nil URL, a system source, and an offline marketplace (graceful,
  no crash); `ark_check_updates` keeps a marketplace package out of the update set
  when no URL is configured; a held package is excluded from the query set; and a
  stub `avail_fn` drives the positive path (a newer version yields a detected
  update read back via `nupd_*`). Adversarially reviewed (8 findings, all low; no
  correctness bugs — core wiring verified). Host + agnos builds, bench, fuzz green.

### Fixed (from review)

- **Held packages are no longer queried.** `ark_check_updates` builds its query
  set via `ark_installed_for_updates`, which skips held packages — avoiding a
  wasted mela `/latest` round-trip per held marketplace package and not leaking
  held-package names to the server. (Pin filtering still happens in the plan
  builder, since it needs the resolved available version.)
- Removed the now-dead `nous_check_updates` wrapper (all callers moved to
  `ark_check_updates`).

### Notes

- Update detection now reaches the network (mela `/latest` per installed
  marketplace package) **only when a `marketplace_url` is configured**; with none
  set it makes no network calls. Native-source detection plugs into the same
  `avail_fn` seam once nous ships the `SOURCE_NATIVE` index.

## [1.1.2] - 2026-06-29 — PackageDb persistent round-trip (the reload fix)

Closes the pre-existing bug that quietly undercut everything downstream: `PackageDb`
never reloaded from disk, so the installed-set — and holds, pins, and owned-file
records — vanished between CLI processes. This is the highest-leverage follow-up
from 1.1.0/1.1.1: it makes the native store **authoritative across processes** and
makes ark-managed **holds/pins actually persist**.

### Fixed

- **`pkg_db_load` now reconstructs entries** (it previously read only
  `schema_version` and returned an empty map). Root cause of why the obvious fix
  hadn't worked: bayan's DOM parser `json_v_parse(Str)` mis-dispatches under
  cyrius 6.3.5 — the `_str` overload routing reroutes it to
  `json_v_parse_str(Str, ?)` with a dropped `len`, so it reads 0 bytes and
  returns 0. The loader now calls `bayan_json_v_parse_str(str_data, str_len)`
  directly (and looks up object keys with raw cstr literals, the other
  marshaling rule), which parses the nested DB correctly.
- **`pkg_db_save` now persists the full entry** for a faithful round-trip — it
  previously dropped **pins, owned files, and per-file hashes**. Added to the
  serialization: `pinned_version` / `pinned_source` (with the `-1` "unset"
  sentinel emitted explicitly — `0` is a real source), the `files` array, the
  `file_checksums` object, plus `installed_at` / `installed_by` / `checksum`.
  Schema bumped **v2 → v3**; older files load with defaults for absent fields.
- **Large-DB truncation could destroy the database** (adversarial-review #2, the
  serious one). The old read capped at 512 KiB with no truncation signal — and
  v3 entries are much larger (full file manifests + per-file hashes), so a real
  system's DB can exceed it. A truncated read parses as empty, and the next save
  would then atomically overwrite the real file with `{}` — silent total loss.
  Now `pkg_db_load` grows the read buffer until it captures the whole file (64
  MiB ceiling), and if a load is ever truncated **or corrupt**, the DB is marked
  not-loaded and **`pkg_db_save` refuses to overwrite the on-disk file** (it's
  left intact). Also fixed the original 64 KiB-alloc/512 KiB-cap overflow.
- **`json_escape_str` control-byte escaping** (review #1) — control bytes
  `0x00–0x1F` other than `\n\r\t` were all written as a literal `\0`,
  corrupting the real byte to NUL on reload (e.g. an exotic file path). They now
  emit the correct `\u00XX`.

### Result

- **Holds and pins now persist across CLI processes** for ark-managed
  (native/marketplace) packages — so `ark upgrade`'s held-skip + pin-skip (1.1.1)
  are now effective in a real run, not just in-process. (apt/system packages
  still live in dpkg, not ark's `PackageDb` — apt's `apt-mark` governs their
  holds; that's unchanged.)
- The **native store is authoritative across processes**: an install in one
  process is seen — with its source, owned files, and hashes — by the next.
- +25 tests (391 total): full save→load round-trip (held / pins / files /
  per-file hashes / provenance survive a fresh load; unset pin reloads as `-1`,
  not `0`; pin enforced after reload), the cross-process reload assertions
  deferred in 1.1.0/1.1.1 re-enabled, and DB-safety (a corrupt load refuses to
  clobber; control bytes escape to `\u00XX`). Host + agnos builds, bench, fuzz
  green. Adversarially reviewed (5 findings; 2 fixed in code, 1 documented).

### Notes

- **One-time migration cost:** loading a pre-v3 DB (which never stored
  `installed_at`) stamps each package's install time to the load time, since the
  original is unrecoverable. v3-written DBs round-trip `installed_at` correctly.

## [1.1.1] - 2026-06-29 — M4 (host slice): real `ark upgrade`

Turns `ark upgrade` from a plan-display stub into a real **apply-on-native**
upgrade — the un-gated, host-side half of M4. The whole-system atomic image
swap stays gated on the agnos boot-slot primitive (agnos 1.51.x (b)); this is
the per-package upgrade path, which rides the 1.1.0 machinery (the native `.ark`
installer + plan-wide transaction + source-aware `ark_rollback`) — an engine
swap under the already-proven package chassis.

### Changed

- **`ark upgrade` now applies.** It builds a real `InstallPlan` from the
  available updates (`ark_build_upgrade_plan`) and runs it through the executor:
  `--dry-run` shows the concrete commands, `--apply` upgrades in a rollback-able
  transaction, plan-only (default) describes. **Backend-aware** (ADR 0002): each
  update lowers to `apt-get install pkg=<available>` under `apt`, or the native
  `.ark` installer under `native` — so the same command follows the sovereignty
  cutover with no code change. Drops **held** and **pin-violating** ark-managed
  packages from the set (`pkg_db_is_held` / `pkg_db_upgrade_allowed`), and honors
  the **package filter** (`ark upgrade nginx curl` upgrades only those; bare
  `ark upgrade` does all). Source-aware step kinds mirror `ark install`.

### Added

- `pkg_db_is_held`, the nous update-object accessors (`nupd_*`), and `name_in_vec`.
- `CMD_UPGRADE` is covered by `needs_confirmation`, so a real `--apply` upgrade
  prompts (gated on `confirm_system_installs`).
- +20 upgrade tests (366 total): plan from updates, held-skip, pin-skip, package
  filter, source-aware + backend-aware lowering, empty-updates → empty plan, and
  upgrade-replace (no orphaned files).

### Fixed

- **`ark upgrade --system-backend <mode>`** no longer treats the mode value as a
  package to upgrade (the upgrade arg parser now skips the flag's value).
- **Version/source pins are enforced on upgrade.** `ark_build_upgrade_plan` now
  calls `pkg_db_upgrade_allowed` (previously dead code), so a pinned ark-managed
  package isn't moved off its pin by an upgrade.
- **Native upgrade no longer orphans the prior version's files.**
  `ark_pkg_install_inner` unlinks a previously-registered package's owned files
  before laying down the new version (shared paths are re-materialized), so files
  the new version drops don't linger.

### Notes

- **Update detection is apt-system today.** `nous_check_updates` parses
  `apt list --upgradable` (`sysdb_updates`), so on a Debian/apt host `ark upgrade`
  really upgrades; on agnos (no apt) it finds nothing until nous ships
  native/marketplace update detection (the same M2 producer gate). The
  apply/rollback mechanism is in place regardless of detection source.
- **Holds/pins gate ark-managed (native/marketplace) packages only.** apt/system
  packages live in dpkg, not ark's `PackageDb`, so ark can't hold/pin them —
  apt's own `apt-mark` is the mechanism there. And because `pkg_db_load` does not
  yet reconstruct entries (the pre-existing reload follow-up), hold/pin state set
  in one CLI process is not yet seen by a later `ark upgrade` process; the
  skip-logic is correct and becomes effective once that reload lands. The
  package **filter** works regardless (it needs no DB).
- **Rollback of an upgrade reverts the install (removes the package)** rather
  than restoring the prior version — `ark_rollback` reverses install→remove.
  True downgrade-to-previous needs from-version tracking on the step/op (the
  `OP_UPGRADE` + `from_version` slots exist) plus the prior artifact cached;
  noted as a follow-up.

## [1.1.0] - 2026-06-29 — AGNOS compatibility: the host-side AGNOS track (M0–M3)

Brings up ark's share of the 1.1.x AGNOS track end to end on the host — the
system-backend seam (M0), the native `.ark` installer wired into the executor
(M1), the native backend + authoritative store (M2 ark-share), and syscall
portability so the agnos cross-build compiles (M3) — and moves the toolchain to
**cyrius 6.3.5**. The remaining AGNOS-track work (the nous `SOURCE_NATIVE`
resolver, on-hardware QEMU validation, M4) stays future; the 1.0.x
marketplace/UX enhancement arc is pushed back behind this release.

### Added

- **System-backend seam (M0, ADR 0002).** `ArkConfig.system_backend` selects
  `apt` / `apt-agnos` / `native`; `exec.cyr`'s new `step_to_argv_be(step,
  backend, wrapper)` lowers each step accordingly — `apt-agnos` **wrapper-fronts**
  `apt-get` with a configurable `apt_wrapper`, `native` routes `SOURCE_SYSTEM`
  steps to ark's own installer. The mode decides the *inner* command; shakti
  still escalates. New `--system-backend <mode>` CLI flag and `system_backend` /
  `apt_wrapper` TOML keys. The 1-arg `step_to_argv` is retained as an apt-default
  convenience, so existing call sites and tests are byte-identical.
- **Native installer wired into the executor (M1).** `exec_plan` now routes
  marketplace/community (and native system) steps to ark's `.ark`
  installer/remover (`exec_native_step`) instead of the old "not yet wired"
  dead-end. A resolved plan installs from a **local `.ark` via `--apply`**,
  recorded in the **plan-wide transaction** so `ark_rollback` reverses it. New
  `ark_pkg_remove_inner` (store-driven native remove: unregister + unlink).
- **Resolve-"latest" (M1).** `ark install name --marketplace URL` with no
  `@version` resolves the concrete version from mela's manifest endpoint
  (`ark_marketplace_resolve_latest`, composing `build_latest_url` →
  `manifest_from_json` → `man_version`; mela 1.0.0, no new dependency).
- **Native backend + authoritative store (M2, ark's share).** Under
  `BACKEND_NATIVE`, `SOURCE_SYSTEM` packages install from ark's local `.ark`
  store into the **`PackageDb` — the authoritative installed set (ark never
  shells `dpkg-query`)**. The default flips to `native` on agnos
  (`#ifdef CYRIUS_TARGET_AGNOS`). The nous `SOURCE_NATIVE` resolver + signed
  index + lockfile are a producer gate (see Notes).
- **Syscall portability layer (M3, ADR 0003).** New `src/portable.cyr` with
  `#ifdef`-guarded shims — `ark_now_secs` / `ark_fsync` / `ark_rename` /
  `ark_unlink` / `ark_symlink` — so each operation resolves to the correct Sys
  number per target. The **agnos cross-build (`cyrius build --agnos`) now
  compiles**; it previously failed on the undefined `sys_symlink`.
- **+45 tests (292 → 337):** `system_backend` (M0), `native_apply` + resolve-
  latest (M1), `native_backend` (M2), `portable` (M3); host and agnos builds,
  bench, and fuzz all green.

### Changed

- **Toolchain pin `6.2.21` → `6.3.5`** (`.cyrius-toolchain`, `cyrius.cyml`):
  stdlib re-synced (picks up the arena-aware `tls_accept_*` sandhi needs), deps
  re-resolved. 6.3.5 promotes reachable-undefined functions to a hard error —
  the bench and fuzz harnesses were updated to include the source files defining
  the `cli.cyr` symbols they reference, and 3 files were re-formatted for the
  6.3.5 continuation-line rule.
- **Dependency alignment for 6.3.5.** sigil `3.8.0` → `3.9.7` (provides
  `sha384_init_into`, which the 6.3.5 stdlib TLS now requires, and drops the
  external **agnosys** dep — internalized via the agnosys→agnodrm decomposition;
  the stale vendored `lib/agnosys.cyr` is removed), sandhi `1.6.7` → `1.7.0`, and
  **mela `1.0.0` → `1.0.1`** (a patch that bumps mela's own transitive
  sigil/sandhi/sankoch pins to match, so the whole graph resolves on one sigil).
  Without this the clean-checkout CI build failed on an undefined
  `sha384_init_into` (sigil 3.8.0 was too old for the 6.3.5 TLS).
- **`ark_rollback` is now source-aware.** A reversed op uses the original op's
  source — a native/marketplace install rolls back through the native remover
  (not `apt`) and at the recorded version — instead of always emitting system
  (apt) steps.
- **`ArkConfig` grew** (`ACFG_SZ` 104 → 128): `system_backend`, `apt_wrapper`,
  and `native_root` (the executor's staging root for native installs, `""` =
  real root). Internal struct only — there is no serialized config format.

### Fixed

- **`confirm()` stack over-read** (`cli.cyr`) — it read 128 bytes into a
  16-byte stack buffer; capped the read at the buffer size (only the first
  reply character is used).
- **De-magic'd syscall literals** — the Linux-host raw numbers (`time 201`,
  `fsync 74`, `rename 82`, `O_WRONLY|O_CREAT|O_EXCL 193`) now go through the
  portable shims / named flags, so they invoke the right syscall on agnos
  instead of a wrong-numbered one.
- **Adversarial-review fixes** (found verifying the M0–M3 diff):
  - **Source attribution** — a native `SOURCE_SYSTEM` install no longer
    registers in `PackageDb` as `SOURCE_MARKETPLACE`; the installer takes the
    source from the step/caller (`ark_pkg_install_inner(..., source)`).
  - **Durable store writes** — install/remove now `pkg_db_save` after the
    register/unregister (previously only pin/hold persisted), and the secure
    temp write uses portable `file_open`/`file_write`/`file_close` (so the
    earlier M3 portability claim holds on the open path too, not just fsync/rename).
  - **Rollback covers every source** — flutter installs roll back via `agpkg`
    (not `apt`); and a remove records the installed version so a rollback-of-
    remove reinstalls at the right version instead of an empty-version cache miss.
  - **CLI/validation** — an invalid `--system-backend` value now errors (was
    silently ignored); resolve-"latest" validates the package name before it is
    concatenated into the manifest URL.

### Notes

- **Producer gate (nous).** M2's resolver half is **not** ark's to ship: nous
  1.2.7 (HEAD == 1.2.7, verified per-tag) has no `SOURCE_NATIVE`, no signed
  native index, and no lockfile. ark is the wired-and-ready consumer; the
  acceptance "apt absent → nous resolves a name to `SOURCE_NATIVE`" is
  unattainable until nous ships it (and the store stays empty until takumi
  builds a real zugot recipe → `.ark` and indexes it).
- **Deferred (M3, compiling ≠ working).** On-agnos QEMU+iron runtime validation
  is the remaining M3 acceptance gate. agnos has no `fsync` (durability is a
  no-op there) and no `symlink` syscall (filed agnos 1.51.x) — a `.ark` carrying
  a symlink entry is rejected on agnos until it lands; scope agnos fixtures to
  symlink-free.
- **Pre-existing gap discovered: `PackageDb` does not reload from disk.**
  `pkg_db_save` writes correct JSON, but `pkg_db_load` only reads
  `schema_version` and never reconstructs the `packages` object — so the
  installed-set does not survive across CLI processes. This predates 1.1.0 (it
  affected pin/hold persistence too) and is **not** introduced here; install/
  remove now at least write the store durably. The fix needs a working object
  parser (the bundled DOM JSON parser, `json_v_parse`, returns 0 in-build; only
  the flat key→string parser works) — tracked as a roadmap follow-up.

## [1.0.0] - 2026-06-18 — Cyrius port complete

ark's first stable release. The full package manager is in Cyrius, end to end:
**resolve → plan → execute**, native `.ark` packages, and a trust-anchored
marketplace path.

### Highlights — what 1.0 delivers

- **Plan-first execution backend** — `install`/`remove` build a typed plan;
  `--dry-run` shows the concrete commands; `--apply` runs them, escalating
  privileged steps through **shakti** (ark never holds privilege). ADR 0001.
- **Native `.ark` packages** — read + verify (SHA-256 root hash + Ed25519
  signature + per-file hashes), materialize, register in `PackageDb`. Install a
  local file (`ark install ./pkg.ark`) or from a **mela** marketplace
  (`name@version --marketplace <url>`), trust-anchored and fail-closed.
- **Resolution via nous**, recipes via takumi/zugot, transactions with
  begin/commit/rollback, integrity checking, hold/pin/verify/history/rollback/
  backup/bazaar.
- **Quality-gated (0.9.0):** full zugot-corpus parse (563/563), fuzz harnesses,
  real-nous integration tests, a CVE-grounded security audit + 4 bug fixes
  (recipe heap-overflow, zip-slip, decompression bomb, OOB reads).
- **Security-hardened (0.9.1):** symlink two-pass materialization, marketplace
  fail-closed signing, trust set from mela's keyring.
- **Docs reconciled** to the shipped surface (README, architecture overview,
  getting-started guide, worked install example).

### Deferred (post-1.0, documented in the roadmap)

- Marketplace enhancements: resolve-"latest", transparency-log verification,
  mirror support, ratings, typosquatting detection; `STEP_MARKETPLACE_INSTALL`
  wiring for resolved (no-URL) installs; progress UI.
- **1.1.x AGNOS track:** real `--apply` on AGNOS hardware (apt+shakti), the
  `apt-agnos` syscall-wrapper bridge, and native apt-independence (ADR 0002).
- Tooling caveat: `cyrius coverage` is non-actionable; coverage is evidenced by
  the test suite (292 tests + fuzz) rather than a percentage.

## [0.9.1] - 2026-06-18 — Security hardening follow-ups

Closes the audit follow-ups carried out of 0.9.0.

### Security

- **Symlink hardening (audit #5, CWE-22).** `ark_pkg_install` now materializes in **two passes** — directories + regular/config files first, symlinks last — so a malicious symlink entry can't be planted and then have a same-archive write redirected through it. Legitimate symlinks still install.
- **Marketplace installs force `require_signed`.** A marketplace install (untrusted source) now requires a trusted signer **regardless** of the global `require_signed` default — fail closed. (Global default stays off, so explicit local `ark install ./pkg.ark` is user-discretion.)
- **Trust set from mela's keyring.** `ark_trust_from_keyring` / `ark_trust_load_keyring` build ark's trust set from a mela publisher keyring (`mela_keyring_*` / `mela_kv_public_key_hex`), so `.ark` signatures anchor to the marketplace's publisher identities instead of a separately-provisioned key file.
- +3 regression assertions (292 total): keyring-sourced trust accepts the signer, empty keyring rejects, and an untrusted-signer marketplace install is refused. Audit findings #4 and #5 now resolved (`docs/audit/2026-06-18-pre-v1-audit.md`).

## [0.9.0] - 2026-06-18 — Quality-gate milestone

Caps the quality-gate milestone built across 0.8.11–0.8.15: full zugot-corpus
parse validation (563/563), fuzz harnesses (txn-log / package-name / recipe),
real-nous integration tests, documentation (guides + examples), a CVE-grounded
security audit, and — in this release — **trust-anchored package signing**.
Four real bugs were fixed along the way (recipe heap-overflow, zip-slip,
decompression bomb, OOB reads). Caveats carried to v1.0: the `cyrius coverage`
metric is non-actionable (real coverage rose via the added tests), and the
signing follow-ups below (default-on for marketplace, mela-keyring trust source,
symlink-target constraint).

### Security — trust-anchored signing (audit finding #4)

- **Trust-anchored `.ark` signature verification.** Previously the signature was verified only against the package's *own embedded pubkey* (self-signed passes) and unsigned packages were accepted — integrity, but no publisher authenticity. Now:
  - `ark_pkg_read` exposes the signer's pubkey (`apkg_pubkey`).
  - A configurable **trust set** of hex Ed25519 pubkeys (`ark_trust_load` from `ArkConfig.trust_keys`, default `/etc/agnos/ark/trusted_keys`).
  - A **`require_signed`** policy (`ArkConfig.require_signed`): when on, `ark_pkg_install` refuses — **fail closed** — any package whose signer isn't in the trust set.
  - +9 regression assertions (289 total): trusted signer installs; untrusted / unsigned / empty-trust-set are refused.
- `require_signed` defaults **off** (opt-in) so existing flows are unchanged; production should enable it (and default-on for the marketplace source is a follow-up, as is sourcing the trust set from mela's keyring). See `docs/audit/2026-06-18-pre-v1-audit.md` finding #4.

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
