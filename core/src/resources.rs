use crate::error::{Result, RvaError};
use crate::model::Scene;
use crate::package;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Where an asset's resource bytes come from: an unpacked directory or a
/// self-contained `.rva` package held in memory.
enum Store {
    Dir(PathBuf),
    Memory {
        data: Arc<Vec<u8>>,
        index: BTreeMap<String, (u64, u64)>,
    },
}

/// A loaded RVA asset. Resources are resolved through the store so that a
/// directory asset and a packaged asset render through the exact same code path.
pub struct Asset {
    pub scene: Scene,
    store: Store,
}

impl Asset {
    /// Load an asset from a package directory or a path to a `scene.json` file.
    ///
    /// Resource paths are resolved relative to the package root. When given a
    /// directory, the loader first looks for `scene.json` at the root and then
    /// in immediate subdirectories (the demo asset keeps it under
    /// `05_examples/`).
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let (root, scene_path) = if path.is_file() {
            let root = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            (root, path.to_path_buf())
        } else {
            let scene_path = find_scene(path).ok_or_else(|| {
                RvaError::Validation(format!("no scene.json found under {}", path.display()))
            })?;
            (path.to_path_buf(), scene_path)
        };

        let raw = std::fs::read_to_string(&scene_path)?;
        let scene: Scene = serde_json::from_str(&raw)?;
        Ok(Asset {
            scene,
            store: Store::Dir(root),
        })
    }

    /// Open either a directory asset or a `.rva` package, detected by extension
    /// or by the container magic. A plain `scene.json` file is loaded as a
    /// directory-rooted asset.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.is_file() {
            let bytes = std::fs::read(path)?;
            let is_rva = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case(package::EXTENSION))
                .unwrap_or(false);
            if is_rva || bytes.starts_with(package::MAGIC) {
                return Asset::from_bytes(bytes);
            }
        }
        Asset::load(path)
    }

    /// Construct an asset directly from `.rva` container bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let (scene, index) = package::read(bytes)?;
        Ok(Asset {
            scene,
            store: Store::Memory {
                data: index.data,
                index: index.index,
            },
        })
    }

    /// Serialize this asset into a self-contained `.rva` package.
    pub fn pack(&self) -> Result<Vec<u8>> {
        package::pack(self)
    }

    pub fn is_packaged(&self) -> bool {
        matches!(self.store, Store::Memory { .. })
    }

    /// Human-readable asset summary shared by all adapters.
    pub fn describe(&self) -> String {
        let scene = &self.scene;
        format!(
            "{} {} · {} resource(s) · {} element(s) · {} topology(ies)",
            scene
                .name
                .clone()
                .unwrap_or_else(|| "RVA asset".to_string()),
            scene.format_version,
            scene.resources.len(),
            scene.elements.len(),
            scene.topologies.len(),
        )
    }

    /// The path a reference resolves to, used for format/MIME detection.
    pub fn relative_for(&self, name: &str) -> String {
        self.scene
            .resources
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    /// Read the bytes behind a reference that may be a declared resource id or
    /// a raw path relative to the root (the latter only for directory assets).
    pub fn reference_bytes(&self, name: &str) -> Result<Vec<u8>> {
        match &self.store {
            Store::Dir(root) => {
                let rel = self.relative_for(name);
                Ok(std::fs::read(root.join(rel))?)
            }
            Store::Memory { data, index } => {
                let (offset, length) = index.get(name).copied().ok_or_else(|| {
                    RvaError::Validation(format!("resource '{name}' not found in package"))
                })?;
                let start = offset as usize;
                let end = start + length as usize;
                if end > data.len() {
                    return Err(RvaError::Validation(format!(
                        "resource '{name}' is out of bounds"
                    )));
                }
                Ok(data[start..end].to_vec())
            }
        }
    }

    /// Read the bytes of a declared resource id.
    pub fn resource_bytes(&self, id: &str) -> Result<Vec<u8>> {
        if !self.scene.resources.contains_key(id) {
            return Err(RvaError::Validation(format!(
                "resource id '{id}' is not declared"
            )));
        }
        self.reference_bytes(id)
    }

    /// Whether a reference can be resolved with the current store.
    pub fn has_reference(&self, name: &str) -> bool {
        match &self.store {
            Store::Dir(root) => {
                let rel = self.relative_for(name);
                root.join(rel).exists()
            }
            Store::Memory { index, .. } => index.contains_key(name),
        }
    }

    /// Intrinsic pixel size of a declared resource. Supports raster formats via
    /// the `image` crate and SVGs via width/height/viewBox parsing.
    pub fn intrinsic_size(&self, id: &str) -> Result<(f32, f32)> {
        let bytes = self.reference_bytes(id)?;
        let rel = self.relative_for(id);
        let ext = extension_of(&rel);

        match ext.as_str() {
            "svg" => parse_svg_intrinsic(&String::from_utf8_lossy(&bytes)).ok_or_else(|| {
                RvaError::Validation(format!("could not determine SVG size for '{id}'"))
            }),
            _ => {
                let size = imagesize::blob_size(&bytes).map_err(|e| {
                    RvaError::Validation(format!("could not read image '{id}': {e}"))
                })?;
                Ok((size.width as f32, size.height as f32))
            }
        }
    }

    /// Extract this asset back into a directory form (scene.json + resources).
    pub fn unpack_to(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        for (id, rel) in &self.scene.resources {
            let bytes = self.resource_bytes(id)?;
            let path = dir.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, bytes)?;
        }

        let scene_json = serde_json::to_vec_pretty(&self.scene)?;
        std::fs::write(dir.join("scene.json"), scene_json)?;
        Ok(())
    }
}

fn extension_of(rel: &str) -> String {
    Path::new(rel)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Locate `scene.json` at a package root or one level below it.
fn find_scene(root: &Path) -> Option<PathBuf> {
    let direct = root.join("scene.json");
    if direct.exists() {
        return Some(direct);
    }

    let mut subdirs: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    subdirs.sort();
    subdirs
        .into_iter()
        .map(|dir| dir.join("scene.json"))
        .find(|candidate| candidate.exists())
}

pub fn parse_svg_intrinsic(svg: &str) -> Option<(f32, f32)> {
    if let Some(vb) = attr(svg, "viewBox") {
        let nums: Vec<f32> = vb
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<f32>().ok())
            .collect();
        if nums.len() == 4 && nums[2] > 0.0 && nums[3] > 0.0 {
            return Some((nums[2], nums[3]));
        }
    }
    let w = attr(svg, "width").and_then(|s| parse_len(&s));
    let h = attr(svg, "height").and_then(|s| parse_len(&s));
    match (w, h) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}

fn parse_len(s: &str) -> Option<f32> {
    let trimmed = s.trim().trim_end_matches("px").trim();
    trimmed.parse::<f32>().ok()
}

/// Find the value of the first `name="..."` attribute in an SVG root.
fn attr(svg: &str, name: &str) -> Option<String> {
    let head = &svg[..svg.len().min(4096)];
    let mut search_from = 0usize;
    while let Some(pos) = head[search_from..].find(name) {
        let abs = search_from + pos;
        let after = &head[abs + name.len()..];
        let after_trimmed = after.trim_start();
        if let Some(rest) = after_trimmed.strip_prefix('=') {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('"') {
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            } else if let Some(rest) = rest.strip_prefix('\'') {
                if let Some(end) = rest.find('\'') {
                    return Some(rest[..end].to_string());
                }
            }
        }
        search_from = abs + name.len();
    }
    None
}

/// Return the inner markup of an SVG document (everything between the root
/// `<svg ...>` open tag and the closing `</svg>`).
pub fn svg_inner(svg: &str) -> &str {
    let open_end = svg.find('>').map(|i| i + 1).unwrap_or(0);
    let close_start = svg.rfind("</svg>").unwrap_or(svg.len());
    if open_end >= close_start {
        return "";
    }
    &svg[open_end..close_start]
}
