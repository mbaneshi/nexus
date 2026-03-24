//! Axum REST API server for Nexus with embedded SvelteKit frontend.

mod routes;
mod static_files;

#[cfg(test)]
mod tests;

use axum::Router;
use nexus_core::Result;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

/// Build the API router.
pub fn api_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", routes::routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Build the full router with API + embedded frontend.
pub fn full_router(state: AppState) -> Router {
    Router::new()
        .nest("/api", routes::routes())
        .with_state(state)
        .fallback(static_files::static_handler)
        .layer(CorsLayer::permissive())
}

/// Start the server on the given host and port.
pub async fn run(host: &str, port: u16, db: Connection) -> Result<()> {
    let state = AppState {
        db: Arc::new(Mutex::new(db)),
    };

    let app = full_router(state);
    let addr = format!("{host}:{port}");

    tracing::info!("Nexus server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| nexus_core::NexusError::Internal(e.to_string()))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| nexus_core::NexusError::Internal(e.to_string()))?;

    Ok(())
}
