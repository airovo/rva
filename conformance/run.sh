#!/usr/bin/env bash
# Build everything the conformance check needs and run it.
#
#   ./conformance/run.sh [extra-fixture.rva ...]
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"

echo "building native + C ABI..."
cargo build --manifest-path "$root/Cargo.toml" -p rva-cli -p rva-ffi

echo "refreshing WASM bindings..."
node "$root/adapters/node/build.js" >/dev/null
(cd "$root" && npm run build -w @airovo/rva-node >/dev/null)

echo "compiling C ABI runner..."
lib="$root/target/debug"
cc "$here/cabi_dump.c" \
  -I "$root/ffi/include" \
  -L "$lib" \
  -lrva_ffi \
  -Wl,-rpath,"$lib" \
  -o "$here/cabi_dump"

echo "running cross-runtime conformance..."
node "$here/run.mjs" "$@"
