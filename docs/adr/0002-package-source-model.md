# 0002 — Package source model: pluggable system backend, apt-compat bridge, native target

- **Status**: accepted (direction); native backend deferred to v2
- **Date**: 2026-06-17
- **Deciders**: ark maintainers
- **Related**: [ADR 0001 — Execution backend via shakti](0001-shakti-execution-backend.md);
  roadmap "v2.0 — Native, apt-independent package management"

## Context

ark resolves and installs from several sources. `SOURCE_MARKETPLACE`/
`SOURCE_COMMUNITY` build from zugot recipes via takumi; `SOURCE_FLUTTER_APP`
uses agpkg; `SOURCE_SYSTEM` **is apt** — nous's `sysdb` shells out to
`apt-cache`/`dpkg-query` for resolution and the executor (ADR 0001) lowers
system steps to `apt-get`.

That coupling has a hard consequence: **AGNOS is not a Debian system.** apt is a
large Debian-native toolchain that assumes the Linux/Debian syscall surface,
dpkg's `/var/lib/dpkg` state, and Debian repos. It will not run on AGNOS as-is.
So today there are exactly two ways ark can serve the system source on AGNOS:

1. Build the **native** AGNOS package backend (the v2 goal) — substantial work
   spanning ark, nous, takumi, and a signed native repo.
2. **Run apt on AGNOS through a compatibility shim** — an "AGNOS syscall
   wrapper" that translates/services the syscalls apt and dpkg make, so the
   unmodified Debian toolchain runs under AGNOS until native lands.

Without one of these, **AGNOS cannot use ark for system packages at all.** The
native backend is not going to be ready for AGNOS's first usable releases, so
the apt-compat shim is the interim bridge — and it must not become load-bearing
architecture we can't back out of.

The risk if we don't decide now: the apt assumption is currently spread across
nous (`sysdb`) and ark (executor lowering). If it stays implicit, swapping
apt → native (or apt-direct → apt-via-wrapper) becomes a cross-cutting rewrite
instead of a backend swap.

## Decision

**Introduce an explicit *system backend* abstraction now (v1), with apt as the
first implementation, so the interim compat bridge and the v2 native backend are
interchangeable implementations behind one seam — not forks of the code.**

A system backend answers two questions for the `SOURCE_SYSTEM` path:

- **Resolution** (nous): does package X exist, at what version, with what deps?
- **Execution** (ark, via ADR 0001's shakti path): what argv installs/removes X?

Define three backend modes, selected by config (`ArkConfig.system_backend`, and
the matching nous resolver source), defaulting per host:

| Mode | Resolution | Execution lowering | Use |
|---|---|---|---|
| `apt` | `apt-cache`/`dpkg-query` (today) | `apt-get …` | Debian dev/CI, Debian-base hosts |
| `apt-agnos` | apt tools **via the AGNOS syscall wrapper** | wrapper-fronted `apt-get …` | **AGNOS, interim** — until native lands |
| `native` | native repo index + ark `PackageDb` store | native install/remove | **AGNOS, v2 target** |

Key points:

- **The apt-compat bridge is a backend mode, not a special case.** `apt-agnos`
  reuses all the apt resolution/lowering logic; the only difference is that the
  apt/dpkg invocations are fronted by the AGNOS syscall wrapper (an
  AGNOS-provided shim — ark/shakti invoke apt *through* it). ark does not
  implement the wrapper; it only needs to know to route apt commands through it
  for this mode. The exact prefix/path is configurable so AGNOS owns the shim's
  shape.
- **The lowering already routes through shakti** (ADR 0001). The backend mode
  decides the *inner* command (`apt-get …` vs wrapper-fronted vs native), shakti
  still provides privilege. So this composes cleanly with the executor.
- **`native` demotes apt to optional.** Once the native store + resolver exist,
  the default on AGNOS flips to `native`; `apt`/`apt-agnos` remain available as
  a compatibility strategy and for Debian interop, behind a capability flag.
- **Resolution stays in nous.** ark selects the source mode; nous gains the
  `SOURCE_NATIVE` resolver backend in v2. ark never reimplements resolution.

## Consequences

- **Near-term (v1):** a small, mostly-mechanical refactor — thread a
  `system_backend` setting through `ArkConfig` and the executor's lowering, and
  through nous's `sysdb`/strategy selection. apt behavior is unchanged when the
  mode is `apt`. This is cheap to do now and expensive to retrofit later.
- **`apt-agnos` is testable before AGNOS hardware** using the same stubbed-shakti
  / recording-command harness planned for the executor: assert that the apt
  argv is correctly wrapper-fronted, without needing a live wrapper.
- ark takes a **documented runtime dependency on the AGNOS syscall wrapper** for
  the `apt-agnos` mode. Its absence/misconfig must produce a clear diagnostic.
  This dependency is explicitly **temporary** — it exists to make AGNOS usable
  before v2 and is expected to be retired (or kept only for Debian interop) once
  `native` ships.
- **Migration:** v2 adds the `native` backend and a one-time importer for an
  existing dpkg set; the cutover is a default-mode change plus the capability
  flag, not a rewrite — which is the whole point of fixing the seam now.
- Adds a config surface and a small amount of indirection in the system path.
  Acceptable: it replaces an implicit, untyped apt assumption with an explicit,
  testable one.

## Alternatives considered

- **Stay apt-only until native is ready.** Rejected: AGNOS cannot run apt
  unaided, so this means AGNOS has no system package management until the entire
  v2 native backend ships — too long a gap, and it leaves the apt coupling
  implicit and hard to remove.
- **Jump straight to native, skip the apt bridge.** Rejected: blocks any usable
  AGNOS release on completing v2; throws away the working apt path for Debian
  dev/CI and interop.
- **Wrap apt but hard-code the wrapper invocation inline in the executor.**
  Rejected: bakes the temporary bridge into load-bearing code and conflates
  "which backend" with "how to escalate" (ADR 0001). The backend seam keeps the
  bridge swappable and the native cutover a config change.
- **ark implements the AGNOS syscall wrapper itself.** Rejected: the wrapper is
  OS-level syscall translation — AGNOS's responsibility, not the package
  manager's. ark only routes apt commands through it.
