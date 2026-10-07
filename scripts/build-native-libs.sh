#!/usr/bin/env bash
# Build every native FFI library for RVA from the *current* rva-core, so the
# Swift / Kotlin / Flutter / React Native adapters never link a stale resolver.
#
#   scripts/build-native-libs.sh [--skip-apple] [--skip-android] [--skip-kotlin-aar]
#
# Outputs:
#   Apple   : adapters/swift/RVAFFI.xcframework
#             adapters/react-native/ios/RVAFFI.xcframework        (copy)
#   Android : adapters/kotlin/src/main/jniLibs/<abi>/librva_ffi.so  (JNI)
#             adapters/flutter/native/android/<abi>/librva_ffi.so  (JNI)
#   macOS   : adapters/flutter/native/macos/librva_ffi.dylib
#   Kotlin  : adapters/kotlin/build/outputs/aar/rva-kotlin-release.aar
#
# Requirements:
#   Apple   : macOS + Xcode (xcodebuild, lipo) + rustup Apple targets
#   Android : Android NDK (set ANDROID_NDK_HOME, or an SDK with ndk/<ver>) +
#             rustup Android targets (aarch64-linux-android, x86_64-linux-android)
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
cd "$root"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

DO_APPLE=1 DO_ANDROID=1 DO_AAR=1
for arg in "$@"; do
  case "$arg" in
    --skip-apple) DO_APPLE=0 ;;
    --skip-android) DO_ANDROID=0; DO_AAR=0 ;;
    --skip-kotlin-aar) DO_AAR=0 ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) die "unknown option: $arg" ;;
  esac
done

# --------------------------------------------------------------- Apple -------
if [ "$DO_APPLE" = 1 ]; then
  [ "$(uname -s)" = "Darwin" ] || die "Apple libraries require macOS. Use --skip-apple."

  log "Apple: building RVAFFI.xcframework (macOS + iOS device + simulator)"
  bash adapters/swift/build-xcframework.sh

  log "Apple: copying xcframework into react-native/ios"
  mkdir -p adapters/react-native/ios
  rm -rf adapters/react-native/ios/RVAFFI.xcframework
  cp -R adapters/swift/RVAFFI.xcframework adapters/react-native/ios/RVAFFI.xcframework

  # Flutter (macOS) loads a dylib, not the static xcframework slice.
  case "$(uname -m)" in
    arm64) host_target="aarch64-apple-darwin" ;;
    x86_64) host_target="x86_64-apple-darwin" ;;
    *) die "unknown macOS arch: $(uname -m)" ;;
  esac
  log "Apple: building librva_ffi.dylib for Flutter ($host_target)"
  cargo build --release --target "$host_target" -p rva-ffi
  mkdir -p adapters/flutter/native/macos
  cp "target/$host_target/release/librva_ffi.dylib" adapters/flutter/native/macos/librva_ffi.dylib
fi

# -------------------------------------------------------------- Android ------
if [ "$DO_ANDROID" = 1 ]; then
  ndk="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
  if [ -z "$ndk" ]; then
    sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
    ndk="$(ls -d "$sdk"/ndk/* 2>/dev/null | sort -V | tail -1 || true)"
  fi
  [ -n "$ndk" ] && [ -d "$ndk" ] || die "Android NDK not found; set ANDROID_NDK_HOME."

  case "$(uname -s)" in
    Darwin) host_tag="darwin-x86_64" ;;
    Linux) host_tag="linux-x86_64" ;;
    *) die "unsupported host for the Android NDK: $(uname -s)" ;;
  esac
  bin="$ndk/toolchains/llvm/prebuilt/$host_tag/bin"
  [ -d "$bin" ] || die "NDK toolchain not found at $bin"
  api="${ANDROID_API:-24}"

  build_android() {
    local target="$1" abi="$2" clang="$3"
    local cc="$bin/$clang" upper
    upper="$(printf '%s' "$target" | tr '[:lower:]-' '[:upper:]_')"
    [ -x "$cc" ] || die "missing NDK clang: $cc"

    log "Android: $abi ($target)"
    rustup target add "$target" >/dev/null 2>&1 || true
    env \
      "CC_${target}=$cc" \
      "CC_${target//-/_}=$cc" \
      "AR_${target//-/_}=$bin/llvm-ar" \
      "RANLIB_${target//-/_}=$bin/llvm-ranlib" \
      "CARGO_TARGET_${upper}_LINKER=$cc" \
      cargo build --release --target "$target" -p rva-ffi --features jni

    local so="target/$target/release/librva_ffi.so"
    mkdir -p "adapters/kotlin/src/main/jniLibs/$abi" "adapters/flutter/native/android/$abi"
    cp "$so" "adapters/kotlin/src/main/jniLibs/$abi/librva_ffi.so"
    cp "$so" "adapters/flutter/native/android/$abi/librva_ffi.so"
  }

  build_android aarch64-linux-android arm64-v8a "aarch64-linux-android${api}-clang"
  build_android x86_64-linux-android  x86_64      "x86_64-linux-android${api}-clang"
fi

# ----------------------------------------------------------- Kotlin AAR ------
if [ "$DO_AAR" = 1 ]; then
  if [ -x adapters/kotlin/gradlew ]; then
    log "Kotlin: assembling rva-kotlin-release.aar (JNI + jniLibs)"
    (cd adapters/kotlin && ./gradlew :assembleRelease --offline)
    log "Kotlin: -> adapters/kotlin/build/outputs/aar/rva-kotlin-release.aar"
  else
    log "Kotlin: gradlew not found; skipped the AAR"
  fi
fi

log "done."
