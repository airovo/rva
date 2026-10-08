// @airovo/rva-node — Node.js / server adapter for RVA.
//
// Uniform adapter surface (see adapters/contract.json):
//   open(bytes)                 -> handle
//   resolve(handle, w, h)       -> resolved scene object
//   renderSvg(handle, w, h)     -> { scene, svg }
//
// The Rust core (compiled to WebAssembly) decides what to draw; this adapter
// renders the resolved scene to a self-contained SVG document.

import { load, type RvaHandle } from "./pkg/rva_wasm.js";
import { mimeFor, renderToSvg } from "./render-svg.js";
import { readSource, type RvaSource } from "./source.js";
import type { ResolvedScene } from "@airovo/rva-types";

export type { RvaHandle } from "./pkg/rva_wasm.js";
export type { ResolvedScene, ResolvedItem, Scene } from "@airovo/rva-types";
export { readSource };
export type { RvaSource };

export function open(bytes: Uint8Array): RvaHandle {
  return load(bytes);
}

/**
 * Convenience: open from a filesystem path, `file://` URL, `http(s)://` URL or
 * `data:` URL (or pass through `Uint8Array`). `open(bytes)` remains normative.
 */
export async function openSource(input: RvaSource): Promise<RvaHandle> {
  return open(await readSource(input));
}

export function resolve(handle: RvaHandle, width: number, height: number): ResolvedScene {
  return JSON.parse(handle.resolve(width, height)) as ResolvedScene;
}

export function renderSvg(
  handle: RvaHandle,
  width: number,
  height: number
): { scene: ResolvedScene; svg: string } {
  const scene = resolve(handle, width, height);
  return { scene, svg: renderToSvg(handle, scene) };
}

export { renderToSvg, mimeFor };
export { fontStack } from "./render-svg.js";
