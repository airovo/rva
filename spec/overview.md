# RVA — Core Model & Primitive API (working draft)

Status: experimental, **not frozen**. This document records the rules the
reference implementation follows today.

## The rule

RVA separates a normative core from renderer convenience:

```
.rva
 ↓  parse
 ↓  validate
 ↓  resolve(width, height, environment)
ResolvedScene
```

Normative resolver rules in profile `rva-resolve/0.4`:
- **Hard:** non-croppable elements fit within the canvas; protected (hard) focal
  regions stay visible; elements drawn on top of a protected region are
  deterministically separated, and if separation is impossible the topology is
  non-viable.
- **Soft:** protected regions covered by elements on top, and low-priority
  clutter around protected regions, are penalized. Below the viability
  threshold, optional elements are hidden lowest-priority-first (degradation).
- **Non-viable:** text that still overflows `maxLines` at `minSize` marks a
  topology non-viable. When no topology is viable, the canonical fallback is used.

`ResolvedScene` is the interop boundary. Everything after it is renderer work:

```
ResolvedScene
   ├── PNG / server            (reference rasterizer; convenience)
   ├── Browser Canvas/SVG/DOM  (web adapter)
   ├── Core Graphics / Core Text (Apple)
   ├── Android Canvas            (Kotlin)
   └── Flutter / Skia            (Flutter)
```

### Determinism invariant

```
same .rva
+ same viewport
+ same spec version
+ same resolver profile
────────────────────────────
= same ResolvedScene
```

The reference resolver profile is `rva-resolve/0.4` (`rva_core::RESOLVER_PROFILE`).
Rendering pixels may differ between platforms; resolution semantics must not.

## Primitive API (normative)

These five operations are the conceptual cross-platform RVA API. Language
bindings differ, semantics do not.

| Primitive | Input | Output | Semantics |
|---|---|---|---|
| `open` | bytes | handle | Parse + integrity-validate a `.rva`. |
| `describe` | handle | string | Human-readable summary (informational). |
| `resolve` | handle, width, height | ResolvedScene JSON | Deterministic absolute geometry. |
| `resource` | handle, reference | bytes | Raw bytes for a declared resource id. |
| `has_resource` | handle, reference | boolean | Whether a declared id exists. |

```text
Rust    asset.resolve(...)
JS      asset.resolve(...)
Swift   asset.resolve(...)
Kotlin  asset.resolve(...)
Dart    asset.resolve(...)
```

All of them produce the **same conceptual ResolvedScene**.

## Convenience API (non-normative)

These exist for adapters/tooling but do **not** define RVA semantics:

- `relative_for(reference)` — filename/MIME helper. Not a fundamental RVA
  operation; may be folded into `resolve` metadata or removed before the public
  contract is frozen.
- `render_png(handle, w, h)` — reference rasterization (resvg/tiny-skia). A
  third-party implementation **must not** need resvg or PNG support to claim
  RVA conformance.

The smaller the normative surface, the better.

## Typography

Cross-platform text is the hardest conformance area. The spec distinguishes:

- **Layout-deterministic typography** — line wrapping, line count, line height,
  first baseline, and therefore the constraint solution. These MUST be identical
  across runtimes.
- **Renderer-dependent rasterization** — glyph outlines, hinting, shaping
  features, subpixel metrics. These MAY differ.

Therefore the resolver does **not** use system fonts. It measures with a fixed,
platform-independent advance model (`core/src/fonts.rs`). Real fonts (browser,
Core Text, Android, Skia) are used only when *drawing* the resolved lines. So a
`ResolvedScene`'s text geometry is stable everywhere, while the drawn glyphs
remain native.

Changing the advance model is a resolver-profile change.

## Conformance

`conformance/run.sh` resolves a fixture across a continuous size matrix through
three independent runtimes and asserts byte-identical `ResolvedScene` JSON:

- native — Rust CLI (`rva inspect --compact`)
- WASM — `wasm-bindgen` core via `@airovo/rva-node`
- C ABI — a C program linking `librva_ffi`

```
same inputs → same bytes  (native == wasm == c-abi)
```

## Security

`.rva` is untrusted binary input. The parser is panic-safe across the C ABI,
bounds every offset/length, caps decompressed sizes, and rejects malformed input
with errors. See `spec/container.md`.
