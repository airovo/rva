//! Fallback diagnostics must name the topology and the offending element so a
//! validation failure is actionable.

use rva_core::{resolve, Asset, Fonts};

fn asset(scene: &str) -> Asset {
    let dir = std::env::temp_dir().join(format!(
        "rva-diag-{}-{}",
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

const OVERFLOW: &str = r##"{
  "format": "RVA",
  "formatVersion": "0.1-draft",
  "designSpace": { "width": 1280, "height": 720 },
  "resources": {},
  "text": {
    "headline": {
      "value": "A very long headline that cannot possibly fit on one line",
      "fontFamily": "system-ui",
      "weight": 800,
      "minSize": 80,
      "preferredSize": 80,
      "maxSize": 80,
      "maxLines": 1
    }
  },
  "elements": [ { "id": "headline", "type": "text", "role": "headline", "focalRegions": [] } ],
  "topologies": [ { "id": "base", "when": {}, "layout": { "headline": { "x": 0.1, "y": 0.1, "w": 0.3 } } } ]
}"##;

#[test]
fn fallback_names_topology_and_element() {
    let asset = asset(OVERFLOW);
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 320, 600).unwrap();

    assert_eq!(resolved.topology, "fallback");
    let joined = resolved.diagnostics.join("\n");
    assert!(
        joined.contains("no topology is viable"),
        "expected the fallback note, got: {joined}"
    );
    assert!(
        joined.contains("topology 'base'"),
        "expected the topology named, got: {joined}"
    );
    assert!(
        joined.contains("text 'headline'"),
        "expected the element named, got: {joined}"
    );
    assert!(
        joined.contains("Max lines"),
        "expected an actionable hint, got: {joined}"
    );
}
