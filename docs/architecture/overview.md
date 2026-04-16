# Architecture Overview

## Module Map

```
ark (Cyrius binary, v5.1.3)
  src/main.cyr      -- All types, engine, CLI parser, config, confirmation
  tests/ark.tcyr    -- 141 assertions across 9 test groups
  tests/ark.bcyr    -- 9 benchmarks (manual timing)
  tests/ark.fcyr    -- Fuzz harness (skeleton)
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
    +-- install/remove/upgrade --> nous stubs --> InstallPlan
    |
    +-- search/list/info/update --> nous stubs --> ArkOutput
    |
    +-- status --> ArkOutput (no resolver)
    |
    +-- hold/unhold --> PackageDb mutation
    |
    +-- verify --> pkg_db_check_integrity
    |
    +-- history --> txn_log_recent
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

### InstallPlan (iplan_*)
Plan-only model: ark never calls apt-get or installs packages directly.
Produces `InstallStep` list for shakti to execute with privileges.

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
| ArkCommand | 72B | tag, packages, query, group, package, source, force, purge, count |
| ArkOutputLine | 64B | tag, text, name, version, source, description, key, value |
| InstallStep | 32B | tag, package, version, purge |
| InstallPlan | 24B | steps, requires_root, estimated_size_bytes |
| ArkResult | 32B | success, message, packages_affected, source |
| ArkConfig | 72B | strategy, confirm_sys, confirm_rm, auto_update, color, mkt_dir, cache_dir, db_path, log_path |
| TransactionOp | 56B | op_type, package, version, source, status, error, from_version |
| Transaction | 56B | id, started_at, completed_at, status, error_msg, operations, user |
| TransactionLog | 24B | transactions, next_id, log_path |
| PackageDbEntry | 88B | name, version, source, installed_at, installed_by, size_bytes, checksum, files, dependencies, transaction_id, held |
| PackageDb | 16B | packages (hashmap), db_path |
| NousResolver | 24B | marketplace_dir, cache_dir, strategy |
| ArkPackageManager | 32B | config, resolver, package_db, transaction_log |

## Package Sources

| Source | Resolution | Install Method |
|--------|-----------|----------------|
| System | nous checks apt cache | apt-get install via shakti |
| Marketplace | nous checks marketplace index | Download + verify + extract |
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

- **nous** -- Dependency resolution (stubbed until ported to Cyrius)
- **stdlib** (18 modules) -- string, fmt, alloc, vec, str, syscalls, io,
  args, assert, hashmap, tagged, toml, json, fs, fnptr, callback, bench, regex

## Consumers

- CLI binary (direct user interaction)
- HTTP API (future -- programmatic access)
- System services (future -- auto-update, integrity checks)
