//! Generate TypeScript definitions from the Rust model so the web/Node/React
//! adapters cannot drift from the core.
//!
//!   cargo run -p rva-core --bin export_types -- adapters/types/src/generated

use rva_core::{ResolvedScene, Scene};
use ts_rs::TS;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../adapters/types/src/generated".to_string());
    std::fs::create_dir_all(&dir)?;

    Scene::export_all_to(&dir)?;
    ResolvedScene::export_all_to(&dir)?;

    println!("TypeScript bindings written to {dir}");
    Ok(())
}
