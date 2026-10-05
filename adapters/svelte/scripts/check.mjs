// Compile the Svelte component to verify it parses/builds.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { compile } from "svelte/compiler";

const here = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(join(here, "..", "src", "RVAImage.svelte"), "utf8");

const { js } = compile(source, { filename: "RVAImage.svelte", generate: "client" });
if (!js || !js.code) {
  throw new Error("svelte compile produced no output");
}
console.log("svelte component compiled OK");
