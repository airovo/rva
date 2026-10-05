use rva_server::Renderer;

fn hero() -> Renderer {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/node/hero.rva");
    Renderer::from_path(path).expect("load hero.rva")
}

#[test]
fn resolves_and_renders_png() {
    let renderer = hero();
    assert!(renderer.describe().contains("Nexa Hero Demo"));

    let scene = renderer.resolve(1080, 1080).unwrap();
    assert_eq!(scene.topology, "balanced");
    assert_eq!(scene.items.len(), 8);

    let png = renderer.render_png(1080, 1080).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(renderer.cache_len(), 1);

    // A second call is served from cache.
    let again = renderer.render_png(1080, 1080).unwrap();
    assert!(std::sync::Arc::ptr_eq(&png, &again));
    assert_eq!(renderer.cache_len(), 1);
}
