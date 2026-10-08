# rva-wasm

[![crates.io](https://img.shields.io/crates/v/rva-wasm?color=3b82f6)](https://crates.io/crates/rva-wasm)
[![docs.rs](https://img.shields.io/docsrs/rva-wasm)](https://docs.rs/rva-wasm)
[![license](https://img.shields.io/crates/l/rva-wasm?color=22c55e)](#license)

WebAssembly bindings (`wasm-bindgen`) for the lean [RVA](https://rva.airovo.tech)
core: parse + resolve only (no font metrics, no rasterizer, no image codecs). This
is the engine behind the browser and Node adapters.

It is normally consumed **through** the JS packages rather than directly:

- [`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) — `<rva-image>` element
- [`@airovo/rva-node`](https://www.npmjs.com/package/@airovo/rva-node) — Node/server
- React / Vue / Svelte / React Native wrappers

## Build

```bash
wasm-pack build --target web   # or:
./adapters/web/build.sh
```

Part of the [RVA](https://rva.airovo.tech) project — [source](https://github.com/airovo/rva).
