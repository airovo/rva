#!/usr/bin/env bash
# Build the C ABI and run the smoke test against a .rva asset.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
lib="$root/target/debug"

cargo build --manifest-path "$root/Cargo.toml" -p rva-ffi

cc "$here/smoke.c" \
  -I "$here/../include" \
  -L "$lib" \
  -lrva_ffi \
  -Wl,-rpath,"$lib" \
  -o "$here/smoke"

"$here/smoke" "${1:-$root/examples/node/hero.rva}"
