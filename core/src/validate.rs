use crate::model::{Background, CtaSource};
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
        if let Some(parent) = &element.parent {
            match scene
                .elements
                .iter()
                .find(|candidate| &candidate.id == parent)
            {
                None => issues.push(format!(
                    "element '{}' references missing group '{parent}'",
                    element.id
                )),
                Some(group) if group.kind != "group" => issues.push(format!(
                    "element '{}' parent '{parent}' is not a group",
                    element.id
                )),
                _ => {}
            }
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

    let mut seen_regions: Vec<&str> = Vec::new();
    for region in &scene.cta_regions {
        if region.id.trim().is_empty() {
            issues.push("cta region id must not be empty".to_string());
        }
        if seen_regions.contains(&region.id.as_str()) {
            issues.push(format!("duplicate cta region id '{}'", region.id));
        }
        seen_regions.push(region.id.as_str());

        if let Some(padding) = region.padding {
            if !padding.is_finite() || padding < 0.0 {
                issues.push(format!(
                    "cta region '{}' padding must be a non-negative finite number",
                    region.id
                ));
            }
        }

        let element_exists = |id: &str| scene.elements.iter().any(|element| element.id == id);
        match &region.source {
            CtaSource::Element { element_id } => {
                if !element_exists(element_id) {
                    issues.push(format!(
                        "cta region '{}' references missing element '{element_id}'",
                        region.id
                    ));
                }
            }
            CtaSource::Elements { element_ids } => {
                if element_ids.is_empty() {
                    issues.push(format!(
                        "cta region '{}' has an empty element set",
                        region.id
                    ));
                }
                for element_id in element_ids {
                    if !element_exists(element_id) {
                        issues.push(format!(
                            "cta region '{}' references missing element '{element_id}'",
                            region.id
                        ));
                    }
                }
            }
            CtaSource::Region { x, y, w, h } => {
                if *w <= 0.0 || *h <= 0.0 {
                    issues.push(format!(
                        "cta region '{}' manual rect must have positive size",
                        region.id
                    ));
                }
                let epsilon = 1e-4;
                if *x < -epsilon || *y < -epsilon || x + w > 1.0 + epsilon || y + h > 1.0 + epsilon
                {
                    issues.push(format!(
                        "cta region '{}' manual rect must lie within 0..1",
                        region.id
                    ));
                }
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
