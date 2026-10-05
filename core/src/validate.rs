use crate::model::Background;
use crate::resources::Asset;

/// Lightweight structural validation for prototype assets. Returns a list of
/// human-readable issues; an empty list means the asset is usable.
pub fn validate(asset: &Asset) -> Vec<String> {
    let scene = &asset.scene;
    let mut issues = Vec::new();

    if scene.format != "RVA" {
        issues.push(format!(
            "format must be \"RVA\", found \"{}\"",
            scene.format
        ));
    }
    if scene.format_version.trim().is_empty() {
        issues.push("formatVersion must not be empty".to_string());
    }
    if scene.topologies.is_empty() {
        issues.push("at least one topology is required".to_string());
    }

    let known_kinds = ["raster", "vector", "text", "group", "mask"];
    for element in &scene.elements {
        if !known_kinds.contains(&element.kind.as_str()) {
            issues.push(format!(
                "element '{}' has unsupported type '{}'",
                element.id, element.kind
            ));
        }
    }

    for (id, relative) in &scene.resources {
        if !asset.has_reference(id) {
            issues.push(format!("resource '{id}' is missing at '{relative}'"));
        }
    }

    for (id, text) in &scene.text {
        if text.min_size <= 0.0 || text.preferred_size <= 0.0 || text.max_size <= 0.0 {
            issues.push(format!("text '{id}' has non-positive size bounds"));
        }
        if text.min_size > text.preferred_size || text.preferred_size > text.max_size {
            issues.push(format!(
                "text '{id}' requires minSize <= preferredSize <= maxSize"
            ));
        }
    }

    for topology in &scene.topologies {
        if let Some(Background::Resource(background)) = &topology.background {
            if !scene.resources.contains_key(background) {
                issues.push(format!(
                    "topology '{}' references unknown background resource '{background}'",
                    topology.id
                ));
            }
        }
        for element_id in topology.layout.keys() {
            if !scene.elements.iter().any(|e| &e.id == element_id) {
                issues.push(format!(
                    "topology '{}' lays out unknown element '{element_id}'",
                    topology.id
                ));
            }
        }
    }

    if let Some(fallback) = &scene.fallback {
        if !asset.has_reference(&fallback.resource) {
            issues.push(format!(
                "fallback references missing resource '{}'",
                fallback.resource
            ));
        }
    }

    issues
}
