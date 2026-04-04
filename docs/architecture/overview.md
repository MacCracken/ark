# Architecture Overview

## Module Map

```
ark (binary crate)
├── lib.rs          — ArkPackageManager engine, CLI parser, group resolution
├── types.rs        — All public types, enums, data structures
└── tests.rs        — Unit tests (82 active, 2 ignored)
```

## Data Flow

```
User Input
    │
    ▼
parse_args(&[&str]) ──→ ArkCommand
    │
    ▼
ArkPackageManager::execute(&ArkCommand)
    │
    ├── install/remove/upgrade ──→ nous::NousResolver
    │                                  │
    │                                  ▼
    │                              ResolvedPackage (source, version, deps)
    │                                  │
    │                                  ▼
    │                              InstallPlan { steps, requires_root, estimated_size }
    │
    ├── search/list/info/update ──→ nous::NousResolver ──→ ArkOutput
    │
    └── status ──→ ArkOutput (no resolver call)
    │
    ▼
ArkResult { success, message, packages_affected, source }
```

## Key Components

### ArkPackageManager

The main engine. Holds an `ArkConfig` and a `nous::NousResolver`. All operations go through `execute()`, which dispatches to specialized methods.

### InstallPlan

A plan-only model: ark never directly calls apt-get or installs packages. It produces a list of `InstallStep` variants that an external caller (with privileges) can execute. This separation is a deliberate security design.

### TransactionLog

Append-only JSONL persistence for crash recovery. Every state-changing operation is wrapped in a transaction: begin -> add_op -> commit/rollback/fail. Survives corrupt entries on reload.

### PackageDb

Unified registry of all installed packages across all sources. Tracks files per package for clean removal and integrity checking. Supports ownership queries (which package owns a file) and topological dependency ordering.

## Package Sources

| Source | Resolution | Install Method |
|--------|-----------|----------------|
| System | `nous` checks apt cache | `apt-get install` via shakti |
| Marketplace | `nous` checks marketplace index | Download + verify + extract |
| FlutterApp | `nous` checks flutter registry | `agpkg install` |
| Community | `nous` resolves from community repo | Build locally via takumi |

## Consumers

All AGNOS users interact with ark for package management. Ark is consumed by:

- CLI binary (direct user interaction)
- HTTP API (programmatic access)
- System services (auto-update, integrity checks)

## Dependencies

- **nous** — All dependency resolution, package search, update checking
- **serde/serde_json** — Serialization of all types (required for IPC, persistence, API)
- **chrono** — Timestamps in transactions and package database
- **sha2** — Integrity verification
- **tracing** — Structured logging throughout
- **uuid** — Transaction ID generation
- **anyhow** — Error handling with context
