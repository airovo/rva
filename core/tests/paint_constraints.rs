//! Paint backgrounds, typography styling and authored relationships.

use rva_core::model::Paint;
use rva_core::resolve::ItemKind;
use rva_core::{resolve, Asset, Fonts};

fn scratch_scene(scene: &str) -> Asset {
    let dir = std::env::temp_dir().join(format!(
        "rva-paint-constraints-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("scene.json"), scene).unwrap();
    Asset::load(&dir).expect("scene loads")
}

const SCENE: &str = r##"{
  "format": "RVA",
  "formatVersion": "0.1-draft",
  "name": "Paint + Constraints",
  "designSpace": { "width": 1280, "height": 720 },
  "resources": {},
  "text": {
    "headline": {
      "value": "Hello there",
      "fontFamily": "system-ui",
      "weight": 700,
      "minSize": 20,
      "preferredSize": 48,
      "maxSize": 72,
      "maxLines": 2,
      "color": "#FF0000",
      "letterSpacing": 2.0,
      "wordSpacing": 4.0,
      "align": "center"
    },
    "sub": {
      "value": "World",
      "fontFamily": "system-ui",
      "weight": 400,
      "minSize": 14,
      "preferredSize": 24,
      "maxSize": 32,
      "maxLines": 2,
      "background": "#00FF00"
    }
  },
  "elements": [
    { "id": "headline", "type": "text", "role": "headline", "opacity": 0.5, "focalRegions": [] },
    { "id": "sub", "type": "text", "role": "subheadline", "focalRegions": [] }
  ],
  "topologies": [
    {
      "id": "base",
      "when": {},
      "background": { "type": "color", "color": "#0A0F1C" },
      "layout": {
        "headline": { "x": 0.05, "y": 0.2, "w": 0.5 },
        "sub": { "x": 0.05, "y": 0.5, "w": 0.5 }
      }
    }
  ],
  "constraints": [
    { "id": "c1", "type": "align", "subject": "headline", "target": "canvas", "edge": "center-x", "targetEdge": "center-x", "hard": true },
    { "id": "c2", "type": "gap", "subject": "sub", "target": "headline", "edge": "top", "targetEdge": "bottom", "value": 12.0, "hard": true }
  ]
}"##;

#[test]
fn painted_background_and_styled_text_resolve() {
    let asset = scratch_scene(SCENE);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1280, 720).unwrap();

    // Solid paint background.
    let background = resolved.background.expect("background present");
    match background.kind {
        ItemKind::Paint { paint } => {
            assert_eq!(paint, Paint::Color { color: "#0A0F1C".to_string() });
        }
        other => panic!("expected paint background, got {other:?}"),
    }

    let headline = resolved
        .items
        .iter()
        .find(|item| item.id == "headline")
        .expect("headline item");
    let sub = resolved
        .items
        .iter()
        .find(|item| item.id == "sub")
        .expect("sub item");

    // Element opacity is carried through.
    assert!((headline.opacity - 0.5).abs() < 0.001);

    // Typography styling is carried through.
    match &headline.kind {
        ItemKind::Text {
            color,
            letter_spacing,
            word_spacing,
            align,
            ..
        } => {
            assert_eq!(color.as_deref(), Some("#FF0000"));
            assert!((*letter_spacing - 2.0).abs() < 0.001);
            assert!((*word_spacing - 4.0).abs() < 0.001);
            assert_eq!(align, "center");
        }
        other => panic!("expected text item, got {other:?}"),
    }
    match &sub.kind {
        ItemKind::Text { background, .. } => {
            assert_eq!(background.as_deref(), Some("#00FF00"));
        }
        other => panic!("expected text item, got {other:?}"),
    }

    // align (center-x on canvas): headline is horizontally centered.
    assert!((headline.x + headline.w / 2.0 - 640.0).abs() < 1.0);

    // gap: sub.top = headline.bottom + 12.
    let gap = sub.y - (headline.y + headline.h);
    assert!((gap - 12.0).abs() < 1.0, "gap was {gap}");
}
