<!--
  @rva/svelte — Svelte wrapper for the framework-neutral <rva-image> web
  component. Importing "@rva/web" registers the custom element.

    <script>
      import RVAImage from "@rva/svelte";
    </script>
    <RVAImage src="/hero.rva" alt="Hero" style="width:100%;aspect-ratio:16/9" />
-->
<script>
  import { createEventDispatcher } from "svelte";
  import "@rva/web";

  const dispatch = createEventDispatcher();

  /** @type {string} */
  export let src;
  /** @type {string} */
  export let alt = "";
</script>

<rva-image
  {src}
  {alt}
  on:rva-load={(event) => dispatch("rva-load", event.detail)}
  on:rva-render={(event) => dispatch("rva-render", event.detail)}
  on:rva-error={(event) => dispatch("rva-error", event.detail)}
  {...$$restProps}
/>
