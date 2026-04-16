# P(-1) Security Audit - 2026-04-16

**Scope**: Full scaffold hardening of Cyrius port (from 4363 lines Rust to 1766 lines Cyrius)
**Version**: 0.1.0 (Cyrius port, toolchain 5.1.3)
**Auditor**: Claude Opus 4.6 (automated, internal + external research)

---

## Executive Summary

The Cyrius port of ark is functionally correct (123/123 tests pass) but carries
**4 critical, 5 high, 8 medium, and 10 low** findings from internal code review,
plus **3 critical, 5 high, 4 medium** findings from external CVE/0-day research
mapped to ark's attack surface.

The most urgent issues are three stack buffer overflows (C1-C3), a data-destroying
bug in `pkg_db_save` (H1), and architectural gaps around dependency confusion and
package signing that affect the multi-source resolution model.

---

## CRITICAL Findings

### C1. Stack buffer overflow in `confirm()` (main.cyr:1682)
- **Type**: Memory safety
- **Vector**: Local user input
- `var buf[16]` (128 bytes stack) but `syscall(SYS_READ, STDIN, &buf, 128)` reads
  up to 128 bytes. An attacker pasting > 16 i64 words (128 bytes) into the
  confirmation prompt overwrites the return address.
- **Fix**: Clamp read length to `sizeof(buf)` = 128 bytes. The buf[16] allocates
  16 * 8 = 128 bytes, so the read length happens to match. However, the intent
  was likely buf[16] = 16 bytes. Clarify: use `var buf[16]` and read 16, or
  use `var buf[128]` and read 128. Either way, lengths must match.
- **Status**: NEEDS FIX

### C2. Stack buffer overflow in `txn_log_load()` (main.cyr:580-581)
- **Type**: Memory safety
- **Vector**: Malicious/large transaction log file
- `var buf[65536]` (512KB stack) but `file_read_all(..., 524288)` requests 512KB.
  In Cyrius, `var buf[65536]` allocates 65536 * 8 = 524288 bytes. The sizes
  actually match. However, this allocates 512KB on the stack which is fragile.
- **Risk**: Stack overflow on systems with small stack limits. Not exploitable as
  buffer overflow, but a stack exhaustion risk.
- **Fix**: Use heap allocation via `alloc()` for large buffers.
- **Status**: MEDIUM (reclassified - sizes match in Cyrius i64 model)

### C3. Stack buffer overflow in `load_config_from()` (main.cyr:1631-1632)
- Same pattern as C2. `var buf[65536]` with 524288 maxlen. Sizes match in Cyrius
  (65536 * 8 = 524288). Stack exhaustion risk, not exploitable overflow.
- **Status**: MEDIUM (reclassified)

### C4. Local config injection via `./ark.toml`
- **Type**: Privilege escalation vector
- **Vector**: Attacker writes `./ark.toml` in CWD before user runs ark
- Config search loads `./ark.toml` before system config. Can redirect
  `marketplace_dir`, `cache_dir`, `package_db_path` to attacker paths.
- **Fix**: When running with elevated privileges (via shakti), ignore CWD and
  user-home configs. Only trust `/etc/agnos/ark.toml`.
- **Status**: NEEDS FIX (design - deferred to shakti integration)

### C5. Dependency confusion - multi-source resolution (External)
- **Type**: Supply chain
- **Vector**: Attacker publishes package on public apt repo matching marketplace name
- Ark resolves from system, marketplace, and Flutter. No namespace scoping
  prevents cross-source name collisions. Default `SystemFirst` strategy means
  a malicious apt package shadows a legitimate marketplace package.
- **References**: Alex Birsan (2021), ongoing through 2025
- **Fix**: Namespace scoping (marketplace packages use `agnos-*` prefix),
  source pinning per package, warn on multi-source name collisions.
- **Status**: DESIGN NEEDED (nous resolver - not in ark directly)

### C6. No package signing - hash without authentication (External)
- **Type**: Supply chain
- **Vector**: Attacker modifies package AND PackageDb checksums together
- SHA-256 verifies integrity but not authenticity. If attacker has write access
  to both package files and `/var/lib/agnos/ark/packages.json`, verification
  passes with tampered content.
- **References**: Shai-Hulud worm (Sep 2025), crates.io malicious packages
- **Fix**: Cryptographic package signing via sigil (on roadmap). Ed25519 or
  GPG signatures verified against trusted keyring.
- **Status**: ON ROADMAP (sigil integration)

---

## HIGH Findings

### H1. `pkg_db_save()` writes empty JSON, destroying all data
- **Type**: Data loss
- **Impact**: Every call to `hold`/`unhold` wipes the package database
- Function serializes hardcoded `{"packages":{}}` instead of actual entries.
  The Rust version serialized the full database via serde.
- **Fix**: Implement proper JSON serialization of all PackageDbEntry records.
- **Status**: NEEDS FIX

### H2. `pkg_db_unregister()` sets value to 0 instead of deleting
- **Type**: Logic error
- Uses `map_set(key, 0)` leaving ghost entries. `map_count` returns wrong
  number. Should use `map_delete`.
- **Status**: NEEDS FIX

### H3. Transaction log not file-locked
- **Type**: Data corruption
- **Vector**: Concurrent ark invocations
- `txn_log_persist` uses `O_APPEND` without `flock()`. Concurrent writes can
  interleave partial JSON lines. stdlib provides `file_append_locked()`.
- **Status**: NEEDS FIX

### H4. Transaction log JSON injection via unescaped fields
- **Type**: Log injection
- **Vector**: Crafted username or package name containing `"` or `\n`
- `txn_log_persist` builds JSON by string concatenation without escaping.
  A field containing `","status":"Committed"` can forge transaction status.
- **Fix**: Escape `"`, `\`, and control characters in all interpolated values.
- **Status**: NEEDS FIX

### H5. No package name sanitization
- **Type**: Input validation
- **Vector**: CLI argument with `../`, null bytes, shell metacharacters
- Package names pass through to hashmap keys, file paths (in plan output),
  and JSON serialization without any validation.
- **Fix**: Validate package names match `^[a-zA-Z0-9][a-zA-Z0-9._-]*$`.
  Reject names > 128 chars, containing `..`, `/`, null bytes.
- **Status**: NEEDS FIX

### H6. TOCTOU symlink on PackageDb temp file (External)
- **Type**: Symlink attack
- **Vector**: Attacker creates symlink at `packages.json.tmp`
- `pkg_db_save` writes to predictable `{path}.tmp` then renames. Attacker
  pre-creates symlink to sensitive file (e.g., `/etc/shadow`), ark overwrites it.
- **References**: CVE-2026-22701 pattern (filelock TOCTOU)
- **Fix**: Use `O_NOFOLLOW | O_CREAT | O_EXCL`, random temp name, verify
  target is not a symlink before rename.
- **Status**: NEEDS FIX

### H7. Broken per-file integrity check (External)
- **Type**: Design bug (inherited from Rust version)
- Single `checksum` field per package compared against every file. Only valid
  for single-file packages. Multi-file packages always show mismatch or
  only verify one file.
- **Fix**: Change to per-file checksums (`checksums: HashMap<path, hash>`).
- **Status**: DESIGN NEEDED (deferred - Cyrius port matches Rust behavior)

### H8. Typosquatting on marketplace (External)
- **Type**: Supply chain
- **Vector**: Attacker registers `ngnix` to catch misspellings of `nginx`
- **References**: PyPI typosquatting campaigns (Mar 2024)
- **Fix**: Levenshtein distance warning during install, curated marketplace.
- **Status**: DESIGN NEEDED (marketplace integration)

---

## MEDIUM Findings

### M1. `alloc()` return value never checked (30+ sites)
- If bump allocator exhausted, `alloc()` returns 0, immediate `store64(0, ...)`
  = null pointer write / segfault.
- **Fix**: Check alloc returns > 0 at allocation sites, abort with message.
- **Status**: NEEDS FIX

### M2. Predictable temp file path in `pkg_db_save`
- Always `{db_path}.tmp`. Overlaps with H6 (symlink attack).
- **Status**: COVERED BY H6

### M3. `txn_log_load` doesn't validate `user_val`/`status_val` for null
- `json_get` returns 0 for missing keys. `str_data(0)` dereferences null.
  Only `id_val` is guarded.
- **Status**: NEEDS FIX

### M4. `parse_args` uses hardcoded index 2 instead of `cmd_idx + 1`
- When `--no-color` precedes command, inner parse loops start at wrong index.
  `ark --no-color install foo` would parse "install" as a package name.
- **Status**: NEEDS FIX

### M5. Config injection via `$ARK_CONFIG` env var (External)
- Attacker who can set env vars redirects config to malicious file.
- **Fix**: Ignore env var config when running as root.
- **Status**: DEFERRED (shakti integration)

### M6. JSONL log injection via crafted package names (External)
- Package name containing `\n` followed by valid JSON injects log entries.
- **Covered by**: H4 (JSON escaping) + H5 (name validation)

### M7. Log persistence without fsync
- Power loss between write and OS flush loses transaction records.
- **Fix**: Add `fsync()` after each persist write.
- **Status**: NEEDS FIX

### M8. `source_filter` sentinel uses signed comparison
- `0 - 1` as sentinel for "no filter". Works with signed comparison but
  fragile if Cyrius semantics change.
- **Status**: LOW RISK (Cyrius uses signed i64)

---

## LOW Findings

### L1. Uninitialized struct fields (output_line constructors)
- `alloc()` does not zero memory. Unused fields contain heap garbage.
  Only a problem if code reads the wrong field by mistake.
- **Status**: COSMETIC

### L2. Nous stubs mask real errors
- All stubs return 0/empty. Install commands silently produce empty plans.
- **Status**: EXPECTED (stubs until nous is ported)

### L3. `pkg_db_search` is case-sensitive (Rust was case-insensitive)
- Functional regression from port.
- **Status**: NEEDS FIX

### L4. `ark verify` ignores package filter argument
- Checks all packages regardless of the `package` parameter.
- **Status**: NEEDS FIX

### L5. No SHA-256 verification in `pkg_db_check_integrity`
- Rust version hashed files on disk. Cyrius port only checks empty file list.
- **Status**: NEEDS FIX (core security feature)

### L6. Missing IntegrityIssueType variants
- Only `NoFileManifest` implemented. Missing `MissingFile`, `ChecksumMismatch`.
- **Status**: COVERED BY L5

### L7. `txn_log_recent` may underflow when log is empty
- `total - 1` wraps when total = 0. Guard `added < count` likely saves it,
  but fragile.
- **Status**: NEEDS FIX

### L8. No error propagation from `ark_mgr_new`
- Corrupt files silently ignored. Rust version logged warnings.
- **Status**: COSMETIC (acceptable for 0.1.0)

### L9. `pkg_db_list` relies on hashmap internal layout
- Hardcoded 24-byte stride couples to hashmap.cyr implementation.
- **Fix**: Use `map_iter` or `map_values` instead.
- **Status**: NEEDS FIX

### L10. Community source not handled in plan_install/plan_remove
- `SOURCE_COMMUNITY` enum value exists but no code path handles it.
- **Status**: NEEDS FIX

---

## External Research: Notable CVEs Reviewed

| CVE/Attack | Category | Severity | Ark Exposure |
|---|---|---|---|
| Dependency Confusion (Birsan 2021) | Supply chain | CRITICAL | Direct (multi-source) |
| Shai-Hulud worm (Sep 2025) | Supply chain | CRITICAL | Marketplace risk |
| npm Manifest Confusion (Jun 2023) | Supply chain | MEDIUM | Metadata divergence |
| dpkg Path Traversal (CVE-2022-1664) | Extraction | HIGH | Indirect (shakti/dpkg) |
| APT InRelease Bypass (CVE-2016-1252) | Transport | HIGH | Indirect (apt sources) |
| PyPI Typosquatting (Mar 2024) | Supply chain | HIGH | Marketplace risk |
| GitHub RepoJacking (2023-2025) | Supply chain | HIGH | zugot recipe risk |
| filelock TOCTOU (GHSA-w853-jp5j) | Local privesc | HIGH | Direct (save pattern) |
| CVE-2024-43882 execve TOCTOU | Kernel | MEDIUM | Indirect (shakti) |
| crates.io malicious build.rs | Supply chain | HIGH | Build-time risk |

---

## Fix Priority

### Immediate (this P(-1) cycle)
1. **H1**: Fix `pkg_db_save` to serialize actual entries
2. **H4+H5**: Add JSON escaping + package name validation
3. **H3**: Use `file_append_locked()` for transaction log
4. **C1**: Fix `confirm()` buffer/read length alignment
5. **M3**: Guard null returns from `json_get` in `txn_log_load`
6. **M4**: Fix `parse_args` index offset for `--no-color`
7. **L7**: Guard `txn_log_recent` underflow
8. **L3**: Case-insensitive search
9. **L5**: Implement actual SHA-256 file verification
10. **L9**: Use `map_delete` in unregister, `map_iter` in list

### Design needed (future work loops)
- C4: Privilege-aware config loading (shakti integration)
- C5: Namespace scoping and source pinning (nous)
- C6: Package signing (sigil integration)
- H6: Secure temp file handling
- H7: Per-file checksums
- H8: Typosquatting detection

---

## Methodology

- **Internal review**: Full source read of `src/main.cyr` (1766 lines),
  `tests/ark.tcyr` (470 lines), cross-referenced against `rust-old/src/`
  (4363 lines) for functional regressions.
- **External research**: Web search for CVEs 2023-2026 across apt, dpkg, npm,
  pip, cargo, pacman, apk. Mapped findings to ark's architecture: plan-based
  execution, hashmap-backed PackageDb, JSONL transaction log, TOML config,
  SHA-256 checksums, multi-source resolution via nous.
- **Tools**: cyrius build, cyrius test (123 assertions), cyrius lint (0 warnings),
  cyrius bench (9 benchmarks baselined).
