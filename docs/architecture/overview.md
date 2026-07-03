# Architecture Overview

## Module Map

```
ark (Cyrius binary; toolchain cyrius 6.3.38)
  src/types.cyr       -- constants, enums, struct accessors (ArkConfig, ArkCommand, InstallPlan/Step)
  src/portable.cyr    -- host/agnos syscall portability shims (ADR 0003)
  src/output.cyr      -- ArkOutputLine, ArkOutput, rendering
  src/transaction.cyr -- TransactionOp / Transaction / TransactionLog (JSONL, crash-safe)
  src/db.cyr          -- PackageDbEntry / PackageDb, integrity, signing, persistent round-trip
  src/engine.cyr      -- nous glue, ArkPackageManager, command dispatch, upgrade detection
  src/exec.cyr        -- executor: backend-aware lowering, shakti wrap, exec_plan, native routing
  src/recipe.cyr      -- zugot recipe parsing (.cyml)
  src/ark_package.cyr -- `.ark` reader/verify + native install/remove
  src/marketplace.cyr -- mela marketplace download, resolve-"latest", avail-lookup
  src/bazaar.cyr      -- community recipe catalog browse
  src/cli.cyr         -- parse_args, config load, confirmation, main()
  src/main.cyr        -- include barrel + entrypoint
  tests/ark.tcyr      -- 399 assertions across the test groups
  tests/ark.bcyr      -- benchmarks (manual timing)
  fuzz/ark.fcyr       -- fuzz harnesses (txn-log / pkg-name / CLI / JSON-escape / recipe)
```

## Data Flow

```
User Input
    |
    v
parse_args(ac, av) --> ArkCommand (tagged struct)
    |
    v
ark_execute(mgr, cmd)
    |
    +-- install/remove/upgrade --> nous resolver --> InstallPlan --> (executor, on --apply)
    |   install ./pkg.ark / name@ver --marketplace --> ark_pkg_install (verify+materialize)
    |
    +-- search/list/info/update --> nous resolver --> ArkOutput
    |
    +-- status --> ArkOutput (no resolver)
    |
    +-- hold/unhold --> PackageDb mutation
    |
    +-- pin/unpin --> PackageDb mutation (version + source pins)
    |
    +-- verify --> pkg_db_check_integrity
    |
    +-- history --> txn_log_recent
    |
    +-- rollback --> ark_rollback (source-aware reverse plan --> executor)
    |
    +-- backup/restore --> PackageDb snapshot to/from path
    |
    +-- bazaar --> community catalog browse (local)
    |
    v
ArkResult { success, message, packages_affected, source }
    |
    v
ark_output_render(output, color) --> stdout
```

## Key Components

### ArkPackageManager (amgr_*)
The main engine. Holds config, resolver, package_db, transaction_log.
All operations go through `ark_execute()`.

### InstallPlan + executor (iplan_* / src/exec.cyr)
Plan-first: `install`/`remove` build an `InstallStep` list and, by default, only
display it. `--dry-run` renders the concrete commands; `--apply` runs them — the
executor lowers each step (`step_to_argv_be`) and wraps privileged ones as
`shakti -- …` (ark never holds privilege itself). The **system backend**
(`ArkConfig.system_backend`: `apt` / `apt-agnos` / `native`; ADR 0002) decides
the inner command for `SOURCE_SYSTEM` steps — direct `apt-get`, wrapper-fronted
`apt-get`, or ark's native `.ark` installer. Native and marketplace/community
steps route to `ark_pkg_install` / `ark_pkg_remove_inner` (`exec_native_step`):
verify (root hash + signature + per-file hashes) → materialize → register in
`PackageDb`, recording the op in the plan-wide transaction so `ark_rollback`
reverses it. See [ADR 0001](../adr/0001-shakti-execution-backend.md) and
[ADR 0002](../adr/0002-package-source-model.md).

### Portability shims (ark_* / src/portable.cyr)
Host/AGNOS-divergent syscalls (`time`, `fsync`, `rename`, `unlink`, `symlink`)
go through `#ifdef`-guarded wrappers so one source tree compiles for both
x86_64-Linux and AGNOS, each resolving to the correct Sys number/ABI. See
[ADR 0003](../adr/0003-syscall-portability-layer.md).

### TransactionLog (txn_log_*)
Append-only JSONL persistence via `file_append_locked()` for crash
recovery and concurrent access safety. Transaction lifecycle:
begin -> add_op -> mark_op_complete/failed -> commit/rollback/fail.

### PackageDb (pkg_db_*)
Hashmap-backed registry of installed packages across all sources.
Tracks files per package, ownership queries, dependency ordering
(topological sort with cycle detection), hold/unhold state.
Atomic save via temp+rename pattern.

### ArkConfig (acfg_*)
TOML config with search order: $ARK_CONFIG, ./ark.toml,
~/.config/ark/ark.toml, /etc/agnos/ark.toml, defaults.

## Struct Layout (Cyrius i64 model)

All structs use explicit offsets with accessor functions (load64/store64)
due to compiler stdin pipe limitation with dot notation.

| Struct | Size | Fields |
|--------|------|--------|
| ArkCommand | 88B | tag, packages, query, group, package, source, force, purge, count, root, mkt_url |
| ArkOutputLine | 64B | tag, text, name, version, source, description, key, value |
| InstallStep | 32B | tag, package, version, purge |
| InstallPlan | 24B | steps, requires_root, estimated_size_bytes |
| ArkResult | 32B | success, message, packages_affected, source |
| ArkConfig | 136B | strategy, confirm_sys, confirm_rm, auto_update, color, mkt_dir, cache_dir, db_path, log_path, shakti_path, apply, trust_keys, require_signed, system_backend, apt_wrapper, native_root, marketplace_url |
| TransactionOp | 56B | op_type, package, version, source, status, error, from_version |
| Transaction | 56B | id, started_at, completed_at, status, error_msg, operations, user |
| TransactionLog | 32B | transactions, next_id, log_path, index |
| PackageDbEntry | 128B | name, version, source, installed_at, installed_by, size_bytes, checksum, files, dependencies, transaction_id, held, file_checksums, signature, signer_key_id, pinned_version, pinned_source |
| PackageDb | 24B | packages (hashmap), db_path, loaded_ok |
| NousResolver | 24B | marketplace_dir, cache_dir, strategy |
| ArkPackageManager | 32B | config, resolver, package_db, transaction_log |

## Package Sources

| Source | Resolution | Install Method |
|--------|-----------|----------------|
| System | nous checks apt cache (`apt`/`apt-agnos`) or native index (`native`) | backend-aware (ADR 0002): `apt-get` via shakti, wrapper-fronted apt, or ark's native `.ark` installer |
| Marketplace | nous checks marketplace index | Download + verify + extract → PackageDb |
| FlutterApp | nous checks flutter registry | agpkg install |
| Community | nous resolves from community repo | Build locally via takumi |

## Security Design

- **Plan-based execution**: Never executes installs directly
- **Package name validation**: Rejects traversal, special chars, null bytes
- **JSON escaping**: All serialized fields escaped to prevent injection
- **Locked log writes**: file_append_locked() for concurrent safety
- **Atomic DB writes**: temp+rename pattern for crash safety
- **Input sanitization**: CLI args validated before processing

## Dependencies

ark consumes single-file `dist/*.cyr` library bundles (pinned in `cyrius.lock`)
plus the Cyrius stdlib. It never reimplements resolution, crypto, or transport.

- **nous** (1.3.0) -- Dependency resolution across system / marketplace /
  flutter / community; pluggable `resolver_check_updates` for update detection.
- **sigil** (3.9.7) -- SHA-256 + Ed25519 (package integrity & signatures).
- **mela** (1.0.1) -- Marketplace client (download, `/latest` resolve) +
  publisher keyring (the trust set for signature verification).
- **sandhi** (1.7.0) -- HTTP/TLS transport for marketplace downloads (direct
  dep; also pulled transitively by mela).
- **agnostik** (1.3.1) -- Shared AGNOS manifest types (direct dep; also mela's).
- **stdlib** (`bayan` TOML/JSON, `sankoch` DEFLATE, `bench`, etc.) -- string,
  fmt, alloc, vec, str, syscalls, io, args, assert, hashmap, tagged, toml, json,
  fs, fnptr, callback, bench, regex.

## Consumers

- CLI binary (direct user interaction)
- HTTP API (future -- programmatic access)
- System services (future -- auto-update, integrity checks)
