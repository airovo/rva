// Build .rva fixtures from the RVA Torture Corpus source assets.
//
// Authoring is parametric: a base scene (background, logo, subject, product,
// text, three topologies) with per-fixture overrides that target one question.
//
//   node corpus/build.mjs
//
// Writes <fixture>/scene.json into each asset folder and packs
// corpus/fixtures/<id>.rva with the current experimental toolchain.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(new URL(import.meta.url).pathname, "..", "..");
const assets = path.join(root, "rva-torture-corpus-assets");
const outDir = path.join(root, "corpus", "fixtures");
const rva = path.join(root, "target", "debug", "rva");

fs.mkdirSync(outDir, { recursive: true });

const L = (x, y, w, extra = {}) => ({ x, y, w, ...extra });

// Default per-topology placements by element id.
const PLACE = {
  landscape: {
    logo: L(0.05, 0.06, 0.12),
    headline: L(0.07, 0.26, 0.36),
    subheadline: L(0.07, 0.55, 0.36),
    subject: L(0.60, 0.10, 0.30, { anchor: "bottom-right" }),
    subjectA: L(0.58, 0.12, 0.28, { anchor: "bottom-right" }),
    subjectB: L(0.36, 0.20, 0.24, { anchor: "bottom-center" }),
    product: L(0.40, 0.52, 0.30, { anchor: "bottom-center" }),
    "wide-product": L(0.38, 0.45, 0.42, { anchor: "bottom-center" }),
    "portrait-subject": L(0.58, 0.08, 0.30, { anchor: "bottom-right" }),
    "anchor-object": L(0.62, 0.12, 0.26, { anchor: "bottom-right" }),
    "secondary-object": L(0.40, 0.30, 0.20),
    "foreground-focal": L(0.10, 0.20, 0.30),
    "pagoda-like-mark": L(0.78, 0.10, 0.16),
  },
  balanced: {
    logo: L(0.07, 0.05, 0.18),
    headline: L(0.07, 0.16, 0.5),
    subheadline: L(0.07, 0.42, 0.42),
    subject: L(0.56, 0.36, 0.4, { anchor: "bottom-right" }),
    subjectA: L(0.54, 0.38, 0.38, { anchor: "bottom-right" }),
    subjectB: L(0.1, 0.5, 0.32, { anchor: "bottom-left" }),
    product: L(0.07, 0.66, 0.44, { anchor: "bottom-left" }),
    "wide-product": L(0.20, 0.60, 0.58, { anchor: "bottom-center" }),
    "portrait-subject": L(0.48, 0.30, 0.44, { anchor: "bottom-right" }),
    "anchor-object": L(0.50, 0.34, 0.42, { anchor: "bottom-right" }),
    "secondary-object": L(0.12, 0.52, 0.32),
    "foreground-focal": L(0.10, 0.30, 0.40),
    "pagoda-like-mark": L(0.70, 0.12, 0.20),
  },
  portrait: {
    logo: L(0.08, 0.04, 0.24),
    headline: L(0.08, 0.12, 0.80),
    subheadline: L(0.08, 0.32, 0.80),
    subject: L(0.26, 0.46, 0.54, { anchor: "bottom-center" }),
    subjectA: L(0.28, 0.44, 0.50, { anchor: "bottom-center" }),
    subjectB: L(0.06, 0.58, 0.40, { anchor: "bottom-left" }),
    product: L(0.02, 0.74, 0.50, { anchor: "bottom-left" }),
    "wide-product": L(0.05, 0.72, 0.72, { anchor: "bottom-center" }),
    "portrait-subject": L(0.24, 0.42, 0.56, { anchor: "bottom-center" }),
    "anchor-object": L(0.28, 0.44, 0.50, { anchor: "bottom-center" }),
    "secondary-object": L(0.10, 0.60, 0.40),
    "foreground-focal": L(0.08, 0.40, 0.60),
    "pagoda-like-mark": L(0.62, 0.10, 0.26),
  },
};

// Decorations stack over the product (the protected owner) so they create
// viability pressure and exercise degradation.
const DECOR_POS = {
  landscape: L(0.40, 0.52, 0.30),
  balanced: L(0.07, 0.66, 0.44),
  portrait: L(0.02, 0.74, 0.5),
};

function layoutFor(id, topo, fallbackPos) {
  if (id.startsWith("decor-")) return DECOR_POS[topo];
  return PLACE[topo][id] ?? fallbackPos;
}

// Spread indexed elements (decor-*, element-*, critical-*) over a grid so they
// don't stack on one another.
function gridLayout(index, count) {
  const cols = Math.ceil(Math.sqrt(count));
  const rows = Math.ceil(count / cols);
  const col = index % cols;
  const row = Math.floor(index / cols);
  const w = Math.min(0.16, 2.6 / cols);
  const x = 0.06 + (col + 0.5) * (0.88 / cols) - w / 2;
  const y = 0.1 + (row + 0.5) * (0.8 / rows) - w / 2;
  return L(Number(x.toFixed(4)), Number(y.toFixed(4)), w);
}

function topology(id, min, max, ids) {
  const when = {};
  if (min != null) when.minAspectRatio = min;
  if (max != null) when.maxAspectRatio = max;
  const layout = {};
  for (const elementId of ids) {
    const pos =
      layoutFor(elementId, id === "landscape" ? "landscape" : id, null) ??
      L(0.1, 0.2, 0.3);
    layout[elementId] = pos;
  }
  return { id, when, background: "background", layout };
}

function decorElement(i, priority, overrides = {}) {
  return {
    id: `decor-${String(i).padStart(2, "0")}`,
    type: "raster",
    role: "decoration",
    priority,
    visibility: "optional",
    ...overrides,
  };
}

function build(id, overrides = {}) {
  const dir = path.join(assets, id);
  const files = fs.readdirSync(dir);
  const resources = { background: "background.png", logo: "logo.svg" };
  const elements = [
    { id: "background", type: "raster", role: "background", priority: 100, cropPolicy: "cover" },
  ];

  const push = (el, file) => {
    resources[el.id] = file;
    elements.push(el);
  };

  const stdSubject = (elId, file, role = "primary-subject", focal) =>
    push(
      {
        id: elId,
        type: "raster",
        role,
        priority: 100,
        cropPolicy: "protected",
        focalRegions: [
          focal ?? { name: "face", x: 0.34, y: 0.08, w: 0.36, h: 0.34, hard: true },
        ],
      },
      file
    );

  const stdProduct = (elId, file) =>
    push({ id: elId, type: "raster", role: "product", priority: overrides.productPriority ?? 85, cropPolicy: "none" }, file);

  // Decorations first (lower z), then subjects, product, logo, text.
  const ids = [];

  if (overrides.decorations) {
    for (const d of overrides.decorations) push(d.element, d.file);
  }
  if (overrides.decorElementCount) {
    for (let i = 1; i <= overrides.decorElementCount; i++) {
      const num = String(i).padStart(2, "0");
      push(decorElement(i, 60 - i), `element-${num}.png`);
    }
  }

  if (overrides.manyElements) {
    for (let i = 1; i <= 32; i++) {
      const num = String(i).padStart(2, "0");
      push(decorElement(i, 60 - i), `element-${num}.png`);
    }
  }

  if (overrides.subjects) {
    for (const s of overrides.subjects) stdSubject(s.id, s.file, s.role, s.focal);
  } else if (overrides.noSubject !== true) {
    stdSubject("subject", overrides.subjectFile ?? "subject.png");
  }

  if (overrides.product) {
    stdProduct(overrides.product.id, overrides.product.file);
  } else if (overrides.productFile) {
    stdProduct("product", overrides.productFile);
  }

  push({ id: "logo", type: "vector", role: "logo", priority: 95, cropPolicy: "none" }, "logo.svg");

  // Text.
  const headline = overrides.headline ?? "Create Without Limits";
  const subheadline = overrides.subheadline ?? "Design. Build. Launch. All in one place.";
  elements.push({ id: "headline", type: "text", role: "headline", priority: 100 });
  elements.push({ id: "subheadline", type: "text", role: "subheadline", priority: 80 });

  if (overrides.extraResources) Object.assign(resources, overrides.extraResources);

  const idList = elements.map((e) => e.id);
  const topologies = overrides.topologies ?? [
    topology("landscape", 1.35, null, idList),
    topology("balanced", 0.82, 1.35, idList),
    topology("portrait", null, 0.82, idList),
  ];

  // Spread indexed elements over a grid (no stacking).
  const indexed = idList.filter((id) => /^(decor|element|critical)-\d+$/.test(id));
  indexed.forEach((id, index) => {
    const pos = gridLayout(index, indexed.length);
    for (const topo of topologies) if (topo.layout[id]) topo.layout[id] = pos;
  });

  // Per-element, per-topology layout overrides.
  if (overrides.layout) {
    for (const [id, perTopo] of Object.entries(overrides.layout)) {
      for (const topo of topologies) if (perTopo[topo.id]) topo.layout[id] = perTopo[topo.id];
    }
  }

  const scene = {
    format: "RVA",
    formatVersion: "0.1-draft",
    name: id,
    description: `Torture corpus fixture ${id}`,
    designSpace: { width: 1600, height: 900 },
    resources,
    text: {
      headline: {
        value: headline,
        fontFamily: "system-ui",
        weight: 800,
        minSize: overrides.minSize ?? 28,
        preferredSize: overrides.preferredSize ?? 72,
        maxSize: overrides.maxSize ?? 96,
        maxLines: overrides.maxLines ?? 3,
      },
      subheadline: {
        value: subheadline,
        fontFamily: "system-ui",
        weight: 500,
        minSize: 14,
        preferredSize: 26,
        maxSize: 34,
        maxLines: 3,
      },
    },
    elements,
    topologies,
    fallback: { resource: overrides.fallbackResource ?? "background", fit: "cover" },
  };

  fs.writeFileSync(path.join(dir, "scene.json"), JSON.stringify(scene, null, 2));
  const out = path.join(outDir, `${id}.rva`);
  execFileSync(rva, ["--asset", dir, "pack", "--out", out, "--optimize", "--quality", "88"], {
    stdio: "ignore",
  });
  return out;
}

// --- Fixture definitions ----------------------------------------------------

build("01-basic-hero", { product: { id: "product", file: "product.png" } });

build("02-long-headline", {
  headline:
    "Design. Build. Launch. Ship responsive campaigns without limits across every surface imaginable.",
  maxLines: 6,
  minSize: 18,
  preferredSize: 64,
  maxSize: 88,
  product: { id: "product", file: "product.png" },
});

build("03-multiple-subjects", {
  subjects: [
    { id: "subjectA", file: "subject-a.png" },
    { id: "subjectB", file: "subject-b.png", role: "secondary-subject" },
  ],
});

build("04-product-and-person", {
  productPriority: 99,
  product: { id: "product", file: "product.png" },
});

build("05-overlapping-focal-regions", {
  subjects: [
    { id: "subjectA", file: "subject-a.png" },
    { id: "subjectB", file: "subject-b.png", role: "secondary-subject" },
  ],
  decorations: [
    {
      element: {
        id: "foreground-focal",
        type: "raster",
        role: "product",
        priority: 30,
        visibility: "optional",
        cropPolicy: "none",
        focalRegions: [{ name: "focal", x: 0.25, y: 0.25, w: 0.5, h: 0.5, hard: true }],
      },
      file: "foreground-focal.png",
    },
  ],
});

build("06-extreme-landscape", {
  product: { id: "wide-product", file: "wide-product.png" },
  // Shallow canvas: keep the subject small so it is not oversized/cropped.
  layout: {
    subject: {
      landscape: L(0.78, 0.16, 0.12, { anchor: "bottom-right" }),
      balanced: L(0.6, 0.3, 0.24, { anchor: "bottom-right" }),
      portrait: L(0.3, 0.5, 0.4, { anchor: "bottom-center" }),
    },
    "wide-product": {
      landscape: L(0.36, 0.42, 0.3, { anchor: "bottom-center" }),
      balanced: L(0.2, 0.6, 0.5, { anchor: "bottom-center" }),
      portrait: L(0.05, 0.72, 0.7, { anchor: "bottom-center" }),
    },
  },
});

build("07-extreme-portrait", {
  subjectFile: "portrait-subject.png",
});

const decorLayout = {};
for (let i = 0; i < 8; i++) {
  const id = `decor-${String(i + 1).padStart(2, "0")}`;
  decorLayout[id] = {
    // Landscape: a row near the top, away from the product -> decorations stay.
    landscape: L(0.05 + i * 0.11, 0.08, 0.09),
    // Balanced/portrait: large, low-priority decorations overlapping the
    // protected product -> heavy clutter -> removed lowest-priority-first.
    balanced: L(0.08 + (i % 4) * 0.2, 0.62 + Math.floor(i / 4) * 0.14, 0.2),
    portrait: L(0.05 + (i % 3) * 0.16, 0.72 + Math.floor(i / 3) * 0.1, 0.2),
  };
}
build("08-decoration-degradation", {
  // Decorations (low priority) clutter the protected product owner in narrow
  // topologies; the clutter penalty forces progressive removal there while the
  // landscape keeps them.
  noSubject: true,
  layout: decorLayout,
  decorations: [
    ...Array.from({ length: 8 }, (_, i) => ({
      element: {
        id: `decor-${String(i + 1).padStart(2, "0")}`,
        type: "raster",
        role: "decoration",
        priority: 8 - i,
        visibility: "optional",
        focalRegions: [],
      },
      file: `decor-${String(i + 1).padStart(2, "0")}.png`,
    })),
    {
      element: {
        id: "product",
        type: "raster",
        role: "product",
        priority: 90,
        cropPolicy: "none",
        focalRegions: [{ name: "focus", x: 0.1, y: 0.1, w: 0.8, h: 0.8, hard: true }],
      },
      file: "product.png",
    },
  ],
});

build("09-topology-boundary", {
  subjects: [{ id: "anchor-object", file: "anchor-object.png" }],
  decorations: [
    {
      element: { id: "secondary-object", type: "raster", role: "decoration", priority: 40, visibility: "optional" },
      file: "secondary-object.png",
    },
  ],
});

const critLayout = {};
for (let i = 0; i < 5; i++) {
  const x = 0.05 + i * 0.185;
  const id = `critical-${String(i + 1).padStart(2, "0")}`;
  critLayout[id] = {
    landscape: L(x, 0.6, 0.13),
    balanced: L(x, 0.62, 0.15),
    portrait: L(x, 0.66, 0.16),
  };
}
build("10-impossible-constraints", {
  // Five critical (hard-focal) elements must all stay visible with the headline
  // clear of every protected region; placed out of the text band so they can.
  noSubject: true,
  decorations: Array.from({ length: 5 }, (_, i) => ({
    element: {
      id: `critical-${String(i + 1).padStart(2, "0")}`,
      type: "raster",
      role: "primary-subject",
      priority: 100 - i,
      cropPolicy: "none",
      focalRegions: [{ name: "core", x: 0.3, y: 0.25, w: 0.4, h: 0.45, hard: true }],
    },
    file: `critical-${String(i + 1).padStart(2, "0")}.png`,
  })),
  layout: critLayout,
});

build("11-many-elements", {
  manyElements: true,
  product: { id: "product", file: "product.png" },
});

build("12-missing-resource", {
  decorations: [
    {
      element: {
        id: "product-to-remove",
        type: "raster",
        role: "product",
        priority: 85,
        cropPolicy: "none",
      },
      file: "product-to-remove.png",
    },
    {
      element: {
        id: "control-resource",
        type: "raster",
        role: "decoration",
        priority: 40,
        visibility: "optional",
      },
      file: "control-resource.png",
    },
  ],
  layout: {
    "product-to-remove": {
      landscape: L(0.44, 0.54, 0.3, { anchor: "bottom-center" }),
      balanced: L(0.08, 0.66, 0.44, { anchor: "bottom-left" }),
      portrait: L(0.36, 0.7, 0.5, { anchor: "bottom-center" }),
    },
    "control-resource": {
      landscape: L(0.09, 0.12, 0.12),
      balanced: L(0.7, 0.12, 0.16),
      portrait: L(0.68, 0.1, 0.2),
    },
  },
});

build("13-large-resources", {
  subjectFile: "subject-large.png",
  product: { id: "product", file: "product-large.png" },
});

const unicode = fs
  .readFileSync(path.join(assets, "14-unicode-text", "unicode-samples.txt"), "utf8")
  .trim()
  .split("\n")
  .join("  ");

build("14-unicode-text", {
  headline: `A Wider World  ${unicode}`,
  maxLines: 8,
  minSize: 14,
  preferredSize: 46,
});

build("15-fallback", {
  // Text overflow past maxLines at minimum size marks every topology
  // non-viable, forcing the canonical fallback representation.
  subjectFile: "subject.png",
  extraResources: { fallback: "fallback.png" },
  fallbackResource: "fallback",
  headline:
    "This headline is deliberately far too long to ever fit on a single line at the declared minimum font size so every topology overflows",
  minSize: 300,
  preferredSize: 300,
  maxSize: 300,
  maxLines: 1,
});

console.log("built fixtures ->", outDir);
