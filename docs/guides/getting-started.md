# Getting started with ark

ark is the AGNOS package manager CLI. It resolves what to install (via
[nous](https://github.com/MacCracken/nous)), builds a typed **plan**, and — only
when you ask — executes it, escalating privileged steps through
[shakti](https://github.com/MacCracken/shakti). Packages are takumi-built
`.ark` artifacts, verified (SHA-256 root hash + Ed25519 signature + per-file
hashes) before anything touches disk.

> **Plan-first, by default.** `ark install`/`ark remove` produce a plan and do
> **not** change your system. Add `--dry-run` to see the exact commands, or
> `--apply` to actually run them.

## The execution model

```
ark install nginx              # → resolves + prints the plan (nothing runs)
ark install nginx --dry-run    # → prints the concrete commands it WOULD run,
                               #   e.g.  shakti -- apt-get install -y nginx
ark install nginx --apply      # → runs the plan (prompts to confirm first)
```

- **Plan** (default): resolve + show what would happen. Safe, read-only.
- **`--dry-run`**: lower each step to its concrete command (`shakti -- …` for
  privileged steps) and print it — still runs nothing.
- **`--apply`**: execute. System/marketplace installs and removals prompt for
  confirmation (unless disabled in config). Each step is recorded to the
  transaction log so a failure can be rolled back.

Privileged steps are wrapped as `shakti -- <cmd>`; shakti authenticates,
checks policy, and runs them. ark itself never holds elevated privilege.

> **Backend-aware lowering.** The `apt-get …` command above is the concrete
> lowering only for the default `apt` system backend. Under `apt-agnos` the same
> `apt-get` is fronted by the configured AGNOS syscall wrapper, and under
> `native` (the default on AGNOS) system installs route to ark's own `.ark`
> installer with no external apt command at all. shakti still escalates
> regardless. Select with `--system-backend` or the `system_backend` config key
> (see [ADR 0002](../adr/0002-package-source-model.md)).

## Installing packages

```bash
# By name (resolved via nous: system / marketplace / Flutter)
ark install nginx
ark install nginx --apply

# A package group (short name expands to a meta-package: desktop → agnos-desktop)
ark install --group desktop

# A local .ark file directly (verifies, then installs)
ark install ./nginx-1.25.0.ark
ark install ./nginx-1.25.0.ark --root /tmp/stage   # stage under a dir (no root needed)

# From a mela marketplace (downloads the .ark, verifies, installs)
ark install nginx@1.25.0 --marketplace https://market.agnos.org/
ark install nginx --marketplace https://market.agnos.org/   # omit @version → resolve "latest"
```

`--root <dir>` materializes the install under a directory instead of `/` —
useful for inspection or unprivileged testing. The default (real root) needs
appropriate privilege.

## Removing packages

```bash
ark remove nginx                # plan
ark remove nginx --purge        # also remove configuration
ark remove nginx --apply        # execute (prompts to confirm)
```

## Updating & upgrading

```bash
ark update                      # check for updates (no changes made)
ark upgrade                     # plan an upgrade of everything eligible
ark upgrade nginx curl          # plan an upgrade of just these packages
ark upgrade --dry-run           # show the concrete upgrade commands
ark upgrade --apply             # execute (prompts to confirm)
```

`ark upgrade` is plan-first like install/remove. It builds a **backend-aware**
plan, **skips held packages and version/source-pinned ones**, accepts an
optional package filter, and applies in a rollback-able transaction. `ark update`
just reports what's available.

Update detection is multi-source: apt via nous's sysdb, plus marketplace/
community via mela's `/latest`. **Marketplace/community detection only fires when
`marketplace_url` is configured** — otherwise both commands stay apt-only and
make no network calls.

## Inspecting & querying

```bash
ark search <query>              # search across sources
ark list                        # installed packages
ark list --marketplace          # filter by source: --marketplace/--market, --system/--apt, --flutter
ark info <package>              # package details
ark status                      # version, strategy, marketplace/cache dirs
ark history [count]             # recent transactions
ark verify [package]            # re-check installed files against stored SHA-256
```

## Holds, pins, rollback, backup

```bash
ark hold <pkg>     /  ark unhold <pkg>     # prevent / allow upgrades
ark pin <pkg>      /  ark unpin <pkg>      # lock version (--source to also pin the source)
ark rollback [txn-id]                      # reverse a committed transaction (latest if omitted)
ark backup         /  ark restore <path>   # snapshot / restore the package DB
ark bazaar <subcmd> <query>                # browse the local community catalog
```

## Global flags

- `--no-color` — disable ANSI color output.
- `--apply` / `--dry-run` — see the execution model above (install/remove/upgrade).
- `--root <dir>` — install root for local/marketplace `.ark` installs.
- `--system-backend <apt|apt-agnos|native>` — how `SOURCE_SYSTEM` steps lower
  (default `native` on AGNOS, `apt` elsewhere). Mirrors the `system_backend`
  config key. See [ADR 0002](../adr/0002-package-source-model.md).

## Configuration

ark reads an optional TOML config (first match wins, then defaults):
`$ARK_CONFIG` → `./ark.toml` → `~/.config/ark/ark.toml` → `/etc/agnos/ark.toml`.
User-writable sources are skipped when running as root.

| Key | Meaning |
|-----|---------|
| `color_output` | ANSI color |
| `confirm_system_installs` / `confirm_removals` | prompt before applying |
| `auto_update_check` | check for updates on relevant commands |
| `marketplace_dir` / `cache_dir` | local marketplace + download cache |
| `system_backend` / `apt_wrapper` | mirror `--system-backend` + the apt-agnos wrapper command |
| `marketplace_url` | base URL for marketplace update detection; empty (default) ⇒ `ark update`/`ark upgrade` stay apt-only with no network calls |

## How a package gets installed (the chain)

```
zugot recipe (.cyml)
   → takumi build            → produces a signed .ark
   → mela (marketplace)      → serves/publishes the .ark
   → ark install             → download (via mela) → verify (root hash + sig +
                               per-file hashes) → materialize files →
                               register in PackageDb → record transaction
```

See [ADR 0001](../adr/0001-shakti-execution-backend.md) (execution backend) and
[ADR 0002](../adr/0002-package-source-model.md) (source model) for the design,
and [the install example](../examples/install-a-package.md) for a worked
end-to-end run.
