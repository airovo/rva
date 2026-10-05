//! WebAssembly boundary for the RVA core.
//!
//! The core is the authority on *what* to draw. This module exposes it to
//! JavaScript so a web adapter can own *how* it is drawn:
//!
//! - `load(bytes)` parses and integrity-validates a `.rva` package.
//! - `RvaHandle.resolve(w, h)` returns the deterministic resolved scene as JSON.
//! - `RvaHandle.resource(id)` returns raw resource bytes for the adapter.
//!
//! Platform-neutral: no browser APIs are assumed here.

use rva_core::{Asset, Fonts, ResolvedScene};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parse a `.rva` package (or a directory asset is not possible in the browser).
#[wasm_bindgen]
pub fn load(bytes: &[u8]) -> Result<RvaHandle, JsValue> {
    let asset = Asset::from_bytes(bytes.to_vec()).map_err(err)?;
    Ok(RvaHandle {
        asset,
        fonts: Fonts::load_system(),
    })
}

#[wasm_bindgen]
pub struct RvaHandle {
    asset: Asset,
    fonts: Fonts,
}

#[wasm_bindgen]
impl RvaHandle {
    /// Human-readable asset summary.
    pub fn describe(&self) -> Result<String, JsValue> {
        Ok(self.asset.describe())
    }

    /// Resolve the asset for a viewport, returning the resolved scene as JSON.
    pub fn resolve(&self, width: u32, height: u32) -> Result<String, JsValue> {
        let resolved: ResolvedScene =
            rva_core::resolve(&self.asset, &self.fonts, width, height).map_err(err)?;
        serde_json::to_string(&resolved).map_err(err)
    }

    /// Raw bytes for a resource reference (declared id or raw path).
    pub fn resource(&self, name: &str) -> Result<Vec<u8>, JsValue> {
        self.asset.reference_bytes(name).map_err(err)
    }

    /// Whether a resource reference exists in the package.
    pub fn has_resource(&self, name: &str) -> bool {
        self.asset.has_reference(name)
    }

    /// The path a reference resolves to, for MIME detection.
    pub fn relative_for(&self, name: &str) -> String {
        self.asset.relative_for(name)
    }
}

fn err(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
