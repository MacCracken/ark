# 0001 — Execution backend via shakti

- **Status**: accepted (review; implementation pending)
- **Date**: 2026-06-16
- **Deciders**: ark maintainers

## Context

ark plans package operations but does not yet execute them. `engine.cyr`
builds a typed `InstallPlan` of `InstallStep`s
(`STEP_SYSTEM_INSTALL`, `STEP_SYSTEM_REMOVE`, `STEP_MARKETPLACE_INSTALL`,
`STEP_FLUTTER_INSTALL`, `STEP_SYSTEM_UPDATE`, …) and renders it for the
user. **No step is ever run.** `process.cyr` is pulled in through the
stdlib but is never called from `src/`. The plan carries an
`iplan_root` flag marking whether it needs elevated privilege.

Installing system packages, writing under `/usr`, `/etc`, and the package
database requires root. ark must not itself be setuid. AGNOS already
ships **shakti** (0.7.0, Cyrius, pinned to the same toolchain 6.2.12) — a
`sudo`/`doas`-class privilege-escalation tool with PAM auth, a TOML
policy (`/etc/agnos/sudoers.toml`), audit logging, environment
sanitization, capability-based least-privilege, and a per-TTY credential
cache. The roadmap has always named the target shape: **plan → shakti →
system**. This ADR records the review of shakti's fitness and fixes the
integration contract before any executor code is written.

shakti offers two surfaces:

1. **Setuid CLI** — `build/shakti`, installed setuid-root at
   `/usr/bin/shakti`. Contract:
   `shakti [OPTIONS] [--] COMMAND [ARGS...]`, options `-u/--user`
   (default root), `-p/--policy`, `-k/--invalidate`, `-l/--list`,
   `-c/--check`. It authenticates the caller, authorizes against policy,
   sanitizes the environment, execs the command as the target user, and
   **propagates the child's exit code** (`WEXITSTATUS`, else
   `128+signal`; `1` on auth/policy failure).
2. **Library API** — `dist/shakti.cyr` (excludes `main.cyr`), exposing
   `evaluate(...)` / `evaluate_with_policy(...)` which return an
   evaluation struct (authorized?, resolved command, error code/msg,
   granted capabilities, MAC context). This decides authorization but
   **cannot itself escalate** — escalation lives in the setuid binary.

## Decision

ark will execute privileged steps by **invoking the setuid `shakti` CLI
as a subprocess** via `process.cyr`, not by linking `dist/shakti.cyr`.

- The executor walks `iplan_steps(plan)` in order. Each step is lowered
  to a concrete system command.
- When `iplan_root == 1`, the command is run as
  `shakti -- <argv...>` (adding `-u/--user` only for non-root targets).
  Unprivileged steps run directly through `process.cyr`.
- The executor checks the returned exit code per step, appends the result
  to the transaction log, and runs a post-step integrity check. A
  non-zero step aborts the plan; the recorded transaction enables
  `ark_rollback` to build (and, once the executor exists, run) the
  reverse plan.
- The shakti **binary path is configurable** (default `/usr/bin/shakti`)
  via `ArkConfig`, so tests and non-AGNOS hosts can substitute a stub.

Rationale for the exec model over the library:

- Escalation **requires** the setuid boundary; a library call inside
  ark's own unprivileged process cannot raise privilege.
- It keeps ark's TCB small: ark never holds elevated privilege, never
  parses `/etc/shadow`, never touches PAM. All of that stays behind
  shakti's audited boundary.
- It matches the established AGNOS pattern (one auditable escalation
  path, one audit log) and the documented `plan → shakti → system` flow.
- shakti's exit-code propagation gives ark exactly the per-step success
  signal it needs.

The library API is **not** discarded: ark may later use shakti's
`-l/--list` (or a thin `evaluate`-backed pre-check) to fail fast with a
clear "not permitted by policy" message *before* attempting a plan,
improving UX without changing the escalation path.

## Consequences

- ark gains a hard **runtime dependency on a correctly installed,
  setuid-root `/usr/bin/shakti`** plus a policy granting ark's operations.
  Absence/misconfig must produce a clear diagnostic, not a crash.
- A new executor module is needed in `engine.cyr` (or a dedicated
  `exec.cyr`): step lowering, subprocess invocation, exit-code handling,
  transaction recording, post-step verification. This is **large** — it
  will be broken into bites (lowering → unprivileged exec → shakti exec →
  rollback execution → dry-run), each gated on the standard
  cleanliness + test + benchmark loop.
- Testing must not require real root: the configurable shakti path lets
  the suite point at a recording stub and assert on the argv/exit-code
  contract. End-to-end-on-hardware remains a separate v1.0 criterion.
- **Plan signing for shakti verification** (backlog) layers on top: ark
  signs the plan/command set so shakti's policy can verify provenance.
- shakti is pre-1.0 (0.7.0); its CLI contract is stable but not frozen.
  ark should pin a shakti version and treat the argv/exit-code contract
  as the integration boundary to watch.

## Alternatives considered

- **Link `dist/shakti.cyr` and call `evaluate` directly.** Rejected:
  cannot escalate from ark's own process; would also drag PAM/policy
  parsing into ark's TCB.
- **ark itself setuid-root.** Rejected: enormous TCB, duplicates
  shakti's audited logic, defeats the reason shakti exists.
- **Shell out to `sudo`/`doas`.** Rejected: not the AGNOS-native path;
  loses shakti's policy model, capability dropping, and unified audit log.
