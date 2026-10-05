"use strict";

// Regenerate the Node.js WASM bindings used by this adapter.
//
//   rustup target add wasm32-unknown-unknown
//   cargo install wasm-bindgen-cli --version 0.2.129
//   node adapters/node/build.js

const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const adapterDir = __dirname;
const root = path.resolve(adapterDir, "..", "..");
const wasm = path.join(
  root,
  "target",
  "wasm32-unknown-unknown",
  "release",
  "rva_wasm.wasm"
);

function run(command, args) {
  console.log(`$ ${command} ${args.join(" ")}`);
  execFileSync(command, args, { stdio: "inherit", cwd: root });
}

run("cargo", [
  "build",
  "-p",
  "rva-wasm",
  "--target",
  "wasm32-unknown-unknown",
  "--release",
]);
run("wasm-bindgen", [wasm, "--out-dir", path.join(adapterDir, "src", "pkg"), "--target", "nodejs"]);

// Optional further shrink if binaryen is installed.
const output = path.join(adapterDir, "src", "pkg", "rva_wasm_bg.wasm");
try {
  execFileSync("wasm-opt", ["-Oz", "--strip-debug", output, "-o", output], { stdio: "inherit" });
} catch {
  /* wasm-opt not installed; skip */
}

console.log("node bindings written to", path.join(adapterDir, "src", "pkg"));
