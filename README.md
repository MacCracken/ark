# Ark

**Ark** — Unified package manager for AGNOS.

The vessel that carries the [zugot](https://github.com/MacCracken/zugot) (recipes) and builds the world from their definitions. Named after the ark that preserves knowledge through destruction.

## What Ark Does

Ark is the user-facing CLI for all package operations on AGNOS. It translates user commands into execution plans, using [nous](https://github.com/MacCracken/nous) for dependency resolution and [takumi](https://github.com/MacCracken/takumi) recipes from [zugot](https://github.com/MacCracken/zugot) for build instructions.

Ark does **not** directly execute package operations. It generates `InstallPlan` instructions — a deliberate security design choice. Execution requires appropriate privileges via [shakti](https://github.com/MacCracken/shakti).

## Commands

```bash
ark install <package>          # Install a package
ark install --group <group>    # Install a package group (e.g., agnos-desktop)
ark remove <package>           # Remove a package
ark remove --purge <package>   # Remove package and its configuration
ark search <query>             # Search across all sources
ark list                       # List installed packages
ark list --source marketplace  # List packages from a specific source
ark info <package>             # Show package details
ark update                     # Refresh package indices
ark upgrade                    # Upgrade all packages with available updates
ark upgrade <package>          # Upgrade specific packages
ark status                     # Show system package status
```

## Architecture

```
User → ark (CLI/API)
         ├── nous (resolver) → dependency graph
         ├── zugot (recipes) → build instructions
         └── InstallPlan → shakti (privilege) → execution
```

### Key Design Decisions

- **Plan-based execution**: Ark generates plans, not side effects. The plan can be inspected, approved, and audited before anything is installed.
- **Source-aware**: Ark knows where packages come from — system, marketplace, or app bundle. Each source has its own install/remove/upgrade path.
- **Transactional**: Every operation is wrapped in a transaction with begin/commit/rollback. Failed installs don't leave the system in a broken state.
- **Integrity checking**: `PackageDb` tracks installed files with SHA-256 hashes. Detects corruption, tampering, and missing files.

## Package Sources

| Source | Description | Install Method |
|--------|-------------|----------------|
| **System** | Base OS packages built from zugot recipes | takumi build + ark install |
| **Marketplace** | AGNOS crates and consumer apps from [mela](https://github.com/MacCracken/mela) | Download signed .ark bundle |
| **Bazaar** | Community-contributed packages | `ark bazaar install <package>` |

## Types

### Core

| Type | Description |
|------|-------------|
| `ArkPackageManager` | Main engine — wraps config + nous resolver |
| `ArkCommand` | Parsed CLI command (Install, Remove, Search, List, Info, Update, Upgrade, Status) |
| `ArkResult` | Operation result (success, message, affected packages, source) |
| `ArkConfig` | Configuration (directories, default strategy, sources) |

### Planning

| Type | Description |
|------|-------------|
| `InstallPlan` | Ordered list of steps to execute, with root requirement flag |
| `InstallStep` | Individual operation (SystemInstall, MarketplaceInstall, FlutterInstall, Remove variants) |

### Transactions

| Type | Description |
|------|-------------|
| `TransactionLog` | Persistent log of all package operations |
| `Transaction` | Single atomic operation (begin → ops → commit/rollback/fail) |
| `TransactionOp` | Individual step within a transaction |

### Package Database

| Type | Description |
|------|-------------|
| `PackageDb` | Registry of installed packages with file tracking |
| `PackageDbEntry` | Single installed package (name, version, source, files, hashes, size) |
| `IntegrityIssue` | Result of integrity check (missing, corrupted, orphaned files) |

## Dependencies

| Crate | Purpose |
|-------|---------|
| [nous](https://github.com/MacCracken/nous) | Dependency resolution |
| anyhow | Error handling |
| serde / serde_json | Serialization |
| sha2 | Integrity hashing |
| tracing | Structured logging |
| uuid | Transaction IDs |
| chrono | Timestamps |

## Package Groups

Ark supports meta-package groups for bulk installation:

| Group | Installs |
|-------|----------|
| `agnos-desktop` | Full desktop environment |
| `agnos-edge` | Edge/IoT minimal profile |
| `agnos-dev` | Development tools |
| `agnos-ai` | AI/ML stack |
| `agnos-science` | Science crate collection |

## Related

- [nous](https://github.com/MacCracken/nous) — Package resolver (the mind that figures out dependencies)
- [takumi](https://github.com/MacCracken/takumi) — Build system (the craftsman that builds from recipes)
- [zugot](https://github.com/MacCracken/zugot) — Recipe repository (the paired definitions that ark carries)
- [mela](https://github.com/MacCracken/mela) — Marketplace (where packages are discovered)
- [sigil](https://github.com/MacCracken/sigil) — Trust verification (package signing)
- [shakti](https://github.com/MacCracken/shakti) — Privilege escalation (execution permissions)
- [AGNOS Philosophy](https://github.com/MacCracken/agnosticos/blob/main/docs/philosophy.md) — Why ark is named after the vessel that preserves knowledge

## License

GPL-3.0-only
