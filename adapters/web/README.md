# @airovo/rva-web

Framework-neutral `<rva-image>` web component backed by the RVA WebAssembly core. The core decides *what* to draw; the element paints it.

```bash
npm i @airovo/rva-web
```

```html
<script type="module">
  import "@airovo/rva-web";
</script>

<rva-image src="/hero.rva" alt="Hero" style="width:100%"></rva-image>
```

**Events:** `rva-load`, `rva-render`, `rva-error`, `rva-cta`.
**Callbacks:** `onRender`, `onCtaRegions`, `onCta`, `ctaHandlers`.
**Sources:** a URL (fetched), or set `bytes` / `srcResolver` for local or custom sources.

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, created by Airovo Technologies.
