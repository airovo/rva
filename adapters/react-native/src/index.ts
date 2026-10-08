// @airovo/rva-react-native — React Native adapter for RVA.
//
// Architecture (TS API -> native module -> Kotlin/Swift -> Rust core):
//
//   React Native
//        │
//   TypeScript API           this file (RVAImage, RNRva binding)
//        │
//   Native module            RNRva (TurboModule/NativeModule)
//      ↙        ↘
//   Kotlin      Swift        android/  |  ios/  (thin marshalling)
//      ↘        ↙
//   Rust core                librva_ffi.so (JNI)  |  RVAFFI.xcframework (C ABI)
//
// The native modules call the shared Rust core, which resolves the composition
// and rasterizes a PNG for the requested size. The TS component measures its
// container, asks the native module for a PNG, and draws it with <Image>.
//
// A WebView fallback uses the browser <rva-image> element when the native module
// is not linked (e.g. Expo Go), so the component still works everywhere.

import {
  createElement,
  forwardRef,
  useCallback,
  useRef,
  useState,
  type ReactElement,
} from "react";
import {
  Image,
  NativeModules,
  StyleSheet,
  View,
  type LayoutChangeEvent,
  type ViewProps,
} from "react-native";
import { WebView } from "react-native-webview";

/** Contract implemented by the native modules (see android/ and ios/). */
export interface RNRvaSpec {
  /** ResolvedScene JSON for a viewport. */
  resolve(uri: string, width: number, height: number): Promise<string>;
  /** Base64 PNG for a viewport. */
  renderPng(uri: string, width: number, height: number): Promise<string>;
}

const RNRva = (NativeModules as Record<string, unknown>).RNRva as RNRvaSpec | undefined;

/** True when the native core module is linked. */
export function hasNativeCore(): boolean {
  return RNRva !== undefined;
}

export interface RVAImageProps extends ViewProps {
  /** `.rva` asset: a bundled resource URI (require(...)) or an http(s)/file URL. */
  src: string;
  /** Accessible description. */
  alt?: string;
  /** Base URL for the WebView fallback's `rva-image.js` (default: unpkg). */
  baseUrl?: string;
}

const DEFAULT_BASE_URL = "https://unpkg.com/@airovo/rva-web";

/** Build the WebView fallback document hosting <rva-image>. */
export function rvaHtml(src: string, alt: string, baseUrl: string): string {
  const safeAlt = alt.replace(/"/g, "&quot;");
  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" />
    <style>
      html, body { margin: 0; height: 100%; background: transparent; }
      rva-image { display: block; width: 100%; height: 100%; }
    </style>
  </head>
  <body>
    <rva-image src="${src}" alt="${safeAlt}"></rva-image>
    <script type="module">import "${baseUrl.replace(/\/$/, "")}/rva-image.js";</script>
  </body>
</html>`;
}

const NativeRVAImage = forwardRef<View, RVAImageProps>(function NativeRVAImage(
  { src, alt = "", style, ...rest },
  ref
): ReactElement {
  const [dataUri, setDataUri] = useState<string | null>(null);
  const size = useRef({ width: 0, height: 0 });

  const onLayout = useCallback(
    (event: LayoutChangeEvent) => {
      const { width, height } = event.nativeEvent.layout;
      if (width < 1 || height < 1) return;
      const w = Math.round(width);
      const h = Math.round(height);
      if (w === size.current.width && h === size.current.height) return;
      size.current = { width: w, height: h };
      RNRva!.renderPng(src, w, h)
        .then((base64) => setDataUri(`data:image/png;base64,${base64}`))
        .catch((error) => console.warn("[rva] renderPng failed", error));
    },
    [src]
  );

  return createElement(
    View,
    { ref, onLayout, style: [styles.container, style], ...rest },
    dataUri
      ? createElement(Image, {
          source: { uri: dataUri },
          style: styles.image,
          resizeMode: "cover",
          accessibilityLabel: alt,
        })
      : null
  );
});

const WebViewRVAImage = forwardRef<View, RVAImageProps>(function WebViewRVAImage(
  { src, alt = "", baseUrl = DEFAULT_BASE_URL, style, ...rest },
  ref
): ReactElement {
  return createElement(
    View,
    { ref, style: [styles.container, style], ...rest },
    createElement(WebView, {
      originWhitelist: ["*"],
      source: { html: rvaHtml(src, alt, baseUrl) },
      style: styles.image,
      allowsInlineMediaPlayback: true,
      mediaPlaybackRequiresUserAction: false,
      androidLayerType: "hardware",
    })
  );
});

export const RVAImage = forwardRef<View, RVAImageProps>(function RVAImage(props, ref) {
  return RNRva
    ? createElement(NativeRVAImage, { ...props, ref })
    : createElement(WebViewRVAImage, { ...props, ref });
});

const styles = StyleSheet.create({
  container: { overflow: "hidden", backgroundColor: "#ffffff" },
  image: { flex: 1, width: "100%", backgroundColor: "transparent" },
});
