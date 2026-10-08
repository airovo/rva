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
import type { Paint, ResolvedCtaRegion, ResolvedItem, ResolvedScene } from "@airovo/rva-types";

/** A CTA region activation: the stable id plus its resolved geometry. */
export interface RvaCtaActivation {
  id: string;
  region: ResolvedCtaRegion;
}

export interface RvaLoadDetail {
  description: string;
  src: string | null;
}

export interface RvaErrorDetail {
  error: unknown;
  src: string | null;
}

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

async function toBytes(value: Uint8Array | ArrayBuffer | Blob): Promise<Uint8Array> {
  if (value instanceof Uint8Array) return value;
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  return new Uint8Array(await value.arrayBuffer());
}

/**
 * Turn a resolved `Paint` (solid colour or gradient) into a canvas fill style.
 * Mirrors the reference renderer so gradients match the core/viewer.
 */
function paintStyle(
  ctx: CanvasRenderingContext2D,
  paint: Paint,
  x: number,
  y: number,
  w: number,
  h: number
): string | CanvasGradient {
  if (paint.type === "color") return paint.color;
  let gradient: CanvasGradient;
  if (paint.type === "radialGradient") {
    const cx = x + w / 2;
    const cy = y + h / 2;
    gradient = ctx.createRadialGradient(cx, cy, 0, cx, cy, Math.max(w, h) / 2);
  } else {
    const rad = (paint.angle * Math.PI) / 180;
    const x1 = x + (0.5 - 0.5 * Math.cos(rad)) * w;
    const y1 = y + (0.5 - 0.5 * Math.sin(rad)) * h;
    const x2 = x + (0.5 + 0.5 * Math.cos(rad)) * w;
    const y2 = y + (0.5 + 0.5 * Math.sin(rad)) * h;
    gradient = ctx.createLinearGradient(x1, y1, x2, y2);
  }
  for (const stop of paint.stops) {
    gradient.addColorStop(Math.max(0, Math.min(1, stop.offset)), stop.color);
  }
  return gradient;
}

function fillPaint(
  ctx: CanvasRenderingContext2D,
  paint: Paint,
  x: number,
  y: number,
  w: number,
  h: number
): void {
  ctx.fillStyle = paintStyle(ctx, paint, x, y, w, h);
  ctx.fillRect(x, y, w, h);
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
  private lastScene: ResolvedScene | null = null;
  private readonly ctaBindings = new Map<string, Set<() => void>>();
  private readonly onClick = (event: MouseEvent): void => this.handleCtaClick(event);

  // Callback-style API (parity with the React wrapper's props). These are
  // optional sugar that runs alongside the native `rva-*` events.
  onRender: ((scene: ResolvedScene) => void) | null = null;
  onCtaRegions: ((regions: ResolvedCtaRegion[]) => void) | null = null;
  onCta: ((activation: RvaCtaActivation) => void) | null = null;
  ctaHandlers: Record<string, (activation: RvaCtaActivation) => void> | null = null;
  onLoad: ((detail: RvaLoadDetail) => void) | null = null;
  onError: ((detail: RvaErrorDetail) => void) | null = null;

  /**
   * Optional: supply bytes directly, bypassing `src`/fetch. Useful for bundled
   * or locally-read files (e.g. a `file://` asset a host has already loaded).
   * Set the property, then call `reload()`.
   */
  bytes: Uint8Array | ArrayBuffer | Blob | null = null;

  /**
   * Optional: resolve `src` to bytes when `fetch` cannot (custom protocols,
   * Tauri `convertFileSrc`, authenticated sources…). Set before `src`.
   */
  srcResolver:
    | ((src: string) => Promise<ArrayBuffer | Uint8Array> | ArrayBuffer | Uint8Array)
    | null = null;

  /**
   * Visible CTA regions of the current resolution. Identity + geometry only —
   * the asset carries no behavior; bind it here.
   */
  get ctaRegions(): ResolvedCtaRegion[] {
    return (this.lastScene?.ctaRegions ?? []).filter((region) => region.visible);
  }

  /**
   * Bind a handler to a CTA region id. Returns an unbind function. The host
   * decides what activation means (navigate, open, track…).
   *
   *   rvaImage.bindCta("shop-now", () => router.push("/product"));
   */
  bindCta(id: string, handler: () => void): () => void {
    let set = this.ctaBindings.get(id);
    if (!set) {
      set = new Set();
      this.ctaBindings.set(id, set);
    }
    set.add(handler);
    return () => this.unbindCta(id, handler);
  }

  unbindCta(id: string, handler?: () => void): void {
    if (!handler) {
      this.ctaBindings.delete(id);
      return;
    }
    const set = this.ctaBindings.get(id);
    set?.delete(handler);
    if (set && set.size === 0) this.ctaBindings.delete(id);
  }

  unbindAllCta(): void {
    this.ctaBindings.clear();
  }

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
    this.canvas.addEventListener("click", this.onClick);

    if (!this.observer) {
      this.observer = new ResizeObserver(() => this.scheduleDraw());
      this.observer.observe(this);
    }
    this.syncAlt();
    if (this.getAttribute("src") || this.bytes) void this.load();
  }

  /** Re-load from the current `bytes`/`srcResolver`/`src`. */
  reload(): void {
    void this.load();
  }

  disconnectedCallback(): void {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
    this.canvas?.removeEventListener("click", this.onClick);
    for (const entry of this.images.values()) URL.revokeObjectURL(entry.url);
    this.images.clear();
    this.handle = null;
    this.lastScene = null;
  }

  private handleCtaClick(event: MouseEvent): void {
    const scene = this.lastScene;
    if (!scene || !this.canvas) return;
    const rect = this.canvas.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;
    const x = ((event.clientX - rect.left) / rect.width) * scene.width;
    const y = ((event.clientY - rect.top) / rect.height) * scene.height;
    const region = this.ctaRegions.find(
      (candidate) =>
        x >= candidate.bounds.x &&
        x <= candidate.bounds.x + candidate.bounds.width &&
        y >= candidate.bounds.y &&
        y <= candidate.bounds.y + candidate.bounds.height
    );
    if (!region) return;
    const activation: RvaCtaActivation = { id: region.id, region };
    this.dispatchEvent(new CustomEvent("rva-cta", { detail: activation }));
    this.onCta?.(activation);
    this.ctaHandlers?.[region.id]?.(activation);
    for (const handler of this.ctaBindings.get(region.id) ?? []) handler();
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
    if (!src && !this.bytes) return;
    try {
      await ensureWasm();
      const bytes = await this.readBytes(src);
      this.handle = load(bytes);
      const description = this.handle.describe();
      this.dispatchEvent(new CustomEvent("rva-load", { detail: { description, src } }));
      this.onLoad?.({ description, src });
      this.scheduleDraw();
    } catch (error) {
      this.dispatchEvent(new CustomEvent("rva-error", { detail: { error, src } }));
      this.onError?.({ error, src });
      console.error("[rva-image]", error);
    }
  }

  /**
   * Resolve asset bytes from, in order: an explicit `bytes` value, a
   * `srcResolver`, or `fetch(src)`. Browsers/WebViews block `file://` fetches;
   * for those, supply `bytes` or a `srcResolver` (e.g. Tauri `convertFileSrc`).
   */
  private async readBytes(src: string | null): Promise<Uint8Array> {
    if (this.bytes) return toBytes(this.bytes);
    if (this.srcResolver && src) return toBytes(await this.srcResolver(src));
    if (!src) throw new Error("no src or bytes provided");
    if (src.startsWith("file://")) {
      throw new Error(
        "file:// sources cannot be fetched here — set `bytes` or `srcResolver` " +
          "(Tauri: use convertFileSrc; Node: use @airovo/rva-node openSource)"
      );
    }
    const response = await fetch(src);
    if (!response.ok) throw new Error(`fetch ${src}: ${response.status}`);
    return new Uint8Array(await response.arrayBuffer());
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
    this.lastScene = scene;

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
    } else if (background && background.type === "paint") {
      fillPaint(ctx, background.paint, 0, 0, cssW, cssH);
    }

    for (const item of scene.items) {
      ctx.globalAlpha = item.opacity ?? 1;
      if (item.type === "text") {
        this.drawText(ctx, item);
        continue;
      }
      if (item.type === "paint") {
        fillPaint(ctx, item.paint, item.x, item.y, item.w, item.h);
        continue;
      }
      if (!("resource" in item)) continue;
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

    this.onRender?.(scene);
    this.onCtaRegions?.(scene.ctaRegions ?? []);
    this.dispatchEvent(new CustomEvent("rva-render", { detail: scene }));
  }

  private drawText(ctx: CanvasRenderingContext2D, item: Extract<ResolvedItem, { type: "text" }>): void {
    // Optional solid text-box background.
    if (item.background) {
      ctx.fillStyle = item.background;
      ctx.fillRect(item.x, item.y, item.w, item.h);
    }

    // Fill: `fill` (colour or gradient) wins; then explicit `color`; then the
    // role default. Gradient coordinates are relative to the text box.
    ctx.fillStyle = item.fill
      ? paintStyle(ctx, item.fill, item.x, item.y, item.w, item.h)
      : item.color ?? (item.role === "subheadline" ? "#334155" : "#0F172A");

    ctx.textBaseline = "alphabetic";
    ctx.font = `${item.weight} ${item.size}px ${fontStack(item.fontFamily)}`;

    const align = item.align ?? "left";
    const anchorX =
      align === "center" ? item.x + item.w / 2 : align === "right" ? item.x + item.w : item.x;
    ctx.textAlign = align === "center" ? "center" : align === "right" ? "right" : "left";

    const styled = ctx as CanvasRenderingContext2D & {
      letterSpacing?: string;
      wordSpacing?: string;
    };
    if ("letterSpacing" in styled) styled.letterSpacing = `${item.letterSpacing ?? 0}px`;
    if ("wordSpacing" in styled) styled.wordSpacing = `${item.wordSpacing ?? 0}px`;

    const lines = item.lines.length ? item.lines : [item.value];
    for (let i = 0; i < lines.length; i += 1) {
      ctx.fillText(lines[i], anchorX, item.y + item.ascent + i * item.lineHeight);
    }

    ctx.textAlign = "left";
    if ("letterSpacing" in styled) styled.letterSpacing = "0px";
    if ("wordSpacing" in styled) styled.wordSpacing = "0px";
  }
}

if (!customElements.get("rva-image")) {
  customElements.define("rva-image", RvaImage);
}
