// Cross-runtime conformance check.
//
// Invariant:
//   same .rva + same viewport + same resolver profile
//       = byte-identical ResolvedScene
//
// Compares three independent runtime paths over a continuous size matrix:
//   - native  : the Rust CLI (`rva inspect --compact`)
//   - wasm    : @rva/node's WASM core (wasm-bindgen)
//   - c-abi   : a C program linking librva_ffi (ffi/include/rva.h)
//
// All three serialize the same Rust ResolvedScene via serde_json, so the raw
// JSON strings must match exactly.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const dir = path.dirname(new URL(import.meta.url).pathname);
const root = path.resolve(dir, "..");

const rvaBin = path.join(root, "target", "debug", "rva");
const cabiBin = path.join(dir, "cabi_dump");
const sizesFile = path.join(dir, "sizes.txt");

const fixtures = process.argv.slice(2);
const fixtureList = fixtures.length
  ? fixtures
  : [
      path.join(root, "examples", "node", "hero.rva"),
      // CTA regions: element-bound + padded manual region across two topologies.
      path.join(root, "examples", "node", "cta.rva"),
    ];

// Continuous, deliberately awkward size matrix (not just 16:9/1:1/9:16).
const sizes = [];
for (let i = 0; i < 60; i++) {
  const width = 240 + i * 40; // 240..2600
  const ratio = 0.4 + (i % 7) * 0.25; // 0.4 .. 1.9
  const height = Math.max(160, Math.round(width * ratio));
  sizes.push([width, height]);
}
fs.writeFileSync(sizesFile, sizes.map(([w, h]) => `${w} ${h}`).join("\n"));

const rva = require("@rva/node");
let failures = 0;

for (const fixture of fixtureList) {
  const label = path.relative(root, fixture);

  const native = new Map();
  for (const [w, h] of sizes) {
    const json = execFileSync(
      rvaBin,
      ["--asset", fixture, "inspect", "--width", String(w), "--height", String(h), "--compact"],
      { encoding: "utf8" }
    ).trim();
    native.set(`${w}x${h}`, json);
  }

  const handle = rva.open(new Uint8Array(fs.readFileSync(fixture)));
  const wasm = new Map();
  for (const [w, h] of sizes) wasm.set(`${w}x${h}`, handle.resolve(w, h).trim());

  const cabi = new Map();
  const cabiOut = execFileSync(cabiBin, [fixture, sizesFile], { encoding: "utf8" });
  for (const line of cabiOut.trim().split("\n")) {
    const first = line.indexOf(" ");
    const second = line.indexOf(" ", first + 1);
    const key = `${line.slice(0, first)}x${line.slice(first + 1, second)}`;
    cabi.set(key, line.slice(second + 1).trim());
  }

  let mismatches = 0;
  for (const [w, h] of sizes) {
    const key = `${w}x${h}`;
    const a = native.get(key);
    const b = wasm.get(key);
    const c = cabi.get(key);
    if (a !== b || a !== c) {
      mismatches++;
      if (mismatches <= 3) {
        console.error(`  MISMATCH ${key}`);
        console.error(`    native ${a?.slice(0, 140)}`);
        console.error(`    wasm   ${b?.slice(0, 140)}`);
        console.error(`    c-abi  ${c?.slice(0, 140)}`);
      }
    }
  }

  const verdict = mismatches === 0 ? "OK " : "XX ";
  console.log(
    `${verdict}${label}: ${sizes.length} sizes, native==wasm==c-abi (${mismatches} mismatch)`
  );
  if (mismatches > 0) failures++;
}

if (failures > 0) {
  console.error(`conformance FAILED (${failures} fixture(s))`);
  process.exit(1);
}
console.log("conformance OK");
