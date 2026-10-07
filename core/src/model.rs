use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// The logical RVA scene. Corresponds to `scene.json` in the draft asset model.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Scene {
    pub format: String,
    #[serde(rename = "formatVersion")]
    pub format_version: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "designSpace", default)]
    pub design_space: Option<DesignSpace>,
    /// Optional solid base colour behind every topology background. Lets an
    /// asset with a transparent/partly-transparent background paint over a
    /// deterministic scene colour instead of black/white. Default: transparent.
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub resources: BTreeMap<String, String>,
    #[serde(default)]
    pub text: BTreeMap<String, TextResource>,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub topologies: Vec<Topology>,
    /// Authored relationships / constraints between elements. Optional and
    /// forward-compatible: readers that do not implement a kind ignore it.
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    /// Semantic, actionable spatial regions. CTA Regions carry identity and
    /// geometry only; behavior is bound by the host at runtime. Independent of
    /// the `role: "cta"` visual semantic. Optional and forward-compatible.
    #[serde(rename = "ctaRegions", default)]
    pub cta_regions: Vec<CtaRegion>,
    #[serde(default)]
    pub fallback: Option<Fallback>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
pub struct DesignSpace {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TextResource {
    pub value: String,
    #[serde(rename = "fontFamily", default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_weight")]
    pub weight: u16,
    #[serde(rename = "minSize")]
    pub min_size: f32,
    #[serde(rename = "preferredSize")]
    pub preferred_size: f32,
    #[serde(rename = "maxSize")]
    pub max_size: f32,
    #[serde(rename = "maxLines", default)]
    pub max_lines: Option<u32>,
    /// Fill colour (CSS hex). Falls back to the role default when absent.
    #[serde(default)]
    pub color: Option<String>,
    /// Fill paint (colour or gradient). Takes precedence over `color`.
    #[serde(default)]
    pub fill: Option<Paint>,
    /// Optional solid background drawn behind the text box (supports alpha via
    /// `#RRGGBBAA`).
    #[serde(default)]
    pub background: Option<String>,
    /// Line height as a multiple of font size (default 1.25). Affects the
    /// deterministic layout box.
    #[serde(rename = "lineHeight", default)]
    pub line_height: Option<f32>,
    /// Letter spacing in pixels. Participates in the deterministic advance
    /// model, so wrapping is identical across runtimes.
    #[serde(rename = "letterSpacing", default)]
    pub letter_spacing: Option<f32>,
    /// Word spacing in pixels (added per space), also part of the advance model.
    #[serde(rename = "wordSpacing", default)]
    pub word_spacing: Option<f32>,
    /// Horizontal alignment within the text box: left|center|right.
    #[serde(default)]
    pub align: Option<String>,
}

fn default_font_family() -> String {
    "sans-serif".to_string()
}

fn default_weight() -> u16 {
    400
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    /// Human-readable display name. Independent of `id` (renaming must not
    /// change references) and optional.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub priority: Option<f32>,
    #[serde(rename = "cropPolicy", default)]
    pub crop_policy: Option<String>,
    #[serde(default)]
    pub mask: Option<String>,
    #[serde(rename = "focalRegions", default)]
    pub focal_regions: Vec<FocalRegion>,
    #[serde(default)]
    pub visibility: Option<String>,
    /// Compositing opacity, 0..1 (default 1). Enables translucent elements and
    /// transparent-background cutouts.
    #[serde(default)]
    pub opacity: Option<f32>,
    /// Authoring-only: when true, editors must not manipulate this element.
    /// Does not change runtime semantics (renderers ignore it).
    #[serde(default)]
    pub locked: Option<bool>,
    /// Parent group id (for `type: "group"` containers). Children inherit the
    /// group's offset, opacity and visibility.
    #[serde(default)]
    pub parent: Option<String>,
}

/// A semantic, actionable spatial region associated with a call to action.
///
/// RVA defines the region's *identity*, *source* and *resolved geometry* — never
/// the behavior performed on activation. The stable `id` is the host-facing
/// binding key and must survive save, export, parse and resolve unchanged. A
/// `CtaRegion` is independent of the visual `role: "cta"` semantic: neither
/// requires the other.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CtaRegion {
    pub id: String,
    /// Human-readable label for authoring. Independent of `id`; never used as a
    /// reference.
    #[serde(default)]
    pub name: Option<String>,
    pub source: CtaSource,
    /// Optional deterministic hit padding. A non-negative fraction of the
    /// canvas's shorter side, applied uniformly outward and clamped to the
    /// canvas, enlarging the actionable region without changing visual geometry.
    /// It participates in overlap validation. Absent/0 means no padding.
    #[serde(default)]
    pub padding: Option<f32>,
}

/// How a [`CtaRegion`] derives its geometry. Every variant is deterministic and
/// resolves to bounds in the same logical coordinate space as `ResolvedScene`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CtaSource {
    /// Follow one element. If the element is a group, the region covers the
    /// union of its resolved descendants.
    Element {
        #[serde(rename = "elementId")]
        element_id: String,
    },
    /// Cover several related elements (union of all resolved visible ones).
    Elements {
        #[serde(rename = "elementIds")]
        element_ids: Vec<String>,
    },
    /// An explicitly authored region, normalized `0..1` to the canvas. Scales
    /// with the render size; it does not track topology recomposition (use an
    /// element-bound source for that).
    Region { x: f32, y: f32, w: f32, h: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct FocalRegion {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(default)]
    pub hard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Topology {
    pub id: String,
    #[serde(default)]
    pub when: When,
    #[serde(default)]
    pub background: Option<Background>,
    /// Pan of an image background within the canvas: `(0,0)` shows the top-left
    /// of the image, `(1,1)` the bottom-right, `(0.5,0.5)` centred (default).
    /// Lets important parts of a photo stay visible in different topologies.
    #[serde(rename = "backgroundFocus", default)]
    pub background_focus: Option<Focus>,
    #[serde(default)]
    pub layout: BTreeMap<String, Layout>,
}

/// Normalized focus/pan point for a cover-fit image, `0..1` on each axis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
pub struct Focus {
    pub x: f32,
    pub y: f32,
}

impl Default for Focus {
    fn default() -> Self {
        Focus { x: 0.5, y: 0.5 }
    }
}

/// A topology background. Either a declared resource id (legacy/photo) or a
/// procedural paint (solid colour or linear gradient).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum Background {
    /// Resource id, e.g. `"bgLandscape"`. Kept first for JSON compatibility:
    /// existing assets serialize/deserialize as a bare string.
    Resource(String),
    /// A generated fill.
    Paint(Paint),
}

/// A procedural fill usable as a background.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Paint {
    Color {
        /// CSS hex colour, e.g. `#0A0F1C`.
        color: String,
    },
    LinearGradient {
        /// Direction in degrees (0 = left→right, 90 = top→bottom).
        angle: f32,
        stops: Vec<GradientStop>,
    },
    RadialGradient {
        stops: Vec<GradientStop>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct GradientStop {
    /// Position in the gradient, 0..1.
    pub offset: f32,
    /// CSS hex colour.
    pub color: String,
}

/// An authored relationship/constraint between elements (or an element and the
/// canvas). Applied deterministically by the resolver after base layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Constraint {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    /// Subject element id.
    pub subject: String,
    /// Target element id, or `"canvas"`.
    #[serde(default)]
    pub target: Option<String>,
    /// Subject edge/anchor: left|right|top|bottom|center-x|center-y.
    #[serde(default)]
    pub edge: Option<String>,
    /// Target edge/anchor for relations/containment.
    #[serde(rename = "targetEdge", default)]
    pub target_edge: Option<String>,
    /// Gap in logical pixels (signed) for `gap`; ignored by `align`.
    #[serde(default)]
    pub value: Option<f32>,
    /// Hard constraints must not be silently relaxed.
    #[serde(default)]
    pub hard: bool,
    /// Soft-constraint preference weight.
    #[serde(default = "default_constraint_weight")]
    pub weight: f32,
}

fn default_constraint_weight() -> f32 {
    1.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
pub struct When {
    #[serde(rename = "minAspectRatio", default)]
    pub min_ar: Option<f32>,
    #[serde(rename = "maxAspectRatio", default)]
    pub max_ar: Option<f32>,
}

impl When {
    pub fn min(&self) -> f32 {
        self.min_ar.unwrap_or(0.0)
    }

    pub fn max(&self) -> f32 {
        self.max_ar.unwrap_or(f32::INFINITY)
    }

    pub fn contains(&self, ar: f32) -> bool {
        ar >= self.min() && ar <= self.max()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
pub struct Layout {
    #[serde(default)]
    pub anchor: Option<String>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    #[serde(rename = "minScale", default)]
    pub min_scale: Option<f32>,
    #[serde(rename = "maxScale", default)]
    pub max_scale: Option<f32>,
    /// Per-topology visibility toggle (independent of the element's global
    /// `visibility`). When true the element is hidden in this topology only.
    #[serde(default)]
    pub hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Fallback {
    pub resource: String,
    #[serde(default = "default_fit")]
    pub fit: String,
}

fn default_fit() -> String {
    "cover".to_string()
}
