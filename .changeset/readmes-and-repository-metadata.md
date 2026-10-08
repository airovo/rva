---
"@airovo/rva-types": patch
"@airovo/rva-web": patch
"@airovo/rva-node": patch
"@airovo/rva-react": patch
"@airovo/rva-vue": patch
"@airovo/rva-svelte": patch
"@airovo/rva-react-native": patch
---

Expand package READMEs with full API references, examples, and source-resolution
docs, and add `repository` / `bugs` metadata so each npm page links to GitHub.

Also export previously-missing types and helpers: `Background`, `Constraint` and
`Focus` from `@airovo/rva-types`, and `fontStack` from `@airovo/rva-node`. The React
Native native path no longer forwards the WebView-only `baseUrl` prop to `<View>`.
