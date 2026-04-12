use admin_bff::create_router;
use admin_bff::config::Config;
use admin_bff::state::AdminBffState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "admin_bff=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!(
        host = %config.server_host,
        port = %config.server_port,
        "Starting admin-bff..."
    );

    // Create application state
    let state = AdminBffState::new_with_config(&config).await?;
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.server_host, config.server_port
    ))
    .await?;

    tracing::info!("Admin BFF listening on {}:{}", config.server_host, config.server_port);

    axum::serve(listener, app).await?;

    Ok(())
}
