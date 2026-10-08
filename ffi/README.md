# rva-ffi

[![crates.io](https://img.shields.io/crates/v/rva-ffi?color=3b82f6)](https://crates.io/crates/rva-ffi)
[![docs.rs](https://img.shields.io/docsrs/rva-ffi)](https://docs.rs/rva-ffi)
[![license](https://img.shields.io/crates/l/rva-ffi?color=22c55e)](#license)

Stable C ABI for the [RVA](https://rva.airovo.tech) core, used by the native
adapters (Swift, Kotlin, Flutter, C). Built as `staticlib` + `cdylib`.

## Functions

| Symbol | Purpose |
| --- | --- |
| `rva_open` | Parse + integrity-validate a `.rva` package from bytes; returns a handle. |
| `rva_resolve` | Resolve a viewport to `ResolvedScene` JSON. |
| `rva_render_png` | Rasterize a viewport to PNG bytes. |
| `rva_free_handle` / `rva_free_string` / `rva_free_buffer` | Release returned memory. |
| `rva_last_error` | Last error message for the current thread. |

The `jni` feature adds the JNI entry points used by the Android adapter.

## Build

```bash
cargo build -p rva-ffi --release
# macOS/iOS: scripts/build-native-libs.sh produces the XCFramework / dylib
```

Part of the [RVA](https://rva.airovo.tech) project — [source](https://github.com/airovo/rva).
