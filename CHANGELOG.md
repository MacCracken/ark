# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.5.1] - 2026-11-13

**cyrius 6.7.9 → 6.7.18, every lib and dep on the 6.7.18 bundled / pinned version, and the trust
set drops expired keys.** Build OK (1,861,896 B, half of 1.5.0's), 453 passed / 0 failed, fuzz
passes, bench mostly flat.

**nous 1.7.4 was cut for this release.** nous 1.5.3–1.7.3 could not be built into one program
with mela, and did not compile for agnos. Both are fixed in nous 1.7.4 (`83c5c57`). ark was first
verified against it in local mode (`CYRIUS_LOCAL=nous`), then built from the pushed tag.

### Changed — deps on the 6.7.18 line

| dep | was | now |
|---|---|---|
| nous | 1.5.0 | **1.7.4** (git; see *nous 1.5.1–1.7.4* below) |
| mela | 1.1.6 | **1.5.2** (git; the functions ark calls are unchanged) |
| agnostik | 1.12.0 | **2.0.0** (git; what mela 1.5.2 pins. ark calls no agnostik name) |
| bayan | 1.5.13 | **1.11.1** (stdlib; see *bayan 1.11* below) |
| sigil | 3.13.11 | **3.13.15** (stdlib) |
| sandhi | 1.10.9 | **1.13.7** (stdlib) |
| sankoch | 2.8.3 | **2.8.9** (stdlib) |
| sakshi | 2.5.8 | **2.5.10** (stdlib) |

- **nous's own bayan dependency.** nous 1.7.3+ declares a git dep on bayan 1.11.1's split
  `bayan-toml` / `bayan-json` bundles. `cyrius deps` leaves them out, because the stdlib's
  `lib/bayan.cyr` is that package, and pins only the commit. bayan documents compiling a sublib
  beside the full fold as a compile-time refusal.
- **Vendoring.** `lib/nous.cyr`, `lib/mela.cyr` and `lib/agnostik.cyr` are byte-identical to
  their tags' `dist/`, and each tag's commit is the pin `cyrius.lock` records. Every stdlib leaf in
  `lib/` matches `~/.cyrius/versions/6.7.18/lib`.
- **The lock.** `cyrius.lock` was regenerated from scratch, as 6.7.18's `cyrius deps` suggests for
  stale pins. The 1.4.3- and 1.5.0-era commit rows for old tags are gone, every hash line is
  unchanged, and there are four commit pins: nous, mela, agnostik and bayan.

### Fixed — ⛔ nous and mela could not be compiled together (nous 1.7.4)

nous 1.5.3 added `manifest_from_json(data)` and `update_from_json(data)`. mela has defined both
names for years, with a typed `json: Str` parameter, and cyrius 6.7.18 refuses a duplicate fn whose
parameter types differ. ark is the only repo that vendors both.

- **The fix is in nous**, the newcomer. 1.7.4 renames them to `nous_manifest_from_json` /
  `nous_update_from_json`. Its policy is no breaking change without a major version, so the bare
  names stay as `#deprecated` aliases until 2.0.0.
- **ark leaves the aliases out** with the new `NOUS_NO_BARE_JSON` define. It is in `[build]
  defines` and in a new `[test] defines`, because `[build] defines` do not reach test / bench / fuzz
  builds (checked on a scratch project).
- ark's one call (`src/marketplace.cyr`, the marketplace `/latest` lookup) wants mela's
  `manifest_from_json`, and now gets it unambiguously.
- The same 1.7.4 fixes nous for agnos (next section).

### Fixed — ⛔ the agnos cross-build (nous 1.7.4)

`cyrius build --agnos` failed on two nous 1.5.3 additions:

- **`proc_timeout_ms` / `proc_set_timeout_ms`.** cyrius leaves these out of `process_agnos.cyr` on
  purpose: a deadline there would bound nothing, so the call is meant to fail to compile.
- **The 2-argument Linux `sys_lstat`.** agnos's takes a length and fills a different 48-byte
  record.

nous 1.7.4 skips the idle deadline on agnos and reads agnos's `STAT_MODE` / `STAT_SIZE`. It
compiles for x86_64-agnos again, and so does ark: 3,861,760 B, up from 3,687,328 B at 1.5.0. The
older raw `SYS_STAT` sites in nous still read x86_64 offsets on agnos; that is nous's 2.0.0
portability item.

### Security — the trust set holds only key versions valid now

`ark_trust_from_keyring` turns a mela keyring into ark's set of trusted signer keys. It took every
version `mela_keyring_get_all_versions` returns, and by mela's contract that includes expired ones.

- **Now it keeps a version only if `mela_kv_is_valid_at` accepts it at the current time**
  (`valid_from` ≤ now ≤ `valid_until`, when the version has an expiry). Both window edges count as
  inside.
- `ark_trust_from_keyring_at(kr, now)` takes an explicit time, and `ark_trust_from_keyring(kr)`
  passes the wall clock.
- Overlapping versions of one key id, as in a rotation window, are each trusted while they are
  valid.
- ⚠ **Consequence.** The `.ark` carries no signing time, so validity is judged at install time. A
  package signed while its key was valid stops installing once that key expires, and has to be
  re-signed under a current key.
- ⚠ **Scope.** The install trust gate does not use the keyring today. It reads the flat
  `trust_keys` file (`/etc/agnos/ark/trusted_keys`), which has no validity dates.
  `ark_trust_load_keyring` has no caller. So this filter takes effect once a keyring is wired into
  installs, which needs a decision (roadmap, "Found at 1.5.1"). The overview no longer says the
  keyring is the install trust set.
- **Tests:** 8 new assertions cover expired, not-yet-valid, both window edges, an unexpired
  expiry, a rotation (old version expired, successor current) and the wall-clock wrapper. With the
  filter removed, 5 of them fail.

### Fixed — a package could make `packages.json` unloadable (bayan 1.11)

bayan 1.11.1 refuses a whole JSON document in which any object repeats a key; before, the first
value was answered.

- **How ark could write one.** `pkg_db_save` wrote one `file_checksums` key per entry in a
  package's file list, and the installer pushes one path per `.ark` index entry. So an `.ark` whose
  index lists a path twice produced a repeated key.
- **What followed.** On the next run `pkg_db_load` would fail, ark would see no installed packages,
  and every save would refuse ("package DB failed to load").
- **Fix.** `pkg_db_save` writes each path's checksum once.
- **Test.** A new round-trip test writes a twice-listed path, then reloads and saves again. It fails
  on 3 assertions without the fix.
- ⚠ A `packages.json` that already contains a repeated key, which only such an `.ark` installed
  under an earlier ark could produce, does not load under 1.5.1. Remove the duplicate by hand.

### Fixed — a failed update check said "All packages up to date"

ark turned any nous error from update detection into an empty list. Since nous 1.5.1 a failed apt
query fails the whole check instead of being skipped: a capture past 16 MiB, or one cut by nous's
60 s idle deadline. So `ark update` reported "All packages up to date" and dropped the marketplace
results with it.

- `ark_check_updates_r` returns the nous error. `ark_check_updates` keeps its vec contract.
- `ark update` and `ark upgrade` now report "Update check failed: <nous's message>", and `upgrade`
  exits non-zero.
- The rendering is split out (`ark_update_render`) so the failure path is tested with a synthetic
  error; no test host can make apt fail on cue.
- `ark search` and `ark list` still show a failed query as "nothing found" (roadmap).

### Changed — what nous 1.5.1–1.7.4 changes for ark users

From nous's CHANGELOG and its "ark → nous" coordination notes, checked against the bundle diff. The
first two were run on both binaries.

- **An unknown package is an error.**
  - 1.5.0: `ark install zzghostpkg` printed "OK: Install plan (0 packages …)" and exited 0.
  - 1.5.1: it prints "package \`zzghostpkg\` not found: not in any source" and exits 1.
  - A missing dependency is the same error, naming who needed it.
- **The plan prints in install order.** "deps-first" used to be printed over the reverse order.
  Against the zugot tree, `ark install curl`:
  - 1.5.0: `curl, ca-certificates, openssl, zlib, glibc, kernel-edge`.
  - 1.5.1: `linux-api-headers, glibc, zlib, openssl, xz, lz4, zstd, libnghttp2, curl`.
  - Resolution is deterministic now. It was hash-seeded per process.
- **Which recipe wins changed.** nous loads every zugot subdirectory, which adds `desktops/`. For a
  name in several directories it keeps the first in byte order, `base` first. Before, the last one
  scanned won. 23 names change recipe, curl among them (edge → base).
- **Recipe arrays.** A list written over several lines (`runtime = [` on its own line, in 47 zugot
  files) now parses, so groups / runtime / build / hardening are filled where they read empty.
- **Version constraints.** Every constraint is intersected and the chosen version is checked, so
  there are more, correct, version conflicts. Compound ranges parse (1.7.1). A pre-release
  satisfies a constraint only when the constraint names one (1.7.0).
- **Flutter apps** are labelled flutter-app and install through the Flutter step.
- **Update comparison.** Marketplace versions compare by SemVer precedence (`1.0.0` is now offered
  over `1.0.0-rc.1`). Community and system versions compare by Debian ordering.
- **apt.** Updates come from `apt-get -s dist-upgrade` (held-back packages are not listed), search
  escapes its query, and output is no longer cut at 64 KiB.

### Changed — bayan 1.11 deprecations

- `toml_get` → `bayan_toml_get_by_cstr` (35 sites: `recipe.cyr`, `ark_package.cyr`, `cli.cyr`).
  Same answers; `toml_get` already forwarded to it.
- `json_get(pairs, str_from("k"))` → `bayan_json_get_by_cstr(pairs, "k")` (3 sites in
  `transaction.cyr`). Same answers.
- `fuzz/ark.fcyr` called `json_get` with **cstring** keys, which a Str-keyed lookup never matches,
  so its transaction-log target never reached past the lookup. It uses `_by_cstr` now.
- bayan's tree parser now also refuses leading-zero numbers. ark writes numbers only through
  `str_builder_add_int`, so it never writes one.

### Changed — nous's error enum

`ERR_INVALID_PACKAGE_NAME` → `NOUS_ERR_INVALID_PACKAGE_NAME` (nous 1.5.1; value unchanged).
Two sites in `src/engine.cyr`.

### Audited — the 6.7.10–6.7.18 changes that alter already-compiling code do not reach ark

Read in full by three parallel reviews. 6.7.13 is not in the migration notes, so its CHANGELOG was
read directly.

- **ark's code has none of the shapes those releases change:** no structs, tuples, `?`, typed
  pointers, slices, generics, floats, closures, asm, `const` or bool values. Its 9 enums use `;`
  separators.
- **No name collides** with the 6.7.18 stdlib or the dep bundles, apart from the nous/mela pair
  above.
- **Stdlib functions ark calls that changed** do so only on edge inputs or other platforms:
  - DEFLATE rejects code-length sets zlib rejects, so the package fails closed.
  - `dir_list` fails at once on a FIFO instead of blocking.
  - `args_init` reads a command line past 2 MiB whole.
  - The native TLS client refuses non-conformant servers.
- **CI:** unchanged except as below.

### CI

- **Smoke test.** The shipped binary is the `CYRIUS_DCE=1` build, and CI had never run it. 6.7.18
  now compacts it, removing 2,043,167 B of dead code. The new step runs `ark history` and a real
  signed-`.ark` install into a scratch root, then checks the file and the one journal line.
- `cyrius deps --locked`: `lib/` and `cyrius.lock` are exactly what the tags resolve.

### Size and benchmarks

- **Binary:** 3,731,336 → **1,861,896 B (−50.1 %)**. 6.7.18's DCE compacts on x86_64 ELF; 1.5.0's
  build declined to past 4,096 dead-code runs, so it only NOP-filled.
- **Warnings:** 7 → 3. mela 1.5.x dropped its three. One of the two `src/engine.cyr` "assigning
  non-pointer to typed pointer" warnings, `ark_install_dir`'s `last_err = ares_message(r)`, no
  longer fires. That code is unchanged, and the cause was not chased.

`cyrius bench tests/ark.bcyr`, median of 3 alternating runs per side:

| bench | 1.5.0 / 6.7.9 (µs) | 1.5.1 / 6.7.18 (µs) | Δ |
|---|---|---|---|
| cmd_create (100k) | 26,117 | 25,984 | −0.5 % |
| config_default (100k) | 42,323 | 38,830 | **−8.3 %** |
| source_to_str (100k) | 3,921 | 3,691 | **−5.9 %** |
| group_meta_pkg (100k) | 5,226 | 5,829 | **+11.5 %** |
| db_search_20 (10k) | 18,388 | 18,657 | +1.5 % |
| db_total_size_20 (10k) | 5,756 | 5,752 | −0.1 % |
| db_integrity_20 (10k) | 17,271 | 17,235 | −0.2 % |
| txn_begin_commit (10k) | 38,678 | 38,130 | −1.4 % |
| output_display (10k) | 6,470 | 6,584 | +1.8 % |

- **Real, with no overlap between runs:** `config_default` −8.3 % (41,417–43,267 vs 38,044–38,987 µs),
  `source_to_str` −5.9 %, and `group_meta_pkg` **+11.5 %** (5,179–5,280 vs 5,677–5,939 µs).
- ⚖️ **`group_meta_pkg` is one `streq`**, and `streq`, `strlen` and `memeq` are unchanged in
  source. So the ~6 ns per call comes from 6.7.18's codegen or layout. Not chased: it maps a group
  name, not a hot path. nous saw the same kind of shift at its 6.7.9 move (`detect_source`
  +6.8 %).
- `output_display` +1.8 % is noise: 1.5.1's runs were 6,202 / 8,239 / 6,584 µs.
- Appended to `bench-history.csv` (tagged `2026-11-13-1.5.1`).
- Not benched: bayan 1.11's JSON parsing costs about 1.5× what it did by bayan's measure. The
  `pkg_db_load` path that uses it has no bench.

### Verified (default mode, from the pushed tags)

- `cyrius deps --locked`: clean. `cyrius.lock` carries four commit pins: nous 1.7.4 @ `83c5c57`,
  mela 1.5.2, agnostik 2.0.0 and bayan 1.11.1.
- `cyrius test tests/ark.tcyr`: 453 / 0. `cyrius fuzz fuzz/ark.fcyr`: pass.
- fmt (every module, both test files, the fuzz harness), lint (13 modules, 0 warnings), vet, deny:
  clean. `cyrius doc src/*.cyr --check`: 95 documented / 304 undocumented (91 / 305 at 1.5.0).
- The DCE binary: two real `ark install --apply --root <dir>` runs leave `txn-000001` /
  `txn-000002` in the journal and a `packages.json` that parses. `history` and `update` exit 0. The
  new CI smoke steps pass locally.
- `cyrius build --agnos src/main.cyr`: OK.

## [1.5.0] - 2026-11-13

**cyrius 6.6.20 → 6.7.9, every lib and dep on the 6.7.9 bundled / pinned version.** Build OK
(3,731,336 B), 434 passed / 0 failed, fuzz passes, bench flat. The only ark source change is
`ARK_VERSION_STR`. The toolchain needed no `src/` change; the fuzz harness did (see Fixed).

### Changed — deps on the 6.7.9 line

| dep | was | now |
|---|---|---|
| nous | 1.4.1 | **1.5.0** (git; no public API change) |
| mela | 1.0.5 | **1.1.6** (git; see *Inherited from mela 1.1.x*) |
| agnostik | 1.7.1 | **1.12.0** (git; additive only, the version mela 1.1.6 pins) |
| sigil | 3.13.10 | **3.13.11** (stdlib) |
| sandhi | 1.10.8 | **1.10.9** (stdlib) |
| sankoch | 2.8.1 | **2.8.3** (stdlib) |
| bayan | 1.5.12 | **1.5.13** (stdlib) |
| sakshi | 2.5.7 | **2.5.8** (stdlib) |

Each git tag is the last one that pins cyrius 6.7.9. All three were checked against
`git ls-remote`, and each tag's commit matches the pin `cyrius.lock` records. `lib/nous.cyr`,
`lib/mela.cyr` and `lib/agnostik.cyr` are byte-identical to their tags' `dist/`. All 12 mela
functions ark calls, including the private `_rc_http_get`, keep their 1.0.5 signatures.

### Changed — vendored stdlib re-synced to the 6.7.9 snapshot

`cyrius lib sync --full --relock`, then `cyrius deps`. Every stdlib leaf in `lib/` (including
`lib/unicode/`) is byte-identical to `~/.cyrius/versions/6.7.9/lib`, there are no orphans, and
`cyrius deps --locked` reports that `lib/` and `cyrius.lock` are exactly what the tags resolve.

### Audited — the 6.7.x changes that alter already-compiling code do not reach ark

A green suite proves nothing about these, because the code still compiles. Method: a comment- and
string-stripped scan of `src/`, `tests/` and `fuzz/` (5,411 code lines) for each shape.

| Change (cyrius) | What it changed | What ark has |
|---|---|---|
| A lone `!` (6.7.3) | Silently dropped before 6.7.3, so `if (!ok)` ran as `if (ok)` | None. ark spells it `== 0` |
| Unary `-` / `~` on a constant before a literal operator (bug 6.6.10–6.7.8, fixed 6.7.9) | `-K + 10` was 15 | No unary `-` or `~` anywhere (ark writes `0 - 1`), so 1.4.3 on 6.6.20 was not affected either |
| Typed `*T` step leaking through operators (fixed 6.7.9); narrow `*p` reads and writes the element (6.7.8) | `(x == q) + 100` stepped by `sizeof(T)`; `*p` through a `*u8` read 8 bytes | No typed pointers, no `*p` dereference |
| Signed slice elements read negative (6.7.8) | `[i8]` 0xFD read 253, now -3 | No slices |
| Method / generic results typed by the method (fixed 6.7.9); by-value struct argument order (6.7.7) | `p.m(1.5) * 2` multiplied bit patterns | No methods, generics, floats or `struct` declarations |
| `fncall8` arguments 7 and 8 (6.7.6); `fhm_set` overwrite (6.7.9) | swapped / rebuilt the table | No `fncall*` and no `hashmap_fast` in `src/` |
| The exec family resets child signal state (6.7.9) | Children start with SIGPIPE / SIGXFSZ at default and an empty mask | ark never ignores or blocks a signal, so its children start the same as before. The one `signal_ignore` in `lib/` is sandhi's SIGPIPE guard, which runs only in sandhi's server loops; ark, mela, nous and agnostik call none of them |

The 6.7.x changes that turn a silent miscompile into a compile error, such as an unmatched `}` at
module scope, a `for` step on an undeclared name, or `true` / `do` / `dyn` / `const` as names,
would have stopped the build. It builds with the same 7 warnings as 1.4.3.

### Inherited from mela 1.1.x — what ark users see

- **Marketplace downloads over 256 KiB work.** mela 1.1.3 (D7): artifact downloads failed past
  256 KiB and could hang forever. `ark install --marketplace` goes through `mela_fetch_artifact`.
- **A failed download leaves nothing at the cache path.** `mela_fetch_artifact` now removes
  `dest_path` on failure. ark treats an existing file there as a cache hit, so a leftover file was
  installed from and failed verification (fail-closed) until removed by hand.
- **`resolve-"latest"` is stricter.** `manifest_from_json` returns 0 for a manifest with an
  unknown, missing or non-string `category`, and ark reports the latest version as unknown.
- **Key files must spell out `has_until`.** A key version without it used to decode as "never
  expires" into ark's trust set. It is now rejected.
- **`sanitize_filename` keeps non-ASCII letters,** so a non-ASCII package name or version gets a
  different cache file name (the next install re-downloads it). ASCII names are unchanged.
- mela now compiles in the stdlib Unicode tables, which by mela's own measure add about 170 KB.

### Fixed — the fuzz harness had not compiled since 1.4.2

`fuzz/ark.fcyr` still called `recipe_parse` in the pre-6.6 single-value form
(`result_unwrap(rr)`). 1.4.2's value-form migration covered `src/` only, and CI never ran the
harness, so nothing showed it. It now binds both values, as `src/` does, and passes. **CI runs
it** (a `Fuzz` step, about 3.5 s).

### CI — `cyrius deps --locked`

CI and release resolved with `cyrius deps`, which rewrites the tracked `lib/` and `cyrius.lock`
to match the tags and so hides drift. They now use `cyrius deps --locked` (cyrius 6.7.6+), which
writes nothing and fails naming each difference.

### Known

- **The trust set includes expired publisher keys** (pre-existing, unchanged here).
  `ark_trust_from_keyring` takes every version `mela_keyring_get_all_versions` returns, and by
  mela's contract that includes expired ones. A policy decision, tracked in the roadmap.
- **Toolchain finding, not an ark defect:** `cyrius.lock` keeps the commit pin of every tag ever
  resolved. The 1.4.3 pins for nous 1.4.1, mela 1.0.5 and agnostik 1.7.1 stay beside the new ones,
  and `cyrius deps --relock` keeps them too. Nothing reads a pin for a tag ark does not use, and
  mela 1.1.6's own lock shows the same. Left as cyrius writes it.

### Size and benchmarks

The binary grows 3,328,800 → **3,731,336 B (+12.1 %)**: +333,788 B reachable and +68,748 B dead
code, all in the newer vendored bundles. The agnos build grows 3,284,760 → 3,687,328 B.
`cyrius bench tests/ark.bcyr`, median of 3 alternating runs per side:

| bench | 1.4.3 / 6.6.20 (µs) | 1.5.0 / 6.7.9 (µs) | Δ |
|---|---|---|---|
| cmd_create (100k) | 26,862 | 26,193 | −2.5 % |
| config_default (100k) | 41,708 | 41,632 | −0.2 % |
| source_to_str (100k) | 4,206 | 3,890 | **−7.5 %** |
| group_meta_pkg (100k) | 5,242 | 5,214 | −0.5 % |
| db_search_20 (10k) | 18,386 | 18,530 | +0.8 % |
| db_total_size_20 (10k) | 5,702 | 5,872 | +3.0 % |
| db_integrity_20 (10k) | 17,306 | 17,300 | 0.0 % |
| txn_begin_commit (10k) | 38,406 | 38,473 | +0.2 % |
| output_display (10k) | 6,406 | 6,408 | 0.0 % |

- `source_to_str` −7.5 % is real: the runs spanned 4,169–4,219 µs on 1.4.3 and 3,849–3,896 µs on
  1.5.0, with no overlap. Its path is unchanged in `src/`, so this comes from the toolchain.
- `db_total_size_20` +3.0 % is noise: 1.5.0's three runs were 5,719 / 5,872 / 6,759 µs.
- Everything else overlaps run to run. Appended to `bench-history.csv` (tagged `2026-11-13-1.5.0`).

### Verified

- `cyrius test tests/ark.tcyr`: 434 / 0. `cyrius fuzz fuzz/ark.fcyr`: pass.
- fmt (every module, both test files and the fuzz harness), lint (13 modules, 0 warnings), vet,
  deny: clean. `cyrius doc src/*.cyr --check` is still 91 documented / 305 undocumented.
- Two real `ark install --apply --root <dir>` runs leave `txn-000001` and `txn-000002` in the
  root's `transaction.log`, so the journal appends and ids continue across processes.

## [1.4.3] - 2026-11-13

**cyrius 6.6.2 → 6.6.20, every lib and dep on the 6.6.20 bundled / pinned version, and three bugs
the new toolchain's warnings led to.** Build OK (3,328,800 B), 434 passed / 0 failed, bench passes.

### Changed — sigil and sandhi come in as stdlib leaves

Both ship folded into the 6.6.20 stdlib, so their `[deps.*]` git blocks are gone and they are
listed in `[deps] stdlib` (sankoch already was). Under 6.6.2 the git dep and the stdlib leaf
competed for the same `lib/` file — `cyrius deps` warned "refusing to overwrite stdlib leaf" and
kept the snapshot — so this states what was already being built. The remaining git deps move to
the last tag of each that pins cyrius 6.6.20, the same set mela 1.0.4/1.0.5 moved to.

| dep | was | now |
|---|---|---|
| sigil | 3.12.16 (git) | **3.13.10** (stdlib) |
| sandhi | 1.9.16 (git) | **1.10.8** (stdlib) |
| sankoch | 2.7.14 (stdlib) | **2.8.1** (stdlib) |
| bayan | 1.5.5 (stdlib) | **1.5.12** (stdlib) |
| sakshi | 2.5.1 (stdlib) | **2.5.7** (stdlib) |
| nous | 1.4.0 | **1.4.1** |
| mela | 1.0.3 | **1.0.5** |
| agnostik | 1.6.0 | **1.7.1** |

`lib/nous.cyr`, `lib/mela.cyr` and `lib/agnostik.cyr` are byte-identical to their tags' `dist/`;
each tag was checked against `git ls-remote`, and its commit matches the pin `cyrius.lock` records.

### Changed — `path = "../<dep>"` overrides removed from `cyrius.cyml`

Under 6.6.x a `path` beside `git`/`tag` wins locally, so any local `cyrius build` compiled the
sibling checkout's **main** — nous, mela and agnostik main all pin cyrius 6.7.18 — and rewrote
`lib/` and `cyrius.lock` with it, while CI cloned the tag. A local build now resolves the same tags
CI does. (From cyrius 6.7.6 a `path` is a local-mode-only override; see the roadmap.)

### Changed — vendored stdlib re-synced to the 6.6.20 snapshot

`cyrius lib sync --full --relock`: every stdlib leaf in `lib/` is byte-identical to
`~/.cyrius/versions/6.6.20/lib`. New leaves: `alloc_cx`, `poison`, `tls_hostid`. No orphans.

### Fixed — ⛔ the config file's booleans and `system_backend` were ignored

`load_config_from` compared each TOML value with `streq(str_data(val), "false")`. bayan's values
are **slices into the file buffer**, not NUL-terminated, so the comparison read on into the next
line and never matched: `color_output`, `confirm_system_installs`, `confirm_removals`,
`auto_update_check` and `system_backend` had **no effect** from any config file. The string keys
(`marketplace_dir`, `cache_dir`, `apt_wrapper`, `marketplace_url`) were stored as the same slices,
so each one handed to a cstring consumer carried the rest of the file after it. Found by 6.6.20's
new "passing a Str's data pointer to a cstring parameter" warning. Each value is now a terminated
copy (`cfg_toml_str`). New `load_config_from` test group (the loader had none): run against the
1.4.2 sources, 9 of its 12 assertions fail.

### Fixed — ⛔ the transaction log never reloaded

`txn_log_load` looked up `"id"` / `"user"` / `"status"` with `json_get`, which compares keys with
`str_eq` — **Str** keys. A cstring literal never matched, so every process started with an empty
log: `ark history` showed only the current run, and every run's first transaction was
`txn-000001` again (two installs on 1.4.2 write two `txn-000001` records). The keys are Strs now,
the reloaded id is a terminated copy (it is a cstring map key), and a `Failed` record reloads as
failed. Ids continue from the log. New `txn_log_reload` test group: 8 of its 14 assertions fail
against the 1.4.2 sources.

⚠ **Consequence for `ark rollback`.** The log stores a transaction's op *count*, not its ops, so a
transaction reloaded from an earlier run has nothing to reverse. `ark rollback` now refuses one
("Transaction txn-… has no recorded operations to reverse") instead of reporting a successful
zero-step rollback. Before this release it could not find one at all ("No transaction found to
rollback"), so no working path is lost. Rolling back across processes needs the ops persisted —
not done here.

### Fixed — ⛔ heap overflow on any config, log or backup file past 64 KiB

Six reads paired `alloc(65536)` with a `file_read_all(..., 524288)` cap: the config loader, the
transaction-log loader, and both files in `ark backup` and `ark restore`. A file past 64 KiB wrote
up to 448 KiB beyond its buffer. The transaction log grows with every install, so this was
reachable in normal use. (`recipe.cyr` fixed the same pairing earlier.) All six now go through
`ark_read_whole` (`src/portable.cyr`), which sizes the buffer to the file up to 64 MiB — the same
grow-until-EOF loop `pkg_db_load` uses. Backup and restore no longer silently truncate at 512 KiB.
The loader also drops its raw `syscall(SYS_OPEN)` existence probe.

### Changed — 6.6.20 deprecations and warnings

- `bayan_json_v_obj_get` → `bayan_json_v_obj_get_by_cstr` (5 sites in `src/db.cyr`; every key is a
  literal). No behaviour change: the deprecated name already forwarded to `_by_cstr`.
- `ark bazaar`'s sub-command match: `streq(str_data(subcmd), …)` → `str_eq_cstr(subcmd, …)`.
- Build warnings 12 → 7. What remains all predates this release: 3 "assigning non-pointer to
  typed pointer" in `src/`, 3 in `lib/mela.cyr`, and "large static data". The 6.6.2 "refusing to
  overwrite stdlib leaf" and duplicate `_hex_nibble` warnings are gone.

### Changed — `ARK_VERSION_STR` drift

`src/types.cyr` said `1.4.1` through all of 1.4.2. Now `1.4.3`.

### CI — `-D ARK_MAIN` dropped

cyrius 6.6.17+ reads `[build] defines` on every `cyrius build`. The flag was redundant: the binary
is byte-identical with and without it.

### Size and benchmarks

The binary grows 2,930,816 → **3,328,800 B (+13.6 %)**: about +269 KB reachable code and +128 KB
dead code, all in the newer vendored bundles (`CYRIUS_DCE=1` only NOP-fills on x86_64 ELF here, so
it does not shrink the file). `cyrius bench tests/ark.bcyr`, median of 3 alternating runs per side:

| bench | 6.6.2 (µs) | 6.6.20 (µs) | Δ |
|---|---|---|---|
| cmd_create (100k) | 26,486 | 27,215 | +2.8 % |
| config_default (100k) | 48,479 | 41,677 | −14.0 % |
| source_to_str (100k) | 5,174 | 4,346 | −16.0 % |
| group_meta_pkg (100k) | 5,289 | 5,433 | +2.7 % |
| db_search_20 (10k) | 18,406 | 19,148 | +4.0 % |
| db_total_size_20 (10k) | 5,753 | 5,966 | +3.7 % |
| db_integrity_20 (10k) | 17,825 | 18,065 | +1.3 % |
| txn_begin_commit (10k) | 39,459 | 39,588 | +0.3 % |
| output_display (10k) | 7,369 | 6,639 | −9.9 % |

None of the benched paths changed in `src/`; the movement is toolchain and stdlib. Appended to
`bench-history.csv`.

### Tests

392 → 434 assertions. 16 of those are the zugot-corpus assertions, which only run when `../zugot`
exists (it did not for the 1.4.2 figure). The 26 new ones are the `txn_log_reload` and
`load_config_from` groups: reload of committed / rolled-back / failed records, id continuation,
a >64 KiB log and config, terminated string values, and the rollback refusal.

## [1.4.2] - 2026-09-10

**cyrius 6.3.38 → 6.6.2, every dep brought current, and a silent mis-bind removed.**
Build OK (2,930,816 B), tests pass, bench passes.

### Fixed — ⛔ `agent_info_new` silently mis-bound its arguments

ark vendors **both** agnostik and nous, and both defined `agent_info_new` — agnostik's takes 4
args over a 32-byte {id, name, type, status} struct, nous's took 3 over a 24-byte
{name, version, desc} one. "Last definition wins", so calls to the other arity had their
**arguments silently shifted**. cyrius 6.6.2 promotes an arity-disagreeing duplicate from a
warning to a hard error, which is the only reason it surfaced.

Fixed upstream in **nous 1.4.0** (`nous_agent_info_new`), not worked around here. agnostik keeps
the bare name: it is the shared-types library for AGNOS and agent identity/status is its domain.
⚖️ Measured across every `src/` under `~/Repos`: zero callers outside the two defining repos.

### Changed — every dep tag brought current

| dep | was | now |
|---|---|---|
| sigil | 3.9.7 | **3.12.16** |
| nous | 1.3.1 | **1.4.0** |
| mela | 1.0.1 | **1.0.3** |
| agnostik | 1.3.1 | **1.6.0** |
| sandhi | 1.7.0 | **1.9.16** |

All six artifacts now resolve byte-identical to their upstream `dist/`, and `lib/` carries zero
calls to the deleted `payload()`/`tag()`.

### Changed — value-form migration

`payload(r)` → the second bound variable, predicates take the tag, `result_unwrap(r)` →
`result_unwrap(r_t, r)`. 5 files. ark has no hand-rolled `tagged_new` boxes, so no `boxed_*`.
**Zero mixed-return warnings** — the error-reads-as-success shape does not occur here; the one
`Result`-returning fn (`nous_resolve_all`) already returned a pair on both paths.

### Fixed — stale `bayan_json_v_parse_str` (src/db.cyr)

bayan renamed its explicit buf+len JSON entry `_str` → `_buf` in its own 6.6 migration. ark sat on
6.3.38, so the call had gone stale and the build would not link.

### Fixed — `cyrius.cyml` hardcoded ark's OWN version, and it drifted

`version = "1.4.1"` was written literally in the manifest while `VERSION` said 1.4.2 — a second
source of truth for ark's own version, missed on the first pass of this release and caught by the
maintainer. Now `version = "${file:VERSION}"`, the form every other repo in the ecosystem already
uses (agnostik, agnova, mela, nous). ⚠ Same class as the `.cyrius-toolchain` removal below: this
release found **three** copies of the toolchain version disagreeing and **two** copies of ark's own
version disagreeing. Every one of them is now read rather than restated.

### Fixed — ⛔ THREE disagreeing toolchain versions

`.cyrius-toolchain` said **6.3.38**, `CONTRIBUTING.md` said **6.3.5**, `cyrius.cyml` pinned
**6.6.2**. CI read the first, so it installed one toolchain while the pin redirect demanded
another. `.cyrius-toolchain` is **removed** — a second source of truth that had drifted — and both
workflows now read the manifest pin and hand it to the upstream `install.sh` (patra's pattern).
The old step also copied into a **flat** `~/.cyrius`, with no `versions/<v>/` layout and no
`current`, which the pin redirect has required since 6.5.37. The number is not restated in
CONTRIBUTING: a fresh stamp in an unchecked place only restarts the clock.

### Fixed — 7 files not canonically formatted

Pre-existing drift; cyrfmt only began tracking paren depth at v6.5.28 and ark was pinned 6.3.38.

## [Unreleased]

## [1.4.1] — 2026-07-03 — end recipe-fn shadowing vs vendored nous

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
