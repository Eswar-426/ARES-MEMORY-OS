use ares_agent::config::AgentConfig;
use ares_api::create_router;
use ares_app::AppState;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Observability Setup
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ares_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting ARES API Platform"
    );

    let project_path = std::env::current_dir()
        .expect("Cannot determine current directory")
        .to_string_lossy()
        .to_string();

    let config = AgentConfig::load(&project_path)?;
    let app_state = AppState::new(config).await?;

    let app = create_router(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    info!("API listening on http://127.0.0.1:8080");
    info!("Swagger UI available at http://127.0.0.1:8080/swagger-ui");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutting down ARES API gracefully...");
}
