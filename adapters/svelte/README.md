# @airovo/rva-svelte

[![npm](https://img.shields.io/npm/v/@airovo/rva-svelte?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-svelte)
[![license](https://img.shields.io/npm/l/@airovo/rva-svelte?color=22c55e)](./LICENSE)

Svelte wrapper for the [`<rva-image>`](https://www.npmjs.com/package/@airovo/rva-web)
web component. Ships as a source component (`.svelte`), so there is no build step.

## Install

```bash
npm i @airovo/rva-svelte
```

Peer dependency: `svelte >= 4`.

## Quick start

```svelte
<script>
  import RVAImage from "@airovo/rva-svelte";
  const onRender = (e) => console.log(e.detail.topology);
</script>

<RVAImage src="/hero.rva" alt="Hero" style="width:100%" on:rva-render={onRender} />
```

## Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `src` | `string` | — (required) | Asset URL. |
| `alt` | `string` | `""` | Accessible label. |

Other attributes are spread onto `<rva-image>` via `$$restProps`.

## Events

Forwarded: `rva-load`, `rva-render`, `rva-error`.

> `rva-cta` is not currently forwarded by the wrapper. Bind the element directly to
> handle CTA clicks:

```svelte
<script>
  import RVAImage from "@airovo/rva-svelte";
  let el;
  const ready = () => (el.ctaHandlers = { "shop-now": () => console.log("shop") });
</script>

<RVAImage bind:this={el} src="/hero.rva" on:rva-render={ready} />
```

Event payloads are documented in
[`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web).

## Related

- [`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) — the element itself
- [`@airovo/rva-react`](https://www.npmjs.com/package/@airovo/rva-react) · [`@airovo/rva-vue`](https://www.npmjs.com/package/@airovo/rva-vue) · [`@airovo/rva-node`](https://www.npmjs.com/package/@airovo/rva-node)

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
