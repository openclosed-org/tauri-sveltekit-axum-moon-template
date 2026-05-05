//! BFF request context extracted at the HTTP boundary.

use authn_oidc_verifier::VerifiedIdentity;
use axum::extract::Request;
use contracts_events::ActorRef;
use counter_service::contracts::service::CounterCommandContext;
use observability::current_trace_context;

/// Request context extracted at the server boundary and forwarded into service calls.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_sub: String,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub actor: ActorRef,
}

impl RequestContext {
    pub fn from_verified_identity(identity: VerifiedIdentity, request_id: Option<String>) -> Self {
        let trace_context = current_trace_context();
        Self {
            user_sub: identity.sub.clone(),
            tenant_id: identity.tenant_id,
            roles: identity.roles,
            request_id,
            trace_id: trace_context
                .as_ref()
                .map(|context| context.trace_id.clone()),
            span_id: trace_context.map(|context| context.span_id),
            actor: ActorRef {
                actor_id: identity.sub.clone(),
                actor_type: "user".to_string(),
                subject: Some(identity.sub),
            },
        }
    }

    pub fn from_dev_headers(req: &Request) -> Option<Self> {
        let user_sub = req
            .headers()
            .get("x-dev-user-sub")
            .and_then(|value| value.to_str().ok())?
            .trim()
            .to_string();
        if user_sub.is_empty() {
            return None;
        }

        let tenant_id = req
            .headers()
            .get("x-dev-tenant-id")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let roles = req
            .headers()
            .get("x-dev-user-roles")
            .and_then(|value| value.to_str().ok())
            .map(parse_dev_roles)
            .unwrap_or_default();
        let request_id = request_id(req);
        let trace_context = current_trace_context();

        Some(Self {
            user_sub: user_sub.clone(),
            tenant_id,
            roles,
            request_id,
            trace_id: trace_context
                .as_ref()
                .map(|context| context.trace_id.clone()),
            span_id: trace_context.map(|context| context.span_id),
            actor: ActorRef {
                actor_id: user_sub.clone(),
                actor_type: "user".to_string(),
                subject: Some(user_sub),
            },
        })
    }

    pub fn to_counter_command_context(&self) -> CounterCommandContext {
        CounterCommandContext {
            correlation_id: self.request_id.clone(),
            causation_id: self.request_id.clone(),
            actor: Some(self.actor.clone()),
            trace_id: self.trace_id.clone(),
            span_id: self.span_id.clone(),
        }
    }
}

pub fn request_id(req: &Request) -> Option<String> {
    req.headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn parse_dev_roles(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}
