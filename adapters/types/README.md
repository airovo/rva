# @airovo/rva-types

[![npm](https://img.shields.io/npm/v/@airovo/rva-types?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-types)
[![license](https://img.shields.io/npm/l/@airovo/rva-types?color=22c55e)](./LICENSE)

TypeScript definitions for the **RVA** scene model (`Scene`) and the deterministic
resolved output (`ResolvedScene`) produced by the RVA core. **Types only — no runtime.**

The declarations are generated from the Rust core with
[`ts-rs`](https://github.com/Aleph-Alpha/ts-rs), so they always match the resolver's
actual output.

## Install

```bash
npm i -D @airovo/rva-types
```

## Usage

```ts
import type { Scene, ResolvedScene, ResolvedCtaRegion } from "@airovo/rva-types";

function onResolved(scene: ResolvedScene) {
  console.log(scene.topology, scene.viability, scene.ctaRegions);
}

function ctaBounds(region: ResolvedCtaRegion) {
  return region.bounds; // { x, y, width, height }
}
```

## Exported types

| Type | Description |
| --- | --- |
| `Scene` | The source document: resources, elements, topologies, fallback. |
| `DesignSpace` | Base design canvas (`width`, `height`). |
| `Element` | A source element (raster, vector, text, paint). |
| `Background` | A topology background: a resource id or a procedural `Paint`. |
| `Constraint` | An authored relationship between elements (or an element and the canvas). |
| `Topology` | A layout variant selected by an aspect-ratio `when` range. |
| `When` | Aspect-ratio bounds that select a topology. |
| `Layout` | Normalized placement of an element within a topology. |
| `Paint` | Fill: solid color, linear gradient, or radial gradient. |
| `GradientStop` | A stop in a gradient. |
| `TextResource` | Text content and style. |
| `FocalRegion` | A protected region (e.g. a face). |
| `Focus` | Normalized focus/pan point for a cover-fit image. |
| `Fallback` | Degradation / hidden rules. |
| `Bounds` | Pixel rectangle (`x`, `y`, `width`, `height`). |
| `CtaRegion` | A source call-to-action region. |
| `CtaSource` | CTA target metadata. |
| `ResolvedScene` | Deterministic result for a given viewport. |
| `ResolvedItem` | A resolved, absolutely-positioned draw item. |
| `ResolvedCtaRegion` | A resolved CTA region with pixel + normalized bounds. |

## Regenerating

From the core repository, after changing the Rust model:

```bash
npm run generate -w @airovo/rva-types
```

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
