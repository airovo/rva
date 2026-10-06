use crate::error::{Result, RvaError};
use crate::fonts::Fonts;
use crate::model::Paint;
use crate::resolve::{ItemKind, ResolvedScene};
use crate::resources::{svg_inner, Asset};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use std::path::Path;

/// Rasterize a resolved scene to PNG bytes using a deterministic reference
/// renderer (resvg + tiny-skia). Used by the CLI, server and native adapters.
pub fn render_to_png(asset: &Asset, resolved: &ResolvedScene, _fonts: &Fonts) -> Result<Vec<u8>> {
    let svg = build_svg(asset, resolved)?;

    // Rendering uses the platform font stack; resolution did not.
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    let options = usvg::Options {
        fontdb: std::sync::Arc::new(fontdb),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg, &options)
        .map_err(|e| RvaError::Render(format!("svg parse failed: {e}")))?;

    let mut pixmap = tiny_skia::Pixmap::new(resolved.width, resolved.height)
        .ok_or_else(|| RvaError::Render("invalid canvas size".into()))?;

    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    pixmap
        .encode_png()
        .map_err(|e| RvaError::Render(format!("png encode failed: {e}")))
}

/// Rasterize a resolved scene to a PNG file.
pub fn render_to_file(
    asset: &Asset,
    resolved: &ResolvedScene,
    fonts: &Fonts,
    out: &Path,
) -> Result<()> {
    let bytes = render_to_png(asset, resolved, fonts)?;
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(out, bytes)?;
    Ok(())
}

pub fn build_svg(asset: &Asset, resolved: &ResolvedScene) -> Result<String> {
    let w = resolved.width;
    let h = resolved.height;
    let mut defs = String::new();
    let mut body = String::new();

    // Optional solid base colour behind the background (else transparent).
    let base_rect = resolved
        .base_color
        .as_ref()
        .map(|color| format!("<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{color}\"/>"))
        .unwrap_or_default();

    if let Some(background) = &resolved.background {
        match &background.kind {
            ItemKind::Image { resource, fit, focus } => {
                let uri = data_uri(
                    &asset.reference_bytes(resource)?,
                    &asset.relative_for(resource),
                );
                let (iw, ih) = asset.intrinsic_size(resource).unwrap_or((0.0, 0.0));
                if fit == "cover" && iw > 0.0 && ih > 0.0 {
                    // Pan via focus instead of always centering.
                    let fx = focus.map(|f| f.x).unwrap_or(0.5);
                    let fy = focus.map(|f| f.y).unwrap_or(0.5);
                    let scale = (w as f32 / iw).max(h as f32 / ih);
                    let dw = iw * scale;
                    let dh = ih * scale;
                    let dx = (w as f32 - dw) * fx;
                    let dy = (h as f32 - dh) * fy;
                    defs.push_str(&format!(
                        "<clipPath id=\"bgclip\"><rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\"/></clipPath>"
                    ));
                    body.push_str(&format!(
                        "<g clip-path=\"url(#bgclip)\"><image x=\"{dx}\" y=\"{dy}\" width=\"{dw}\" height=\"{dh}\" preserveAspectRatio=\"none\" href=\"{uri}\" xlink:href=\"{uri}\"/></g>"
                    ));
                } else {
                    let par = if fit == "cover" { "xMidYMid slice" } else { "none" };
                    body.push_str(&format!(
                        "<image x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" preserveAspectRatio=\"{par}\" href=\"{uri}\" xlink:href=\"{uri}\"/>"
                    ));
                }
            }
            ItemKind::Paint { paint } => {
                let fill = paint_fill(paint, &mut defs, "bg");
                body.push_str(&format!(
                    "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"{fill}\"/>"
                ));
            }
            _ => {}
        }
    }

    for item in &resolved.items {
        match &item.kind {
            ItemKind::Image { resource, .. } => {
                let uri = data_uri(
                    &asset.reference_bytes(resource)?,
                    &asset.relative_for(resource),
                );
                let image = format!(
                    "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\" xlink:href=\"{}\"/>",
                    item.x, item.y, item.w, item.h, uri, uri
                );
                if let Some(mask) = &item.mask {
                    let id = format!("m-{}", item.id);
                    let mask_uri =
                        data_uri(&asset.reference_bytes(mask)?, &asset.relative_for(mask));
                    defs.push_str(&format!(
                        "<mask id=\"{id}\" maskUnits=\"userSpaceOnUse\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\">\
                         <image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{}\" xlink:href=\"{}\"/></mask>",
                        item.x, item.y, item.w, item.h, item.x, item.y, item.w, item.h, mask_uri, mask_uri
                    ));
                    body.push_str(&format!("<g mask=\"url(#{id})\">{image}</g>"));
                } else {
                    body.push_str(&image);
                }
            }
            ItemKind::Vector { resource } => {
                let bytes = asset.reference_bytes(resource)?;
                let text = String::from_utf8_lossy(&bytes);
                let (iw, _ih) = asset.intrinsic_size(resource)?;
                let scale = if iw > 0.0 { item.w / iw } else { 1.0 };
                body.push_str(&format!(
                    "<g transform=\"translate({},{}) scale({})\">{}</g>",
                    item.x,
                    item.y,
                    scale,
                    svg_inner(&text)
                ));
            }
            ItemKind::Text {
                lines,
                size,
                font_family,
                weight,
                line_height,
                ascent,
                color,
                fill: fill_paint,
                background,
                letter_spacing,
                word_spacing,
                align,
                ..
            } => {
                let family = Fonts::normalize_family(font_family);
                let ascent = *ascent;
                let fill = match fill_paint {
                    Some(paint) => paint_fill(paint, &mut defs, &item.id),
                    None => color.clone().unwrap_or_else(|| match item.role.as_deref() {
                        Some("subheadline") => "#334155".to_string(),
                        Some("headline") => "#0F172A".to_string(),
                        _ => "#0F172A".to_string(),
                    }),
                };

                if let Some(background) = background {
                    body.push_str(&format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
                        item.x, item.y, item.w, item.h, background
                    ));
                }

                let anchor = match align.as_str() {
                    "center" => "middle",
                    "right" => "end",
                    _ => "start",
                };
                let text_x = match anchor {
                    "middle" => item.x + item.w / 2.0,
                    "end" => item.x + item.w,
                    _ => item.x,
                };
                let spacing = format!(
                    " letter-spacing=\"{}\" word-spacing=\"{}\"",
                    letter_spacing, word_spacing
                );
                for (index, line) in lines.iter().enumerate() {
                    let baseline = item.y + ascent + index as f32 * line_height;
                    body.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" fill=\"{}\" text-anchor=\"{}\"{} xml:space=\"preserve\">{}</text>",
                        text_x,
                        baseline,
                        family,
                        size,
                        weight,
                        fill,
                        anchor,
                        spacing,
                        escape_xml(line)
                    ));
                }
            }
            ItemKind::Paint { paint } => {
                let fill = paint_fill(paint, &mut defs, &item.id);
                body.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
                    item.x, item.y, item.w, item.h, fill
                ));
            }
        }
    }

    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\
         <defs>{defs}</defs>\
         {base_rect}\
         {body}</svg>"
    ))
}

/// Resolve a paint into an SVG `fill` value, registering gradient defs as needed.
fn paint_fill(paint: &Paint, defs: &mut String, id_prefix: &str) -> String {
    match paint {
        Paint::Color { color } => color.clone(),
        Paint::LinearGradient { angle, stops } => {
            let safe: String = id_prefix
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect();
            let id = format!("g-{safe}");
            let rad = angle.to_radians();
            let x1 = 0.5 - 0.5 * rad.cos();
            let y1 = 0.5 - 0.5 * rad.sin();
            let x2 = 0.5 + 0.5 * rad.cos();
            let y2 = 0.5 + 0.5 * rad.sin();
            let mut stops_svg = String::new();
            for stop in stops {
                stops_svg.push_str(&format!(
                    "<stop offset=\"{}\" stop-color=\"{}\"/>",
                    stop.offset.clamp(0.0, 1.0),
                    stop.color
                ));
            }
            defs.push_str(&format!(
                "<linearGradient id=\"{id}\" x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\">{stops_svg}</linearGradient>"
            ));
            format!("url(#{id})")
        }
        Paint::RadialGradient { stops } => {
            let safe: String = id_prefix
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect();
            let id = format!("rg-{safe}");
            let mut stops_svg = String::new();
            for stop in stops {
                stops_svg.push_str(&format!(
                    "<stop offset=\"{}\" stop-color=\"{}\"/>",
                    stop.offset.clamp(0.0, 1.0),
                    stop.color
                ));
            }
            defs.push_str(&format!(
                "<radialGradient id=\"{id}\" cx=\"0.5\" cy=\"0.5\" r=\"0.5\">{stops_svg}</radialGradient>"
            ));
            format!("url(#{id})")
        }
    }
}

fn data_uri(bytes: &[u8], relative: &str) -> String {
    let mime = match Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("svg") => "image/svg+xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("gif") => "image/gif",
        _ => "image/png",
    };
    format!("data:{mime};base64,{}", STANDARD.encode(bytes))
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
