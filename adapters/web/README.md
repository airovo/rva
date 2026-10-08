# @airovo/rva-web

[![npm](https://img.shields.io/npm/v/@airovo/rva-web?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-web)
[![license](https://img.shields.io/npm/l/@airovo/rva-web?color=22c55e)](./LICENSE)

Framework-neutral `<rva-image>` web component powered by the RVA WebAssembly core.
The core decides *what* to draw from the viewport; the element paints it on a canvas.
Drop it into any framework (or none) — it re-resolves on resize.

## Install

```bash
npm i @airovo/rva-web
```

## Quick start

```html
<script type="module">
  import "@airovo/rva-web";
</script>

<rva-image src="/hero.rva" alt="Hero" style="display:block;width:100%"></rva-image>
```

```js
const el = document.querySelector("rva-image");
el.addEventListener("rva-render", (e) => console.log(e.detail.topology));
```

## Attributes

| Attribute | Type | Description |
| --- | --- | --- |
| `src` | `string` | Asset URL. Changing it re-loads the asset (while connected). |
| `alt` | `string` | Accessible label, mirrored to `aria-label` on the canvas. |

`bytes`, `srcResolver`, and the callbacks below are **JS properties**, not attributes.

## Properties

| Property | Type | Description |
| --- | --- | --- |
| `bytes` | `Uint8Array \| ArrayBuffer \| Blob \| null` | Asset bytes; bypasses fetching. Call `reload()` after changing. |
| `srcResolver` | `(src: string) => Promise<ArrayBuffer \| Uint8Array> \| ArrayBuffer \| Uint8Array \| null` | Custom loader. Call `reload()` after changing. |
| `onRender` | `(scene: ResolvedScene) => void` | Called on every (re)resolution. |
| `onCtaRegions` | `(regions: ResolvedCtaRegion[]) => void` | All CTA regions after a render. |
| `onCta` | `(activation: RvaCtaActivation) => void` | Any CTA region activated (click). |
| `ctaHandlers` | `Record<string, (a: RvaCtaActivation) => void>` | Per-region-id click handlers. |
| `onLoad` | `(detail: RvaLoadDetail) => void` | Asset parsed and loaded. |
| `onError` | `(detail: RvaErrorDetail) => void` | Loading or resolution failed. |
| `ctaRegions` *(getter)* | `ResolvedCtaRegion[]` | The visible CTA regions of the last scene. |

## Methods

| Method | Description |
| --- | --- |
| `reload()` | Re-read the asset from `bytes` / `srcResolver` / `src`. |
| `bindCta(id, handler)` | Subscribe to a specific region id; returns an unbind function. |
| `unbindCta(id, handler?)` | Remove one (or all) handler(s) for a region id. |
| `unbindAllCta()` | Remove every bound CTA handler. |

## Events

| Event | `event.detail` |
| --- | --- |
| `rva-load` | `{ description: string; src: string \| null }` |
| `rva-render` | `ResolvedScene` |
| `rva-error` | `{ error: unknown; src: string \| null }` |
| `rva-cta` | `{ id: string; region: ResolvedCtaRegion }` |

## CTA regions

`rva-cta` fires on click when the pointer lands inside a visible CTA region. The
element then also calls `onCta`, `ctaHandlers[id]`, and every `bindCta` handler.

```js
el.onCta = ({ id }) => track("rva_cta", id);
el.ctaHandlers = { "shop-now": () => navigate("/product") };
const off = el.bindCta("learn-more", () => openModal());
// later: off();
```

## Sources

The asset is resolved in this order: `bytes` → `srcResolver(src)` → `fetch(src)`.

`file://` is intentionally rejected in browsers/WebViews (they block it). For local
files, set `bytes` or `srcResolver` — e.g. Tauri's `convertFileSrc`. On the server,
use [`@airovo/rva-node`](https://www.npmjs.com/package/@airovo/rva-node).

```js
import { convertFileSrc } from "@tauri-apps/api/core";

el.srcResolver = (p) => fetch(convertFileSrc(p)).then((r) => r.arrayBuffer());
el.src = "/path/to/hero.rva";
```

## Styling

Rendering happens in an open shadow root on an internal `<canvas role="img">`; size
the host element and the canvas fills it. Device pixel ratio is clamped to 2.

```css
rva-image {
  display: block;
  width: 100%;
  aspect-ratio: 16 / 9;
}
```

## Framework wrappers

React · Vue · Svelte · React Native — see
[`@airovo/rva-react`](https://www.npmjs.com/package/@airovo/rva-react),
[`@airovo/rva-vue`](https://www.npmjs.com/package/@airovo/rva-vue),
[`@airovo/rva-svelte`](https://www.npmjs.com/package/@airovo/rva-svelte),
[`@airovo/rva-react-native`](https://www.npmjs.com/package/@airovo/rva-react-native).

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
