//! Per-topology visibility: an element can be hidden in one topology and shown
//! in another.

use rva_core::{resolve, Asset, Fonts};

fn asset(scene: &str) -> Asset {
    let dir = std::env::temp_dir().join(format!(
        "rva-visibility-{}-{}",
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
  "name": "Per-topology visibility",
  "designSpace": { "width": 1280, "height": 720 },
  "resources": {},
  "text": {
    "deco": {
      "value": "Cloud",
      "fontFamily": "system-ui",
      "weight": 400,
      "minSize": 16,
      "preferredSize": 32,
      "maxSize": 48,
      "maxLines": 1
    }
  },
  "elements": [
    { "id": "deco", "type": "text", "role": "headline", "focalRegions": [] }
  ],
  "topologies": [
    {
      "id": "wide",
      "when": { "minAspectRatio": 1.3 },
      "layout": { "deco": { "x": 0.1, "y": 0.1, "w": 0.4, "hidden": true } }
    },
    {
      "id": "tall",
      "when": { "maxAspectRatio": 0.8 },
      "layout": { "deco": { "x": 0.1, "y": 0.1, "w": 0.4 } }
    }
  ]
}"##;

#[test]
fn element_hidden_in_one_topology_only() {
    let asset = asset(SCENE);
    let fonts = Fonts::load_system();

    let wide = resolve(&asset, &fonts, 1920, 800).unwrap();
    assert_eq!(wide.topology, "wide");
    assert!(
        wide.items.iter().all(|item| item.id != "deco"),
        "element hidden in the 'wide' topology must not be drawn there"
    );

    let tall = resolve(&asset, &fonts, 600, 900).unwrap();
    assert_eq!(tall.topology, "tall");
    assert!(
        tall.items.iter().any(|item| item.id == "deco"),
        "element must be drawn in the 'tall' topology"
    );
}
