//! Serve embedded SvelteKit frontend assets.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../../frontend/build/"]
struct FrontendAssets;

/// Handler for static files — serves the embedded SvelteKit build.
/// Falls back to index.html for SPA client-side routing.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try exact path first
    if let Some(content) = FrontendAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data,
        )
            .into_response();
    }

    // Try path with .html extension (SvelteKit prerendered pages)
    let html_path = format!("{path}.html");
    if let Some(content) = FrontendAssets::get(&html_path) {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            content.data,
        )
            .into_response();
    }

    // SPA fallback: serve index.html for all unmatched routes
    match FrontendAssets::get("index.html") {
        Some(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            content.data,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "Frontend not built").into_response(),
    }
}
