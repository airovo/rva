// A dependency-free Node/server renderer for RVA.
//
// Mirrors the reference renderer in `core/src/render.rs`: it takes the resolved
// scene produced by the WASM core and composites it into an SVG document. SVG
// keeps text semantic (no bundled fonts required) while still embedding the
// raster/vector resources as data URIs.

import type { ResolvedItem, ResolvedScene } from "@rva/types";
import type { RvaHandle } from "./pkg/rva_wasm.js";

const FONT_STACKS: Record<string, string> = {
  "system-ui": "system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif",
  "ui-sans-serif": "system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif",
  sans: "system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif",
  serif: "Georgia, Times New Roman, serif",
  monospace: "ui-monospace, SFMono-Regular, Menlo, monospace",
};

export function fontStack(family: string): string {
  const stack = FONT_STACKS[family] ?? `${family}, sans-serif`;
  return stack.replace(/"/g, "");
}

export function mimeFor(relative: string): string {
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

function dataUri(handle: RvaHandle, name: string): string {
  const bytes = handle.resource(name);
  const mime = mimeFor(handle.relative_for(name));
  return `data:${mime};base64,${Buffer.from(bytes).toString("base64")}`;
}

function svgIntrinsic(svg: string): [number, number] {
  const viewBox = /\bviewBox\s*=\s*"([^"]+)"/i.exec(svg);
  if (viewBox) {
    const nums = viewBox[1]
      .split(/[\s,]+/)
      .map(Number)
      .filter((n) => Number.isFinite(n));
    if (nums.length === 4 && nums[2] > 0 && nums[3] > 0) return [nums[2], nums[3]];
  }
  const width = /\bwidth\s*=\s*"([0-9.]+)/i.exec(svg);
  const height = /\bheight\s*=\s*"([0-9.]+)/i.exec(svg);
  if (width && height) return [Number(width[1]), Number(height[1])];
  return [0, 0];
}

function svgInner(svg: string): string {
  const open = svg.indexOf(">");
  const close = svg.lastIndexOf("</svg>");
  if (open < 0 || close <= open) return "";
  return svg.slice(open + 1, close);
}

function escapeXml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

interface RenderedMask {
  defs: string;
  body: string;
}

function renderItem(handle: RvaHandle, item: ResolvedItem): string | RenderedMask {
  if (item.type === "text") {
    const family = fontStack(item.fontFamily);
    const fill = item.role === "subheadline" ? "#334155" : "#0F172A";
    return item.lines
      .map((line, index) => {
        const y = item.y + item.ascent + index * item.lineHeight;
        return (
          `<text x="${item.x}" y="${y}" font-family="${family}" ` +
          `font-size="${item.size}" font-weight="${item.weight}" ` +
          `fill="${fill}" xml:space="preserve">${escapeXml(line)}</text>`
        );
      })
      .join("");
  }

  if (item.type === "vector") {
    const text = Buffer.from(handle.resource(item.resource)).toString("utf8");
    const [iw] = svgIntrinsic(text);
    const scale = iw > 0 ? item.w / iw : 1;
    return `<g transform="translate(${item.x},${item.y}) scale(${scale})">${svgInner(text)}</g>`;
  }

  const uri = dataUri(handle, item.resource);
  const image =
    `<image x="${item.x}" y="${item.y}" width="${item.w}" height="${item.h}" ` +
    `preserveAspectRatio="none" href="${uri}"/>`;
  if (!item.mask) return image;

  const id = `m-${item.id}`;
  const maskUri = dataUri(handle, item.mask);
  const mask =
    `<mask id="${id}" maskUnits="userSpaceOnUse" x="${item.x}" y="${item.y}" ` +
    `width="${item.w}" height="${item.h}"><image x="${item.x}" y="${item.y}" ` +
    `width="${item.w}" height="${item.h}" preserveAspectRatio="none" href="${maskUri}"/></mask>`;
  return { defs: mask, body: `<g mask="url(#${id})">${image}</g>` };
}

export function renderToSvg(handle: RvaHandle, scene: ResolvedScene): string {
  const { width, height } = scene;
  const defs: string[] = [];
  const body: string[] = [];

  if (scene.background && scene.background.type === "image") {
    const uri = dataUri(handle, scene.background.resource);
    const par = scene.background.fit === "cover" ? "xMidYMid slice" : "none";
    body.push(
      `<image x="0" y="0" width="${width}" height="${height}" ` +
        `preserveAspectRatio="${par}" href="${uri}"/>`
    );
  }

  for (const item of scene.items) {
    const rendered = renderItem(handle, item);
    if (typeof rendered === "string") {
      body.push(rendered);
    } else {
      defs.push(rendered.defs);
      body.push(rendered.body);
    }
  }

  return (
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" ` +
    `viewBox="0 0 ${width} ${height}"><defs>${defs.join("")}</defs>` +
    `<rect width="${width}" height="${height}" fill="#ffffff"/>` +
    `${body.join("")}</svg>`
  );
}
