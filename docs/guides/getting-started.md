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

## Installing packages

```bash
# By name (resolved via nous: system / marketplace / Flutter)
ark install nginx
ark install nginx --apply

# A package group (expands to a meta-package)
ark install --group agnos-desktop

# A local .ark file directly (verifies, then installs)
ark install ./nginx-1.25.0.ark
ark install ./nginx-1.25.0.ark --root /tmp/stage   # stage under a dir (no root needed)

# From a mela marketplace (downloads the .ark, verifies, installs)
ark install nginx@1.25.0 --marketplace https://market.agnos.org/
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

## Inspecting & querying

```bash
ark search <query>              # search across sources
ark list                        # installed packages
ark list --source marketplace   # filter by source (--system / --flutter too)
ark info <package>              # package details
ark status                      # version, strategy, marketplace/cache dirs
ark history [count]             # recent transactions
ark verify [package]            # re-check installed files against stored SHA-256
```

## Holds, pins, rollback, backup

```bash
ark hold <pkg>     /  ark unhold <pkg>     # prevent / allow upgrades
ark pin <pkg>      /  ark unpin <pkg>      # lock version/source (--force to pin source)
ark rollback [txn-id]                      # reverse a committed transaction (latest if omitted)
ark backup         /  ark restore <path>   # snapshot / restore the package DB
ark bazaar <subcmd> <query>                # browse the local community catalog
```

## Global flags

- `--no-color` — disable ANSI color output.
- `--apply` / `--dry-run` — see the execution model above (install/remove).
- `--root <dir>` — install root for local/marketplace `.ark` installs.

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
