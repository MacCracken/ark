# References & Sources

ark is largely a **delegating crate**: the heavy domain algorithms it relies on
live in dependencies (crypto in sigil, compression in sankoch, dependency
resolution and semver in nous), so ark cites *their* sources rather than
re-deriving them. What ark owns directly is the `.ark` reader/writer (a binary
container format defined by takumi) and the command-lowering for each package
source. This file maps each module to the source that governs it, so a reviewer
can trace any behavior back to its specification.

| Module | Domain | Source |
|--------|--------|--------|
| `src/ark_package.cyr` | `.ark` v1 binary format (read/verify/materialize) | takumi `docs/adr/0001-ark-binary-format.md` — the authoritative container spec (manifest, file index, DEFLATE data block, SHA-256 root hash, Ed25519 signature) |
| `src/db.cyr`, `src/ark_package.cyr` | Package integrity & signing (SHA-256 root hash, per-file hashes, Ed25519 signatures) | [sigil](https://github.com/MacCracken/sigil) — SHA-256 (FIPS 180-4) and Ed25519 (RFC 8032) |
| `src/ark_package.cyr` | DEFLATE decompression of the `.ark` data block | [sankoch](https://github.com/MacCracken/sankoch) — DEFLATE (RFC 1951) |
| `src/engine.cyr` | Dependency resolution, semver comparison, update detection | [nous](https://github.com/MacCracken/nous) — resolution strategy + `resolver_check_updates`; SemVer 2.0.0 |
| `src/exec.cyr` | System-source command lowering | apt/dpkg CLIs (`apt-get`, `dpkg-query`) for the `apt`/`apt-agnos` backends; agpkg for Flutter apps; ark's native `.ark` installer for the `native` backend (ADR 0002) |
| `src/marketplace.cyr` | Marketplace download + `/latest` resolution | [mela](https://github.com/MacCracken/mela) — marketplace client + publisher keyring; [sandhi](https://github.com/MacCracken/sandhi) HTTP/TLS transport |
| `src/recipe.cyr` | zugot recipe parsing (`.cyml`) | [zugot](https://github.com/MacCracken/zugot) recipe format; `bayan` TOML |

## Design records

Decisions and their rationale are recorded as ADRs:

- [ADR 0001 — Execution backend via shakti](adr/0001-shakti-execution-backend.md)
- [ADR 0002 — Package source model (pluggable system backend)](adr/0002-package-source-model.md)
- [ADR 0003 — Syscall portability layer](adr/0003-syscall-portability-layer.md)

## No undocumented constants

ark introduces no domain-specific magic numbers of its own — sizes and limits
(e.g. the `.ark` format offsets, the DB read ceiling) are either dictated by the
formats above or documented inline at their definition.
