"use strict";

// Node.js example for the @airovo/rva-node adapter (installed from the workspace).
//
//   node index.js [asset] [outDir]
//
// `asset` may be a filesystem path, a file:// URL, an http(s):// URL, or a
// data: URL. One .rva asset is resolved and rendered across the canonical
// viewport matrix.

const fs = require("fs");
const path = require("path");

const rva = require("@airovo/rva-node");

const asset = process.argv[2] || path.join(__dirname, "hero.rva");
const outDir = process.argv[3] || path.join(__dirname, "out");

const CANONICAL_SIZES = [
  [1920, 500, "wide"],
  [1280, 720, "desktop"],
  [768, 1024, "tablet"],
  [1080, 1080, "square"],
  [430, 932, "mobile"],
  [2560, 800, "ultrawide"],
];

async function main() {
  fs.mkdirSync(outDir, { recursive: true });

  const handle = await rva.openSource(asset);
  console.log(handle.describe());
  console.log("asset:", asset.startsWith("http") ? asset : path.relative(process.cwd(), asset));
  console.log("");

  console.log(
    "size".padEnd(12) +
      "topology".padEnd(11) +
      "viability".padEnd(11) +
      "items".padEnd(7) +
      "output"
  );

  for (const [width, height, name] of CANONICAL_SIZES) {
    const { scene, svg } = rva.renderSvg(handle, width, height);
    const file = path.join(outDir, `${name}-${width}x${height}.svg`);
    fs.writeFileSync(file, svg);

    console.log(
      `${`${width}x${height}`.padEnd(12)}` +
        `${scene.topology.padEnd(11)}` +
        `${scene.viability.toFixed(2).padEnd(11)}` +
        `${String(scene.items.length).padEnd(7)}` +
        `${path.relative(process.cwd(), file)}`
    );
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
