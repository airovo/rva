# rva_flutter

[![pub](https://img.shields.io/pub/v/rva_flutter?color=3b82f6)](https://pub.dev/packages/rva_flutter)
[![license](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

The **Flutter** adapter for RVA (Responsive Visual Asset). It binds the shared Rust
core over `dart:ffi` and renders the resolved scene with a Flutter `CustomPainter`.

The core decides *what* to draw (deterministic geometry for a viewport); Flutter
decides *how*.

## Platforms

Android, iOS and macOS. The core ships as a prebuilt binary via the plugin's
platform folders (`librva_ffi.so` for Android, the dynamic `RVAFFI.framework` for
Apple) — no toolchain or build step is required in your app.

## Install

```bash
flutter pub add rva_flutter
```

## Usage

```dart
import 'package:flutter/services.dart';
import 'package:rva_flutter/rva_flutter.dart';

// Render a bundled asset.
final bytes = (await rootBundle.load('assets/hero.rva')).buffer.asUint8List();

RVAImageView(bytes: bytes);
```

Lower-level Dart API (`dart:ffi` over the C ABI):

```dart
final image = await RVAImage.openSource('/path/hero.rva'); // path | file:// | http(s)://
print(image.describe());

final scene = image.resolveJSON(1280, 720);   // ResolvedScene JSON
final bytes = image.resource('bgLandscape');   // raw resource bytes
image.close();
```

| Member | Description |
| --- | --- |
| `RVAImage.open(bytes)` | Parse + validate a `.rva` from bytes. |
| `RVAImage.openSource(src)` | Load from a path, `file://` or `http(s)://` URL. |
| `describe()` | Human-readable asset summary. |
| `resolveJSON(w, h)` | Deterministic `ResolvedScene` JSON for a viewport. |
| `resource(id)` / `hasResource(id)` / `relativeFor(id)` | Resource helpers. |
| `close()` | Release the native handle. |

## The core

The native core is provided by the [`airovo/rva`](https://github.com/airovo/rva)
repository and is the same C ABI used by the Swift, Kotlin and React Native
adapters. For local development against a freshly built core, set `RVA_FFI_LIB` to
the library path.

## License

MIT — see [LICENSE](./LICENSE).

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by
[Airovo Technologies](https://airovo.tech).
