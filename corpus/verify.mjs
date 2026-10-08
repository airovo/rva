// Adversarial verification of the RVA Torture Corpus.
//
// Two dimensions:
//   1. CROSS-RUNTIME : native (Rust CLI) == WASM (@airovo/rva-node) == C ABI (librva_ffi)
//                      over a continuous matrix plus boundary fuzzing around
//                      every topology threshold (…-0.00001, exact, +0.00001…).
//   2. SPEC BEHAVIOR : the ResolvedScene satisfies semantic invariants
//                      (protected focal regions visible, logo visible, headline
//                      never over a protected focal region, non-croppable
//                      elements not clipped, degradation follows priority).
//
//   node corpus/verify.mjs

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const here = path.dirname(new URL(import.meta.url).pathname);
const root = path.resolve(here, "..");
const assets = path.join(root, "rva-torture-corpus-assets");
const fixturesDir = path.join(here, "fixtures");
const rvaBin = path.join(root, "target", "debug", "rva");
const cabiBin = path.join(here, "cabi_dump");
const sizesFile = path.join(here, "sizes.txt");

const rva = require("@airovo/rva-node");

// --- size matrices ----------------------------------------------------------

const BASE_ASPECTS = [0.30, 0.40, 0.55, 0.70, 0.82, 1.0, 1.2, 1.35, 1.6, 2.0, 2.67, 3.2, 3.55];

function baseSizes() {
  return BASE_ASPECTS.map((ar) => {
    const width = Math.min(2600, Math.max(240, Math.round(900 * Math.sqrt(ar))));
    return [width, Math.max(120, Math.round(width / ar))];
  });
}

function thresholdSizes(scene) {
  const thresholds = new Set();
  for (const topology of scene.topologies ?? []) {
    for (const key of ["minAspectRatio", "maxAspectRatio"]) {
      const value = topology.when?.[key];
      if (typeof value === "number" && value > 0) thresholds.add(value);
    }
  }
  const sizes = [];
  // A large logical height so sub-1e-5 aspect-ratio deltas map to distinct
  // integer dimensions (no rounding collapse at the threshold).
  const height = 100000;
  for (const threshold of thresholds) {
    const deltas = [-0.01, -0.001, -0.0001, -0.00001, 0, 0.00001, 0.0001, 0.001, 0.01];
    for (const delta of deltas) {
      const ar = threshold + delta;
      if (ar <= 0) continue;
      sizes.push([Math.max(1, Math.round(height * ar)), height]);
    }
  }
  return sizes;
}

function dedupe(sizes) {
  const seen = new Set();
  const out = [];
  for (const [w, h] of sizes) {
    const key = `${w}x${h}`;
    if (!seen.has(key) && w > 0 && h > 0) {
      seen.add(key);
      out.push([w, h]);
    }
  }
  return out;
}

// --- runtimes ---------------------------------------------------------------

function nativeResolve(fixture, w, h) {
  return execFileSync(
    rvaBin,
    ["--asset", fixture, "inspect", "--width", String(w), "--height", String(h), "--compact"],
    { encoding: "utf8" }
  ).trim();
}

function wasmResolver(fixture) {
  const handle = rva.open(new Uint8Array(fs.readFileSync(fixture)));
  return (w, h) => handle.resolve(w, h).trim();
}

function cabiResolveAll(fixture, sizes) {
  fs.writeFileSync(sizesFile, sizes.map(([w, h]) => `${w} ${h}`).join("\n"));
  const out = execFileSync(cabiBin, [fixture, sizesFile], { encoding: "utf8" });
  const map = new Map();
  for (const line of out.trim().split("\n")) {
    const a = line.indexOf(" ");
    const b = line.indexOf(" ", a + 1);
    map.set(`${line.slice(0, a)}x${line.slice(a + 1, b)}`, line.slice(b + 1).trim());
  }
  return map;
}

// --- invariants -------------------------------------------------------------

function rectOf(item) {
  return { x: item.x, y: item.y, w: item.w, h: item.h };
}

function rectsOverlap(a, b) {
  const x = Math.max(0, Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x));
  const y = Math.max(0, Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y));
  return x * y;
}

function checkInvariants(scene, resolved) {
  // Fallback is a defined terminal state; item-level invariants do not apply.
  if (resolved.topology === "fallback") return [];
  const issues = [];
  const W = resolved.width;
  const H = resolved.height;
  const items = resolved.items ?? [];
  const byId = new Map(items.map((item) => [item.id, item]));
  const elements = new Map((scene.elements ?? []).map((e) => [e.id, e]));

  // 1. Hard focal regions must be visible (inside the canvas).
  for (const item of items) {
    for (const region of item.focalRegions ?? []) {
      if (!region.hard) continue;
      const fx = item.x + region.x * item.w;
      const fy = item.y + region.y * item.h;
      const fw = region.w * item.w;
      const fh = region.h * item.h;
      if (fx < -0.5 || fy < -0.5 || fx + fw > W + 0.5 || fy + fh > H + 0.5) {
        issues.push(`protected region '${region.name}' of '${item.id}' is not fully visible`);
      }
    }
  }

  // 2. Logo must remain visible.
  for (const [id, element] of elements) {
    if (element.role === "logo" && !byId.has(id)) {
      issues.push(`logo '${id}' is not visible`);
    }
  }

  // 3. Headline must not overlap a protected focal region.
  const headline = items.find((item) => item.role === "headline");
  if (headline) {
    const hRect = rectOf(headline);
    for (const item of items) {
      for (const region of item.focalRegions ?? []) {
        if (!region.hard) continue;
        const fRect = {
          x: item.x + region.x * item.w,
          y: item.y + region.y * item.h,
          w: region.w * item.w,
          h: region.h * item.h,
        };
        if (rectsOverlap(hRect, fRect) > 1) {
          issues.push(`headline overlaps protected region '${region.name}' of '${item.id}'`);
        }
      }
    }
  }

  // 4. Non-croppable elements must not be clipped.
  for (const item of items) {
    if (item.cropPolicy !== "none") continue;
    if (item.x < -0.5 || item.y < -0.5 || item.x + item.w > W + 0.5 || item.y + item.h > H + 0.5) {
      issues.push(`non-croppable '${item.id}' is clipped`);
    }
  }

  // 5. Degradation must hide the lowest-priority optional elements first.
  if (resolved.degraded) {
    const hiddenPriorities = (resolved.hidden ?? []).map((id) => elements.get(id)?.priority ?? 0);
    const visibleOptional = items
      .filter((item) => elements.get(item.id)?.visibility === "optional")
      .map((item) => elements.get(item.id)?.priority ?? 0);
    for (const hp of hiddenPriorities) {
      for (const vp of visibleOptional) {
        if (hp > vp) issues.push(`degradation hid priority ${hp} before visible priority ${vp}`);
      }
    }
  }

  return issues;
}

// --- main -------------------------------------------------------------------

const only = process.argv[2];
const ids = fs
  .readdirSync(fixturesDir)
  .filter((f) => f.endsWith(".rva"))
  .map((f) => f.replace(/\.rva$/, ""))
  .filter((id) => !id.endsWith(".corrupt"))
  .sort()
  .filter((id) => !only || id.includes(only));

let crossFails = 0;
let invariantFails = 0;
let totalSizes = 0;

for (const id of ids) {
  const fixture = path.join(fixturesDir, `${id}.rva`);
  const scene = JSON.parse(fs.readFileSync(path.join(assets, id, "scene.json"), "utf8"));

  // Fixture 12: corrupt a copy and require deterministic failure everywhere.
  if (id === "12-missing-resource") {
    const bytes = fs.readFileSync(fixture);
    for (let i = bytes.length - 64; i < bytes.length; i++) bytes[i] ^= 0xff;
    const corrupt = path.join(fixturesDir, `${id}.corrupt.rva`);
    fs.writeFileSync(corrupt, bytes);

    let nativeFails = false;
    let wasmFails = false;
    let cabiFails = false;
    try {
      execFileSync(rvaBin, ["--asset", corrupt, "inspect", "--width", "1080", "--height", "1080", "--compact"], { stdio: "ignore" });
    } catch { nativeFails = true; }
    try { rva.open(new Uint8Array(fs.readFileSync(corrupt))); } catch { wasmFails = true; }
    try {
      fs.writeFileSync(sizesFile, "1080 1080");
      execFileSync(cabiBin, [corrupt, sizesFile], { stdio: "ignore" });
    } catch { cabiFails = true; }

    const ok = nativeFails && wasmFails && cabiFails;
    if (!ok) crossFails++;
    console.log(
      `${ok ? "OK " : "XX "}${id}: corrupt package → native=${nativeFails} wasm=${wasmFails} cabi=${cabiFails} (all must reject)`
    );
    continue;
  }

  const sizes = dedupe([...baseSizes(), ...thresholdSizes(scene)]);
  totalSizes += sizes.length;

  const cabi = cabiResolveAll(fixture, sizes);
  const wasm = wasmResolver(fixture);

  let mismatches = 0;
  let firstMismatch = "";
  const invariantIssues = [];
  let degradedCount = 0;
  let fallbackCount = 0;

  for (const [w, h] of sizes) {
    const key = `${w}x${h}`;
    const a = nativeResolve(fixture, w, h);
    const b = wasm(w, h);
    const c = cabi.get(key);
    if (a !== b || a !== c) {
      mismatches++;
      if (!firstMismatch) firstMismatch = `${key}`;
    }
    const resolved = JSON.parse(a);
    if (resolved.degraded) degradedCount++;
    if (resolved.topology === "fallback") fallbackCount++;
    const issues = checkInvariants(scene, resolved);
    if (issues.length) invariantIssues.push(`${key}: ${issues[0]}`);
  }

  const crossOk = mismatches === 0;
  if (!crossOk) crossFails++;
  if (invariantIssues.length) invariantFails++;

  const invSummary = invariantIssues.length
    ? `invariant FAIL (${invariantIssues.length}/${sizes.length}) e.g. ${invariantIssues[0]}`
    : "invariants OK";
  const coverage = `degraded=${degradedCount} fallback=${fallbackCount}`;
  const finding = id === "15-fallback" && fallbackCount === 0 ? " [FINDING: fallback never selected in current profile]" : "";
  console.log(
    `${crossOk ? "OK " : "XX "}${id}: cross-runtime ${sizes.length} sizes ${crossOk ? "" : `mismatch @${firstMismatch}`}; ${invSummary}; ${coverage}${finding}`
  );
}

console.log(
  `\nsummary: ${ids.length} fixtures, ${totalSizes} resolutions; cross-runtime failures=${crossFails}, fixtures with invariant failures=${invariantFails}`
);
process.exit(crossFails > 0 || invariantFails > 0 ? 1 : 0);
