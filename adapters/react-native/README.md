# @airovo/rva-react-native

React Native adapter. Uses the native core module when linked, and a WebView
fallback over `@airovo/rva-web` otherwise.

```bash
npm i @airovo/rva-react-native
```

```tsx
import { RVAImage } from "@airovo/rva-react-native";

<RVAImage src="http://host/hero.rva" baseUrl="http://host" alt="Hero" />
```

Peer dependencies: `react-native`, `react-native-webview`.

## Native libraries

The optional native module (iOS/Android) links the shared RVA core. The prebuilt
binaries are **not** bundled in this package (they are large). Build them from the
core repository and vendor them into `ios/` / `android/`:

```bash
./scripts/build-native-libs.sh
```

Without a linked native module the component automatically uses the WebView
fallback over `@airovo/rva-web`.

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, created by Airovo Technologies.
