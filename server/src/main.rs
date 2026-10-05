//! `rva-server` — serve a `.rva` asset over HTTP for explicit dimensions.
//!
//!   rva-server --asset hero.rva --addr 127.0.0.1:8787
//!
//!   GET /health
//!   GET /describe/:id
//!   GET /resolve/:id?w=1280&h=720        -> ResolvedScene JSON
//!   GET /render/:id?w=1280&h=720         -> image/png
//!
//! The asset id defaults to the file stem (e.g. `hero.rva` -> `hero`).

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use rva_server::Renderer;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path as StdPath;
use std::sync::Arc;

type AppState = Arc<HashMap<String, Arc<Renderer>>>;

#[derive(Parser)]
#[command(name = "rva-server", version, about)]
struct Cli {
    /// Path(s) to .rva assets to serve (id = file stem).
    #[arg(long = "asset", required = true)]
    asset: Vec<String>,

    /// Listen address.
    #[arg(long, default_value = "127.0.0.1:8787")]
    addr: String,
}

#[derive(Deserialize)]
struct RenderQuery {
    w: u32,
    h: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut assets: HashMap<String, Arc<Renderer>> = HashMap::new();
    for path in &cli.asset {
        let id = StdPath::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("asset")
            .to_string();
        let renderer = Arc::new(Renderer::from_path(path)?);
        println!("loaded '{id}': {}", renderer.describe());
        assets.insert(id, renderer);
    }
    let state: AppState = Arc::new(assets);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/describe/:id", get(describe))
        .route("/resolve/:id", get(resolve_scene))
        .route("/render/:id", get(render))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(&cli.addr).await?;
    println!(
        "rva-server listening on http://{} ({} asset(s))",
        cli.addr,
        state.len()
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn describe(State(app): State<AppState>, Path(id): Path<String>) -> Response {
    match app.get(&id) {
        Some(renderer) => renderer.describe().into_response(),
        None => not_found(&id),
    }
}

async fn resolve_scene(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RenderQuery>,
) -> Response {
    let Some(renderer) = app.get(&id) else {
        return not_found(&id);
    };
    match renderer.resolve(query.w, query.h) {
        Ok(scene) => match serde_json::to_string(&scene) {
            Ok(json) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                json,
            )
                .into_response(),
            Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
        },
        Err(error) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

async fn render(
    State(app): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RenderQuery>,
) -> Response {
    let Some(renderer) = app.get(&id).cloned() else {
        return not_found(&id);
    };

    // Rendering is CPU-bound; keep it off the async runtime.
    let result = tokio::task::spawn_blocking(move || renderer.render_png(query.w, query.h)).await;

    match result {
        Ok(Ok(bytes)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes.to_vec(),
        )
            .into_response(),
        Ok(Err(error)) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

fn not_found(id: &str) -> Response {
    (StatusCode::NOT_FOUND, format!("unknown asset '{id}'")).into_response()
}
