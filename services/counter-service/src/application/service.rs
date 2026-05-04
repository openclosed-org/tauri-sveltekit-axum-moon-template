//! CounterService implementation backed by CounterRepository.
//!
//! Implements the full mutation chain:
//!   load current state → repository transaction → CAS mutation + outbox event write
//!
//! ## Idempotency
//! When an idempotency_key is provided, the repository reserves, completes, and
//! replays the key at the same transaction boundary as the mutation and outbox
//! write. If the key was already processed for the same request fingerprint, the
//! cached result is returned without re-executing the operation.
//!
//! ## CAS (Compare-And-Swap)
//! Every mutation passes the current `version` to the repository. If another
//! writer has changed the counter between load and update, the CAS check fails
//! and the repository returns a CAS conflict error — the application layer retries
//! with the latest version.
//!
//! ## Event Publishing
//! After a successful mutation, a `counter.changed` event is written to the
//! outbox table. The outbox-relay worker picks up these entries and publishes
//! them to the event bus asynchronously (guaranteed delivery).

use crate::contracts::service::{CounterCommandContext, CounterError, CounterService};
use async_trait::async_trait;
use contracts_events::{
    AppEvent, CounterChanged, CounterOperation as EventCounterOperation, EventEnvelope,
    event_type_name,
};
use kernel::id::correlation_id as new_correlation_id;
use tracing::debug;

use crate::domain::CounterId;
use crate::ports::{CommitOutcome, CounterMutation, CounterOperation, CounterRepository};

/// `CounterService` backed by any `CounterRepository` implementation.
///
/// ## Type parameters
/// - `R`: The repository implementation. Allows swapping libsql,
///   Turso cloud, SurrealDB, or in-memory stubs without touching this code.
pub struct RepositoryBackedCounterService<R: CounterRepository> {
    repo: R,
}

#[derive(Clone, Copy)]
enum MutationKind {
    Increment,
    Decrement,
    Reset,
}

impl MutationKind {
    fn operation(self) -> &'static str {
        match self {
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::Reset => "reset",
        }
    }

    fn log_name(self) -> &'static str {
        match self {
            Self::Increment => "counter.increment",
            Self::Decrement => "counter.decrement",
            Self::Reset => "counter.reset",
        }
    }
}

impl<R: CounterRepository> RepositoryBackedCounterService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: CounterRepository> CounterService for RepositoryBackedCounterService<R> {
    async fn get_value(&self, tenant_id: &CounterId) -> Result<i64, CounterError> {
        let counter = self
            .repo
            .load(tenant_id)
            .await
            .map_err(CounterError::Database)?;

        let value = counter.map(|c| c.value).unwrap_or(0);
        debug!(counter_id = %tenant_id, value, "counter.get_value");
        Ok(value)
    }

    async fn increment(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        self.increment_with_context(
            tenant_id,
            idempotency_key,
            &CounterCommandContext::default(),
        )
        .await
    }

    async fn increment_with_context(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        let outcome = self
            .run_mutation(tenant_id, idempotency_key, context, MutationKind::Increment)
            .await?;
        debug!(counter_id = %tenant_id, value = outcome.value, version = outcome.version, "counter.increment");
        Ok(outcome.value)
    }

    async fn decrement(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        self.decrement_with_context(
            tenant_id,
            idempotency_key,
            &CounterCommandContext::default(),
        )
        .await
    }

    async fn decrement_with_context(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        let outcome = self
            .run_mutation(tenant_id, idempotency_key, context, MutationKind::Decrement)
            .await?;
        debug!(counter_id = %tenant_id, value = outcome.value, version = outcome.version, "counter.decrement");
        Ok(outcome.value)
    }

    async fn reset(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        self.reset_with_context(
            tenant_id,
            idempotency_key,
            &CounterCommandContext::default(),
        )
        .await
    }

    async fn reset_with_context(
        &self,
        tenant_id: &CounterId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        let outcome = self
            .run_mutation(tenant_id, idempotency_key, context, MutationKind::Reset)
            .await?;
        debug!(counter_id = %tenant_id, version = outcome.version, "counter.reset");
        Ok(outcome.value)
    }
}

struct MutationOutcome {
    value: i64,
    version: i64,
    delta: i64,
}

impl<R: CounterRepository> RepositoryBackedCounterService<R> {
    async fn run_mutation(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
        kind: MutationKind,
    ) -> Result<MutationOutcome, CounterError> {
        self.run_atomic_mutation(counter_id, idempotency_key, context, kind)
            .await
    }

    /// Execute the mutation chain atomically using the repository transaction boundary.
    ///
    /// CAS mutation, outbox write, and idempotency state are combined into a
    /// single `commit_mutation` call at the repository layer.
    async fn run_atomic_mutation(
        &self,
        counter_id: &CounterId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
        kind: MutationKind,
    ) -> Result<MutationOutcome, CounterError> {
        let mut retries = 0;

        loop {
            let current = self
                .repo
                .load(counter_id)
                .await
                .map_err(CounterError::Database)?;
            let current_value = current.as_ref().map(|c| c.value).unwrap_or(0);
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            let new_value = match kind {
                MutationKind::Increment => current_value + 1,
                MutationKind::Decrement => current_value - 1,
                MutationKind::Reset => 0,
            };
            let new_version = expected_version + 1;
            let delta = new_value - current_value;

            let (event_id, event_type, event_payload, source_service, correlation_id) = self
                .build_event(
                    counter_id,
                    kind,
                    new_value,
                    delta,
                    new_version,
                    context,
                    idempotency_key,
                )?;

            let mutation = CounterMutation {
                counter_id,
                operation: match kind {
                    MutationKind::Increment => CounterOperation::Increment,
                    MutationKind::Decrement => CounterOperation::Decrement,
                    MutationKind::Reset => CounterOperation::Reset,
                },
                new_value,
                new_version,
                event_id: &event_id,
                event_type: &event_type,
                event_payload: &event_payload,
                source_service: &source_service,
                correlation_id: correlation_id.as_deref(),
            };

            match self
                .repo
                .commit_mutation(&mutation, idempotency_key)
                .await
                .map_err(CounterError::Database)?
            {
                CommitOutcome::Committed {
                    new_value,
                    new_version,
                } => {
                    debug!(
                        counter_id = %counter_id,
                        value = new_value,
                        version = new_version,
                        "counter commit_mutation succeeded"
                    );
                    return Ok(MutationOutcome {
                        value: new_value,
                        version: new_version,
                        delta,
                    });
                }
                CommitOutcome::IdempotentReplay { value, version } => {
                    return Ok(MutationOutcome {
                        value,
                        version,
                        delta: 0,
                    });
                }
                CommitOutcome::IdempotencyConflict => {
                    return Err(CounterError::IdempotencyConflict);
                }
                CommitOutcome::CasConflict if retries < 3 => {
                    retries += 1;
                    debug!(
                        counter_id = %counter_id,
                        retries,
                        operation = kind.operation(),
                        "CAS conflict, retrying"
                    );
                }
                CommitOutcome::CasConflict => {
                    return Err(CounterError::CasConflict);
                }
            }
        }
    }

    /// Build the event envelope for a mutation (used by atomic commit).
    ///
    /// Returns (event_id, event_type, event_payload, source_service, correlation_id)
    /// so the repository layer can write it atomically in the CTE.
    #[allow(clippy::type_complexity)]
    fn build_event(
        &self,
        counter_id: &CounterId,
        kind: MutationKind,
        new_value: i64,
        delta: i64,
        new_version: i64,
        context: &CounterCommandContext,
        idempotency_key: Option<&str>,
    ) -> Result<(String, String, String, String, Option<String>), CounterError> {
        let event = CounterChanged {
            tenant_id: counter_id.as_str().to_string(),
            counter_key: counter_id.as_str().to_string(),
            operation: match kind {
                MutationKind::Increment => EventCounterOperation::Increment,
                MutationKind::Decrement => EventCounterOperation::Decrement,
                MutationKind::Reset => EventCounterOperation::Reset,
            },
            new_value,
            delta,
            version: new_version,
        };

        let mut envelope = EventEnvelope::new(AppEvent::CounterChanged(event), "counter-service");
        let correlation_id = context
            .correlation_id
            .clone()
            .or_else(|| idempotency_key.map(std::borrow::ToOwned::to_owned))
            .unwrap_or_else(new_correlation_id);
        let causation_id = context
            .causation_id
            .clone()
            .or_else(|| idempotency_key.map(std::borrow::ToOwned::to_owned))
            .unwrap_or_else(|| correlation_id.clone());
        envelope = envelope
            .with_correlation_id(correlation_id.clone())
            .with_causation_id(causation_id);
        if let Some(actor) = &context.actor {
            envelope.metadata.actor = Some(actor.clone());
        }
        if let Some(trace_id) = &context.trace_id {
            envelope.metadata.trace_id = Some(trace_id.clone());
        }
        if let Some(span_id) = &context.span_id {
            envelope.metadata.span_id = Some(span_id.clone());
        }

        let payload =
            serde_json::to_string(&envelope).map_err(|e| CounterError::Database(Box::new(e)))?;

        Ok((
            envelope.id.to_string(),
            event_type_name(&envelope.event).to_string(),
            payload,
            envelope.source_service,
            Some(correlation_id),
        ))
    }
}

/// Tenant-scoped CounterService — accepts TenantId, isolates by tenant.
///
/// This is the **primary** service used by the BFF layer.
/// Each tenant gets an independent counter with no cross-tenant leakage.
pub struct TenantScopedCounterService<R: CounterRepository> {
    inner: RepositoryBackedCounterService<R>,
}

impl<R: CounterRepository> TenantScopedCounterService<R> {
    pub fn new(repo: R) -> Self {
        Self {
            inner: RepositoryBackedCounterService::new(repo),
        }
    }

    /// Get counter value for a specific tenant.
    pub async fn get_value(&self, tenant_id: &kernel::TenantId) -> Result<i64, CounterError> {
        let id = counter_id_for_tenant(tenant_id);
        let value = self.inner.get_value(&id).await?;
        debug!(tenant_id = %tenant_id, value, "counter.get_value_for_tenant");
        Ok(value)
    }

    /// Increment counter for a specific tenant.
    pub async fn increment(
        &self,
        tenant_id: &kernel::TenantId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        self.increment_with_context(
            tenant_id,
            idempotency_key,
            &CounterCommandContext::default(),
        )
        .await
    }

    pub async fn increment_with_context(
        &self,
        tenant_id: &kernel::TenantId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        let value = self
            .inner
            .increment_with_context(&counter_id_for_tenant(tenant_id), idempotency_key, context)
            .await?;
        debug!(tenant_id = %tenant_id, value, "counter.increment_for_tenant");
        Ok(value)
    }

    /// Decrement counter for a specific tenant.
    pub async fn decrement(
        &self,
        tenant_id: &kernel::TenantId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        self.decrement_with_context(
            tenant_id,
            idempotency_key,
            &CounterCommandContext::default(),
        )
        .await
    }

    pub async fn decrement_with_context(
        &self,
        tenant_id: &kernel::TenantId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        let value = self
            .inner
            .decrement_with_context(&counter_id_for_tenant(tenant_id), idempotency_key, context)
            .await?;
        debug!(tenant_id = %tenant_id, value, "counter.decrement_for_tenant");
        Ok(value)
    }

    /// Reset counter for a specific tenant.
    pub async fn reset(
        &self,
        tenant_id: &kernel::TenantId,
        idempotency_key: Option<&str>,
    ) -> Result<i64, CounterError> {
        self.reset_with_context(
            tenant_id,
            idempotency_key,
            &CounterCommandContext::default(),
        )
        .await
    }

    pub async fn reset_with_context(
        &self,
        tenant_id: &kernel::TenantId,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<i64, CounterError> {
        let value = self
            .inner
            .reset_with_context(&counter_id_for_tenant(tenant_id), idempotency_key, context)
            .await?;
        debug!(tenant_id = %tenant_id, "counter.reset_for_tenant");
        Ok(value)
    }
}

fn counter_id_for_tenant(tenant_id: &kernel::TenantId) -> CounterId {
    CounterId::new(tenant_id.as_str())
}
