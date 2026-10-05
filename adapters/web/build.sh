#!/usr/bin/env bash
# Build the RVA core to WebAssembly and generate the browser bindings.
#
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version 0.2.129
set -euo pipefail

adapter_dir="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$adapter_dir/../.." && pwd)"

cargo build \
  --manifest-path "$root/Cargo.toml" \
  -p rva-wasm \
  --target wasm32-unknown-unknown \
  --release

wasm_bin="$root/target/wasm32-unknown-unknown/release/rva_wasm.wasm"
wasm-bindgen "$wasm_bin" --out-dir "$adapter_dir/src/pkg" --target web

# Optional further shrink if binaryen is installed.
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Oz --strip-debug \
    "$adapter_dir/src/pkg/rva_wasm_bg.wasm" \
    -o "$adapter_dir/src/pkg/rva_wasm_bg.wasm"
fi

echo "web bindings written to $adapter_dir/src/pkg"
