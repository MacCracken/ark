# Worked example: build a `.ark` and install it with ark

This walks the full native chain end to end — **takumi builds a signed `.ark`,
ark verifies and installs it** — using the same fixtures ark's own test suite
uses (`tests/fixtures/sample-{signed,unsigned}.ark`).

## 1. The artifact

A `.ark` is takumi's output: a versioned binary container (see takumi
`docs/adr/0001-ark-binary-format.md`) holding a TOML manifest, a file index, a
DEFLATE-compressed data block, a SHA-256 **root hash** over the whole prefix,
and (optionally) an Ed25519 **signature** over that root hash. The test fixture
`sample-signed.ark` is a signed package named `myapp` 2.1.0 with two files:

```
/etc/app.conf      (config)
/usr/bin/app       (regular)
```

## 2. Install a local `.ark`

```bash
$ ark install ./tests/fixtures/sample-signed.ark --root /tmp/demo
OK: Installed myapp 2.1.0 (2 files)

$ ls -R /tmp/demo
/tmp/demo/etc/app.conf
/tmp/demo/usr/bin/app
```

`--root /tmp/demo` stages the install under a directory (no privilege needed).
Omit it to install to the real root (`/`), which requires appropriate privilege.

**What ark did, in order:**

1. **Read + verify** the `.ark` (`ark_pkg_read`): recompute the SHA-256 root
   hash and compare; verify the Ed25519 signature against the embedded pubkey;
   inflate the data block; **re-check every per-file SHA-256**. Any mismatch →
   the install aborts before touching disk (returns a clean error, no partial
   write).
2. **Materialize** each entry under the root, honoring its type — directories
   (`mkdir -p`), regular/config files (write the verified bytes), symlinks.
3. **Register** the package in `PackageDb` with its version, file list, per-file
   hashes, and signature.
4. **Record** an install transaction (begin → op → commit).

A tampered or truncated archive fails verification and installs nothing — the
conformance test flips a byte and asserts the read returns an error.

## 3. Install from a marketplace

When the `.ark` lives on a [mela](https://github.com/MacCracken/mela)
marketplace rather than on disk:

```bash
$ ark install myapp@2.1.0 --marketplace https://market.agnos.org/
$ ark install myapp --marketplace https://market.agnos.org/   # omit @version → resolve "latest"
```

ark builds a mela client (`registry_client_new`) and calls `mela_fetch_artifact`
to download the `.ark` into its cache (mela validates the name/version + builds
the URL; sandhi provides the HTTP/TLS transport), then runs the **exact same**
verify → materialize → register flow as the local install above. A cache hit
(the `.ark` already in the cache dir) skips the network. Omitting `@version`
resolves the latest published version first
(`ark_marketplace_resolve_latest`, mela's manifest endpoint) and installs that.

## 4. Verify an installed package later

```bash
$ ark verify myapp
```

re-reads the files recorded in `PackageDb` and re-computes their SHA-256, so you
can detect corruption or tampering after the fact. `PackageDb` is the
**authoritative, persistent** native store (schema v3 — it round-trips version,
files, per-file hashes, holds, and pins to disk), which is why this `ark verify`
works in a *separate, later* CLI invocation: the registration from step 2
survives across processes.

## Notes

- ark does **not** build packages — that's takumi's job. ark consumes the `.ark`
  takumi produces.
- `.ark` (system/app packages) is distinct from mela's `.agnos-agent` agent
  bundles; ark installs the former, mela's own pipeline handles the latter.
