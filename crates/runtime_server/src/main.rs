//! Standalone server binary — runs the Axum HTTP server.
//!
//! Usage: cargo run -p runtime_server
//!
//! Listens on 0.0.0.0:3001 by default.
//! Override with SERVER_PORT env var.

use runtime_server::{create_router, state::AppState};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid port number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Initialize shared application state
    let state = AppState::new_dev().await?;
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Runtime server listening on {addr}");
    println!("   Health: http://localhost:{port}/healthz");
    println!("   Ready:  http://localhost:{port}/readyz");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
