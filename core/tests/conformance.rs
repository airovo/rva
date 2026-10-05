use rva_core::{render_to_file, resolve, validate, Asset, Fonts};

fn demo_asset() -> Asset {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../rva-demo-assets");
    Asset::load(root).expect("demo asset loads")
}

#[test]
fn demo_asset_validates() {
    let issues = validate(&demo_asset());
    assert!(issues.is_empty(), "unexpected issues: {issues:?}");
}

#[test]
fn topology_selection_matrix() {
    let asset = demo_asset();
    let fonts = Fonts::load_system();

    let cases = [
        (1920, 500, "landscape"),
        (1280, 720, "landscape"),
        (2560, 800, "landscape"),
        (1080, 1080, "balanced"),
        (768, 1024, "portrait"),
        (430, 932, "portrait"),
    ];

    for (width, height, expected) in cases {
        let resolved = resolve(&asset, &fonts, width, height).unwrap();
        assert_eq!(
            resolved.topology, expected,
            "unexpected topology for {width}x{height}"
        );
        assert!(!resolved.items.is_empty());
        assert!(resolved.diagnostics.is_empty());
    }
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rva-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn package_renders_identically_to_directory() {
    let asset = demo_asset();
    let fonts = Fonts::load_system();

    let dir = scratch("pkg-parity");
    let from_dir = dir.join("from-dir.png");
    let from_pkg = dir.join("from-pkg.png");
    let resolved = resolve(&asset, &fonts, 1280, 720).unwrap();
    render_to_file(&asset, &resolved, &fonts, &from_dir).unwrap();

    let packed = asset.pack().unwrap();
    assert_eq!(
        packed,
        asset.pack().unwrap(),
        "packing must be deterministic"
    );

    let packaged = Asset::from_bytes(packed).unwrap();
    assert!(packaged.is_packaged());
    let resolved_pkg = resolve(&packaged, &fonts, 1280, 720).unwrap();
    render_to_file(&packaged, &resolved_pkg, &fonts, &from_pkg).unwrap();

    assert_eq!(
        std::fs::read(&from_dir).unwrap(),
        std::fs::read(&from_pkg).unwrap(),
        "package and directory renders must be byte-identical"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compression_and_optimization_round_trip() {
    use rva_core::package::{pack_with, PackOptions};

    let asset = demo_asset();
    let fonts = Fonts::load_system();

    let plain = pack_with(
        &asset,
        PackOptions {
            compress: false,
            optimize: false,
            ..Default::default()
        },
    )
    .unwrap();
    let optimized = pack_with(
        &asset,
        PackOptions {
            compress: true,
            optimize: true,
            lossless: false,
            quality: 85,
        },
    )
    .unwrap();

    assert!(
        optimized.len() < plain.len(),
        "optimized ({} B) should beat plain ({} B)",
        optimized.len(),
        plain.len()
    );

    // The optimized package still opens, resolves, and renders.
    let reopened = Asset::from_bytes(optimized).unwrap();
    assert!(reopened.has_reference("bgLandscape"));
    let scene = resolve(&reopened, &fonts, 1080, 1080).unwrap();
    assert_eq!(scene.items.len(), 8);
    let png = rva_core::render_to_png(&reopened, &scene, &fonts).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn tampered_package_is_rejected() {
    let asset = demo_asset();
    let mut bytes = asset.pack().unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    assert!(
        Asset::from_bytes(bytes).is_err(),
        "integrity validation must reject modified resource data"
    );
}

#[test]
fn unpack_round_trip_preserves_scene() {
    let asset = demo_asset();
    let packaged = Asset::from_bytes(asset.pack().unwrap()).unwrap();

    let dir = scratch("unpack");
    packaged.unpack_to(&dir).unwrap();
    let reloaded = Asset::load(&dir).unwrap();

    let fonts = Fonts::load_system();
    let first = resolve(&packaged, &fonts, 1080, 1080).unwrap();
    let second = resolve(&reloaded, &fonts, 1080, 1080).unwrap();
    assert_eq!(first.topology, second.topology);
    assert_eq!(first.items.len(), second.items.len());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn overlapping_ranges_resolve_deterministically() {
    let asset = demo_asset();
    let fonts = Fonts::load_system();

    // Aspect 1.35 is claimed by both `landscape` (>=1.35) and `balanced`
    // (<=1.35). Selection is deterministic (by viability cost, then order).
    let first = resolve(&asset, &fonts, 1350, 1000).unwrap();
    let second = resolve(&asset, &fonts, 1350, 1000).unwrap();
    assert_eq!(first.topology, second.topology);
    assert!(first.topology == "landscape" || first.topology == "balanced");
    assert!(first.viability >= rva_core::solver::MIN_VIABILITY);
}

#[test]
fn extreme_aspects_stay_viable() {
    let asset = demo_asset();
    let fonts = Fonts::load_system();

    for (width, height) in [(2100, 600), (500, 1400)] {
        let resolved = resolve(&asset, &fonts, width, height).unwrap();
        assert!(
            resolved.viability >= rva_core::solver::MIN_VIABILITY,
            "{width}x{height} dropped below the viability threshold"
        );
        assert!(!resolved.topology.is_empty());
    }
}

#[test]
fn render_is_deterministic() {
    let asset = demo_asset();
    let fonts = Fonts::load_system();
    let resolved = resolve(&asset, &fonts, 1080, 1080).unwrap();

    let dir = std::env::temp_dir().join(format!("rva-conformance-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let first = dir.join("first.png");
    let second = dir.join("second.png");

    render_to_file(&asset, &resolved, &fonts, &first).unwrap();
    render_to_file(&asset, &resolved, &fonts, &second).unwrap();

    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );

    let _ = std::fs::remove_dir_all(&dir);
}
