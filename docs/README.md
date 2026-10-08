# RVA documentation

A single entry point into the RVA repository. New here? Start with the
[README](../README.md) for the concept, then the
[specification overview](../spec/overview.md).

## Concept & specification

- [RVA overview](../README.md) — what a Responsive Visual Asset is.
- [Specification · overview](../spec/overview.md) — the model, resolver and
  primitive API (working draft).
- [Specification · container](../spec/container.md) — the `.rva` binary container.

## Core & runtimes

- [`core/`](../core) — `rva-core`: model, validator, deterministic resolver and
  reference renderer.
- [`cli/`](../cli) — the `rva` developer CLI (`validate`, `inspect`, `render`,
  `pack`, `unpack`).
- [`wasm/`](../wasm) — `rva-wasm`: parse + resolve for the browser.
- [`ffi/`](../ffi) — `rva-ffi`: stable C ABI (Swift / Kotlin / Flutter / C) with a
  JNI bridge.
- [`server/`](../server) — `rva-server`: resolve/render over HTTP.

## Adapters

One contract, many platforms — see [`adapters/contract.json`](../adapters/contract.json).

- Web: [`@airovo/rva-web`](../adapters/web/README.md) — the `<rva-image>` element.
- Node/server: [`@airovo/rva-node`](../adapters/node/README.md).
- React: [`@airovo/rva-react`](../adapters/react/README.md) ·
  Vue: [`@airovo/rva-vue`](../adapters/vue/README.md) ·
  Svelte: [`@airovo/rva-svelte`](../adapters/svelte/README.md).
- React Native: [`@airovo/rva-react-native`](../adapters/react-native/README.md).
- Types: [`@airovo/rva-types`](../adapters/types) — generated from the Rust core.
- Native: [Swift](../adapters/swift) · [Kotlin](../adapters/kotlin) ·
  [Flutter](../adapters/flutter).

## Conformance & testing

- [`conformance/`](../conformance) — the cross-runtime harness: native (CLI) ==
  WASM (`@airovo/rva-node`) == C ABI (`rva_ffi`) must produce byte-identical
  `ResolvedScene` JSON.
- [`corpus/`](../corpus) — the torture corpus generator and fixtures.
- Policy: [`deny.toml`](../deny.toml) (licenses, advisories, sources).

## Examples

Runnable consumers in [`examples/`](../examples) — web, React, Vue, Svelte, Node,
React Native, Swift, Kotlin and Flutter, plus a sample `.rva`.

## Project

- [Releasing](./RELEASING.md) — npm (Changesets) and crates.io, via OIDC.
- [Contributing](../CONTRIBUTING.md)
- [Code of Conduct](../CODE_OF_CONDUCT.md)
- [Security policy](../SECURITY.md)
- [License (MIT)](../LICENSE)

---

RVA was created by [Airovo Technologies](https://airovo.tech). Website:
[rva.airovo.tech](https://rva.airovo.tech).
