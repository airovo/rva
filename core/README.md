# rva-core

[![crates.io](https://img.shields.io/crates/v/rva-core?color=3b82f6)](https://crates.io/crates/rva-core)
[![docs.rs](https://img.shields.io/docsrs/rva-core)](https://docs.rs/rva-core)
[![license](https://img.shields.io/crates/l/rva-core?color=22c55e)](#license)

Reference core for **RVA (Responsive Visual Asset)**: parse, validate, resolve and
render `.rva` assets deterministically. This is the implementation behind the CLI,
FFI, server, WASM bindings and every platform adapter.

The core is a **resolver, not a runtime**: for the same asset + viewport + resolver
profile it produces the same `ResolvedScene`, and platform adapters only paint that
result with their native graphics stack.

## Use

```toml
[dependencies]
rva-core = "0.1"
```

```rust
use rva_core::{Asset, Fonts, resolve};

let asset = Asset::load("hero.rva")?;
let fonts = Fonts::load_system();
let scene = resolve(&asset, &fonts, 1280, 720)?;
println!("{} {}", scene.topology, scene.viability);
```

## Features

- `render` — reference rasterizer (resvg/tiny-skia); used by the CLI, server and FFI.
- `packaging` — full image decode/encode for pack-time optimization (PNG → WebP/JPEG).

Both are enabled by default; disable with `default-features = false` for the lean
parse + resolve path used in the browser.

Part of the [RVA](https://rva.airovo.tech) project — [source](https://github.com/airovo/rva).
