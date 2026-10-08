#!/usr/bin/env bash
# Build the Rust core for Apple platforms and package it as RVAFFI.xcframework.
#
#   ./scripts/build-xcframework.sh
#
# The result (target/RVAFFI.xcframework) is attached to core releases and consumed
# by the Swift package (airovo/rva-swift) and the React Native iOS adapter.
#
# Requires: rustup + Xcode.
#
# The xcframework wraps a *dynamic* `RVAFFI.framework` (built from the cdylib),
# not a static library. A Rust `staticlib` bundles all of std (~9 MB per arch)
# and is only dead-stripped when the consuming app links, so the archive is
# ~135 MB; the `cdylib` is linked and stripped here, so the whole xcframework is
# ~10 MB. SwiftPM embeds the dynamic framework automatically for app targets.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
out="$root/target/RVAFFI.xcframework"

version="$(grep -m1 -E '^version = ' "$root/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')"
install_name="@rpath/RVAFFI.framework/RVAFFI"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Build the cdylib for a target and echo the resulting .dylib path.
build_dylib() {
    local target="$1"
    rustup target add "$target" >/dev/null 2>&1 || true
    RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-install_name -C link-arg=$install_name" \
        cargo build --manifest-path "$root/Cargo.toml" -p rva-ffi --release --target "$target" \
        --config 'profile.release.strip="symbols"' >/dev/null
    echo "$root/target/$target/release/librva_ffi.dylib"
}

# make_framework <dylib> <dest.framework> <CFBundleSupportedPlatforms> <min-os>
make_framework() {
    local dylib="$1" fw="$2" platform="$3" minos="$4"
    rm -rf "$fw"
    mkdir -p "$fw/Headers" "$fw/Modules"
    cp "$dylib" "$fw/RVAFFI"
    cp "$root/ffi/include/rva.h" "$fw/Headers/rva.h"
    cat > "$fw/Modules/module.modulemap" <<'MAP'
framework module RVAFFI {
    header "rva.h"
    export *
}
MAP
    cat > "$fw/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key><string>en</string>
	<key>CFBundleExecutable</key><string>RVAFFI</string>
	<key>CFBundleIdentifier</key><string>tech.airovo.rva.ffi</string>
	<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
	<key>CFBundleName</key><string>RVAFFI</string>
	<key>CFBundlePackageType</key><string>FMWK</string>
	<key>CFBundleShortVersionString</key><string>${version}</string>
	<key>CFBundleVersion</key><string>${version}</string>
	<key>MinimumOSVersion</key><string>${minos}</string>
	<key>CFBundleSupportedPlatforms</key><array><string>${platform}</string></array>
</dict>
</plist>
PLIST
}

echo "Building RVAFFI.xcframework (dynamic frameworks)..."

# macOS (Apple Silicon)
make_framework "$(build_dylib aarch64-apple-darwin)" "$tmp/macos/RVAFFI.framework" "MacOSX" "13.0"

# iOS device
make_framework "$(build_dylib aarch64-apple-ios)" "$tmp/ios/RVAFFI.framework" "iPhoneOS" "15.0"

# iOS simulator (arm64 + x86_64 fat)
sim_arm="$(build_dylib aarch64-apple-ios-sim)"
sim_x86="$(build_dylib x86_64-apple-ios)"
lipo -create "$sim_arm" "$sim_x86" -output "$tmp/librva_ffi_sim.dylib"
make_framework "$tmp/librva_ffi_sim.dylib" "$tmp/sim/RVAFFI.framework" "iPhoneSimulator" "15.0"

rm -rf "$out"
xcodebuild -create-xcframework \
    -framework "$tmp/macos/RVAFFI.framework" \
    -framework "$tmp/ios/RVAFFI.framework" \
    -framework "$tmp/sim/RVAFFI.framework" \
    -output "$out"

echo "XCFramework written to $out"
