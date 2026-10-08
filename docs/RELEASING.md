# Releasing

Two channels are automated today. Both use **trusted publishing (OIDC)** — no
long-lived tokens.

## npm — `@airovo/rva-*` (Changesets)

Workflow: [`.github/workflows/release.yml`](../.github/workflows/release.yml).

1. Add a changeset with each PR:
   ```bash
   npm run changeset
   ```
2. On merge to `main`, the Release workflow opens/updates a **"Version Packages"**
   PR.
3. Merging that PR bumps versions, generates `CHANGELOG.md`s and publishes to npm
   via OIDC (provenance is generated automatically when the repository is public).

One-time setup: for each of the 7 packages, npmjs.com → package → Settings →
**Trusted Publisher** → GitHub Actions, with owner `airovo`, repo `rva`, workflow
`release.yml`. A package must already exist before OIDC works.

## crates.io — Rust workspace

Workflow: [`.github/workflows/release-crates.yml`](../.github/workflows/release-crates.yml).
Publishes in dependency order (`rva-core` first) via:

```bash
cargo publish --workspace
```

### Bootstrapping (first publish)

A crate must already exist before a trusted publisher can be configured, so the
very first publish uses a token:

```bash
cargo login                 # crates.io API token with publish scope
cargo publish --workspace   # wait for the index between crates
```

Then, on crates.io, for **each** crate → Settings → **Trusted Publishing** →
add GitHub, owner `airovo`, repo `rva`, workflow `release-crates.yml`.

### Subsequent releases

Bump `version` in the root `Cargo.toml` (`[workspace.package]`), then tag:

```bash
git tag v0.1.1
git push origin v0.1.1
```

The tag triggers the workflow, which authenticates over OIDC.

## Versioning

RVA is a single contract spanning every runtime, so the resolver profile must match
across ecosystems. Prefer releasing all channels at the **same version** (e.g. a
`vX.Y.Z` tag alongside the npm bump), and treat `adapters/contract.json` as the
source of truth for the contract version.

## Not yet automated

- Swift (SPM tags + an `RVAFFI.xcframework` release asset).
- Kotlin / Android (Maven Central or GitHub Packages).
- Flutter (pub.dev automated publishing).
