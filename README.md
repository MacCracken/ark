# Ark

**Ark** — Unified package manager for AGNOS. (v1.1.3 — system-backend seam + native `.ark` installer + real `ark upgrade`.)

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
ark install --group <group>        # a package group (e.g., desktop → agnos-desktop)
ark install ./pkg.ark [--root DIR] # install a local .ark (verify → materialize)
ark install name[@ver] --marketplace <url>  # marketplace install (omit @ver to resolve "latest")
ark install <package> --system-backend native   # apt | apt-agnos | native (ADR 0002)

ark remove <package> [--purge] [--apply]    # remove (and optionally its config)
ark search <query>                 # search across sources
ark list [--marketplace]           # installed packages (--system / --flutter / --marketplace filters)
ark info <package>                 # package details
ark update                         # check for updates (apt + marketplace, when marketplace_url set)
ark upgrade [<package>]            # upgrade all / specific pkgs — honors --dry-run/--apply, holds & pins; detects marketplace updates
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
         └── InstallPlan ──┬── system step → shakti (privilege) → apt-get / wrapper
                           └── native/marketplace step → exec_native_step
                                  → verify → materialize → PackageDb
```

Privileged system steps escalate through shakti; native and marketplace steps
run through ark's own `.ark` installer into the authoritative `PackageDb`.
Syscall-divergent paths route through `src/portable.cyr` (ADR 0003).

### Key Design Decisions

- **Plan-first execution**: `install`/`remove` produce an inspectable plan and run nothing without `--apply`; privileged steps go through shakti.
- **Source-aware**: Ark knows where packages come from — system, marketplace, or app bundle. Each source has its own install/remove/upgrade path.
- **Transactional**: Every operation is wrapped in a transaction with begin/commit/rollback. Failed installs don't leave the system in a broken state.
- **Integrity checking**: `PackageDb` tracks installed files with SHA-256 hashes. Detects corruption, tampering, and missing files.

## Package Sources

| Source | Description | Install Method |
|--------|-------------|----------------|
| **System** | Base OS packages | backend-selectable (`--system-backend`, ADR 0002): `apt-get` via shakti (default on Debian), wrapper-fronted `apt-agnos`, or the native `.ark` store (default on AGNOS) |
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
| [sandhi](https://github.com/MacCracken/sandhi) | HTTP/TLS transport for marketplace downloads (direct dep; also mela's) |
| [agnostik](https://github.com/MacCracken/agnostik) | Shared AGNOS manifest types (direct dep; also mela's) |
| `bayan` / `sankoch` (stdlib) | TOML/JSON + DEFLATE for `.ark` |

## Package Groups

Ark supports meta-package groups for bulk installation:

| `--group` | Meta-package |
|-----------|--------------|
| `desktop` | `agnos-desktop` |
| `ai` / `ml` | `agnos-ai` |
| `shell` | `agnoshi` |
| `edge` / `iot` | `agnos-edge-agent` |

## Configuration

Ark reads an optional TOML config. Search order (first match wins, then
built-in defaults):

1. `$ARK_CONFIG`
2. `./ark.toml`
3. `~/.config/ark/ark.toml`
4. `/etc/agnos/ark.toml`

User-writable sources (1–3) are skipped when running as root (euid 0), so a
privileged invocation only honors `$ARK_CONFIG` and `/etc/agnos/ark.toml`.

| Key | Meaning |
|-----|---------|
| `color_output` | ANSI color (bool) |
| `confirm_system_installs` / `confirm_removals` | prompt before applying (bool) |
| `auto_update_check` | check for updates on relevant commands (bool) |
| `marketplace_dir` / `cache_dir` | local marketplace + download cache dirs |
| `system_backend` | `apt` \| `apt-agnos` \| `native` — mirrors `--system-backend` (ADR 0002; default `native` on AGNOS) |
| `apt_wrapper` | command that fronts apt under the `apt-agnos` backend |
| `marketplace_url` | base URL for marketplace update detection; **empty (default) ⇒ `ark update`/`ark upgrade` stay apt-only and make no network calls** |

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
