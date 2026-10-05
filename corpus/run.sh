#!/usr/bin/env bash
# Build the torture corpus and run adversarial verification.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"

echo "building native + C ABI..."
cargo build --manifest-path "$root/Cargo.toml" -p rva-cli -p rva-ffi

echo "refreshing WASM bindings..."
node "$root/adapters/node/build.js" >/dev/null
(cd "$root" && npm run build -w @rva/node >/dev/null)

echo "compiling C ABI runner..."
cc "$root/conformance/cabi_dump.c" \
  -I "$root/ffi/include" \
  -L "$root/target/debug" \
  -lrva_ffi \
  -Wl,-rpath,"$root/target/debug" \
  -o "$here/cabi_dump"

echo "building fixtures..."
node "$here/build.mjs" >/dev/null

echo "verifying..."
node "$here/verify.mjs" "$@"
