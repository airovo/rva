#!/usr/bin/env node
// Assert the RVA release version is identical across every manifest that carries
// one. When run on a git tag (CI sets GITHUB_REF_TYPE=tag), also assert the tag
// matches. Run in CI and at the top of the publish workflows.
//
//   node scripts/check-versions.mjs          # manifests only
//   node scripts/check-versions.mjs v0.1.1   # also check against an explicit tag

import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => readFileSync(path.join(root, p), "utf8");
const fromJson = (p) => JSON.parse(read(p)).version;

const entries = [];
const add = (label, version) => entries.push([label, version]);

// Cargo workspace (the release version for all crates).
add(
  "Cargo.toml",
  read("Cargo.toml").match(/\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1],
);

// Adapter contract.
add("adapters/contract.json", fromJson("adapters/contract.json"));

// npm packages.
for (const pkg of ["types", "web", "node", "react", "vue", "svelte", "react-native"]) {
  add(`adapters/${pkg}/package.json`, fromJson(`adapters/${pkg}/package.json`));
}

// Flutter.
add(
  "adapters/flutter/pubspec.yaml",
  read("adapters/flutter/pubspec.yaml").match(/^version:\s*([0-9][^\s+]*)/m)?.[1],
);

// Kotlin / Android.
add(
  "adapters/kotlin/build.gradle.kts",
  read("adapters/kotlin/build.gradle.kts").match(/^version\s*=\s*"([^"]+)"/m)?.[1],
);

// React Native iOS podspec.
add(
  "adapters/react-native/RNRva.podspec",
  read("adapters/react-native/RNRva.podspec").match(/^\s*s\.version\s*=\s*"([^"]+)"/m)?.[1],
);

// git tag (CI tags, or an explicit argument).
const tag =
  process.env.GITHUB_REF_TYPE === "tag"
    ? process.env.GITHUB_REF_NAME
    : process.argv[2];
if (tag) add(`git tag ${tag}`, tag.replace(/^v/, ""));

const width = Math.max(...entries.map(([label]) => label.length));
for (const [label, version] of entries) {
  console.log(`  ${label.padEnd(width)}  ${version ?? "<missing>"}`);
}

const missing = entries.filter(([, version]) => !version);
const distinct = new Set(entries.map(([, version]) => version));

if (missing.length > 0 || distinct.size > 1) {
  const expected = entries.find(([, version]) => version)?.[1] ?? "?";
  console.error(`\n::error::version parity failed — expected every manifest to be ${expected}`);
  process.exit(1);
}

console.log(`\nversion parity OK: ${entries[0][1]}`);
