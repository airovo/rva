<script>
  import RVAImage from "@airovo/rva-svelte";

  let status = "resolving…";

  function onRender(event) {
    const detail = event.detail;
    status =
      `topology ${detail.topology} · ${detail.width}×${detail.height} · ` +
      `viability ${detail.viability.toFixed(2)}${detail.degraded ? " · degraded" : ""}`;
  }
</script>

<div class="wrap">
  <h1 style="font-size: 18px; margin: 0">RVA — one asset, any shape (Svelte)</h1>
  <p>
    A single <code>hero.rva</code> rendered by <code>&lt;RVAImage&gt;</code>. Resize the
    window: the box changes shape and the composition recomposes.
  </p>
  <div class="hero">
    <RVAImage src="/hero.rva" alt="Responsive campaign hero" on:rva-render={onRender} />
  </div>
  <div class="badge">{status}</div>
</div>
