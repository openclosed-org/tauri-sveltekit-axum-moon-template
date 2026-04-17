//! Web BFF 入口 — 配置加载 + 服务启动。

use web_bff::{config::Config, create_router, state::BffState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _observability = observability::init_observability("web-bff", "info,web_bff=debug")
        .map_err(std::io::Error::other)?;

    tracing::info!("Starting Web BFF...");

    let config = Config::from_env()?;
    let state = BffState::new(config).await?;
    let port = state.config.server_port;
    let addr = format!("{}:{}", state.config.server_host, port);
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Web BFF listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

    Ok(())
}
