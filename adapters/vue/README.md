# @airovo/rva-vue

[![npm](https://img.shields.io/npm/v/@airovo/rva-vue?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-vue)
[![license](https://img.shields.io/npm/l/@airovo/rva-vue?color=22c55e)](./LICENSE)

Vue 3 wrapper for the [`<rva-image>`](https://www.npmjs.com/package/@airovo/rva-web)
web component. Renders the element and forwards `src` / `alt` plus any other
attributes and listeners.

## Install

```bash
npm i @airovo/rva-vue
```

Peer dependency: `vue >= 3.3`.

## Quick start

```vue
<script setup>
import { RVAImage } from "@airovo/rva-vue";
const onRender = (e) => console.log(e.detail.topology);
</script>

<template>
  <RVAImage src="/hero.rva" alt="Hero" style="width:100%" @rva-render="onRender" />
</template>
```

## Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `src` | `string` | — (required) | Asset URL. |
| `alt` | `string` | `""` | Accessible label. |

Any other attribute (`style`, `class`) or listener (`@rva-render`, `@rva-cta`, …)
falls through to the underlying `<rva-image>`.

## Events and property access

The wrapper is intentionally thin: use the element's native events, and a template
ref for property-only features (`bytes`, `srcResolver`, `ctaHandlers`).

```vue
<script setup>
import { ref, onMounted } from "vue";
import { RVAImage } from "@airovo/rva-vue";

const el = ref(null);
onMounted(() => {
  el.value.ctaHandlers = { "shop-now": () => console.log("shop") };
});
</script>

<template>
  <RVAImage ref="el" src="/hero.rva" @rva-cta="(e) => console.log(e.detail.id)" />
</template>
```

Events: `rva-load`, `rva-render`, `rva-error`, `rva-cta`. See
[`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) for payloads.

## Related

- [`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) — the element itself
- [`@airovo/rva-react`](https://www.npmjs.com/package/@airovo/rva-react) · [`@airovo/rva-svelte`](https://www.npmjs.com/package/@airovo/rva-svelte) · [`@airovo/rva-node`](https://www.npmjs.com/package/@airovo/rva-node)

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
