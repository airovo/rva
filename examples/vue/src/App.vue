<script setup lang="ts">
import { ref } from "vue";
import { RVAImage } from "@rva/vue";

interface RenderDetail {
  topology: string;
  width: number;
  height: number;
  viability: number;
  degraded: boolean;
}

const status = ref("resolving…");

function onRender(event: Event) {
  const detail = (event as CustomEvent<RenderDetail>).detail;
  status.value =
    `topology ${detail.topology} · ${detail.width}×${detail.height} · ` +
    `viability ${detail.viability.toFixed(2)}${detail.degraded ? " · degraded" : ""}`;
}
</script>

<template>
  <div class="wrap">
    <h1 style="font-size: 18px; margin: 0">RVA — one asset, any shape (Vue)</h1>
    <p>
      A single <code>hero.rva</code> rendered by <code>&lt;RVAImage&gt;</code>. Resize the
      window: the box changes shape and the composition recomposes.
    </p>
    <RVAImage
      src="/hero.rva"
      alt="Responsive campaign hero"
      class="hero"
      @rva-render="onRender"
    />
    <div class="badge">{{ status }}</div>
  </div>
</template>
