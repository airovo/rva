#!/usr/bin/env bash
# Build the Rust core for Apple platforms and package it as RVAFFI.xcframework.
#
#   ./adapters/swift/build-xcframework.sh
#
# Requires: rustup + Xcode. The Swift package links the resulting xcframework.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$here/RVAFFI.xcframework"

targets=(aarch64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios)
for target in "${targets[@]}"; do
    rustup target add "$target" >/dev/null 2>&1 || true
    cargo build --manifest-path "$root/Cargo.toml" -p rva-ffi --release --target "$target"
done

tmp="$(mktemp -d)"
lipo -create \
    "$root/target/aarch64-apple-ios-sim/release/librva_ffi.a" \
    "$root/target/x86_64-apple-ios/release/librva_ffi.a" \
    -output "$tmp/librva_ffi_sim.a"

rm -rf "$out"
xcodebuild -create-xcframework \
    -library "$root/target/aarch64-apple-darwin/release/librva_ffi.a" -headers "$root/ffi/include" \
    -library "$root/target/aarch64-apple-ios/release/librva_ffi.a" -headers "$root/ffi/include" \
    -library "$tmp/librva_ffi_sim.a" -headers "$root/ffi/include" \
    -output "$out"

echo "XCFramework written to $out"
