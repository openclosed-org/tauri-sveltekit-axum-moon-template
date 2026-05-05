//! Web BFF 入口 — 配置加载 + 服务启动。

use web_bff::{bootstrap::bootstrap_bff_state, config::Config, create_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _observability = observability::init_observability("web-bff", "info,web_bff=debug")
        .map_err(std::io::Error::other)?;

    tracing::info!("Starting Web BFF...");

    let config = Config::from_env()?;
    let state = bootstrap_bff_state(config).await?;
    let port = state.config().server_port;
    let addr = format!("{}:{}", state.config().server_host, port);
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Web BFF listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received; starting graceful shutdown");
}
