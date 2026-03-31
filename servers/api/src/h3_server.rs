//! HTTP/3 server — Quinn QUIC transport layer.
//!
//! Per D-07/D-08: HTTP/3 (Quinn) as primary transport, HTTP/2 (Axum TCP) as fallback.
//! Per D-09: Dev environment uses TCP (:3001) for debugging. Production can enable h3.
//!
//! STATUS: Scaffolding — full implementation deferred to production readiness phase.
//! Dependencies (quinn, h3, rcgen) are declared and compile-checked.

/// Placeholder for h3 server configuration.
#[derive(Clone)]
pub struct H3Config {
    /// Bind address for QUIC listener (e.g., "0.0.0.0:443").
    pub bind_addr: String,

    /// Path to TLS certificate PEM file.
    pub cert_path: String,

    /// Path to TLS private key PEM file.
    pub key_path: String,
}

impl Default for H3Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:443".to_string(),
            cert_path: "./certs/cert.pem".to_string(),
            key_path: "./certs/key.pem".to_string(),
        }
    }
}

/// Start the HTTP/3 server.
///
/// This is a placeholder that verifies quinn + h3 compile.
/// Full implementation requires:
/// 1. TLS certificate loading (rcgen for dev self-signed)
/// 2. Quinn endpoint creation
/// 3. h3 connection accept loop
/// 4. Request routing to Axum handler
///
/// For now, the Axum TCP server (main.rs) handles all traffic.
pub async fn start_h3_server(_config: H3Config) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("HTTP/3 server: scaffolding ready (full implementation pending)");
    // TODO: Implement QUIC listener with quinn::Endpoint
    // TODO: Implement h3 server connection handling
    // TODO: Route requests through Axum router
    Ok(())
}

/// Generate a self-signed certificate for development.
///
/// Uses rcgen to create a cert valid for localhost.
/// Production should use ACME (Let's Encrypt) or Cloudflare.
pub fn generate_dev_cert() -> Result<(String, String), Box<dyn std::error::Error>> {
    let rcgen::CertifiedKey { cert, key_pair } =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])?;
    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    Ok((cert_pem, key_pem))
}
