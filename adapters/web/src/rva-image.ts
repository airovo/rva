// <rva-image> — a framework-neutral web component for RVA assets.
//
// The Rust core (compiled to WebAssembly) decides *what* to draw; this adapter
// decides *how* by painting the resolved scene onto a canvas with the browser's
// own image decoding and text engine.
//
//   <rva-image src="/hero.rva" alt="Campaign hero" style="width:100%"></rva-image>
//
// The element observes its own container size and recomposes on every material
// resize, exactly like a responsive UI component.

import init, { load, type RvaHandle } from "./pkg/rva_wasm.js";
import type { ResolvedItem, ResolvedScene } from "@rva/types";

let wasmReady: Promise<unknown> | null = null;
function ensureWasm(): Promise<unknown> {
  if (!wasmReady) wasmReady = init();
  return wasmReady;
}

interface ImageEntry {
  img: HTMLImageElement;
  url: string;
  relative: string;
}

const FONT_STACKS: Record<string, string> = {
  "system-ui": "system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif",
  "ui-sans-serif": "system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif",
  sans: "system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif",
  serif: "Georgia, Times New Roman, serif",
  monospace: "ui-monospace, SFMono-Regular, Menlo, monospace",
};

function fontStack(family: string): string {
  return FONT_STACKS[family] ?? `${family}, sans-serif`;
}

function mimeFor(relative: string): string {
  const ext = relative.split(".").pop()?.toLowerCase() ?? "";
  switch (ext) {
    case "svg":
      return "image/svg+xml";
    case "jpg":
    case "jpeg":
      return "image/jpeg";
    case "webp":
      return "image/webp";
    case "avif":
      return "image/avif";
    case "gif":
      return "image/gif";
    default:
      return "image/png";
  }
}

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`failed to decode ${url}`));
    img.src = url;
  });
}

function drawCover(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  x: number,
  y: number,
  w: number,
  h: number
): void {
  const iw = img.naturalWidth || img.width;
  const ih = img.naturalHeight || img.height;
  if (!iw || !ih) {
    ctx.drawImage(img, x, y, w, h);
    return;
  }
  const scale = Math.max(w / iw, h / ih);
  const dw = iw * scale;
  const dh = ih * scale;
  ctx.drawImage(img, x + (w - dw) / 2, y + (h - dh) / 2, dw, dh);
}

function drawMasked(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  mask: HTMLImageElement,
  x: number,
  y: number,
  w: number,
  h: number,
  dpr: number
): void {
  const off = document.createElement("canvas");
  off.width = Math.max(1, Math.round(w * dpr));
  off.height = Math.max(1, Math.round(h * dpr));
  const octx = off.getContext("2d");
  if (!octx) return;
  octx.setTransform(dpr, 0, 0, dpr, 0, 0);
  octx.drawImage(img, 0, 0, w, h);
  octx.globalCompositeOperation = "destination-in";
  octx.drawImage(mask, 0, 0, w, h);
  ctx.drawImage(off, x, y, w, h);
}

export class RvaImage extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["src", "alt"];
  }

  private handle: RvaHandle | null = null;
  private readonly images = new Map<string, ImageEntry>();
  private renderToken = 0;
  private frame = 0;
  private observer: ResizeObserver | null = null;
  private canvas!: HTMLCanvasElement;
  private ctx!: CanvasRenderingContext2D;

  constructor() {
    super();
    this.attachShadow({ mode: "open" });
  }

  connectedCallback(): void {
    if (!this.shadowRoot?.querySelector("canvas")) {
      this.shadowRoot!.innerHTML = `
        <style>
          :host { display: block; position: relative; overflow: hidden; }
          canvas { display: block; width: 100%; height: 100%; }
        </style>
        <canvas role="img"></canvas>`;
    }
    this.canvas = this.shadowRoot!.querySelector("canvas") as HTMLCanvasElement;
    this.ctx = this.canvas.getContext("2d") as CanvasRenderingContext2D;

    if (!this.observer) {
      this.observer = new ResizeObserver(() => this.scheduleDraw());
      this.observer.observe(this);
    }
    this.syncAlt();
    if (this.getAttribute("src")) void this.load();
  }

  disconnectedCallback(): void {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
    for (const entry of this.images.values()) URL.revokeObjectURL(entry.url);
    this.images.clear();
    this.handle = null;
  }

  attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null): void {
    if (name === "alt") this.syncAlt();
    if (name === "src" && oldValue !== newValue && newValue && this.isConnected) {
      void this.load();
    }
  }

  private syncAlt(): void {
    if (this.canvas) this.canvas.setAttribute("aria-label", this.getAttribute("alt") ?? "");
  }

  private async load(): Promise<void> {
    const src = this.getAttribute("src");
    if (!src) return;
    try {
      await ensureWasm();
      const response = await fetch(src);
      if (!response.ok) throw new Error(`fetch ${src}: ${response.status}`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      this.handle = load(bytes);
      this.dispatchEvent(
        new CustomEvent("rva-load", { detail: { description: this.handle.describe(), src } })
      );
      this.scheduleDraw();
    } catch (error) {
      this.dispatchEvent(new CustomEvent("rva-error", { detail: { error, src } }));
      console.error("[rva-image]", error);
    }
  }

  private scheduleDraw(): void {
    if (this.frame) return;
    this.frame = requestAnimationFrame(() => {
      this.frame = 0;
      void this.draw();
    });
  }

  private async image(name: string): Promise<ImageEntry> {
    const cached = this.images.get(name);
    if (cached) return cached;
    const relative = this.handle!.relative_for(name);
    const resource = this.handle!.resource(name) as unknown as BlobPart;
    const blob = new Blob([resource], { type: mimeFor(relative) });
    const url = URL.createObjectURL(blob);
    const img = await loadImage(url);
    const entry = { img, url, relative };
    this.images.set(name, entry);
    return entry;
  }

  private async draw(): Promise<void> {
    if (!this.handle || !this.ctx) return;
    const cssW = Math.max(1, Math.round(this.clientWidth));
    const cssH = Math.max(1, Math.round(this.clientHeight));
    if (cssW <= 1 || cssH <= 1) return;

    const token = ++this.renderToken;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    this.canvas.width = Math.round(cssW * dpr);
    this.canvas.height = Math.round(cssH * dpr);
    const ctx = this.ctx;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, cssW, cssH);

    const scene = JSON.parse(this.handle.resolve(cssW, cssH)) as ResolvedScene;

    const needed = new Set<string>();
    if (scene.background && scene.background.type === "image") {
      needed.add(scene.background.resource);
    }
    for (const item of scene.items) {
      if ("resource" in item) needed.add(item.resource);
      if (item.mask) needed.add(item.mask);
    }
    const entries = new Map<string, ImageEntry>();
    for (const name of needed) {
      try {
        entries.set(name, await this.image(name));
      } catch (error) {
        console.warn("[rva-image] resource failed", name, error);
      }
    }
    if (token !== this.renderToken) return; // superseded by a newer resize

    const background = scene.background;
    if (background && background.type === "image") {
      const entry = entries.get(background.resource);
      if (entry) {
        if (background.fit === "cover") {
          drawCover(ctx, entry.img, 0, 0, cssW, cssH);
        } else {
          ctx.drawImage(entry.img, 0, 0, cssW, cssH);
        }
      }
    }

    for (const item of scene.items) {
      ctx.globalAlpha = item.opacity ?? 1;
      if (item.type === "text") {
        this.drawText(ctx, item);
        continue;
      }
      const entry = entries.get(item.resource);
      if (!entry) continue;
      if (item.type === "vector") {
        ctx.drawImage(entry.img, item.x, item.y, item.w, item.h);
      } else if (item.mask) {
        const mask = entries.get(item.mask);
        if (mask) {
          drawMasked(ctx, entry.img, mask.img, item.x, item.y, item.w, item.h, dpr);
        } else {
          ctx.drawImage(entry.img, item.x, item.y, item.w, item.h);
        }
      } else {
        ctx.drawImage(entry.img, item.x, item.y, item.w, item.h);
      }
    }
    ctx.globalAlpha = 1;

    this.dispatchEvent(new CustomEvent("rva-render", { detail: scene }));
  }

  private drawText(ctx: CanvasRenderingContext2D, item: Extract<ResolvedItem, { type: "text" }>): void {
    ctx.fillStyle = item.role === "subheadline" ? "#334155" : "#0F172A";
    ctx.textBaseline = "alphabetic";
    ctx.font = `${item.weight} ${item.size}px ${fontStack(item.fontFamily)}`;
    const lines = item.lines.length ? item.lines : [item.value];
    for (let i = 0; i < lines.length; i += 1) {
      ctx.fillText(lines[i], item.x, item.y + item.ascent + i * item.lineHeight);
    }
  }
}

if (!customElements.get("rva-image")) {
  customElements.define("rva-image", RvaImage);
}
