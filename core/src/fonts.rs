//! Deterministic text metrics for the resolver.
//!
//! Resolution must be identical across every runtime and machine:
//!
//! ```text
//! same .rva + same viewport + same spec version + same resolver profile
//!   = same ResolvedScene
//! ```
//!
//! So the resolver does NOT use system fonts. It measures text with a fixed,
//! platform-independent advance model. Real fonts (Core Text, browser, Android,
//! Skia) are used only by renderers when *drawing* the already-resolved text.
//!
//! This is the split the spec will need: layout-deterministic typography vs
//! renderer-dependent glyph rasterization. Changing this table is a
//! resolver-profile change and would bump the conformance profile.

/// Deterministic font-metrics context. Carries no state today; kept as a type so
/// renderers and adapters share one measurement entry point.
#[derive(Clone, Default)]
pub struct Fonts;

impl Fonts {
    pub fn load_system() -> Self {
        Fonts
    }

    /// Normalize a declared font family to an SVG-friendly generic.
    pub fn normalize_family(family: &str) -> String {
        let f = family.to_ascii_lowercase();
        if f.contains("system") || f.contains("sans") || f == "ui-sans-serif" {
            "sans-serif".to_string()
        } else if f.contains("serif") {
            "serif".to_string()
        } else if f.contains("mono") {
            "monospace".to_string()
        } else {
            family.to_string()
        }
    }

    /// Width in pixels of `text` at `size`, using the deterministic advance model.
    pub fn measure(&self, text: &str, size: f32, _family: &str, _weight: u16) -> f32 {
        text.chars().map(advance_ratio).sum::<f32>() * size
    }

    /// Width in pixels including letter and word spacing. Spacing is part of the
    /// deterministic model so line wrapping stays identical across runtimes.
    pub fn measure_styled(
        &self,
        text: &str,
        size: f32,
        _family: &str,
        _weight: u16,
        letter_spacing: f32,
        word_spacing: f32,
    ) -> f32 {
        let glyphs: Vec<char> = text.chars().collect();
        let base: f32 = glyphs.iter().map(|c| advance_ratio(*c)).sum::<f32>() * size;
        let letters = glyphs.len().saturating_sub(1) as f32 * letter_spacing.max(0.0);
        let spaces = glyphs.iter().filter(|c| **c == ' ').count() as f32 * word_spacing;
        base + letters + spaces
    }

    /// Ascent in pixels at `size` (first-baseline offset).
    pub fn ascent(&self, size: f32, _family: &str, _weight: u16) -> f32 {
        size * 0.8
    }
}

/// Per-character advance as a fraction of the em size. A fixed table (not a
/// system font) so measurement is bit-identical everywhere.
fn advance_ratio(c: char) -> f32 {
    match c {
        ' ' | '\t' => 0.28,
        'i' | 'l' | 'I' | 'j' | 't' | 'f' | 'r' | '.' | ',' | ':' | ';' | '!' | '|' | '\''
        | '`' => 0.30,
        '(' | ')' | '[' | ']' | '{' | '}' => 0.34,
        'm' | 'w' | 'M' | 'W' => 0.86,
        'A'..='Z' => 0.67,
        '0'..='9' => 0.56,
        // Broad script classes (deterministic, conservative).
        '\u{00C0}'..='\u{024F}' => 0.55, // Latin-1 supplement / extended
        '\u{0590}'..='\u{05FF}' => 0.55, // Hebrew
        '\u{0600}'..='\u{06FF}' => 0.52, // Arabic
        '\u{0900}'..='\u{097F}' => 0.62, // Devanagari
        '\u{0980}'..='\u{09FF}' => 0.64, // Bengali
        '\u{0E00}'..='\u{0E7F}' => 0.55, // Thai
        '\u{3040}'..='\u{30FF}' => 1.00, // Hiragana/Katakana
        '\u{4E00}'..='\u{9FFF}' => 1.00, // CJK unified
        '\u{AC00}'..='\u{D7A3}' => 1.00, // Hangul
        c if (c as u32) > 0x2FF => 0.62, // other non-ASCII
        _ => 0.53,
    }
}
