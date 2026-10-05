//! RVA server-side renderer.
//!
//! Uses the reference core directly (native Rust, no WASM) to resolve and
//! rasterize `.rva` assets for explicit dimensions — the server/CDN side of the
//! PRD's delivery model. Rendered PNGs are cached by `(width, height)`.

use rva_core::{render_to_png, resolve, Asset, Fonts, ResolvedScene};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

const MAX_CACHE_ENTRIES: usize = 64;

/// A loaded asset plus a font context and a small render cache.
pub struct Renderer {
    asset: Asset,
    fonts: Fonts,
    cache: Mutex<HashMap<(u32, u32), Arc<Vec<u8>>>>,
}

impl Renderer {
    /// Load from `.rva` package bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> rva_core::Result<Self> {
        Ok(Self {
            asset: Asset::from_bytes(bytes)?,
            fonts: Fonts::load_system(),
            cache: Mutex::new(HashMap::new()),
        })
    }

    /// Load from a `.rva` file on disk.
    pub fn from_path(path: impl AsRef<Path>) -> rva_core::Result<Self> {
        Self::from_bytes(std::fs::read(path)?)
    }

    pub fn describe(&self) -> String {
        self.asset.describe()
    }

    /// Resolve the scene for a viewport (what to draw).
    pub fn resolve(&self, width: u32, height: u32) -> rva_core::Result<ResolvedScene> {
        resolve(&self.asset, &self.fonts, width, height)
    }

    /// Resolve and rasterize to PNG bytes (cached).
    pub fn render_png(&self, width: u32, height: u32) -> rva_core::Result<Arc<Vec<u8>>> {
        if let Some(hit) = self
            .cache
            .lock()
            .expect("render cache poisoned")
            .get(&(width, height))
            .cloned()
        {
            return Ok(hit);
        }

        let scene = self.resolve(width, height)?;
        let bytes = Arc::new(render_to_png(&self.asset, &scene, &self.fonts)?);

        let mut cache = self.cache.lock().expect("render cache poisoned");
        if cache.len() >= MAX_CACHE_ENTRIES {
            cache.clear();
        }
        cache.insert((width, height), Arc::clone(&bytes));
        Ok(bytes)
    }

    /// Number of cached renders (for diagnostics/tests).
    pub fn cache_len(&self) -> usize {
        self.cache.lock().expect("render cache poisoned").len()
    }
}
