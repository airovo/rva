// @rva/node — Node.js / server adapter for RVA.
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
import type { ResolvedScene } from "@rva/types";

export type { RvaHandle } from "./pkg/rva_wasm.js";
export type { ResolvedScene, ResolvedItem, Scene } from "@rva/types";

export function open(bytes: Uint8Array): RvaHandle {
  return load(bytes);
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
