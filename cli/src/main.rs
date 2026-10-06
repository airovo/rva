use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use rva_core::{render_to_file, resolve, Asset, Fonts, ItemKind, ResolvedItem, ResolvedScene};
use std::path::{Path, PathBuf};

/// RVA developer CLI: validate, inspect and render Responsive Visual Assets.
#[derive(Parser)]
#[command(name = "rva", version, about)]
struct Cli {
    /// Asset directory, a scene.json file, or a packaged .rva file.
    #[arg(long, global = true, default_value = "rva-demo-assets")]
    asset: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate the asset structure and referenced resources.
    Validate,
    /// Describe the asset, or resolve a concrete viewport to a scene graph.
    Inspect {
        /// Logical viewport width in pixels.
        #[arg(long)]
        width: Option<u32>,
        /// Logical viewport height in pixels.
        #[arg(long)]
        height: Option<u32>,
        /// Emit compact JSON (byte-stable; used by conformance).
        #[arg(long)]
        compact: bool,
    },
    /// Resolve and rasterize the asset for a viewport, writing a PNG.
    Render {
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        /// Output PNG path.
        #[arg(long)]
        out: PathBuf,
        /// Render the canonical fallback representation instead of the live
        /// composition.
        #[arg(long)]
        fallback: bool,
    },
    /// Package the asset into a self-contained .rva file.
    Pack {
        /// Output .rva path.
        #[arg(long)]
        out: PathBuf,
        /// Re-encode PNG resources (lossless WebP, or JPEG for opaque images).
        #[arg(long)]
        optimize: bool,
        /// With --optimize, only use lossless encodings (skip JPEG).
        #[arg(long)]
        lossless: bool,
        /// JPEG quality for --optimize (1-100).
        #[arg(long, default_value_t = 85)]
        quality: u8,
        /// Disable zlib compression of the manifest and resource data.
        #[arg(long)]
        no_compress: bool,
    },
    /// Extract a .rva package into a directory.
    Unpack {
        /// Output directory.
        #[arg(long)]
        out_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let asset = Asset::open(&cli.asset)
        .with_context(|| format!("failed to open asset at {}", cli.asset.display()))?;

    match cli.command {
        Command::Validate => validate(&asset),
        Command::Inspect {
            width,
            height,
            compact,
        } => inspect(&asset, width, height, compact),
        Command::Render {
            width,
            height,
            out,
            fallback,
        } => render(&asset, width, height, &out, fallback),
        Command::Pack {
            out,
            optimize,
            lossless,
            quality,
            no_compress,
        } => pack(&asset, &out, optimize, lossless, quality, no_compress),
        Command::Unpack { out_dir } => unpack(&asset, &out_dir),
    }
}

fn pack(
    asset: &Asset,
    out: &Path,
    optimize: bool,
    lossless: bool,
    quality: u8,
    no_compress: bool,
) -> Result<()> {
    let options = rva_core::package::PackOptions {
        compress: !no_compress,
        optimize,
        lossless,
        quality,
    };
    let bytes = rva_core::package::pack_with(asset, options).context("failed to pack asset")?;

    let mut raw_total: u64 = asset
        .scene
        .resources
        .keys()
        .filter_map(|id| asset.reference_bytes(id).ok())
        .map(|bytes| bytes.len() as u64)
        .sum();
    if let Some(fallback) = &asset.scene.fallback {
        if !asset.scene.resources.contains_key(&fallback.resource) {
            raw_total += asset
                .reference_bytes(&fallback.resource)
                .map(|bytes| bytes.len() as u64)
                .unwrap_or(0);
        }
    }

    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(out, &bytes).with_context(|| format!("failed to write {}", out.display()))?;

    let ratio = if raw_total > 0 {
        100.0 * bytes.len() as f64 / raw_total as f64
    } else {
        100.0
    };
    println!(
        "packed {} resource(s) → {} ({} bytes; raw {} bytes; {:.0}% of raw{}{})",
        asset.scene.resources.len(),
        out.display(),
        bytes.len(),
        raw_total,
        ratio,
        if optimize { "; webp" } else { "" },
        if no_compress { "; uncompressed" } else { "" },
    );
    Ok(())
}

fn unpack(asset: &Asset, out_dir: &Path) -> Result<()> {
    if !asset.is_packaged() {
        bail!("--asset is not a packaged .rva file");
    }
    asset
        .unpack_to(out_dir)
        .with_context(|| format!("failed to unpack into {}", out_dir.display()))?;
    println!(
        "unpacked {} resource(s) → {}",
        asset.scene.resources.len(),
        out_dir.display()
    );
    Ok(())
}

fn validate(asset: &Asset) -> Result<()> {
    let issues = rva_core::validate(asset);
    if issues.is_empty() {
        println!(
            "OK  {} · {} resource(s) · {} element(s) · {} topology(ies)",
            asset
                .scene
                .name
                .clone()
                .unwrap_or_else(|| "RVA asset".to_string()),
            asset.scene.resources.len(),
            asset.scene.elements.len(),
            asset.scene.topologies.len(),
        );
        return Ok(());
    }

    eprintln!("{} issue(s):", issues.len());
    for issue in &issues {
        eprintln!("  - {issue}");
    }
    std::process::exit(1);
}

fn inspect(asset: &Asset, width: Option<u32>, height: Option<u32>, compact: bool) -> Result<()> {
    match (width, height) {
        (Some(width), Some(height)) => {
            let fonts = Fonts::load_system();
            let resolved = resolve(asset, &fonts, width, height)?;
            let json = if compact {
                serde_json::to_string(&resolved)?
            } else {
                serde_json::to_string_pretty(&resolved)?
            };
            println!("{json}");
        }
        (None, None) => describe(asset),
        _ => bail!("--width and --height must be provided together"),
    }
    Ok(())
}

fn describe(asset: &Asset) {
    let scene = &asset.scene;
    println!(
        "{} {}",
        scene
            .name
            .clone()
            .unwrap_or_else(|| "RVA asset".to_string()),
        scene.format_version
    );
    if let Some(description) = &scene.description {
        println!("{description}");
    }
    if let Some(space) = scene.design_space {
        println!("design space: {} x {}", space.width, space.height);
    }
    println!("resources: {}", scene.resources.len());
    println!("elements:");
    for element in &scene.elements {
        println!(
            "  - {:<12} {:<8} role={}",
            element.id,
            element.kind,
            element.role.clone().unwrap_or_else(|| "-".to_string())
        );
    }
    println!("topologies:");
    for topology in &scene.topologies {
        let min = topology.when.min_ar.unwrap_or(0.0);
        let max = topology.when.max_ar.unwrap_or(f32::INFINITY);
        println!("  - {:<10} aspect {:.2}..{}", topology.id, min, max);
    }
}

fn render(asset: &Asset, width: u32, height: u32, out: &Path, fallback: bool) -> Result<()> {
    let fonts = Fonts::load_system();
    let mut resolved = resolve(asset, &fonts, width, height)?;

    if fallback {
        resolved = fallback_scene(asset, width, height)?;
    }

    render_to_file(asset, &resolved, &fonts, out)
        .with_context(|| format!("failed to render {}", out.display()))?;

    println!(
        "{} → {} (topology '{}', viability {:.2}, {} item(s), {}x{})",
        asset
            .scene
            .name
            .clone()
            .unwrap_or_else(|| "RVA asset".to_string()),
        out.display(),
        resolved.topology,
        resolved.viability,
        resolved.items.len(),
        width,
        height,
    );
    for note in &resolved.diagnostics {
        eprintln!("  note: {note}");
    }
    Ok(())
}

/// Build a degenerate resolved scene that draws only the canonical fallback.
fn fallback_scene(asset: &Asset, width: u32, height: u32) -> Result<ResolvedScene> {
    let fallback = asset
        .scene
        .fallback
        .as_ref()
        .context("asset declares no fallback")?;

    Ok(ResolvedScene {
        topology: "fallback".to_string(),
        width,
        height,
        topology_fallback: false,
        degraded: false,
        hidden: Vec::new(),
        viability: 0.0,
        base_color: None,
        background: Some(ResolvedItem {
            id: "fallback".to_string(),
            role: Some("fallback".to_string()),
            z: -1,
            x: 0.0,
            y: 0.0,
            w: width as f32,
            h: height as f32,
            opacity: 1.0,
            mask: None,
            priority: 100.0,
            crop_policy: Some(fallback.fit.clone()),
            focal_regions: Vec::new(),
            kind: ItemKind::Image {
                resource: fallback.resource.clone(),
                fit: fallback.fit.clone(),
                focus: None,
            },
        }),
        items: Vec::new(),
        diagnostics: vec!["rendered canonical fallback representation".to_string()],
    })
}
