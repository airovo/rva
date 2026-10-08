# @airovo/rva-node

[![npm](https://img.shields.io/npm/v/@airovo/rva-node?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-node)
[![license](https://img.shields.io/npm/l/@airovo/rva-node?color=22c55e)](./LICENSE)

Node.js / server adapter for RVA: load, resolve, and render `.rva` assets with the
same WebAssembly core the browser uses — producing byte-identical `ResolvedScene`
output across runtimes.

## Install

```bash
npm i @airovo/rva-node
```

## Quick start

```ts
import { openSource, resolve, renderSvg } from "@airovo/rva-node";

const handle = await openSource("file:///assets/hero.rva");

const scene = resolve(handle, 1280, 720);        // ResolvedScene (JS object)
const { svg } = renderSvg(handle, 1280, 720);    // deterministic SVG string
```

## API

### `open(bytes: Uint8Array): RvaHandle`
Parse and integrity-validate a `.rva` package from raw bytes.

### `openSource(input: RvaSource): Promise<RvaHandle>`
Load from a path, `URL`, or bytes, then `open`.

### `resolve(handle: RvaHandle, width: number, height: number): ResolvedScene`
Deterministically resolve the asset for a viewport.

### `renderSvg(handle, width, height): { scene: ResolvedScene; svg: string }`
Resolve and rasterize to an SVG string (convenience; the browser paints with Canvas).

### `renderToSvg(handle, scene: ResolvedScene): string`
Render an already-resolved scene to SVG.

### `mimeFor(relative: string): string`
MIME type for a resource's relative path.

### `fontStack(family: string): string`
Resolve a font family to a CSS font stack (used by the SVG renderer).

### `readSource(input: RvaSource): Promise<Uint8Array>`
Low-level loader (returns bytes as-is).

`RvaHandle` also exposes `describe()`, `has_resource(name)`, `relative_for(name)`,
`resource(name)`, `resolve(width, height)` (JSON string), and `free()`.

## Sources

`RvaSource = string | URL | Uint8Array`.

| Input | Behaviour |
| --- | --- |
| filesystem path (relative/absolute) | read from disk |
| `file://` | read from disk |
| `http://` / `https://` | `fetch` (throws on non-OK) |
| `data:` | base64 / URL-encoded decode |
| `Uint8Array` | used directly |

```ts
const fromUrl = await openSource("https://cdn.example.com/hero.rva");
const fromBuf = await openSource(await readFile("hero.rva"));
```

## Types

Re-exports `ResolvedScene`, `ResolvedItem`, `Scene`, `RvaHandle`, and `RvaSource`.
See [`@airovo/rva-types`](https://www.npmjs.com/package/@airovo/rva-types).

## Related

- Browser: [`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web)
- React / Vue / Svelte / React Native wrappers
- CLI and native runtimes live in the [core repository](https://github.com/airovo/rva)

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
