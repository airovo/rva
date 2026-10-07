//! CTA Regions: identity, deterministic geometry and overlap diagnostics.

use rva_core::{cta_overlaps, resolve, validate, Asset, Fonts, ResolvedScene};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn asset(scene: &str) -> Asset {
    let dir = std::env::temp_dir().join(format!(
        "rva-cta-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("scene.json"), scene).unwrap();
    Asset::load(&dir).expect("scene loads")
}

fn item(resolved: &ResolvedScene, id: &str) -> (f32, f32, f32, f32) {
    let found = resolved
        .items
        .iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| panic!("item '{id}' exists"));
    (found.x, found.y, found.w, found.h)
}

const SINGLE: &str = r##"{
  "format": "RVA",
  "formatVersion": "0.1-draft",
  "name": "CTA single",
  "designSpace": { "width": 1280, "height": 720 },
  "resources": {},
  "text": {
    "btn": { "value": "Buy", "fontFamily": "system-ui", "weight": 600, "minSize": 12, "preferredSize": 20, "maxSize": 40 },
    "label": { "value": "Label", "fontFamily": "system-ui", "weight": 400, "minSize": 12, "preferredSize": 20, "maxSize": 40 }
  },
  "elements": [
    { "id": "btn", "type": "text", "focalRegions": [] },
    { "id": "label", "type": "text", "focalRegions": [] }
  ],
  "ctaRegions": [
    { "id": "buy", "source": { "type": "element", "elementId": "btn" } },
    { "id": "combo", "source": { "type": "elements", "elementIds": ["btn", "label"] } },
    { "id": "banner", "source": { "type": "region", "x": 0.1, "y": 0.2, "w": 0.3, "h": 0.4 } }
  ],
  "topologies": [
    {
      "id": "base",
      "when": {},
      "layout": {
        "btn": { "x": 0.5, "y": 0.5, "w": 0.25 },
        "label": { "x": 0.1, "y": 0.1, "w": 0.3 }
      }
    }
  ]
}"##;

#[test]
fn element_region_matches_final_item_geometry() {
    let asset = asset(SINGLE);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();

    let region = resolved
        .cta_regions
        .iter()
        .find(|region| region.id == "buy")
        .unwrap();
    assert!(region.visible);

    let (x, y, w, h) = item(&resolved, "btn");
    assert!((region.bounds.x - x).abs() < 0.01);
    assert!((region.bounds.y - y).abs() < 0.01);
    assert!((region.bounds.width - w).abs() < 0.01);
    assert!((region.bounds.height - h).abs() < 0.01);
    // normalized convenience mirrors logical bounds.
    assert!((region.normalized_bounds.x - x / 1000.0).abs() < 1e-4);
    assert!((region.normalized_bounds.width - w / 1000.0).abs() < 1e-4);
}

#[test]
fn element_region_follows_recomposition() {
    let asset = asset(SINGLE);
    let fonts = Fonts::load_system();

    let wide = resolve(&asset, &fonts, 1000, 500).unwrap();
    let buy_wide = wide
        .cta_regions
        .iter()
        .find(|r| r.id == "buy")
        .unwrap()
        .bounds;
    // btn at x 0.5, y 0.5 of the canvas.
    assert!((buy_wide.x - 500.0).abs() < 0.01);
    assert!((buy_wide.y - 250.0).abs() < 0.01);

    let tall = resolve(&asset, &fonts, 400, 800).unwrap();
    let buy_tall = tall
        .cta_regions
        .iter()
        .find(|r| r.id == "buy")
        .unwrap()
        .bounds;
    assert!((buy_tall.x - 200.0).abs() < 0.01);
    assert!((buy_tall.y - 400.0).abs() < 0.01);
    // same stable id across sizes.
    assert_eq!(buy_wide.width, 0.25 * 1000.0);
    assert_eq!(buy_tall.width, 0.25 * 400.0);
}

#[test]
fn multi_element_region_is_deterministic_union() {
    let asset = asset(SINGLE);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();
    let region = resolved
        .cta_regions
        .iter()
        .find(|r| r.id == "combo")
        .unwrap()
        .bounds;

    let (bx, by, bw, bh) = item(&resolved, "btn");
    let (lx, ly, lw, lh) = item(&resolved, "label");
    let min_x = bx.min(lx);
    let min_y = by.min(ly);
    let max_x = (bx + bw).max(lx + lw);
    let max_y = (by + bh).max(ly + lh);

    assert!((region.x - min_x).abs() < 0.01);
    assert!((region.y - min_y).abs() < 0.01);
    assert!((region.width - (max_x - min_x)).abs() < 0.01);
    assert!((region.height - (max_y - min_y)).abs() < 0.01);
}

#[test]
fn manual_region_scales_and_keeps_normalized_input() {
    let asset = asset(SINGLE);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();
    let region = resolved
        .cta_regions
        .iter()
        .find(|r| r.id == "banner")
        .unwrap();

    assert!(region.visible);
    assert!((region.bounds.x - 100.0).abs() < 0.01);
    assert!((region.bounds.y - 100.0).abs() < 0.01);
    assert!((region.bounds.width - 300.0).abs() < 0.01);
    assert!((region.bounds.height - 200.0).abs() < 0.01);
    assert!((region.normalized_bounds.x - 0.1).abs() < 1e-4);
    assert!((region.normalized_bounds.height - 0.4).abs() < 1e-4);
}

#[test]
fn group_region_covers_resolved_descendants() {
    let scene = r##"{
      "format": "RVA",
      "formatVersion": "0.1-draft",
      "designSpace": { "width": 1280, "height": 720 },
      "resources": {},
      "text": {
        "a": { "value": "A", "fontFamily": "system-ui", "weight": 400, "minSize": 12, "preferredSize": 20, "maxSize": 40 },
        "b": { "value": "B", "fontFamily": "system-ui", "weight": 400, "minSize": 12, "preferredSize": 20, "maxSize": 40 }
      },
      "elements": [
        { "id": "card", "type": "group", "focalRegions": [] },
        { "id": "a", "type": "text", "parent": "card", "focalRegions": [] },
        { "id": "b", "type": "text", "parent": "card", "focalRegions": [] }
      ],
      "ctaRegions": [
        { "id": "card-action", "source": { "type": "element", "elementId": "card" } }
      ],
      "topologies": [
        {
          "id": "base",
          "when": {},
          "layout": {
            "card": { "x": 0.05, "y": 0.05, "w": 0.1 },
            "a": { "x": 0.1, "y": 0.1, "w": 0.2 },
            "b": { "x": 0.5, "y": 0.4, "w": 0.2 }
          }
        }
      ]
    }"##;
    let asset = asset(scene);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();
    let region = resolved
        .cta_regions
        .iter()
        .find(|r| r.id == "card-action")
        .unwrap();
    assert!(region.visible);

    let (ax, ay, aw, ah) = item(&resolved, "a");
    let (bx, by, bw, bh) = item(&resolved, "b");
    let min_x = ax.min(bx);
    let min_y = ay.min(by);
    let max_x = (ax + aw).max(bx + bw);
    let max_y = (ay + ah).max(by + bh);
    assert!((region.bounds.x - min_x).abs() < 0.01);
    assert!((region.bounds.y - min_y).abs() < 0.01);
    assert!((region.bounds.width - (max_x - min_x)).abs() < 0.01);
    assert!((region.bounds.height - (max_y - min_y)).abs() < 0.01);
}

#[test]
fn hidden_source_resolves_invisible_with_zero_bounds() {
    let scene = SINGLE.replace(
        r#"{ "id": "btn", "type": "text", "focalRegions": [] }"#,
        r#"{ "id": "btn", "type": "text", "visibility": "hidden", "focalRegions": [] }"#,
    );
    let asset = asset(&scene);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();
    let region = resolved
        .cta_regions
        .iter()
        .find(|r| r.id == "buy")
        .unwrap();
    assert!(!region.visible, "hidden source must not stay actionable");
    assert_eq!(region.bounds.width, 0.0);
    assert_eq!(region.bounds.height, 0.0);
}

#[test]
fn overlap_is_detected_with_viewport_and_ratio() {
    let scene = r##"{
      "format": "RVA",
      "formatVersion": "0.1-draft",
      "resources": {},
      "elements": [],
      "ctaRegions": [
        { "id": "left",  "source": { "type": "region", "x": 0.0,  "y": 0.0, "w": 0.5, "h": 1.0 } },
        { "id": "right", "source": { "type": "region", "x": 0.25, "y": 0.0, "w": 0.5, "h": 1.0 } }
      ],
      "topologies": [ { "id": "base", "when": {}, "layout": {} } ]
    }"##;
    let asset = asset(scene);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();

    let conflicts = cta_overlaps(&resolved);
    assert_eq!(conflicts.len(), 1, "one conflicting pair");
    let conflict = &conflicts[0];
    assert_eq!(conflict.regions, vec!["left".to_string(), "right".to_string()]);
    assert_eq!(conflict.width, 1000);
    assert_eq!(conflict.height, 500);
    // overlap 250x500 of a 500x500 region => ratio 0.5.
    assert!((conflict.overlap_ratio - 0.5).abs() < 1e-3);
}

#[test]
fn adjacent_regions_are_not_conflicts() {
    let scene = r##"{
      "format": "RVA",
      "formatVersion": "0.1-draft",
      "resources": {},
      "elements": [],
      "ctaRegions": [
        { "id": "left",  "source": { "type": "region", "x": 0.0, "y": 0.0, "w": 0.5, "h": 1.0 } },
        { "id": "right", "source": { "type": "region", "x": 0.5, "y": 0.0, "w": 0.5, "h": 1.0 } }
      ],
      "topologies": [ { "id": "base", "when": {}, "layout": {} } ]
    }"##;
    let asset = asset(scene);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();
    assert!(cta_overlaps(&resolved).is_empty());
}

#[test]
fn hit_padding_expands_bounds_and_can_create_overlap() {
    // Two regions 0.1 (100px) apart; 0.15 * shorter side (500) = 75px each side
    // bridges the gap and overlaps.
    let scene = r##"{
      "format": "RVA",
      "formatVersion": "0.1-draft",
      "resources": {},
      "elements": [],
      "ctaRegions": [
        { "id": "a", "padding": 0.15, "source": { "type": "region", "x": 0.0, "y": 0.0, "w": 0.2, "h": 0.2 } },
        { "id": "b", "padding": 0.15, "source": { "type": "region", "x": 0.3, "y": 0.0, "w": 0.2, "h": 0.2 } }
      ],
      "topologies": [ { "id": "base", "when": {}, "layout": {} } ]
    }"##;
    let asset = asset(scene);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();

    let a = resolved.cta_regions.iter().find(|r| r.id == "a").unwrap();
    // base x=0,w=200; pad 75 each side, clamped at 0 -> x=0,w=275; y -> h=175.
    assert!((a.bounds.x - 0.0).abs() < 0.01);
    assert!((a.bounds.width - 275.0).abs() < 0.01);
    assert!((a.bounds.height - 175.0).abs() < 0.01);

    assert_eq!(cta_overlaps(&resolved).len(), 1, "padding should introduce overlap");
}

#[test]
fn negative_padding_is_rejected() {
    let scene = r##"{
      "format": "RVA",
      "formatVersion": "0.1-draft",
      "resources": {},
      "elements": [],
      "ctaRegions": [
        { "id": "bad", "padding": -0.2, "source": { "type": "region", "x": 0.1, "y": 0.1, "w": 0.2, "h": 0.2 } }
      ],
      "topologies": [ { "id": "base", "when": {}, "layout": {} } ]
    }"##;
    let asset = asset(scene);
    assert!(validate(&asset).iter().any(|i| i.contains("padding")));
}

#[test]
fn packaged_asset_resolves_cta_identically() {
    let asset = asset(SINGLE);
    let fonts = Fonts::load_system();
    let direct = resolve(&asset, &fonts, 1000, 500).unwrap();
    let packaged = Asset::from_bytes(asset.pack().unwrap()).unwrap();
    let packed = resolve(&packaged, &fonts, 1000, 500).unwrap();
    assert_eq!(
        serde_json::to_string(&direct).unwrap(),
        serde_json::to_string(&packed).unwrap(),
        "packaging must preserve CTA region geometry byte-for-byte",
    );
}

#[test]
fn valid_regions_pass_validation_and_keep_ids() {
    let asset = asset(SINGLE);
    assert!(validate(&asset).is_empty(), "scene must validate clean");

    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1000, 500).unwrap();
    let ids: Vec<&str> = resolved
        .cta_regions
        .iter()
        .map(|region| region.id.as_str())
        .collect();
    assert_eq!(ids, vec!["buy", "combo", "banner"]);
}

#[test]
fn validation_rejects_bad_sources_and_duplicate_ids() {
    let scene = r##"{
      "format": "RVA",
      "formatVersion": "0.1-draft",
      "resources": {},
      "elements": [ { "id": "a", "type": "text", "focalRegions": [] } ],
      "text": { "a": { "value": "A", "fontFamily": "system-ui", "weight": 400, "minSize": 12, "preferredSize": 20, "maxSize": 40 } },
      "ctaRegions": [
        { "id": "dup", "source": { "type": "element", "elementId": "missing" } },
        { "id": "dup", "source": { "type": "region", "x": 0.1, "y": 0.1, "w": 0.0, "h": 0.2 } },
        { "id": "", "source": { "type": "elements", "elementIds": [] } }
      ],
      "topologies": [ { "id": "base", "when": {}, "layout": { "a": { "x": 0.1, "y": 0.1, "w": 0.2 } } } ]
    }"##;
    let asset = asset(scene);
    let issues = validate(&asset);
    assert!(issues.iter().any(|i| i.contains("duplicate cta region id")));
    assert!(issues.iter().any(|i| i.contains("missing element 'missing'")));
    assert!(issues.iter().any(|i| i.contains("positive size")));
    assert!(issues.iter().any(|i| i.contains("must not be empty")));
}
