# Ark

**Ark** — Unified package manager for AGNOS. (v1.0.0 — Cyrius port complete.)

The vessel that carries the [zugot](https://github.com/MacCracken/zugot) (recipes) and builds the world from their definitions. Named after the ark that preserves knowledge through destruction.

## What Ark Does

Ark is the user-facing CLI for all package operations on AGNOS. It translates user commands into execution plans, using [nous](https://github.com/MacCracken/nous) for dependency resolution and [takumi](https://github.com/MacCracken/takumi) recipes from [zugot](https://github.com/MacCracken/zugot) for build instructions.

Ark is **plan-first**: `install`/`remove` resolve and show a plan by default and change nothing. `--dry-run` prints the concrete commands; `--apply` executes them, escalating privileged steps through [shakti](https://github.com/MacCracken/shakti) (ark itself never holds privilege). Native `.ark` packages are verified (SHA-256 root hash + Ed25519 signature + per-file hashes) before any file is written.

## Commands

```bash
# Install — plan by default; --dry-run shows concrete commands; --apply runs them
ark install <package>              # resolve + show the plan (nothing runs)
ark install <package> --apply      # execute (prompts to confirm)
ark install <package> --dry-run    # show the exact commands it would run
ark install --group <group>        # a package group (e.g., agnos-desktop)
ark install ./pkg.ark [--root DIR] # install a local .ark (verify → materialize)
ark install name[@ver] --marketplace <url>  # marketplace install (omit @ver to resolve "latest")
ark install <package> --system-backend native   # apt | apt-agnos | native (ADR 0002)

ark remove <package> [--purge] [--apply]    # remove (and optionally its config)
ark search <query>                 # search across sources
ark list [--source marketplace]    # installed packages (--system / --flutter filters)
ark info <package>                 # package details
ark update                         # refresh package indices
ark upgrade [<package>]            # upgrade all / specific packages
ark status                         # version, strategy, dirs
ark verify [<package>]             # re-check installed files vs stored SHA-256
ark history [count]                # recent transactions
ark hold/unhold <pkg>              # prevent / allow upgrades
ark pin/unpin <pkg>                # lock version/source
ark rollback [txn-id]              # reverse a committed transaction
ark backup / ark restore <path>    # snapshot / restore the package DB
ark bazaar <subcmd> <query>        # browse the community catalog
```

**Plan-first:** `install`/`remove` change nothing without `--apply`. See the
[getting-started guide](docs/guides/getting-started.md) and a
[worked install example](docs/examples/install-a-package.md).

## Architecture

```
User → ark (CLI/API)
         ├── nous (resolver) → dependency graph
         ├── zugot (recipes) → build instructions
         └── InstallPlan → shakti (privilege) → execution
```

### Key Design Decisions

- **Plan-first execution**: `install`/`remove` produce an inspectable plan and run nothing without `--apply`; privileged steps go through shakti.
- **Source-aware**: Ark knows where packages come from — system, marketplace, or app bundle. Each source has its own install/remove/upgrade path.
- **Transactional**: Every operation is wrapped in a transaction with begin/commit/rollback. Failed installs don't leave the system in a broken state.
- **Integrity checking**: `PackageDb` tracks installed files with SHA-256 hashes. Detects corruption, tampering, and missing files.

## Package Sources

| Source | Description | Install Method |
|--------|-------------|----------------|
| **System** | Base OS packages | `apt-get` via shakti today; native `.ark` store is the v2 direction (see roadmap / ADR 0002) |
| **Marketplace** | AGNOS apps & agents from [mela](https://github.com/MacCracken/mela) | download signed `.ark` (verify root hash + trusted Ed25519 signer) → install |
| **Bazaar** | Community-contributed packages | `ark bazaar` (catalog browse) |

## Types

### Core

| Type | Description |
|------|-------------|
| `ArkPackageManager` | Main engine — wraps config + nous resolver |
| `ArkCommand` | Parsed CLI command (Install, Remove, Search, List, Info, Update, Upgrade, Status, Hold/Unhold, Pin/Unpin, Verify, History, Rollback, Backup/Restore, Bazaar) |
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

ark is written in [Cyrius](https://github.com/MacCracken/cyrius); it consumes
single-file `dist/*.cyr` library bundles plus the Cyrius stdlib (`bayan`,
`sankoch`, `bench`, etc.), pinned in `cyrius.lock`.

| Dependency | Purpose |
|------------|---------|
| [nous](https://github.com/MacCracken/nous) | Dependency resolution (system / marketplace / Flutter) |
| [sigil](https://github.com/MacCracken/sigil) | SHA-256 + Ed25519 (integrity & signatures) |
| [mela](https://github.com/MacCracken/mela) | Marketplace client (download) + publisher keyring |
| sandhi (via mela) | HTTP/TLS transport for marketplace downloads |
| agnostik (via mela) | Shared AGNOS manifest types |
| `bayan` / `sankoch` (stdlib) | TOML/JSON + DEFLATE for `.ark` |

## Package Groups

Ark supports meta-package groups for bulk installation:

| `--group` | Meta-package |
|-----------|--------------|
| `desktop` | `agnos-desktop` |
| `ai` / `ml` | `agnos-ai` |
| `shell` | `agnoshi` |
| `edge` / `iot` | `agnos-edge-agent` |

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
