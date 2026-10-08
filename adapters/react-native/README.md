# @airovo/rva-react-native

[![npm](https://img.shields.io/npm/v/@airovo/rva-react-native?color=3b82f6)](https://www.npmjs.com/package/@airovo/rva-react-native)
[![license](https://img.shields.io/npm/l/@airovo/rva-react-native?color=22c55e)](./LICENSE)

React Native adapter for RVA. Uses a native module over the shared RVA C ABI when
linked, and falls back to a WebView running
[`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) otherwise.

## Install

```bash
npm i @airovo/rva-react-native
```

Peer dependencies: `react >= 18`, `react-native >= 0.70`, `react-native-webview >= 13`.

## Quick start

```tsx
import { RVAImage } from "@airovo/rva-react-native";

<RVAImage src="https://cdn.example.com/hero.rva" alt="Hero" />
```

## Props

| Prop | Type | Default | Description |
| --- | --- | --- | --- |
| `src` | `string` | — (required) | Asset URL or path. |
| `alt` | `string` | `""` | Accessible label. |
| `baseUrl` | `string` | `https://unpkg.com/@airovo/rva-web` | WebView fallback base URL for `rva-image.js`. |

Plus all `ViewProps`; a forwarded `ref` is the underlying `View`.

## How it renders

- **Native path** (when `NativeModules.RNRva` is present): measures via `onLayout`,
  calls `RNRva.renderPng(src, width, height)`, and displays the resulting base64 PNG.
- **WebView path**: injects `rvaHtml(...)` and loads `@airovo/rva-web` from `baseUrl`.

`hasNativeCore()` reports which path is active.

```tsx
import { hasNativeCore } from "@airovo/rva-react-native";
console.log(hasNativeCore());
```

## Native libraries

Prebuilt binaries are **not** bundled. Build them from the core repository and
vendor into `ios/` / `android/`:

```bash
./scripts/build-native-libs.sh
```

On iOS the core ships as a **dynamic** `RVAFFI.xcframework` (~11 MB) and CocoaPods
embeds it into the app automatically (see `RNRva.podspec`). Without a linked native
module the WebView fallback is used automatically. The
native module exposes `RNRva.resolve(uri, width, height)` and
`RNRva.renderPng(uri, width, height)`.

## Source support

Native: `http(s)://`, `file://`, and filesystem paths. WebView: whatever the
platform permits over the network.

## Related

- [`@airovo/rva-web`](https://www.npmjs.com/package/@airovo/rva-web) — the WebView fallback
- [`@airovo/rva-react`](https://www.npmjs.com/package/@airovo/rva-react) for the web

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by [Airovo Technologies](https://airovo.tech).
[Source](https://github.com/airovo/rva) · [Issues](https://github.com/airovo/rva/issues)
