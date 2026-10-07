//! CTA Region diagnostics.
//!
//! Overlap is a pure function of a resolved scene so every runtime agrees. Two
//! regions that are separate at one aspect ratio can collide after responsive
//! recomposition, so this must be evaluated per concrete viewport.

use crate::resolve::{Bounds, ResolvedCtaRegion, ResolvedScene};
use serde::Serialize;
use ts_rs::TS;

/// Intersections at or below this area (logical px²) are treated as touching,
/// not overlapping. Small, positive and deterministic.
pub const OVERLAP_EPSILON_AREA: f32 = 1.0;

/// A detected overlap between two CTA Regions at a concrete viewport.
#[derive(Debug, Clone, Serialize, TS)]
pub struct CtaOverlap {
    /// The two conflicting region ids, in scene order.
    pub regions: Vec<String>,
    pub width: u32,
    pub height: u32,
    /// Intersection area in logical px².
    #[serde(rename = "overlapArea")]
    pub overlap_area: f32,
    /// Intersection area relative to the smaller of the two regions, `0..1`.
    #[serde(rename = "overlapRatio")]
    pub overlap_ratio: f32,
}

/// All pairwise overlaps among the *visible* CTA regions, deterministic and
/// ordered by scene order then pair order. An empty result means the regions
/// are unambiguous.
pub fn overlaps(resolved: &ResolvedScene) -> Vec<CtaOverlap> {
    let visible: Vec<&ResolvedCtaRegion> = resolved
        .cta_regions
        .iter()
        .filter(|region| region.visible)
        .collect();

    let mut out = Vec::new();
    for i in 0..visible.len() {
        for j in (i + 1)..visible.len() {
            let a = &visible[i];
            let b = &visible[j];
            if let Some((area, ratio)) = intersection(&a.bounds, &b.bounds) {
                out.push(CtaOverlap {
                    regions: vec![a.id.clone(), b.id.clone()],
                    width: resolved.width,
                    height: resolved.height,
                    overlap_area: area,
                    overlap_ratio: ratio,
                });
            }
        }
    }
    out
}

/// Intersection area and ratio-to-smaller when the overlap exceeds the epsilon.
fn intersection(a: &Bounds, b: &Bounds) -> Option<(f32, f32)> {
    let ix = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
    let iy = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
    if ix <= 0.0 || iy <= 0.0 {
        return None;
    }
    let area = ix * iy;
    if area <= OVERLAP_EPSILON_AREA {
        return None;
    }
    let smaller = (a.width * a.height)
        .min(b.width * b.height)
        .max(1e-6);
    Some((area, (area / smaller).clamp(0.0, 1.0)))
}
