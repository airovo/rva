# rva-android

[![Maven Central](https://img.shields.io/maven-central/v/tech.airovo/rva-android?color=3b82f6)](https://central.sonatype.com/artifact/tech.airovo/rva-android)
[![license](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

The **Android** adapter for RVA (Responsive Visual Asset): JNI bindings to the
shared Rust core, plus an Android `Canvas` renderer. The core decides *what* to
draw; Android decides *how*.

## Install

```kotlin
// build.gradle.kts
dependencies {
    implementation("tech.airovo:rva-android:0.1.1")
}
```

```groovy
// build.gradle
dependencies {
    implementation 'tech.airovo:rva-android:0.1.1'
}
```

`minSdk` 24. The prebuilt `librva_ffi.so` (arm64-v8a, x86_64) is bundled.

## Usage

```kotlin
import dev.rva.RVAImage
import dev.rva.RVARenderer

// bytes, or a filesystem path / file:// / http(s):// URL
val image = RVAImage.openSource("https://cdn.example.com/hero.rva")
println(image.describe())

// Deterministic ResolvedScene JSON for a viewport
val sceneJson = image.resolveJSON(1280, 720)

// Draw with the Android Canvas
RVARenderer.draw(canvas, image, width = 1280f, height = 720f)

image.close()
```

| Member | Description |
| --- | --- |
| `RVAImage.open(bytes)` | Parse + validate a `.rva` from bytes. |
| `RVAImage.openSource(src)` | Load from a path, `file://` or `http(s)://` URL. |
| `describe()` | Human-readable asset summary. |
| `resolveJSON(w, h)` | Deterministic `ResolvedScene` JSON. |
| `renderPng(w, h)` | Optional PNG from the Rust core. |
| `resource(id)` / `hasResource(id)` / `relativeFor(id)` | Resource helpers. |
| `RVARenderer.draw(canvas, image, w, h)` | Draw the resolved scene with Android `Canvas`. |
| `close()` | Release the native handle. |

## The core

Binds the same C ABI (`rva_open` / `rva_resolve` / `rva_render_png`) as the Swift,
Flutter and React Native adapters — see
[`airovo/rva`](https://github.com/airovo/rva).

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by
[Airovo Technologies](https://airovo.tech).
