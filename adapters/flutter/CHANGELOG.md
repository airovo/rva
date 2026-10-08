## 0.1.2

- Maintenance release published through pub.dev automated publishing (CI).

## 0.1.1

- Ship the core as a proper Flutter FFI plugin: Android `jniLibs` and an Apple
  dynamic `RVAFFI.framework` are bundled automatically (`ffiPlugin: true`).
- `RVAImage` (`open`, `openSource`, `describe`, `resolveJSON`, `resource`,
  `hasResource`, `relativeFor`, `close`) and the `RVAImageView` widget.

## 0.1.0

- Initial Flutter adapter: `dart:ffi` binding to the shared Rust core with a
  Flutter canvas renderer.
