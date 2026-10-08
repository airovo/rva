# rva-server

[![crates.io](https://img.shields.io/crates/v/rva-server?color=3b82f6)](https://crates.io/crates/rva-server)
[![docs.rs](https://img.shields.io/docsrs/rva-server)](https://docs.rs/rva-server)
[![license](https://img.shields.io/crates/l/rva-server?color=22c55e)](#license)

Server-side renderer for [RVA](https://rva.airovo.tech): resolve and rasterize
`.rva` assets natively, with caching and a small HTTP endpoint. Built on
[`rva-core`](https://crates.io/crates/rva-core).

## Run

```bash
cargo install rva-server
rva-server --asset hero.rva --addr 127.0.0.1:8787
```

## Endpoints

| Route | Result |
| --- | --- |
| `GET /health` | `ok` |
| `GET /describe/:id` | Human-readable asset summary. |
| `GET /resolve/:id?w=1280&h=720` | `ResolvedScene` JSON. |
| `GET /render/:id?w=1280&h=720` | `image/png`. |

The asset id defaults to the file stem (`hero.rva` → `hero`).

Part of the [RVA](https://rva.airovo.tech) project — [source](https://github.com/airovo/rva).
