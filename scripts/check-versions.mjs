#!/usr/bin/env node
// RVA version / contract check.
//
// The *core* (the Cargo workspace: rva-core / rva-wasm / rva-ffi / rva-cli /
// rva-server) has a single version, and it is what affects cross-runtime
// determinism. Adapters (npm packages, Swift, Kotlin, Flutter) version
// independently — they only need to bundle a compatible core — so they are
// reported but never forced to match the core.
//
// Enforced:
//   1. every workspace crate inherits the workspace version (core-internal parity),
//   2. adapters/contract.json `resolverProfile` == rva_core::RESOLVER_PROFILE,
//   3. when run on a release tag (vX.Y.Z), the tag equals the core version.
//
//   node scripts/check-versions.mjs          # (1) + (2)
//   node scripts/check-versions.mjs v0.1.1   # also (3)

import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => readFileSync(path.join(root, p), "utf8");
const asJson = (p) => JSON.parse(read(p));

const failures = [];

// 1. Core version.
const coreVersion = read("Cargo.toml").match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1];
if (!coreVersion) failures.push("Cargo.toml: [workspace.package] version not found");

for (const crate of ["core", "cli", "ffi", "server", "wasm"]) {
  if (!/^version\.workspace\s*=\s*true/m.test(read(`${crate}/Cargo.toml`))) {
    failures.push(`${crate}/Cargo.toml must use \`version.workspace = true\``);
  }
}

// 2. Resolver profile agreement.
const coreProfile = read("core/src/lib.rs").match(
  /RESOLVER_PROFILE\s*:\s*&str\s*=\s*"([^"]+)"/,
)?.[1];
const contractProfile = asJson("adapters/contract.json").resolverProfile;
if (!coreProfile) {
  failures.push("core/src/lib.rs: RESOLVER_PROFILE not found");
} else if (coreProfile !== contractProfile) {
  failures.push(
    `resolver profile mismatch: core ${coreProfile} vs contract.json ${contractProfile}`,
  );
}

// 3. Release tag (core stream).
const tag =
  process.env.GITHUB_REF_TYPE === "tag" ? process.env.GITHUB_REF_NAME : process.argv[2];
if (tag && /^v?\d/.test(tag)) {
  const tagged = tag.replace(/^v/, "");
  if (tagged !== coreVersion) {
    failures.push(`tag ${tag} does not match core version ${coreVersion}`);
  }
}

// Adapters — independent, reported for visibility only.
const adapters = [
  ...["types", "web", "node", "react", "vue", "svelte", "react-native"].map(
    (p) => [`adapters/${p}/package.json`, asJson(`adapters/${p}/package.json`).version],
  ),
  [
    "adapters/flutter/pubspec.yaml",
    read("adapters/flutter/pubspec.yaml").match(/^version:\s*([0-9][^\s+]*)/m)?.[1],
  ],
  [
    "adapters/kotlin/build.gradle.kts",
    read("adapters/kotlin/build.gradle.kts").match(/^version\s*=\s*"([^"]+)"/m)?.[1],
  ],
  [
    "adapters/react-native/RNRva.podspec",
    read("adapters/react-native/RNRva.podspec").match(/^\s*s\.version\s*=\s*"([^"]+)"/m)?.[1],
  ],
];

console.log(`core version:     ${coreVersion}`);
console.log(`resolver profile: ${coreProfile}`);
console.log("adapters (independent versions, not enforced):");
for (const [name, version] of adapters) {
  console.log(`  ${name.padEnd(36)} ${version ?? "<none>"}`);
}

if (failures.length > 0) {
  for (const failure of failures) console.error(`\n::error::${failure}`);
  process.exit(1);
}
console.log("\ncore version & contract OK");
