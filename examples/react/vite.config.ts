import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @rva/web ships the WASM bindings; keep them out of dep pre-bundling so the
// generated `import.meta.url` asset URL is preserved.
export default defineConfig({
  plugins: [react()],
  optimizeDeps: { exclude: ["@rva/web"] },
  assetsInclude: ["**/*.wasm"],
  server: { fs: { allow: ["..", "../.."] } },
});
