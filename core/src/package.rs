//! The `.rva` single-file container.
//!
//! Container policy (per the working specification):
//! - File extension: `.rva`
//! - Provisional media type: `application/x-rva`
//! - Target registered media type: `image/rva`
//! - Container: binary, self-contained
//! - Signature: reserved; to be finalized before public RVA 0.1
//!
//! Layout (little-endian):
//!
//! | offset | size | field                                   |
//! |-------:|-----:|-----------------------------------------|
//! | 0      | 4    | magic / provisional signature (`RVA1`)  |
//! | 4      | 2    | container major version                 |
//! | 6      | 2    | container minor version                 |
//! | 8      | 4    | flags (bit0: manifest zlib, bit1: data zlib) |
//! | 12     | 8    | reserved signature bytes (currently 0)  |
//! | 20     | 4    | manifest length (stored bytes)          |
//! | 24     | 8    | resource data length (stored bytes)     |
//! | 32     | ...  | manifest JSON (optionally zlib)         |
//! | ...    | ...  | concatenated resource blobs (optionally zlib) |
//!
//! The manifest is a forward-compatible JSON document: unknown optional fields
//! are ignored by readers. Every blob carries a SHA-256 digest for integrity
//! validation, and hard limits guard against pathological resource use.
//!
//! Compression: the manifest and the resource data region are each zlib-deflated
//! when that is smaller than storing them raw (flags bit0/bit1). Offsets/lengths
//! in the manifest are relative to the *uncompressed* data region, and digests
//! are over uncompressed resource bytes.

use crate::error::{Result, RvaError};
use crate::model::Scene;
use crate::resources::Asset;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
#[cfg(feature = "packaging")]
use image::{ExtendedColorType, ImageEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::sync::Arc;

pub const EXTENSION: &str = "rva";
pub const MEDIA_TYPE_PROVISIONAL: &str = "application/x-rva";
pub const MEDIA_TYPE_TARGET: &str = "image/rva";
/// Provisional magic. The final signature is reserved and must be frozen before
/// public RVA 0.1.
pub const MAGIC: &[u8; 4] = b"RVA1";
pub const SIGNATURE_POLICY: &str = "reserved; to be finalized before public RVA 0.1";
pub const CONTAINER_VERSION: &str = "1.1";

const CONTAINER_MAJOR: u16 = 1;
const HEADER_LEN: usize = 32;
const FLAG_MANIFEST_ZLIB: u32 = 1 << 0;
const FLAG_DATA_ZLIB: u32 = 1 << 1;

// Guardrails (see the "must define maximum/guardrail behavior" requirement).
const MAX_MANIFEST_BYTES: u64 = 64 * 1024 * 1024;
const MAX_BLOBS: usize = 4096;
const MAX_BLOB_BYTES: u64 = 512 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
    format: String,
    #[serde(rename = "formatVersion")]
    format_version: String,
    #[serde(rename = "containerVersion")]
    container_version: String,
    #[serde(rename = "mediaType")]
    media_type: String,
    signature: String,
    scene: Scene,
    blobs: Vec<Blob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Blob {
    id: String,
    path: String,
    offset: u64,
    length: u64,
    sha256: String,
}

/// In-memory index of a parsed package: the resource data region plus a map of
/// resource id to `(offset, length)`.
pub struct PackageIndex {
    pub data: Arc<Vec<u8>>,
    pub index: BTreeMap<String, (u64, u64)>,
}

/// Options controlling how a package is written.
#[derive(Debug, Clone, Copy)]
pub struct PackOptions {
    /// zlib-deflate the manifest and the resource data region when smaller.
    pub compress: bool,
    /// Re-encode PNG resources when a smaller encoding exists.
    pub optimize: bool,
    /// With `optimize`, restrict to lossless encodings (WebP only).
    pub lossless: bool,
    /// JPEG quality for opaque resources when optimizing (lossy).
    pub quality: u8,
}

impl Default for PackOptions {
    fn default() -> Self {
        Self {
            compress: true,
            optimize: false,
            lossless: false,
            quality: 85,
        }
    }
}

/// Serialize an asset into a `.rva` container (zlib-compressed, no transcoding).
pub fn pack(asset: &Asset) -> Result<Vec<u8>> {
    pack_with(asset, PackOptions::default())
}

/// Serialize an asset into a `.rva` container with explicit options.
pub fn pack_with(asset: &Asset, options: PackOptions) -> Result<Vec<u8>> {
    pack_scene_with(asset, &asset.scene.clone(), options)
}

/// Serialize an arbitrary scene against an asset's resource store. Used to
/// package a pruned scene (hidden layers stripped, unused resources dropped)
/// while reusing the already-loaded resource bytes.
pub fn pack_scene_with(asset: &Asset, scene: &Scene, options: PackOptions) -> Result<Vec<u8>> {
    let mut scene = scene.clone();

    // Promote a raw fallback path into a declared resource so the package is
    // fully self-describing and re-packable.
    let mut fallback_source: Option<String> = None;
    if let Some(fallback) = scene.fallback.clone() {
        if !scene.resources.contains_key(&fallback.resource) {
            let raw = fallback.resource.clone();
            let mut key = "fallback".to_string();
            if scene.resources.contains_key(&key) {
                key = "fallbackResource".to_string();
            }
            scene.resources.insert(key.clone(), raw.clone());
            if let Some(fb) = scene.fallback.as_mut() {
                fb.resource = key.clone();
            }
            fallback_source = Some(raw);
        }
    }

    // Resources referenced as element masks must not be lossily re-encoded.
    #[cfg(feature = "packaging")]
    let mask_ids: BTreeMap<&str, ()> = scene
        .elements
        .iter()
        .filter_map(|element| element.mask.as_deref())
        .map(|id| (id, ()))
        .collect();

    let mut blobs: Vec<Blob> = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    #[cfg(feature = "packaging")]
    let mut updated_paths: Vec<(String, String)> = Vec::new();
    #[cfg(feature = "packaging")]
    let design_width = scene
        .design_space
        .map(|space| space.width.max(1.0))
        .unwrap_or(1920.0);

    for (id, rel) in &scene.resources {
        let source = if id == "fallback" || id == "fallbackResource" {
            fallback_source.clone().unwrap_or_else(|| id.clone())
        } else {
            id.clone()
        };
        #[cfg(feature = "packaging")]
        let (mut path, mut bytes) = (rel.clone(), asset.reference_bytes(&source)?);
        #[cfg(not(feature = "packaging"))]
        let (path, bytes) = (rel.clone(), asset.reference_bytes(&source)?);

        #[cfg(feature = "packaging")]
        if options.optimize && !mask_ids.contains_key(id.as_str()) {
            let target = resource_target(&scene, id, design_width);
            if let Some((new_path, new_bytes)) = optimize_resource(&path, &bytes, target, options) {
                path = new_path;
                bytes = new_bytes;
                updated_paths.push((id.clone(), path.clone()));
            }
        }

        if bytes.len() as u64 > MAX_BLOB_BYTES {
            return Err(RvaError::Validation(format!(
                "resource '{id}' exceeds the maximum blob size"
            )));
        }
        if data.len() as u64 + bytes.len() as u64 > MAX_TOTAL_BYTES {
            return Err(RvaError::Validation(
                "package exceeds the maximum total resource size".into(),
            ));
        }

        let offset = data.len() as u64;
        let length = bytes.len() as u64;
        let sha256 = sha256_hex(&bytes);
        data.extend_from_slice(&bytes);
        blobs.push(Blob {
            id: id.clone(),
            path,
            offset,
            length,
            sha256,
        });
    }

    #[cfg(feature = "packaging")]
    for (id, path) in updated_paths {
        scene.resources.insert(id, path);
    }

    if blobs.len() > MAX_BLOBS {
        return Err(RvaError::Validation(format!(
            "package contains more than {MAX_BLOBS} resources"
        )));
    }

    let manifest = Manifest {
        format: scene.format.clone(),
        format_version: scene.format_version.clone(),
        container_version: CONTAINER_VERSION.to_string(),
        media_type: MEDIA_TYPE_PROVISIONAL.to_string(),
        signature: SIGNATURE_POLICY.to_string(),
        scene,
        blobs,
    };

    let manifest_bytes = serde_json::to_vec(&manifest)?;
    if manifest_bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(RvaError::Validation(
            "manifest exceeds the maximum size".into(),
        ));
    }

    let (manifest_stored, manifest_zlib) = compress_optional(manifest_bytes, options.compress)?;
    let (data_stored, data_zlib) = compress_optional(data, options.compress)?;

    let flags: u32 = (manifest_zlib as u32) | ((data_zlib as u32) << 1);

    let mut out = Vec::with_capacity(HEADER_LEN + manifest_stored.len() + data_stored.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&CONTAINER_MAJOR.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&[0u8; 8]);
    out.extend_from_slice(&(manifest_stored.len() as u32).to_le_bytes());
    out.extend_from_slice(&(data_stored.len() as u64).to_le_bytes());
    out.extend_from_slice(&manifest_stored);
    out.extend_from_slice(&data_stored);
    Ok(out)
}

fn compress_optional(bytes: Vec<u8>, enabled: bool) -> Result<(Vec<u8>, bool)> {
    if !enabled {
        return Ok((bytes, false));
    }
    let compressed = deflate(&bytes)?;
    if compressed.len() < bytes.len() {
        Ok((compressed, true))
    } else {
        Ok((bytes, false))
    }
}

fn deflate(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

fn inflate_limited(bytes: &[u8], limit: u64) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(bytes).take(limit.saturating_add(1));
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    if out.len() as u64 > limit {
        return Err(RvaError::Validation(
            "decompressed data exceeds the maximum size".into(),
        ));
    }
    Ok(out)
}

/// Largest pixel width a resource is ever displayed at, across all topologies
/// (`None` if the resource is not referenced by any element/background).
#[cfg(feature = "packaging")]
fn resource_target(scene: &Scene, id: &str, design_width: f32) -> Option<f32> {
    let mut max_width = 0.0f32;
    let mut found = false;

    for topology in &scene.topologies {
        if let Some(crate::model::Background::Resource(resource)) = &topology.background {
            if resource == id {
                max_width = max_width.max(design_width);
                found = true;
            }
        }
        for element in &scene.elements {
            let uses = element.id == id
                || element.mask.as_deref() == Some(id)
                || element.role.as_deref() == Some(id);
            if !uses {
                continue;
            }
            if let Some(layout) = topology.layout.get(&element.id) {
                max_width = max_width.max(layout.w * design_width);
                found = true;
            }
        }
    }

    if scene
        .fallback
        .as_ref()
        .is_some_and(|fallback| fallback.resource == id)
    {
        max_width = max_width.max(design_width);
        found = true;
    }

    found.then_some(max_width)
}

/// Downscale a raster to its displayed size (×2 for hi-dpi) and re-encode to the
/// smallest of lossless WebP / lossy WebP / JPEG. Returns `None` when nothing
/// improves on the original bytes.
#[cfg(feature = "packaging")]
fn optimize_resource(
    path: &str,
    bytes: &[u8],
    target_width: Option<f32>,
    options: PackOptions,
) -> Option<(String, Vec<u8>)> {
    // Oversample factor: never keep more than 2× the displayed pixels.
    const MAX_SCALE: f32 = 2.0;

    let decoded = image::load_from_memory(bytes).ok()?;
    let mut rgba = decoded.to_rgba8();
    let (orig_w, orig_h) = (rgba.width(), rgba.height());
    if orig_w == 0 || orig_h == 0 {
        return None;
    }

    let mut resized = false;
    if let Some(display_width) = target_width {
        let cap_w = (display_width * MAX_SCALE).ceil().max(1.0) as u32;
        if cap_w < orig_w {
            let cap_h = ((cap_w as f32) * (orig_h as f32) / (orig_w as f32))
                .round()
                .max(1.0) as u32;
            rgba =
                image::imageops::resize(&rgba, cap_w, cap_h, image::imageops::FilterType::Lanczos3);
            resized = true;
        }
    }
    let (width, height) = (rgba.width(), rgba.height());

    let mut best: Option<(&'static str, Vec<u8>)> = None;
    let mut consider = |ext: &'static str, candidate: Vec<u8>| {
        if candidate.len() >= bytes.len() {
            return;
        }
        // Without a resize, require a meaningful saving.
        if !resized && candidate.len() + 32 >= bytes.len() {
            return;
        }
        if best
            .as_ref()
            .is_none_or(|(_, current)| candidate.len() < current.len())
        {
            best = Some((ext, candidate));
        }
    };

    // Lossless WebP (works with alpha).
    if let Some(webp) = encode_webp_lossless(&rgba) {
        consider("webp", webp);
    }

    if !options.lossless {
        // Lossy WebP — preserves alpha and is by far the biggest win for
        // photographic rasters (including transparent cutouts).
        if let Some(webp) = encode_webp_lossy(&rgba, options.quality) {
            consider("webp", webp);
        }

        // Lossy JPEG for fully opaque images (no alpha channel needed).
        let opaque = rgba.pixels().all(|pixel| pixel.0[3] == 255);
        if opaque {
            if let Some(jpeg) = encode_jpeg(rgba.as_raw(), width, height, options.quality) {
                consider("jpg", jpeg);
            }
        }
    }

    best.map(|(ext, out)| {
        let new_path = std::path::Path::new(path)
            .with_extension(ext)
            .to_string_lossy()
            .replace('\\', "/");
        (new_path, out)
    })
}

#[cfg(feature = "packaging")]
fn encode_webp_lossy(rgba: &image::RgbaImage, quality: u8) -> Option<Vec<u8>> {
    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
    let encoded = encoder.encode(f32::from(quality.clamp(1, 100)));
    Some(encoded.to_vec())
}

#[cfg(feature = "packaging")]
fn encode_webp_lossless(rgba: &image::RgbaImage) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut out);
    encoder
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ExtendedColorType::Rgba8,
        )
        .ok()?;
    Some(out)
}

#[cfg(feature = "packaging")]
fn encode_jpeg(rgba: &[u8], width: u32, height: u32, quality: u8) -> Option<Vec<u8>> {
    let mut rgb = Vec::with_capacity((width as usize * height as usize) * 3);
    for pixel in rgba.as_chunks::<4>().0 {
        rgb.extend_from_slice(&pixel[..3]);
    }
    let mut out = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
    encoder
        .write_image(&rgb, width, height, ExtendedColorType::Rgb8)
        .ok()?;
    Some(out)
}

/// Parse and validate a `.rva` container, returning its scene and blob index.
pub fn read(bytes: Vec<u8>) -> Result<(Scene, PackageIndex)> {
    if bytes.len() < HEADER_LEN {
        return Err(RvaError::Validation(
            "file is too small to be an RVA package".into(),
        ));
    }
    if &bytes[0..4] != MAGIC {
        return Err(RvaError::Validation(
            "missing RVA signature; not an .rva package".into(),
        ));
    }

    let major = u16::from_le_bytes([bytes[4], bytes[5]]);
    let minor = u16::from_le_bytes([bytes[6], bytes[7]]);
    if major != CONTAINER_MAJOR {
        return Err(RvaError::Validation(format!(
            "unsupported container major version {major} (this build supports {CONTAINER_MAJOR}.x, file is {major}.{minor})"
        )));
    }

    let flags = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let manifest_len = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as usize;
    let data_len = u64::from_le_bytes([
        bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
    ]) as usize;

    if manifest_len as u64 > MAX_MANIFEST_BYTES {
        return Err(RvaError::Validation(
            "manifest exceeds the maximum size".into(),
        ));
    }

    let manifest_start = HEADER_LEN;
    let manifest_end = manifest_start
        .checked_add(manifest_len)
        .ok_or_else(|| RvaError::Validation("manifest length overflow".into()))?;
    let data_end = manifest_end
        .checked_add(data_len)
        .ok_or_else(|| RvaError::Validation("resource data length overflow".into()))?;

    if data_end > bytes.len() {
        return Err(RvaError::Validation(
            "package is truncated or has an inconsistent length".into(),
        ));
    }

    let manifest_stored = &bytes[manifest_start..manifest_end];
    let manifest_raw = if flags & FLAG_MANIFEST_ZLIB != 0 {
        inflate_limited(manifest_stored, MAX_MANIFEST_BYTES)?
    } else {
        manifest_stored.to_vec()
    };
    let manifest: Manifest = serde_json::from_slice(&manifest_raw).map_err(|error| {
        RvaError::Validation(format!(
            "manifest parse failed (flags={flags:#x}, manifest_zlib={}, raw_len={}): {error}",
            flags & FLAG_MANIFEST_ZLIB != 0,
            manifest_raw.len()
        ))
    })?;

    if manifest.blobs.len() > MAX_BLOBS {
        return Err(RvaError::Validation(format!(
            "package declares more than {MAX_BLOBS} resources"
        )));
    }

    let data_stored = &bytes[manifest_end..data_end];
    let data = Arc::new(if flags & FLAG_DATA_ZLIB != 0 {
        inflate_limited(data_stored, MAX_TOTAL_BYTES)?
    } else {
        data_stored.to_vec()
    });
    let mut index = BTreeMap::new();

    for blob in &manifest.blobs {
        let end = blob.offset.checked_add(blob.length).ok_or_else(|| {
            RvaError::Validation(format!("blob '{}' has an invalid range", blob.id))
        })?;
        if end > data.len() as u64 {
            return Err(RvaError::Validation(format!(
                "blob '{}' is out of bounds",
                blob.id
            )));
        }
        let slice = &data[blob.offset as usize..end as usize];
        let digest = sha256_hex(slice);
        if digest != blob.sha256 {
            return Err(RvaError::Validation(format!(
                "resource '{}' failed integrity validation",
                blob.id
            )));
        }
        index.insert(blob.id.clone(), (blob.offset, blob.length));
    }

    Ok((manifest.scene, PackageIndex { data, index }))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
