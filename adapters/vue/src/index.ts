// @rva/vue — Vue 3 wrapper for the framework-neutral <rva-image> web component.
//
//   <script setup>
//   import { RVAImage } from "@rva/vue";
//   </script>
//   <RVAImage src="/hero.rva" alt="Hero" style="width:100%;aspect-ratio:16/9" />

import { defineComponent, h, type PropType } from "vue";
import "@rva/web";

export const RVAImage = defineComponent({
  name: "RVAImage",
  props: {
    src: { type: String as PropType<string>, required: true },
    alt: { type: String as PropType<string>, default: "" },
  },
  setup(props, { attrs }) {
    return () =>
      h("rva-image", {
        src: props.src,
        alt: props.alt,
        ...attrs,
      });
  },
});

export default RVAImage;
