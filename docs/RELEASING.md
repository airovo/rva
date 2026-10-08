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

Bump `version` in the root `Cargo.toml` (`[workspace.package]`), then either tag
directly:

```bash
git tag v0.1.1
git push origin v0.1.1
```

…or let the Swift workflow create the tag (see below). Either way the tag triggers
the workflow, which authenticates over OIDC.

## Swift / SPM

Workflow: [`.github/workflows/release-swift.yml`](../.github/workflows/release-swift.yml).
Triggered manually (**Actions → Release (Swift / SPM) → Run workflow**) with a
version that matches `Cargo.toml`. It:

1. builds `RVAFFI.xcframework` (macOS + iOS device + simulator) as a **dynamic**
   framework (~11 MB, ~5 MB zipped; a static lib would be ~135 MB),
2. zips it and computes the SwiftPM checksum,
3. rewrites `adapters/swift/Package.swift`'s binary target to the release
   `url:` + `checksum:`,
4. commits that on a `release-swift/vX.Y.Z` branch, creates and pushes tag
   `vX.Y.Z`,
5. creates the GitHub Release and uploads `RVAFFI.xcframework.zip`.

Why the workflow owns the tag: SwiftPM reads the checksum from `Package.swift` at
the version tag, and the checksum only exists after the binary is built — so the
tag has to be created after the manifest is patched.

`main` keeps the development manifest (`binaryTarget(path: "RVAFFI.xcframework")`);
only the tagged commit points at the release asset. Pushing the tag also triggers
the crates.io workflow, which skips versions already published.

Consumers:

```swift
.package(url: "https://github.com/airovo/rva", from: "0.1.2")
```

## Versioning

There are two version axes:

- **Core** — the Cargo workspace (`rva-core`, `rva-wasm`, `rva-ffi`, `rva-cli`,
  `rva-server`). One version; it is what affects cross-runtime determinism. The
  Swift package shares this stream (its SPM tag equals the core version).
- **Adapters** — each npm package, and Kotlin / Flutter. Versioned **independently**
  (Changesets for npm; the `version` field in each native manifest). An adapter-only
  change (e.g. a DOM fix in `@airovo/rva-web`) ships without touching the core;
  a core change that alters resolution requires re-releasing the adapters that
  bundle the WASM (`rva-web`, `rva-node`) or the native binaries.

`scripts/check-versions.mjs` enforces only what must agree:

1. every workspace crate inherits the workspace version (core-internal parity),
2. `adapters/contract.json` → `resolverProfile` == `rva_core::RESOLVER_PROFILE`,
3. on a release tag `vX.Y.Z`, the tag equals the core version.

Adapter versions are printed for visibility but never forced to match.

```bash
node scripts/check-versions.mjs        # core + contract
node scripts/check-versions.mjs v0.1.1 # also against a tag
```

## Not yet automated

- Kotlin / Android (Maven Central or GitHub Packages).
- Flutter (pub.dev automated publishing).
