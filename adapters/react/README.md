# @airovo/rva-react

[![npm](https://img.shields.io/npm/v/@airovo/rva-react?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-react)
[![license](https://img.shields.io/npm/l/@airovo/rva-react?color=22c55e)](./LICENSE)

React wrapper for the [`<rva-image>`](https://www.npmjs.com/package/@airovo/rva-web)
web component — declarative props, callback events, and a `ref` to the underlying
element.

## Install

```bash
npm i @airovo/rva-react
```

Peer dependency: `react >= 18`. The web component is bundled via
`@airovo/rva-web` and registered on import.

## Quick start

```tsx
import { RVAImage } from "@airovo/rva-react";

export function Hero() {
  return (
    <RVAImage
      src="/hero.rva"
      alt="Hero"
      style={{ width: "100%", aspectRatio: "16 / 9" }}
      onRender={(scene) => setStatus(scene.topology)}
      onCta={({ id }) => track("rva_cta", id)}
      ctaHandlers={{ "shop-now": () => navigate("/product") }}
    />
  );
}
```

## Props

| Prop | Type | Description |
| --- | --- | --- |
| `src` | `string` | Asset URL (required). |
| `alt` | `string` | Accessible label. |
| `onRender` | `(scene) => void` | Every (re)resolution. `scene` has `topology`, `width`, `height`, `viability`, `degraded`, `hidden`, `ctaRegions`. |
| `onCtaRegions` | `(regions) => void` | All CTA regions after a render. |
| `onCta` | `(cta) => void` | Any CTA activation, `{ id, region }`. |
| `ctaHandlers` | `Record<string, (cta) => void>` | Per-region-id handlers. |
| `onLoad` | `(detail) => void` | `{ description, src }`. |
| `onError` | `(detail) => void` | `{ error, src }`. |

All other HTML attributes are forwarded; `className` maps to the element's `class`.

## Ref and advanced usage

`ref` is the raw `HTMLElement`, so the web component's property-only features
(`bytes`, `srcResolver`, `reload()`, `bindCta()`) remain available:

```tsx
import { useEffect, useRef } from "react";
import { RVAImage } from "@airovo/rva-react";

function LocalImage() {
  const ref = useRef<HTMLElement>(null);

  useEffect(() => {
    const el = ref.current!;
    el.srcResolver = () => fetch("/hero.rva").then((r) => r.arrayBuffer());
    el.reload();
  }, []);

  return <RVAImage ref={ref} src="/hero.rva" alt="Hero" />;
}
```

## Related

- [`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) — the element itself
- [`@airovo/rva-vue`](https://www.npmjs.com/package/@airovo/rva-vue) · [`@airovo/rva-svelte`](https://www.npmjs.com/package/@airovo/rva-svelte) · [`@airovo/rva-node`](https://www.npmjs.com/package/@airovo/rva-node)

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
