//! Deterministic constraint evaluation and viability scoring.
//!
//! The resolver builds a raw, unconstrained layout for a candidate topology.
//! This module then:
//! 1. enforces hard constraints (non-croppable elements and protected focal
//!    regions), moving/scaling elements where necessary, and
//! 2. scores the result with soft penalties (chiefly protected content being
//!    occluded by elements drawn on top of it).
//!
//! Lower soft penalty is better; `viability` maps penalty into `(0, 1]`.

use crate::model::{Constraint, FocalRegion};
use crate::resolve::{ItemKind, ResolvedItem};

/// Below this viability the resolver attempts degradation (hiding optional
/// elements) before falling back.
pub const MIN_VIABILITY: f32 = 0.6;
const OCCLUSION_WEIGHT: f32 = 1.0;
/// Weight for low-priority elements cluttering a protected region (drives
/// degradation on crowded compositions).
const CLUTTER_WEIGHT: f32 = 0.7;
const HIDDEN_COST: f32 = 0.01;
const FOCAL_MARGIN: f32 = 2.0;

/// `(x, y, w, h)`
type Rect = (f32, f32, f32, f32);

pub struct Score {
    pub hard: Vec<String>,
    pub soft: f32,
}

pub fn viability(soft: f32) -> f32 {
    1.0 / (1.0 + soft.max(0.0))
}

/// Total ordering cost used to pick between viable candidates. Lower is better;
/// hiding elements is mildly discouraged so an equally-viable non-degraded
/// composition wins.
pub fn cost(soft: f32, hidden: usize) -> f32 {
    soft + HIDDEN_COST * hidden as f32
}

fn rect(item: &ResolvedItem) -> Rect {
    (item.x, item.y, item.w, item.h)
}

fn intersection(a: Rect, b: Rect) -> f32 {
    let x0 = a.0.max(b.0);
    let y0 = a.1.max(b.1);
    let x1 = (a.0 + a.2).min(b.0 + b.2);
    let y1 = (a.1 + a.3).min(b.1 + b.3);
    if x1 > x0 && y1 > y0 {
        (x1 - x0) * (y1 - y0)
    } else {
        0.0
    }
}

fn focal_rect(item: &ResolvedItem, region: &FocalRegion) -> Rect {
    (
        item.x + region.x * item.w,
        item.y + region.y * item.h,
        region.w * item.w,
        region.h * item.h,
    )
}

fn scale_about(item: &mut ResolvedItem, cx: f32, cy: f32, scale: f32) {
    item.x = cx + (item.x - cx) * scale;
    item.y = cy + (item.y - cy) * scale;
    item.w *= scale;
    item.h *= scale;
}

fn is_background(item: &ResolvedItem) -> bool {
    item.role.as_deref() == Some("background")
}

/// Position `br` clear of `fr` (nearest edge first), staying in canvas and
/// preferring a spot that does not overlap other content (`obstacles`, e.g.
/// text). Falls back to the least-overlapping spot if none is clean.
fn escape(
    fr: Rect,
    br: Rect,
    canvas_w: f32,
    canvas_h: f32,
    obstacles: &[Rect],
) -> Option<(f32, f32)> {
    let candidates = [
        (br.0, fr.1 - br.3 - FOCAL_MARGIN), // above
        (br.0, fr.1 + fr.3 + FOCAL_MARGIN), // below
        (fr.0 - br.2 - FOCAL_MARGIN, br.1), // left
        (fr.0 + fr.2 + FOCAL_MARGIN, br.1), // right
    ];
    let mut best: Option<(f32, f32, f32)> = None;
    for (cx, cy) in candidates {
        let x = cx.clamp(0.0, (canvas_w - br.2).max(0.0));
        let y = cy.clamp(0.0, (canvas_h - br.3).max(0.0));
        let candidate = (x, y, br.2, br.3);
        if intersection(fr, candidate) > 1.0 {
            continue; // must clear the protected region
        }
        let penalty: f32 = obstacles.iter().map(|o| intersection(*o, candidate)).sum();
        if penalty <= 1.0 {
            return Some((x, y));
        }
        if best.is_none_or(|(_, _, bp)| penalty < bp) {
            best = Some((x, y, penalty));
        }
    }
    best.map(|(x, y, _)| (x, y))
}

fn is_text(item: &ResolvedItem) -> bool {
    matches!(item.kind, ItemKind::Text { .. })
}

/// Hard constraint: elements drawn on top must not cover a protected (hard)
/// focal region of another element. Occluders are moved out of the way; if no
/// placement is possible the topology is reported non-viable (enables fallback).
fn separate_occluders(items: &mut [ResolvedItem], canvas_w: f32, canvas_h: f32) -> Vec<String> {
    // Owner indices only; focal rects are recomputed each pass because owners
    // can themselves move while avoiding another owner's region.
    let owners: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            !is_background(item) && item.focal_regions.iter().any(|region| region.hard)
        })
        .map(|(index, _)| index)
        .collect();

    let hard_focals = |item: &ResolvedItem| -> Vec<Rect> {
        item.focal_regions
            .iter()
            .filter(|region| region.hard)
            .map(|region| focal_rect(item, region))
            .collect()
    };

    // Bounded deterministic passes (occluders can affect one another).
    for _ in 0..6 {
        let mut changed = false;
        let text_rects: Vec<(usize, Rect)> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| is_text(item))
            .map(|(index, item)| (index, rect(item)))
            .collect();
        for &owner in &owners {
            let owner_z = items[owner].z;
            let focals = hard_focals(&items[owner]);
            for other in 0..items.len() {
                if other == owner || is_background(&items[other]) || items[other].z <= owner_z {
                    continue; // only elements drawn on top occlude
                }
                let obstacles: Vec<Rect> = text_rects
                    .iter()
                    .filter(|(index, _)| *index != other)
                    .map(|(_, rect)| *rect)
                    .collect();
                for focal in &focals {
                    let current = rect(&items[other]);
                    if intersection(*focal, current) > 1.0 {
                        if let Some((x, y)) =
                            escape(*focal, current, canvas_w, canvas_h, &obstacles)
                        {
                            items[other].x = x;
                            items[other].y = y;
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    let mut hard = Vec::new();
    for &owner in &owners {
        let owner_z = items[owner].z;
        let focals = hard_focals(&items[owner]);
        for other in 0..items.len() {
            if other == owner || is_background(&items[other]) || items[other].z <= owner_z {
                continue;
            }
            for focal in &focals {
                if intersection(*focal, rect(&items[other])) > 1.0 {
                    hard.push(format!(
                        "'{}' cannot avoid the protected region of '{}'",
                        items[other].id, items[owner].id
                    ));
                    break;
                }
            }
        }
    }
    hard
}

/// Enforce hard constraints in place and return a soft-penalty score.
pub fn enforce_and_score(items: &mut [ResolvedItem], canvas_w: f32, canvas_h: f32) -> Score {
    let mut hard = Vec::new();

    for item in items.iter_mut() {
        if is_background(item) {
            continue;
        }

        // Non-croppable elements must fit entirely within the canvas.
        if matches!(item.crop_policy.as_deref(), Some("none")) {
            let mut scale = 1.0f32;
            if item.w > canvas_w {
                scale = scale.min(canvas_w / item.w);
            }
            if item.h > canvas_h {
                scale = scale.min(canvas_h / item.h);
            }
            if scale < 1.0 {
                scale_about(item, item.x + item.w / 2.0, item.y + item.h / 2.0, scale);
            }
            item.x = item.x.clamp(0.0, (canvas_w - item.w).max(0.0));
            item.y = item.y.clamp(0.0, (canvas_h - item.h).max(0.0));
        }

        // Protected (hard) focal regions must remain visible.
        let hard_regions: Vec<FocalRegion> = item
            .focal_regions
            .iter()
            .filter(|region| region.hard)
            .cloned()
            .collect();
        for region in &hard_regions {
            let (fx, fy, fw, fh) = focal_rect(item, region);
            if fw > canvas_w || fh > canvas_h {
                let scale = (canvas_w / fw).min(canvas_h / fh).min(1.0);
                scale_about(item, fx + fw / 2.0, fy + fh / 2.0, scale);
                let (fx2, fy2, fw2, fh2) = focal_rect(item, region);
                let dx = if fx2 < 0.0 {
                    -fx2
                } else if fx2 + fw2 > canvas_w {
                    canvas_w - (fx2 + fw2)
                } else {
                    0.0
                };
                let dy = if fy2 < 0.0 {
                    -fy2
                } else if fy2 + fh2 > canvas_h {
                    canvas_h - (fy2 + fh2)
                } else {
                    0.0
                };
                item.x += dx;
                item.y += dy;
            } else {
                let dx = if fx < 0.0 {
                    -fx
                } else if fx + fw > canvas_w {
                    canvas_w - (fx + fw)
                } else {
                    0.0
                };
                let dy = if fy < 0.0 {
                    -fy
                } else if fy + fh > canvas_h {
                    canvas_h - (fy + fh)
                } else {
                    0.0
                };
                item.x += dx;
                item.y += dy;
            }
        }

        for region in &hard_regions {
            let (fx, fy, fw, fh) = focal_rect(item, region);
            if fx < -0.5 || fy < -0.5 || fx + fw > canvas_w + 0.5 || fy + fh > canvas_h + 0.5 {
                hard.push(format!(
                    "protected region '{}' of '{}' cannot be kept visible at this size",
                    region.name, item.id
                ));
            }
        }
    }

    // Hard: elements on top must not cover protected regions.
    hard.extend(separate_occluders(items, canvas_w, canvas_h));

    // Soft penalties around protected regions.
    let mut soft = 0.0f32;
    let owners: Vec<(usize, f32, Rect)> = items
        .iter()
        .enumerate()
        .flat_map(|(index, item)| {
            item.focal_regions
                .iter()
                .filter(|region| region.hard)
                .map(move |region| (index, item.priority, focal_rect(item, region)))
        })
        .collect();

    // (a) residual occlusion by elements drawn on top.
    for &(owner_index, owner_priority, region) in &owners {
        let owner_z = items[owner_index].z;
        let mut covered = 0.0f32;
        for (other_index, other) in items.iter().enumerate() {
            if other_index == owner_index || is_background(other) || other.z <= owner_z {
                continue;
            }
            let overlap = intersection(region, rect(other));
            if overlap > 0.0 {
                let factor = if other.priority >= owner_priority {
                    1.5
                } else {
                    1.0
                };
                covered += overlap * factor;
            }
        }
        soft += (covered / (region.2 * region.3).max(1.0)) * OCCLUSION_WEIGHT;
    }

    // (b) low-priority clutter around protected regions (drives degradation).
    for &(owner_index, _owner_priority, region) in &owners {
        let region_area = (region.2 * region.3).max(1.0);
        for (other_index, other) in items.iter().enumerate() {
            if other_index == owner_index || is_background(other) {
                continue;
            }
            let overlap = intersection(region, rect(other));
            if overlap > 0.0 {
                let weight = (1.0 - (other.priority / 100.0).clamp(0.0, 1.0)) * CLUTTER_WEIGHT;
                soft += (overlap / region_area) * weight;
            }
        }
    }

    Score { hard, soft }
}

/// Tolerance (logical px) below which a constraint is considered satisfied.
const CONSTRAINT_TOLERANCE: f32 = 1.0;

fn edge_position(r: Rect, edge: &str) -> Option<f32> {
    match edge {
        "left" => Some(r.0),
        "right" => Some(r.0 + r.2),
        "top" => Some(r.1),
        "bottom" => Some(r.1 + r.3),
        "center-x" => Some(r.0 + r.2 / 2.0),
        "center-y" => Some(r.1 + r.3 / 2.0),
        _ => None,
    }
}

fn is_horizontal(edge: &str) -> bool {
    matches!(edge, "left" | "right" | "center-x")
}

fn desired_edge(constraint: &Constraint, target: Rect, target_edge: &str) -> Option<f32> {
    let base = edge_position(target, target_edge)?;
    Some(if constraint.kind == "gap" {
        base + constraint.value.unwrap_or(0.0)
    } else {
        base
    })
}

/// Residual violation: pixels for alignment/gap/containment, area for overlap.
fn constraint_violation(constraint: &Constraint, subject: Rect, target: Rect) -> f32 {
    match constraint.kind.as_str() {
        "align" | "gap" => {
            let edge = constraint.edge.as_deref().unwrap_or("left");
            let target_edge = constraint.target_edge.as_deref().unwrap_or(edge);
            match (
                desired_edge(constraint, target, target_edge),
                edge_position(subject, edge),
            ) {
                (Some(desired), Some(current)) => (desired - current).abs(),
                _ => 0.0,
            }
        }
        "contain" => {
            let ox = (target.0 - subject.0).max(0.0)
                + ((subject.0 + subject.2) - (target.0 + target.2)).max(0.0);
            let oy = (target.1 - subject.1).max(0.0)
                + ((subject.1 + subject.3) - (target.1 + target.3)).max(0.0);
            ox + oy
        }
        "no-overlap" => intersection(subject, target),
        _ => 0.0,
    }
}

/// Apply authored relationships/constraints in declaration order. Hard
/// constraints are clamped into the canvas and reported when still unsatisfied;
/// soft constraints adjust the layout and fold residual violation into the soft
/// score. Returns `(hard_violations, soft_penalty)`.
pub fn apply_constraints(
    items: &mut [ResolvedItem],
    constraints: &[Constraint],
    canvas_w: f32,
    canvas_h: f32,
) -> (Vec<String>, f32) {
    let mut hard = Vec::new();
    let mut soft = 0.0f32;

    for constraint in constraints {
        let Some(subject_index) = items.iter().position(|item| item.id == constraint.subject) else {
            continue;
        };
        if is_background(&items[subject_index]) {
            continue;
        }

        let target_rect: Rect = match constraint.target.as_deref() {
            None | Some("canvas") => (0.0, 0.0, canvas_w, canvas_h),
            Some(target_id) => match items.iter().position(|item| item.id == target_id) {
                Some(index) => rect(&items[index]),
                None => continue,
            },
        };

        match constraint.kind.as_str() {
            "align" | "gap" => {
                let edge = constraint.edge.as_deref().unwrap_or("left");
                let target_edge = constraint.target_edge.as_deref().unwrap_or(edge);
                let subject = rect(&items[subject_index]);
                if let (Some(desired), Some(current)) = (
                    desired_edge(constraint, target_rect, target_edge),
                    edge_position(subject, edge),
                ) {
                    let delta = desired - current;
                    if is_horizontal(edge) {
                        items[subject_index].x += delta;
                    } else {
                        items[subject_index].y += delta;
                    }
                }
            }
            "contain" => {
                let item = &mut items[subject_index];
                item.x = item.x.clamp(target_rect.0, (target_rect.0 + target_rect.2 - item.w).max(target_rect.0));
                item.y = item.y.clamp(target_rect.1, (target_rect.1 + target_rect.3 - item.h).max(target_rect.1));
            }
            "no-overlap" => {
                let subject = rect(&items[subject_index]);
                if intersection(subject, target_rect) > 0.0 {
                    let left = (subject.0 + subject.2) - target_rect.0;
                    let right = (target_rect.0 + target_rect.2) - subject.0;
                    let up = (subject.1 + subject.3) - target_rect.1;
                    let down = (target_rect.1 + target_rect.3) - subject.1;
                    let options = [("left", left), ("right", right), ("up", up), ("down", down)];
                    if let Some((direction, amount)) = options
                        .iter()
                        .filter(|(_, amount)| *amount > 0.0)
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    {
                        let item = &mut items[subject_index];
                        match *direction {
                            "left" => item.x -= amount,
                            "right" => item.x += amount,
                            "up" => item.y -= amount,
                            _ => item.y += amount,
                        }
                    }
                }
            }
            _ => {}
        }

        // Hard constraints may not leave the canvas.
        if constraint.hard {
            let item = &mut items[subject_index];
            item.x = item.x.clamp(0.0, (canvas_w - item.w).max(0.0));
            item.y = item.y.clamp(0.0, (canvas_h - item.h).max(0.0));
        }

        let subject_rect = rect(&items[subject_index]);
        let violation = constraint_violation(constraint, subject_rect, target_rect);
        if violation > CONSTRAINT_TOLERANCE {
            if constraint.hard {
                hard.push(format!(
                    "constraint '{}' ({}) on '{}' cannot be satisfied at this size",
                    constraint.id, constraint.kind, constraint.subject
                ));
            } else if constraint.kind == "no-overlap" {
                soft += (violation / (canvas_w * canvas_h).max(1.0)) * constraint.weight;
            } else {
                soft += (violation / canvas_w.max(canvas_h).max(1.0)) * constraint.weight;
            }
        }
    }

    (hard, soft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FocalRegion;
    use crate::resolve::ItemKind;

    fn raster(id: &str, z: i32, x: f32, y: f32, w: f32, h: f32) -> ResolvedItem {
        ResolvedItem {
            id: id.to_string(),
            role: None,
            z,
            x,
            y,
            w,
            h,
            opacity: 1.0,
            mask: None,
            priority: 0.0,
            crop_policy: None,
            focal_regions: Vec::new(),
            kind: ItemKind::Image {
                resource: id.to_string(),
                fit: "none".to_string(),
                focus: None,
            },
        }
    }

    fn face() -> FocalRegion {
        FocalRegion {
            name: "face".to_string(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            hard: true,
        }
    }

    #[test]
    fn non_croppable_element_is_clamped_into_canvas() {
        let mut items = vec![raster("logo", 0, -50.0, -50.0, 200.0, 100.0)];
        items[0].crop_policy = Some("none".to_string());
        let score = enforce_and_score(&mut items, 100.0, 100.0);
        assert!(score.hard.is_empty());
        assert!(items[0].x >= 0.0 && items[0].y >= 0.0);
        assert!(items[0].x + items[0].w <= 100.0 + 0.01);
        assert!(items[0].y + items[0].h <= 100.0 + 0.01);
    }

    #[test]
    fn offscreen_protected_region_is_pulled_into_view() {
        let mut subject = raster("subject", 0, 300.0, 300.0, 100.0, 100.0);
        subject.crop_policy = Some("protected".to_string());
        subject.focal_regions = vec![face()];
        let mut items = vec![subject];
        let score = enforce_and_score(&mut items, 200.0, 200.0);
        assert!(score.hard.is_empty());
        assert!(items[0].x <= 100.0 && items[0].x + items[0].w <= 200.0 + 0.01);
        assert!(items[0].y <= 100.0 && items[0].y + items[0].h <= 200.0 + 0.01);
    }

    #[test]
    fn occluder_is_moved_off_a_protected_region() {
        // Small protected region; the element drawn on top is pushed clear.
        let mut subject = raster("subject", 1, 0.0, 0.0, 100.0, 100.0);
        subject.focal_regions = vec![FocalRegion {
            name: "face".to_string(),
            x: 0.0,
            y: 0.0,
            w: 0.3,
            h: 0.3,
            hard: true,
        }];
        subject.priority = 100.0;
        let mut cover = raster("cover", 2, 0.0, 0.0, 30.0, 30.0);
        cover.priority = 80.0;
        let mut items = vec![subject, cover];
        let score = enforce_and_score(&mut items, 100.0, 100.0);
        assert!(
            score.hard.is_empty(),
            "separable occluder must not be a violation"
        );
        // The cover no longer overlaps the focal region.
        let focal = (0.0, 0.0, 30.0, 30.0);
        assert_eq!(intersection(focal, rect(&items[1])), 0.0);
    }

    #[test]
    fn unavoidable_occlusion_is_a_hard_violation() {
        // Protected region covers the whole canvas; the occluder cannot escape.
        let mut subject = raster("subject", 1, 0.0, 0.0, 100.0, 100.0);
        subject.focal_regions = vec![face()]; // 1x1 of a full-canvas element
        subject.crop_policy = Some("protected".to_string());
        let cover = raster("cover", 2, 0.0, 0.0, 100.0, 100.0);
        let score = enforce_and_score(&mut [subject, cover], 100.0, 100.0);
        assert!(
            !score.hard.is_empty(),
            "unavoidable occlusion must be non-viable"
        );
    }

    #[test]
    fn background_does_not_contribute_clutter() {
        let mut subject = raster("subject", 2, 0.0, 0.0, 100.0, 100.0);
        subject.focal_regions = vec![face()];
        let mut background = raster("background", 1, 0.0, 0.0, 100.0, 100.0);
        background.role = Some("background".to_string());
        let score = enforce_and_score(&mut [subject, background], 100.0, 100.0);
        assert_eq!(score.soft, 0.0);

        // A low-priority non-background element behind the focal adds clutter.
        let mut subject = raster("subject", 2, 0.0, 0.0, 100.0, 100.0);
        subject.focal_regions = vec![face()];
        let mut behind = raster("decor", 1, 0.0, 0.0, 100.0, 100.0);
        behind.priority = 20.0;
        let score = enforce_and_score(&mut [subject, behind], 100.0, 100.0);
        assert!(score.soft > 0.0);
    }

    fn gap(id: &str, subject: &str, target: &str, edge: &str, target_edge: &str, value: f32) -> Constraint {
        Constraint {
            id: id.to_string(),
            kind: "gap".to_string(),
            subject: subject.to_string(),
            target: Some(target.to_string()),
            edge: Some(edge.to_string()),
            target_edge: Some(target_edge.to_string()),
            value: Some(value),
            hard: true,
            weight: 1.0,
        }
    }

    #[test]
    fn gap_relation_places_subject_relative_to_target() {
        let subject = raster("headline", 1, 120.0, 50.0, 40.0, 20.0);
        let target = raster("subject", 0, 0.0, 0.0, 40.0, 40.0);
        // headline.left = subject.right + 10
        let constraints = vec![gap("c", "headline", "subject", "left", "right", 10.0)];
        let mut items = vec![subject, target];
        let (hard, _soft) = apply_constraints(&mut items, &constraints, 200.0, 200.0);
        assert!(hard.is_empty());
        assert!((items[0].x - 50.0).abs() < 0.01);
    }

    #[test]
    fn align_to_canvas_center() {
        let subject = raster("logo", 0, 0.0, 0.0, 40.0, 20.0);
        let constraints = vec![Constraint {
            id: "c".into(),
            kind: "align".into(),
            subject: "logo".into(),
            target: Some("canvas".into()),
            edge: Some("center-x".into()),
            target_edge: Some("center-x".into()),
            value: None,
            hard: true,
            weight: 1.0,
        }];
        let mut items = vec![subject];
        let (hard, _soft) = apply_constraints(&mut items, &constraints, 200.0, 200.0);
        assert!(hard.is_empty());
        assert!((items[0].x + items[0].w / 2.0 - 100.0).abs() < 0.01);
    }

    #[test]
    fn no_overlap_pushes_subject_clear() {
        let subject = raster("badge", 1, 40.0, 0.0, 30.0, 30.0);
        let target = raster("face", 0, 20.0, 0.0, 30.0, 30.0);
        let constraints = vec![Constraint {
            id: "c".into(),
            kind: "no-overlap".into(),
            subject: "badge".into(),
            target: Some("face".into()),
            edge: None,
            target_edge: None,
            value: None,
            hard: true,
            weight: 1.0,
        }];
        let mut items = vec![subject, target];
        let (hard, _soft) = apply_constraints(&mut items, &constraints, 200.0, 200.0);
        assert!(hard.is_empty());
        assert_eq!(intersection(rect(&items[0]), rect(&items[1])), 0.0);
    }

    #[test]
    fn unsatisfiable_hard_constraint_is_reported() {
        // Ask a full-width element to sit 50px to the right of the canvas edge.
        let subject = raster("wide", 0, 0.0, 0.0, 100.0, 20.0);
        let constraints = vec![gap("c", "wide", "canvas", "left", "right", 50.0)];
        let mut items = vec![subject];
        let (hard, _soft) = apply_constraints(&mut items, &constraints, 100.0, 100.0);
        assert!(!hard.is_empty(), "impossible hard constraint must be reported");
    }
}
