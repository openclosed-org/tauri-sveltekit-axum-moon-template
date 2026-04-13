//! Pingora-based reverse proxy — API + web routing.
//!
//! Routing rules:
//!   - `/healthz`      → health check (200 OK, handled locally)
//!   - `/api/*`        → web-bff upstream (3010)
//!   - `/admin/*`      → admin-bff upstream (3020)
//!   - `/*`            → web static upstream (3002)
//!
//! Environment variables:
//!   API_UPSTREAM  — web-bff server address (default: 127.0.0.1:3010)
//!   ADMIN_UPSTREAM — admin-bff server address (default: 127.0.0.1:3020)
//!   WEB_UPSTREAM  — Web static server address (default: 127.0.0.1:3002)
//!   BIND          — Bind address (default: 0.0.0.0:3000)

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use pingora::prelude::*;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_http::ResponseHeader;
use pingora_load_balancing::{selection::RoundRobin, LoadBalancer, Backend, health_check::TcpHealthCheck};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};

// ── Configuration ────────────────────────────────────────────

struct GatewayConfig {
    api_upstream: String,
    admin_upstream: String,
    web_upstream: String,
    bind: String,
}

impl GatewayConfig {
    fn from_env() -> Self {
        Self {
            api_upstream: std::env::var("API_UPSTREAM")
                .unwrap_or_else(|_| "127.0.0.1:3010".to_string()),
            admin_upstream: std::env::var("ADMIN_UPSTREAM")
                .unwrap_or_else(|_| "127.0.0.1:3020".to_string()),
            web_upstream: std::env::var("WEB_UPSTREAM")
                .unwrap_or_else(|_| "127.0.0.1:3002".to_string()),
            bind: std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
        }
    }
}

// ── Gateway proxy ────────────────────────────────────────────

struct Gateway {
    api_upstreams: Arc<LoadBalancer<RoundRobin>>,
    admin_upstreams: Arc<LoadBalancer<RoundRobin>>,
    web_upstreams: Arc<LoadBalancer<RoundRobin>>,
}

impl Gateway {
    fn new(config: &GatewayConfig) -> Self {
        // API upstream (web-bff) with health check
        let mut api_upstreams =
            LoadBalancer::<RoundRobin>::try_from_iter([config.api_upstream.as_str()])
                .expect("valid API upstream address");
        let api_hc = TcpHealthCheck::new();
        api_upstreams.set_health_check(api_hc);
        api_upstreams.health_check_frequency = Some(std::time::Duration::from_secs(5));
        let api_bg = background_service("api-health-check", api_upstreams);
        let api_upstreams = api_bg.task();

        // Admin upstream (admin-bff) with health check
        let mut admin_upstreams =
            LoadBalancer::<RoundRobin>::try_from_iter([config.admin_upstream.as_str()])
                .expect("valid admin upstream address");
        let admin_hc = TcpHealthCheck::new();
        admin_upstreams.set_health_check(admin_hc);
        admin_upstreams.health_check_frequency = Some(std::time::Duration::from_secs(5));
        let admin_bg = background_service("admin-health-check", admin_upstreams);
        let admin_upstreams = admin_bg.task();

        // Web upstream with health check
        let mut web_upstreams =
            LoadBalancer::<RoundRobin>::try_from_iter([config.web_upstream.as_str()])
                .expect("valid web upstream address");
        let web_hc = TcpHealthCheck::new();
        web_upstreams.set_health_check(web_hc);
        web_upstreams.health_check_frequency = Some(std::time::Duration::from_secs(5));
        let web_bg = background_service("web-health-check", web_upstreams);
        let web_upstreams = web_bg.task();

        Self {
            api_upstreams,
            admin_upstreams,
            web_upstreams,
        }
    }
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    /// Handle health check directly (short-circuit, no upstream).
    async fn request_filter(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<bool> {
        let path = session.req_header().uri.path();

        if path == "/healthz" || path == "/health" {
            // Gateway is healthy if it's running
            // Pingora's internal health checks manage upstream status automatically
            let body_str = serde_json::json!({
                "status": "ok",
                "upstreams": {
                    "api": "configured",
                    "admin": "configured",
                    "web": "configured"
                }
            }).to_string();

            let body = Bytes::from(body_str);
            let len = body.len();
            let mut hdr = ResponseHeader::build(200, Some(len))?;
            hdr.set_content_length(len)?;
            hdr.insert_header("content-type", "application/json")?;

            session.write_response_header(Box::new(hdr), false).await?;
            session.write_response_body(Some(body), true).await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Select upstream based on request path.
    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        let (upstreams, sni) = if path.starts_with("/admin/") || path == "/admin" {
            (&self.admin_upstreams, "")
        } else if path.starts_with("/api/") || path == "/api" {
            (&self.api_upstreams, "")
        } else {
            (&self.web_upstreams, "")
        };

        let backend = upstreams.select(b"", 256).unwrap();
        Ok(Box::new(HttpPeer::new(backend, false, sni.to_string())))
    }
}

// ── Entry point ──────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info,pingora_gateway=debug"),
    );

    let config = GatewayConfig::from_env();
    let bind_addr = config.bind.clone();

    log::info!("Starting Pingora gateway on {}", bind_addr);
    log::info!("  API upstream (web-bff):    {}", config.api_upstream);
    log::info!("  Admin upstream (admin-bff): {}", config.admin_upstream);
    log::info!("  Web upstream:              {}", config.web_upstream);

    let mut server = Server::new(Some(Opt::parse_args()))?;
    server.bootstrap();

    let gateway = Gateway::new(&config);

    let mut proxy = http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp(&bind_addr);

    server.add_service(proxy);
    server.run_forever()
}
