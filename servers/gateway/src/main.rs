//! Pingora-based reverse proxy — API + web routing.
//!
//! Routing rules:
//!   - `/healthz`      → health check (200 OK, handled locally)
//!   - `/api/*`        → upstream API server
//!   - `/*`            → upstream web static server
//!
//! Environment variables:
//!   API_UPSTREAM  — API server address (default: 127.0.0.1:3001)
//!   WEB_UPSTREAM  — Web static server address (default: 127.0.0.1:3002)
//!   BIND          — Bind address (default: 0.0.0.0:3000)

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use pingora::prelude::*;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::upstreams::rr::LoadBalancer;
use pingora_http::ResponseHeader;
use pingora_proxy::{http_proxy_service, ProxyHttp};

// ── Configuration ────────────────────────────────────────────

struct GatewayConfig {
    api_upstream: String,
    web_upstream: String,
    bind: String,
}

impl GatewayConfig {
    fn from_env() -> Self {
        Self {
            api_upstream: std::env::var("API_UPSTREAM")
                .unwrap_or_else(|_| "127.0.0.1:3001".to_string()),
            web_upstream: std::env::var("WEB_UPSTREAM")
                .unwrap_or_else(|_| "127.0.0.1:3002".to_string()),
            bind: std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
        }
    }
}

// ── Gateway proxy ────────────────────────────────────────────

struct Gateway {
    api_upstreams: Arc<LoadBalancer<Backend>>,
    web_upstreams: Arc<LoadBalancer<Backend>>,
}

impl Gateway {
    fn new(config: &GatewayConfig) -> Self {
        // API upstream with health check
        let mut api_upstreams =
            LoadBalancer::try_from_iter([config.api_upstream.as_str()])
                .expect("valid API upstream address");
        let mut api_hc = TcpHealthCheck::new();
        api_hc.set_connect_timeout(std::time::Duration::from_secs(2));
        api_upstreams.set_health_check(Box::new(api_hc));
        api_upstreams.health_check_frequency = Some(std::time::Duration::from_secs(5));
        let api_bg = background_service("api-health-check", api_upstreams);
        let api_upstreams = api_bg.task();

        // Web upstream with health check
        let mut web_upstreams =
            LoadBalancer::try_from_iter([config.web_upstream.as_str()])
                .expect("valid web upstream address");
        let mut web_hc = TcpHealthCheck::new();
        web_hc.set_connect_timeout(std::time::Duration::from_secs(2));
        web_upstreams.set_health_check(Box::new(web_hc));
        web_upstreams.health_check_frequency = Some(std::time::Duration::from_secs(5));
        let web_bg = background_service("web-health-check", web_upstreams);
        let web_upstreams = web_bg.task();

        Self {
            api_upstreams,
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
            let body = Bytes::from("ok\n");
            let len = body.len();
            let mut hdr = ResponseHeader::build(200, Some(len))?;
            hdr.set_content_length(len)?;
            hdr.insert_header("content-type", "text/plain")?;

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

        let (upstreams, sni) = if path.starts_with("/api/") || path == "/api" {
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
    log::info!("  API upstream:  {}", config.api_upstream);
    log::info!("  Web upstream:  {}", config.web_upstream);

    let mut server = Server::new(Some(Opt::parse_args()))?;
    server.bootstrap();

    let gateway = Gateway::new(&config);

    let mut proxy = http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp(&bind_addr);

    server.add_service(proxy);
    server.run_forever()
}
