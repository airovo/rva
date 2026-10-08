use crate::error::{Result, RvaError};
use crate::fonts::Fonts;
use crate::model::{
    Background, CtaSource, FocalRegion, Focus, Paint, Scene, TextResource, Topology,
};
use crate::resources::Asset;
use crate::solver;
use serde::Serialize;
use ts_rs::TS;

/// The deterministic "what to draw" output of the core. Platform renderers turn
/// this into pixels using their native graphics stack.
#[derive(Debug, Clone, Serialize, TS)]
pub struct ResolvedScene {
    pub topology: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "topologyFallback")]
    pub topology_fallback: bool,
    /// Whether optional elements were hidden to keep the composition viable.
    pub degraded: bool,
    /// Hidden element ids, in the order they were removed.
    pub hidden: Vec<String>,
    /// 0..1 quality score for the selected composition.
    pub viability: f32,
    /// Optional solid base colour drawn behind the (topology) background.
    #[serde(rename = "baseColor", skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub base_color: Option<String>,
    pub background: Option<ResolvedItem>,
    pub items: Vec<ResolvedItem>,
    /// Resolved, actionable CTA regions. Geometry only — no behavior.
    #[serde(rename = "ctaRegions")]
    pub cta_regions: Vec<ResolvedCtaRegion>,
    pub diagnostics: Vec<String>,
}

/// Logical pixel rectangle in the `ResolvedScene` coordinate space (origin at
/// the top-left of the canvas).
#[derive(Debug, Clone, Copy, Serialize, TS)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The resolved geometry of one CTA region. `visible: false` means the source
/// is not active in the selected composition, so the region must not remain
/// actionable (bounds are then zero).
#[derive(Debug, Clone, Serialize, TS)]
pub struct ResolvedCtaRegion {
    pub id: String,
    pub visible: bool,
    pub bounds: Bounds,
    #[serde(rename = "normalizedBounds")]
    pub normalized_bounds: Bounds,
}

#[derive(Debug, Clone, Serialize, TS)]
pub struct ResolvedItem {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub role: Option<String>,
    pub z: i32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub opacity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mask: Option<String>,
    pub priority: f32,
    #[serde(rename = "cropPolicy", skip_serializing_if = "Option::is_none")]
    #[ts(rename = "cropPolicy", optional)]
    pub crop_policy: Option<String>,
    #[serde(rename = "focalRegions")]
    #[ts(rename = "focalRegions")]
    pub focal_regions: Vec<FocalRegion>,
    #[serde(flatten)]
    pub kind: ItemKind,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ItemKind {
    Image {
        resource: String,
        fit: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        focus: Option<Focus>,
    },
    Vector {
        resource: String,
    },
    Text {
        value: String,
        #[serde(rename = "fontFamily")]
        font_family: String,
        weight: u16,
        size: f32,
        #[serde(rename = "lineHeight")]
        line_height: f32,
        ascent: f32,
        lines: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        fill: Option<Paint>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        background: Option<String>,
        #[serde(rename = "letterSpacing")]
        letter_spacing: f32,
        #[serde(rename = "wordSpacing")]
        word_spacing: f32,
        align: String,
    },
    /// A procedural fill (solid colour or gradient), used for painted
    /// backgrounds.
    Paint {
        paint: Paint,
    },
}

struct Candidate {
    topology: String,
    background: Option<ResolvedItem>,
    items: Vec<ResolvedItem>,
    hard: Vec<String>,
    soft: f32,
    hidden: Vec<String>,
}

impl Candidate {
    fn viable(&self) -> bool {
        self.hard.is_empty()
    }

    fn viability(&self) -> f32 {
        solver::viability(self.soft)
    }

    fn cost(&self) -> f32 {
        solver::cost(self.soft, self.hidden.len())
    }
}

/// Resolve an RVA scene into absolute pixel geometry for a concrete logical
/// viewport. Candidate topologies are scored by viability and the best is
/// selected deterministically.
pub fn resolve(asset: &Asset, fonts: &Fonts, width: u32, height: u32) -> Result<ResolvedScene> {
    if width == 0 || height == 0 {
        return Err(RvaError::Validation(
            "render width and height must be non-zero".into(),
        ));
    }

    let scene = &asset.scene;
    let aspect_ratio = width as f32 / height as f32;
    let mut diagnostics = Vec::new();

    let candidates = candidate_topologies(scene, aspect_ratio);
    let range_fallback = candidates
        .iter()
        .all(|topology| !topology.when.contains(aspect_ratio));

    let mut evaluated: Vec<Candidate> = Vec::new();
    for topology in &candidates {
        evaluated.push(evaluate(asset, fonts, topology, width, height, &[])?);
    }

    // Consider degraded variants only when no non-degraded composition is
    // comfortably viable, mirroring the "degrade then fall back" policy.
    let best_plain = evaluated
        .iter()
        .filter(|candidate| candidate.viable())
        .min_by(|a, b| a.cost().partial_cmp(&b.cost()).unwrap());

    let need_degrade =
        best_plain.is_none_or(|candidate| candidate.viability() < solver::MIN_VIABILITY);
    if need_degrade {
        for topology in &candidates {
            if let Some(degraded) = degrade(asset, fonts, topology, width, height)? {
                evaluated.push(degraded);
            }
        }
    }

    let best = evaluated
        .iter()
        .filter(|candidate| candidate.viable())
        .min_by(|a, b| a.cost().partial_cmp(&b.cost()).unwrap());

    let Some(best) = best else {
        diagnostics
            .push("no topology is viable at this size; using the canonical fallback".to_string());
        // Surface the specific blockers so the failure is actionable (names the
        // topology and the element/constraint that could not be satisfied).
        let mut seen = std::collections::BTreeSet::new();
        for candidate in &evaluated {
            for violation in &candidate.hard {
                if seen.insert(format!("{}|{}", candidate.topology, violation)) {
                    diagnostics.push(format!("topology '{}': {violation}", candidate.topology));
                }
            }
        }
        return Ok(fallback_scene(
            asset,
            width,
            height,
            range_fallback,
            diagnostics,
        ));
    };

    if best.degraded() {
        diagnostics.push(format!(
            "viability below threshold; hid {} optional element(s) in topology '{}'",
            best.hidden.len(),
            best.topology
        ));
    }
    for violation in &best.hard {
        diagnostics.push(violation.clone());
    }
    if range_fallback {
        diagnostics.push(format!(
            "no topology range matched aspect {aspect_ratio:.3}; scored all topologies"
        ));
    }

    Ok(ResolvedScene {
        topology: best.topology.clone(),
        width,
        height,
        topology_fallback: range_fallback,
        degraded: best.degraded(),
        hidden: best.hidden.clone(),
        viability: best.viability(),
        base_color: scene.background.clone(),
        background: best.background.clone(),
        items: best.items.clone(),
        cta_regions: resolve_cta_regions(scene, &best.items, width, height),
        diagnostics,
    })
}

impl Candidate {
    fn degraded(&self) -> bool {
        !self.hidden.is_empty()
    }
}

/// Topologies whose aspect range matches come first; if none match, all are
/// considered (the PRD allows overlapping ranges plus viability scoring).
fn candidate_topologies(scene: &Scene, aspect_ratio: f32) -> Vec<&Topology> {
    let matching: Vec<&Topology> = scene
        .topologies
        .iter()
        .filter(|topology| topology.when.contains(aspect_ratio))
        .collect();
    if matching.is_empty() {
        scene.topologies.iter().collect()
    } else {
        matching
    }
}

fn evaluate(
    asset: &Asset,
    fonts: &Fonts,
    topology: &Topology,
    width: u32,
    height: u32,
    hidden: &[String],
) -> Result<Candidate> {
    let (background, mut items, mut hard) =
        layout_topology(asset, fonts, topology, width, height, hidden)?;

    // Authored relationships first, then built-in clamp/protected-region
    // enforcement. Constraint penalties fold into the soft score.
    let (constraint_hard, constraint_soft) = solver::apply_constraints(
        &mut items,
        &asset.scene.constraints,
        width as f32,
        height as f32,
    );
    let score = solver::enforce_and_score(&mut items, width as f32, height as f32);
    hard.extend(constraint_hard);
    hard.extend(score.hard);
    Ok(Candidate {
        topology: topology.id.clone(),
        background,
        items,
        hard,
        soft: score.soft + constraint_soft,
        hidden: hidden.to_vec(),
    })
}

/// Progressively hide optional elements (least important, then declaration
/// order) and keep the first degraded composition that restores viability.
fn degrade(
    asset: &Asset,
    fonts: &Fonts,
    topology: &Topology,
    width: u32,
    height: u32,
) -> Result<Option<Candidate>> {
    let mut optionals: Vec<(usize, f32, String)> = asset
        .scene
        .elements
        .iter()
        .enumerate()
        .filter(|(_, element)| element.visibility.as_deref() == Some("optional"))
        .filter(|(_, element)| topology.layout.contains_key(&element.id))
        .map(|(index, element)| (index, element.priority.unwrap_or(0.0), element.id.clone()))
        .collect();
    // Lowest priority first; stable by declaration order.
    optionals.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });

    let mut hidden: Vec<String> = Vec::new();
    let mut best: Option<Candidate> = None;
    for (_, _, id) in optionals {
        hidden.push(id);
        let candidate = evaluate(asset, fonts, topology, width, height, &hidden)?;
        if candidate.viable() {
            let improved = best
                .as_ref()
                .is_none_or(|current| candidate.cost() < current.cost());
            if improved {
                best = Some(candidate);
            }
        }
        if best
            .as_ref()
            .is_some_and(|candidate| candidate.viability() >= solver::MIN_VIABILITY)
        {
            break;
        }
    }
    Ok(best)
}

fn layout_topology(
    asset: &Asset,
    fonts: &Fonts,
    topology: &Topology,
    width: u32,
    height: u32,
    hidden: &[String],
) -> Result<(Option<ResolvedItem>, Vec<ResolvedItem>, Vec<String>)> {
    let scene = &asset.scene;
    let mut hard: Vec<String> = Vec::new();
    let canvas_w = width as f32;
    let canvas_h = height as f32;

    // The topology background has four modes: none, image, solid, gradient.
    // Only the **image** mode is backed by the `background` element, so hiding
    // that element hides an image background but never a solid/gradient one.
    let background_element_hidden = scene.elements.iter().any(|element| {
        (element.role.as_deref() == Some("background") || element.id == "background")
            && (element.visibility.as_deref() == Some("hidden")
                || topology
                    .layout
                    .get(&element.id)
                    .and_then(|layout| layout.hidden)
                    .unwrap_or(false))
    });

    let background = match &topology.background {
        Some(Background::Resource(resource)) if !background_element_hidden => Some(ResolvedItem {
            id: "background".to_string(),
            role: Some("background".to_string()),
            z: -1,
            x: 0.0,
            y: 0.0,
            w: canvas_w,
            h: canvas_h,
            opacity: 1.0,
            mask: None,
            priority: 100.0,
            crop_policy: Some("cover".to_string()),
            focal_regions: Vec::new(),
            kind: ItemKind::Image {
                resource: resource.clone(),
                fit: "cover".to_string(),
                focus: topology.background_focus,
            },
        }),
        Some(Background::Paint(paint)) => Some(ResolvedItem {
            id: "background".to_string(),
            role: Some("background".to_string()),
            z: -1,
            x: 0.0,
            y: 0.0,
            w: canvas_w,
            h: canvas_h,
            opacity: 1.0,
            mask: None,
            priority: 100.0,
            crop_policy: None,
            focal_regions: Vec::new(),
            kind: ItemKind::Paint {
                paint: paint.clone(),
            },
        }),
        _ => None,
    };

    let mut items = Vec::new();
    for (index, element) in scene.elements.iter().enumerate() {
        let is_background =
            element.role.as_deref() == Some("background") || element.id == "background";
        // `visibility: "optional"` marks an element as expendable under
        // degradation; `visibility: "hidden"` removes it from the composition
        // entirely (an authoring hide toggle).
        // Groups are containers: they contribute an offset/opacity/visibility to
        // their descendants but are not drawn themselves.
        if element.kind == "group" {
            continue;
        }

        let Some(layout) = topology.layout.get(&element.id) else {
            continue;
        };

        // Hidden globally (all topologies) or hidden in this topology.
        let is_hidden =
            element.visibility.as_deref() == Some("hidden") || layout.hidden == Some(true);
        if is_background || is_hidden || hidden.contains(&element.id) {
            continue;
        }

        // Inherit ancestor-group offset, opacity and visibility.
        let (gdx, gdy, gopacity, ghidden) = group_context(scene, element, topology, hidden);
        if ghidden {
            continue;
        }

        let x = (layout.x + gdx) * canvas_w;
        let y = (layout.y + gdy) * canvas_h;
        let w = layout.w * canvas_w;

        let kind = match element.kind.as_str() {
            "text" => {
                let Some(text_resource) = lookup_text(scene, element) else {
                    continue;
                };
                match layout_text(fonts, text_resource, w, canvas_w, canvas_h) {
                    Some((size, line_height, lines, overflowed)) => {
                        if overflowed {
                            let max_lines = text_resource.max_lines.unwrap_or(0);
                            hard.push(format!(
                                "text '{}' needs more than {} line(s) at its minimum size in a {:.0}px box — increase Max lines, lower Min size, or widen the box",
                                element.id, max_lines, w
                            ));
                        }
                        let ascent =
                            fonts.ascent(size, &text_resource.font_family, text_resource.weight);
                        ItemKind::Text {
                            value: text_resource.value.clone(),
                            font_family: text_resource.font_family.clone(),
                            weight: text_resource.weight,
                            size,
                            line_height,
                            ascent,
                            lines,
                            color: text_resource.color.clone(),
                            fill: text_resource.fill.clone(),
                            background: text_resource.background.clone(),
                            letter_spacing: text_resource.letter_spacing.unwrap_or(0.0).max(0.0),
                            word_spacing: text_resource.word_spacing.unwrap_or(0.0),
                            align: text_resource
                                .align
                                .clone()
                                .unwrap_or_else(|| "left".to_string()),
                        }
                    }
                    None => continue,
                }
            }
            "vector" => {
                let Some(resource) = lookup_resource(asset, element) else {
                    continue;
                };
                ItemKind::Vector { resource }
            }
            _ => {
                let Some(resource) = lookup_resource(asset, element) else {
                    continue;
                };
                ItemKind::Image {
                    resource,
                    fit: "none".to_string(),
                    focus: None,
                }
            }
        };

        let h = match &kind {
            ItemKind::Text {
                lines, line_height, ..
            } => lines.len() as f32 * line_height,
            ItemKind::Image { resource, .. } | ItemKind::Vector { resource } => {
                let (iw, ih) = asset.intrinsic_size(resource)?;
                if iw > 0.0 {
                    w * ih / iw
                } else {
                    w
                }
            }
            // Painted fills are not produced by the element loop today, but the
            // match must stay exhaustive.
            ItemKind::Paint { .. } => w,
        };

        items.push(ResolvedItem {
            id: element.id.clone(),
            role: element.role.clone(),
            z: index as i32,
            x,
            y,
            w,
            h,
            opacity: (element.opacity.unwrap_or(1.0).clamp(0.0, 1.0) * gopacity).clamp(0.0, 1.0),
            mask: element.mask.clone(),
            priority: element.priority.unwrap_or(0.0),
            crop_policy: element.crop_policy.clone(),
            focal_regions: element.focal_regions.clone(),
            kind,
        });
    }

    Ok((background, items, hard))
}

fn fallback_scene(
    asset: &Asset,
    width: u32,
    height: u32,
    range_fallback: bool,
    diagnostics: Vec<String>,
) -> ResolvedScene {
    let background = asset.scene.fallback.as_ref().map(|fallback| ResolvedItem {
        id: "fallback".to_string(),
        role: Some("fallback".to_string()),
        z: -1,
        x: 0.0,
        y: 0.0,
        w: width as f32,
        h: height as f32,
        opacity: 1.0,
        mask: None,
        priority: 100.0,
        crop_policy: Some(fallback.fit.clone()),
        focal_regions: Vec::new(),
        kind: ItemKind::Image {
            resource: fallback.resource.clone(),
            fit: fallback.fit.clone(),
            focus: None,
        },
    });

    ResolvedScene {
        topology: "fallback".to_string(),
        width,
        height,
        topology_fallback: range_fallback,
        degraded: false,
        hidden: Vec::new(),
        viability: 0.0,
        base_color: asset.scene.background.clone(),
        background,
        items: Vec::new(),
        cta_regions: resolve_cta_regions(&asset.scene, &[], width, height),
        diagnostics,
    }
}

/// Resolve every authored CTA region against the final (post-solver) item
/// geometry. Manual regions always resolve; element sources resolve to
/// `visible: false` when their elements are not active in this composition.
fn resolve_cta_regions(
    scene: &Scene,
    items: &[ResolvedItem],
    width: u32,
    height: u32,
) -> Vec<ResolvedCtaRegion> {
    let canvas_w = width as f32;
    let canvas_h = height as f32;

    scene
        .cta_regions
        .iter()
        .map(|region| {
            let bounds = match &region.source {
                CtaSource::Element { element_id } => {
                    element_union(scene, items, std::iter::once(element_id))
                }
                CtaSource::Elements { element_ids } => {
                    element_union(scene, items, element_ids.iter())
                }
                CtaSource::Region { x, y, w, h } => Some(Bounds {
                    x: x * canvas_w,
                    y: y * canvas_h,
                    width: w * canvas_w,
                    height: h * canvas_h,
                }),
            };

            let (visible, bounds) = match bounds {
                Some(bounds) => (true, bounds),
                None => (
                    false,
                    Bounds {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    },
                ),
            };
            let bounds = if visible {
                pad_bounds(
                    bounds,
                    region.padding.unwrap_or(0.0).max(0.0),
                    canvas_w.min(canvas_h),
                    canvas_w,
                    canvas_h,
                )
            } else {
                bounds
            };

            ResolvedCtaRegion {
                id: region.id.clone(),
                visible,
                bounds,
                normalized_bounds: Bounds {
                    x: if canvas_w > 0.0 {
                        bounds.x / canvas_w
                    } else {
                        0.0
                    },
                    y: if canvas_h > 0.0 {
                        bounds.y / canvas_h
                    } else {
                        0.0
                    },
                    width: if canvas_w > 0.0 {
                        bounds.width / canvas_w
                    } else {
                        0.0
                    },
                    height: if canvas_h > 0.0 {
                        bounds.height / canvas_h
                    } else {
                        0.0
                    },
                },
            }
        })
        .collect()
}

/// Expand a region outward by `padding_fraction * base` on every side, then
/// clamp to the canvas. Deterministic; used for optional CTA hit padding.
fn pad_bounds(
    bounds: Bounds,
    padding_fraction: f32,
    base: f32,
    canvas_w: f32,
    canvas_h: f32,
) -> Bounds {
    if padding_fraction <= 0.0 || base <= 0.0 {
        return bounds;
    }
    let pad = padding_fraction * base;
    let x = (bounds.x - pad).max(0.0);
    let y = (bounds.y - pad).max(0.0);
    let right = (bounds.x + bounds.width + pad).min(canvas_w);
    let bottom = (bounds.y + bounds.height + pad).min(canvas_h);
    Bounds {
        x,
        y,
        width: (right - x).max(0.0),
        height: (bottom - y).max(0.0),
    }
}

/// Deterministic union of the resolved bounds of the referenced elements. Group
/// ids expand to their resolved descendants (groups are containers, not items).
fn element_union<'a>(
    scene: &Scene,
    items: &[ResolvedItem],
    ids: impl Iterator<Item = &'a String>,
) -> Option<Bounds> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut found = false;

    for id in ids {
        for item in items {
            if item.id == *id || is_descendant(scene, &item.id, id) {
                found = true;
                min_x = min_x.min(item.x);
                min_y = min_y.min(item.y);
                max_x = max_x.max(item.x + item.w);
                max_y = max_y.max(item.y + item.h);
            }
        }
    }

    if !found {
        return None;
    }
    Some(Bounds {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

/// True when `element_id`'s ancestor chain includes `ancestor_id`.
fn is_descendant(scene: &Scene, element_id: &str, ancestor_id: &str) -> bool {
    let mut current = scene
        .elements
        .iter()
        .find(|element| element.id == element_id)
        .and_then(|element| element.parent.clone());
    let mut guard = 0;
    while let Some(parent_id) = current {
        if parent_id == ancestor_id {
            return true;
        }
        guard += 1;
        if guard > 64 {
            break; // defensive: malformed/cyclic parent chain
        }
        current = scene
            .elements
            .iter()
            .find(|element| element.id == parent_id)
            .and_then(|element| element.parent.clone());
    }
    false
}

fn lookup_text<'a>(
    scene: &'a Scene,
    element: &'a crate::model::Element,
) -> Option<&'a TextResource> {
    scene
        .text
        .get(&element.id)
        .or_else(|| element.role.as_ref().and_then(|role| scene.text.get(role)))
}

/// Accumulated transform/opacity/visibility contributed by a chain of ancestor
/// groups. Offsets are normalized (added to the element's own normalized x/y).
fn group_context(
    scene: &Scene,
    element: &crate::model::Element,
    topology: &Topology,
    hidden: &[String],
) -> (f32, f32, f32, bool) {
    let mut dx = 0.0f32;
    let mut dy = 0.0f32;
    let mut opacity = 1.0f32;
    let mut hidden_flag = false;

    let mut current = element.parent.clone();
    let mut guard = 0;
    while let Some(parent_id) = current {
        guard += 1;
        if guard > 64 {
            break; // defensive: malformed/cyclic parent chain
        }
        let Some(parent) = scene.elements.iter().find(|e| e.id == parent_id) else {
            break;
        };
        let parent_hidden_here = topology
            .layout
            .get(&parent_id)
            .and_then(|layout| layout.hidden)
            .unwrap_or(false);
        if parent.visibility.as_deref() == Some("hidden")
            || parent_hidden_here
            || hidden.iter().any(|id| id == &parent.id)
        {
            hidden_flag = true;
        }
        if let Some(op) = parent.opacity {
            opacity *= op.clamp(0.0, 1.0);
        }
        if let Some(layout) = topology.layout.get(&parent_id) {
            dx += layout.x;
            dy += layout.y;
        }
        current = parent.parent.clone();
    }

    (dx, dy, opacity, hidden_flag)
}

fn lookup_resource(asset: &Asset, element: &crate::model::Element) -> Option<String> {
    if asset.scene.resources.contains_key(&element.id) {
        return Some(element.id.clone());
    }
    if let Some(role) = &element.role {
        if asset.scene.resources.contains_key(role) {
            return Some(role.clone());
        }
    }
    None
}

/// Reference viewport used to scale the authored preferred size. Font size
/// scales with the smaller of the width/height ratios so short viewports do not
/// overflow and tall/portrait viewports stay bounded.
const TEXT_REF_WIDTH: f32 = 1600.0;
const TEXT_REF_HEIGHT: f32 = 900.0;

/// Choose a responsive font size, wrap the value to the element width and apply
/// the declared max-lines policy. If the text still exceeds `maxLines` after
/// shrinking to `minSize`, the topology is marked non-viable (the last flag).
fn layout_text(
    fonts: &Fonts,
    text: &TextResource,
    width: f32,
    canvas_w: f32,
    canvas_h: f32,
) -> Option<(f32, f32, Vec<String>, bool)> {
    let scale = (canvas_w / TEXT_REF_WIDTH).min(canvas_h / TEXT_REF_HEIGHT);
    let mut size = (text.preferred_size * scale).clamp(text.min_size, text.max_size);
    let letter_spacing = text.letter_spacing.unwrap_or(0.0).max(0.0);
    let word_spacing = text.word_spacing.unwrap_or(0.0);

    let mut lines = wrap(
        fonts,
        &text.value,
        width,
        size,
        &text.font_family,
        text.weight,
        letter_spacing,
        word_spacing,
    );

    let mut overflowed = false;
    if let Some(max_lines) = text.max_lines {
        let max_lines = max_lines as usize;
        if max_lines == 0 {
            return None;
        }
        while lines.len() > max_lines && size > text.min_size {
            size = (size * 0.94).max(text.min_size);
            lines = wrap(
                fonts,
                &text.value,
                width,
                size,
                &text.font_family,
                text.weight,
                letter_spacing,
                word_spacing,
            );
        }
        if lines.len() > max_lines {
            overflowed = true;
            lines.truncate(max_lines);
            if let Some(last) = lines.last_mut() {
                last.push('…');
            }
        }
    }

    let line_height = size * text.line_height.unwrap_or(1.25).max(0.1);
    Some((size, line_height, lines, overflowed))
}

#[allow(clippy::too_many_arguments)]
fn wrap(
    fonts: &Fonts,
    text: &str,
    max_width: f32,
    size: f32,
    family: &str,
    weight: u16,
    letter_spacing: f32,
    word_spacing: f32,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        let fits = fonts.measure_styled(
            &candidate,
            size,
            family,
            weight,
            letter_spacing,
            word_spacing,
        ) <= max_width;
        if fits || current.is_empty() {
            current = candidate;
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
