# RVA Demo Assets

This package is a manually prepared proof-of-concept asset set for implementing the RVA (Responsive Visual Asset) idea.

## What is included

- Independent transparent raster elements: subject, product and CTA visual
- Landscape, portrait and square backgrounds
- SVG logo and decorative vectors
- Subject and product-shadow masks
- `scene.json` describing resources, semantic roles, focal regions and three composition topologies
- A minimal draft JSON Schema
- Six flattened reference compositions rendered from the same logical asset set
- A concept reference sheet

## Important

This is **not** the final RVA specification. It is an implementation fixture for the first parser / solver / renderer prototype.

No font files are bundled. Text in `scene.json` uses `system-ui`; production RVA should define its eventual font resource/fallback policy separately.

## Suggested first implementation

1. Parse `05_examples/scene.json`.
2. Select topology from viewport aspect ratio.
3. Resolve normalized element positions/sizes into pixels.
4. Load resources by resource ID.
5. Draw background with `cover` behavior.
6. Composite raster/vector resources by scene order.
7. Render semantic text using the declared size bounds.
8. Resize the host continuously and verify topology switching.
9. Add hard constraints next, especially the subject face focal region.
10. Replace the simple topology rules with the actual RVA constraint solver.

## Test sizes

- 1920×500
- 1280×720
- 768×1024
- 1080×1080
- 430×932
- 2560×800

The same logical elements are used across all six compositions.
