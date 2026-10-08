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

The tag triggers the workflow, which authenticates over OIDC.

## Apple binary (xcframework)

Workflow: [`.github/workflows/release-xcframework.yml`](../.github/workflows/release-xcframework.yml).
Runs on a core tag `vX.Y.Z` (and on `workflow_dispatch`). It builds
`RVAFFI.xcframework` (macOS + iOS device + simulator) as a **dynamic** framework
(~11 MB, ~5 MB zipped; a static lib would be ~135 MB) and attaches
`RVAFFI.xcframework.zip` to the release for that tag. The Swift package and the
React Native iOS adapter consume this asset.

## Swift / SPM

The Swift package lives in a **separate repo**: [`airovo/rva-swift`](https://github.com/airovo/rva-swift).
It is an independent, Apple-only package that pins a core binary by URL + checksum,
so it versions **independently** of the core (and isn't polluted by this repo's
tags, which matters for the Swift Package Index).

Its `Package.swift` reads:

```swift
let rvaFFIUrl = "https://github.com/airovo/rva/releases/download/vX.Y.Z/RVAFFI.xcframework.zip"
let rvaFFIChecksum = "..."
```

To release: in `airovo/rva-swift`, run **Actions → Release** with a new `version`
and the `core_tag` to pin. That workflow downloads the core's `RVAFFI.xcframework.zip`,
recomputes the SwiftPM checksum (the archive's SHA-256), commits the pin to `main`,
and tags the new version. Consumers:

```swift
.package(url: "https://github.com/airovo/rva-swift", from: "0.1.0")
```

## Flutter (pub.dev)

The Flutter adapter (`adapters/flutter`) is a real Flutter **FFI plugin** that
bundles the core — `librva_ffi.so` under `android/src/main/jniLibs/`, and the
dynamic `RVAFFI.xcframework` vendored by the `ios/` and `macos/` podspecs. It
versions independently (its own pubspec `version`).

Workflow: [`.github/workflows/release-flutter.yml`](../.github/workflows/release-flutter.yml).
Triggered manually (**Actions → Release (pub.dev) → Run workflow**); runs
`flutter pub publish` via pub.dev **Automated publishing (OIDC)**.

One-time setup: publish once manually (`cd adapters/flutter && flutter pub publish`),
then pub.dev → `rva_flutter` → Admin → Automated publishing → GitHub Actions, repo
`airovo/rva`, workflow `release-flutter.yml`.

Before running, bump `adapters/flutter/pubspec.yaml` `version` and add a
`CHANGELOG.md` entry.

## Versioning

There are two version axes:

- **Core** — the Cargo workspace (`rva-core`, `rva-wasm`, `rva-ffi`, `rva-cli`,
  `rva-server`). One version; it is what affects cross-runtime determinism. Its
  `vX.Y.Z` tags also name the xcframework release asset.
- **Adapters** — each npm package, Swift, Kotlin, Flutter. Versioned
  **independently** (Changesets for npm; a separate repo/tags for Swift; the
  `version` field for the others). An adapter-only change (e.g. a DOM fix in
  `@airovo/rva-web`) ships without touching the core; a core change that alters
  resolution requires re-releasing the adapters that bundle the WASM
  (`rva-web`, `rva-node`) or the native binaries.

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
