import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Vue treats <rva-image> as a custom element; tell the compiler not to warn.
export default defineConfig({
  plugins: [
    vue({
      template: { compilerOptions: { isCustomElement: (tag) => tag === "rva-image" } },
    }),
  ],
  optimizeDeps: { exclude: ["@rva/web"] },
  assetsInclude: ["**/*.wasm"],
  server: { fs: { allow: ["..", "../.."] } },
});
