# @airovo/rva-svelte

Svelte wrapper for the `<rva-image>` web component.

```bash
npm i @airovo/rva-svelte
```

```svelte
<script>
  import RVAImage from "@airovo/rva-svelte";
</script>

<RVAImage src="/hero.rva" on:rva-render={onRender} />
```

Peer dependency: `svelte >= 4`.

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, created by Airovo Technologies.
