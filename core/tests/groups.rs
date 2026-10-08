//! Group containers: offset, opacity and visibility inherited by children.

use rva_core::{resolve, Asset, Fonts};

fn asset(scene: &str) -> Asset {
    let dir = std::env::temp_dir().join(format!(
        "rva-groups-{}-{}",
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
  "name": "Groups",
  "designSpace": { "width": 1280, "height": 720 },
  "resources": {},
  "text": {
    "title": {
      "value": "Hi",
      "fontFamily": "system-ui",
      "weight": 700,
      "minSize": 20,
      "preferredSize": 40,
      "maxSize": 60,
      "maxLines": 2
    }
  },
  "elements": [
    { "id": "hero", "type": "group", "opacity": 0.5, "focalRegions": [] },
    { "id": "title", "type": "text", "role": "headline", "parent": "hero", "focalRegions": [] }
  ],
  "topologies": [
    {
      "id": "base",
      "when": {},
      "layout": {
        "hero": { "x": 0.1, "y": 0.1, "w": 0.1 },
        "title": { "x": 0.1, "y": 0.1, "w": 0.4 }
      }
    }
  ]
}"##;

#[test]
fn group_offsets_and_lightens_children_without_drawing_itself() {
    let asset = asset(SCENE);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1280, 720).unwrap();

    assert!(
        resolved.items.iter().all(|item| item.id != "hero"),
        "the group container must not be drawn"
    );
    let title = resolved
        .items
        .iter()
        .find(|item| item.id == "title")
        .expect("child item");

    // child (0.1,0.1) + group offset (0.1,0.1) = (0.2,0.2) normalized.
    assert!((title.x - 0.2 * 1280.0).abs() < 0.5, "x was {}", title.x);
    assert!((title.y - 0.2 * 720.0).abs() < 0.5, "y was {}", title.y);
    // opacity inherited from the group.
    assert!(
        (title.opacity - 0.5).abs() < 0.001,
        "opacity was {}",
        title.opacity
    );
}

#[test]
fn hidden_group_hides_its_children() {
    let hidden = SCENE.replace(
        r#"{ "id": "hero", "type": "group", "opacity": 0.5, "focalRegions": [] }"#,
        r#"{ "id": "hero", "type": "group", "visibility": "hidden", "focalRegions": [] }"#,
    );
    let asset = asset(&hidden);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1280, 720).unwrap();
    assert!(
        resolved.items.iter().all(|item| item.id != "title"),
        "children of a hidden group must not be drawn"
    );
}
