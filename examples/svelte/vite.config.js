import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  optimizeDeps: { exclude: ["@airovo/rva-web"] },
  assetsInclude: ["**/*.wasm"],
  server: { fs: { allow: ["..", "../.."] } },
});
