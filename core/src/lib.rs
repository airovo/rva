//! RVA (Responsive Visual Asset) reference core.
//!
//! The core owns the platform-neutral half of RVA: parsing and validating an
//! asset, selecting a composition topology for a viewport, and resolving a
//! deterministic scene of absolute element geometry. Platform adapters are
//! responsible for turning that resolved scene into pixels.

pub mod cta;
pub mod error;
pub mod fonts;
pub mod model;
pub mod package;
#[cfg(feature = "render")]
pub mod render;
pub mod resolve;
pub mod resources;
pub mod solver;
pub mod topology;
pub mod validate;

/// Identifies the resolution semantics a build implements. Two runtimes produce
/// the same `ResolvedScene` only when they share this profile and the spec
/// version. Changing text metrics or layout rules must bump this.
pub const RESOLVER_PROFILE: &str = "rva-resolve/0.4";

pub use error::{Result, RvaError};
pub use fonts::Fonts;
pub use model::Scene;
#[cfg(feature = "render")]
pub use render::{render_to_file, render_to_png};
pub use cta::{overlaps as cta_overlaps, CtaOverlap};
pub use model::{CtaRegion, CtaSource};
pub use resolve::{
    resolve, Bounds, ItemKind, ResolvedCtaRegion, ResolvedItem, ResolvedScene,
};
pub use resources::Asset;
pub use validate::validate;
