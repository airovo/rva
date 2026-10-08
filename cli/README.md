# rva-cli

[![crates.io](https://img.shields.io/crates/v/rva-cli?color=3b82f6)](https://crates.io/crates/rva-cli)
[![license](https://img.shields.io/crates/l/rva-cli?color=22c55e)](#license)

The `rva` developer CLI: validate, inspect, render and package Responsive Visual
Assets. Thin wrapper over [`rva-core`](https://crates.io/crates/rva-core).

## Install

```bash
cargo install rva-cli
```

## Usage

```bash
rva --asset hero.rva validate
rva --asset hero.rva inspect --width 1280 --height 720
rva --asset hero.rva render --width 1280 --height 720 --out hero.png
rva --asset hero.rva pack --out hero.rva --optimize
rva --asset hero.rva unpack --out-dir ./hero
```

`--asset` accepts a directory, a `scene.json`, or a packaged `.rva` file.

Part of the [RVA](https://rva.airovo.tech) project — [source](https://github.com/airovo/rva).
