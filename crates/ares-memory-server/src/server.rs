use axum::Router;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::capabilities;
use crate::routes;

/// Boot the ARES Memory Server on the given address.
///
/// This function is the sole entry point for HTTP serving.
/// It wires the Axum router and attaches Tower middleware.
/// No intelligence logic lives here.
pub async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    // Log the registered capabilities on startup for operational visibility.
    let caps = capabilities::registered_capabilities();
    info!(
        "ARES Memory Server starting. {} capabilities registered.",
        caps.len()
    );
    for cap in &caps {
        info!("  [{}] {} ({})", cap.phase, cap.name, cap.owner_crate);
    }

    let app = Router::new()
        .merge(routes::build_router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
