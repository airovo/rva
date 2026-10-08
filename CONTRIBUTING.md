# Contributing to RVA

Thanks for your interest. RVA is the reference implementation of **Responsive
Visual Asset** — a format and a deterministic resolver. Contributions that keep
the core small, deterministic and platform-neutral are very welcome.

## Guiding principles

- **The core is a resolver, not a runtime.** It parses, validates, selects a
  topology and resolves geometry. No scripting, no behavior, no I/O in the
  resolver.
- **Adapters are thin.** A renderer draws the `ResolvedScene`; it must not
  reimplement layout semantics. Identical input must produce identical geometry
  across Rust, WASM and the C ABI.
- **Determinism is the product.** Any change that alters resolved geometry must
  be explainable and reflected in the resolver profile (`RESOLVER_PROFILE`).

## Layout

| Path | What |
|---|---|
| `core/` | `rva-core`: model, validator, resolver, reference renderer |
| `cli/` | `rva` developer CLI |
| `wasm/` | `rva-wasm` bindings |
| `ffi/` | `rva-ffi` stable C ABI (+ JNI bridge) |
| `server/` | `rva-server`: resolve/render over HTTP |
| `adapters/` | JS + native adapters (`@airovo/rva-*`) |
| `conformance/` | cross-runtime equality harness |
| `corpus/` | torture corpus generator + fixtures |
| `spec/` | working specification |
| `examples/` | runnable consumers per adapter |

## Prerequisites

- Rust **stable** (see `rust-toolchain.toml`), plus `rustfmt` and `clippy`
- Node **22+**
- For WASM bindings: the `wasm32-unknown-unknown` target and `wasm-bindgen`

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129   # match Cargo.lock
npm install
```

## Build & test

```bash
# Rust
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# License / advisory / source policy
cargo install cargo-deny --locked
cargo deny check

# JS adapters (needs the wasm bindings generated first)
npm run build:wasm
npm run build
node examples/node/index.js

# Cross-runtime conformance: native == wasm == c-abi
bash conformance/run.sh
```

If `cargo deny check` flags a new advisory that does not apply, add it to the
`ignore` list in `deny.toml` **with a reason**. Do not allow more licenses
without discussion — the allow list in `deny.toml` is intentionally permissive
only.

Run at least the Rust checks and (if you touched serialization or the resolver)
the conformance harness before opening a PR.

## Adding a fixture to the corpus

1. Add the source under `rva-torture-corpus-assets/<id>/` (or extend
   `corpus/build.mjs`).
2. `node corpus/build.mjs` regenerates `corpus/fixtures/<id>.rva`.
3. `bash conformance/run.sh` includes it.

## Changing the model or resolver

- Update `core/src/model.rs` / `core/src/resolve.rs` and the validator.
- Extend `core/tests/` and, when the output shape changes, the conformance
  fixtures.
- If resolution semantics change, bump `RESOLVER_PROFILE` in `core/src/lib.rs`.
- Regenerate TypeScript types: `npm run generate:types`.

## Changesets

Published packages are the `@airovo/*` adapters. If your change affects one of
them, add a changeset:

```bash
npm run changeset
```

and commit the generated `.changeset/*.md`.

## Pull requests

- Keep changes focused; one concern per PR.
- Fill in the PR template and confirm the checks above.
- Match the existing style (`cargo fmt`, no comments unless they add value).

By contributing you agree your contributions are licensed under the repository's
[MIT License](LICENSE).
