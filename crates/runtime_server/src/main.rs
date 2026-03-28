//! Standalone server binary — runs the Axum HTTP server.
//!
//! Usage: cargo run -p runtime_server
//!
//! Listens on 0.0.0.0:3001 by default.
//! Override with SERVER_PORT env var.

use runtime_server::create_router;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid port number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let app = create_router();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Runtime server listening on {addr}");
    println!("   Health: http://localhost:{port}/healthz");
    println!("   Ready:  http://localhost:{port}/readyz");

    axum::serve(listener, app).await.unwrap();
}
