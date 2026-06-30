# 0003 — Syscall portability layer (host ↔ AGNOS)

- **Status**: accepted
- **Date**: 2026-06-29
- **Deciders**: ark maintainers
- **Related**: [ADR 0002 — Package source model](0002-package-source-model.md);
  roadmap "Post-v1: 1.1.x — AGNOS track" (M3)

## Context

ark targets two platforms from one source tree: x86_64 Linux (Debian dev/CI) and
AGNOS (the native target). Cyrius cross-compiles to AGNOS with `cyrius build
--agnos`, selecting an AGNOS syscall module at build time. Several hot paths in
ark reached for **raw x86_64 Linux syscall numbers** or wrappers that do not
exist (or have a different ABI) on AGNOS:

- `transaction.cyr` / `db.cyr` / `engine.cyr`: `syscall(201, 0)` for wall-clock
  time. On AGNOS, wall-clock is `sys_time_unix` (#46); `201` is a *different*
  syscall there — it compiles but silently does the wrong thing.
- `db.cyr` / `transaction.cyr`: `syscall(74, fd)` (`fsync`). AGNOS has **no
  fsync** syscall (durability is a no-op on its current FS model).
- `db.cyr`: `syscall(82, old, new)` (`rename`, 2-arg). AGNOS `rename` is `#31`
  and takes **four** args (`old, oldlen, new, newlen`).
- `db.cyr`: the open-flags magic literal `193` (`O_WRONLY|O_CREAT|O_EXCL`).
- `ark_package.cyr`: `sys_symlink(target, link)`. AGNOS has **no symlink**
  syscall (filed agnos 1.51.x (a)). This was the one symbol that made the AGNOS
  cross-build *fail to link* — the raw-number calls above compiled but were wrong.

A raw number that "compiles but invokes the wrong syscall on AGNOS" is worse than
a link error: it fails silently at runtime. The coupling was implicit and spread
across four files.

## Decision

**Route every host/AGNOS-divergent syscall through a thin, `#ifdef`-guarded
portability shim in `src/portable.cyr`, so the call site is target-agnostic and
each operation resolves to the correct Sys number/ABI at build time.**

The shims (`ark_now_secs`, `ark_fsync`, `ark_rename`, `ark_unlink`,
`ark_symlink`) each compile to exactly the syscall ark used before on the host,
and to the AGNOS-correct form under `#ifdef CYRIUS_TARGET_AGNOS`:

| Shim | Host (x86_64 Linux) | AGNOS |
|---|---|---|
| `ark_now_secs` | `clock_epoch_secs()` (vDSO `clock_gettime`) | `sys_time_unix` (#46) |
| `ark_fsync` | `sys_fsync` (#74) | `0` — no fsync syscall (no-op) |
| `ark_rename` | `sys_rename(old,new)` (#82) | `sys_rename(old,len,new,len)` (#31) |
| `ark_unlink` | `sys_unlink(path)` | `sys_unlink(path,len)` |
| `ark_symlink` | `sys_symlink(target,link)` (#88) | `-1` — no symlink syscall |

Key points:

- **The host path is byte-identical.** Each shim inlines to the exact syscall
  ark issued before, so x86_64 behavior and the host test suite (337 at 1.1.0;
  399 as of 1.1.3) are unchanged.
- **`ark_symlink` returns `-1` on AGNOS** rather than calling an undefined
  symbol. This is what makes the AGNOS cross-build *link*, and it makes a `.ark`
  carrying a symlink entry **fail clean** on AGNOS (caller checks the result)
  rather than crash. AGNOS `.ark` fixtures are scoped symlink-free until the
  syscall lands.
- **`ark_fsync` is a no-op on AGNOS.** Durability is a no-op on AGNOS's current
  FS model; a no-op is correct there and harmless (the transaction-log fsync on
  the host is unaffected).
- **Magic numbers became named flags.** `193` → `O_WRONLY | O_CREAT | O_EXCL`.

## Consequences

- **The AGNOS cross-build compiles** (`cyrius build --agnos`) — the concrete,
  verifiable M3 gate. The remaining acceptance (compiling ≠ working) is on-AGNOS
  QEMU+iron runtime validation, which stays a roadmap item.
- **AGNOS gaps are documented and fail-clean, not silent.** The absent `fsync`
  and `symlink` syscalls (filed agnos 1.51.x) are handled explicitly at the shim,
  and the rename/unlink ABI deltas are encapsulated in one place. When AGNOS
  gains a symlink syscall, only `ark_symlink` changes.
- **One place to audit.** New host/AGNOS-divergent syscalls go through
  `portable.cyr`; raw numbers at call sites are now a smell to catch in review.
- Adds a small indirection (one function call) on each affected path — measured
  to be within bench noise; the affected paths are I/O-bound (fsync/rename) or
  cold (timestamps), not hot loops.

## Alternatives considered

- **Leave the raw numbers; fix only `sys_symlink` to unbreak the link.**
  Rejected: the build would link but `time`/`fsync`/`rename` would invoke wrong
  syscalls on AGNOS — a silent runtime failure, the worst outcome.
- **Per-call `#ifdef` at each site.** Rejected: scatters the AGNOS coupling
  across four files (the very problem), and duplicates the branch logic.
- **Add the wrappers to the vendored stdlib (`lib/`).** Rejected: `lib/` is
  re-synced from the toolchain pin (`cyrius lib sync`) and would be overwritten;
  the ABI-normalizing wrappers are ark policy, not stdlib.
