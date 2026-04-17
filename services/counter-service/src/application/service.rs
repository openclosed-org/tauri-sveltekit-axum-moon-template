//! CounterService implementation backed by CounterRepository.
//!
//! Implements the full mutation chain:
//!   idempotency check → load current state → CAS mutation → outbox event write
//!
//! ## Idempotency
//! When an idempotency_key is provided, the service checks a local idempotency
//! table before executing the mutation. If the key was already processed, the
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
use chrono::Utc;
use contracts_events::{AppEvent, CounterChanged, EventEnvelope, event_type_name};
use kernel::id::correlation_id as new_correlation_id;
use tracing::debug;

use crate::domain::CounterId;
use crate::ports::CounterRepository;

/// `CounterService` backed by any `CounterRepository` implementation.
///
/// ## Type parameters
/// - `R`: The repository implementation. Allows swapping libsql,
///   Turso cloud, SurrealDB, or in-memory stubs without touching this code.
pub struct RepositoryBackedCounterService<R: CounterRepository> {
    repo: R,
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
        let now = Utc::now();

        // Idempotency check
        if let Some(key) = idempotency_key
            && let Some(cached) = self.check_idempotency(key).await?
        {
            debug!(counter_id = %tenant_id, key, "counter.increment idempotent hit");
            return Ok(cached);
        }

        // CAS mutation with retry
        let mut retries = 0;
        let (value, version) = loop {
            let current = self
                .repo
                .load(tenant_id)
                .await
                .map_err(CounterError::Database)?;
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            match self.repo.increment(tenant_id, expected_version, now).await {
                Ok(result) => break Ok(result),
                Err(_) if retries < 3 => {
                    retries += 1;
                    debug!(counter_id = %tenant_id, retries, "CAS conflict, retrying");
                    continue;
                }
                Err(_) => break Err(CounterError::CasConflict),
            }
        }?;

        // Write outbox event
        let event = CounterChanged {
            tenant_id: tenant_id.as_str().to_string(),
            counter_key: tenant_id.as_str().to_string(),
            operation: "increment".to_string(),
            new_value: value,
            delta: 1,
            version,
        };
        self.write_outbox_event(&event, idempotency_key, context)
            .await?;

        // Cache idempotency result
        if let Some(key) = idempotency_key {
            self.cache_idempotency(key, value, version).await?;
        }

        debug!(counter_id = %tenant_id, value, version, "counter.increment");
        Ok(value)
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
        let now = Utc::now();

        if let Some(key) = idempotency_key
            && let Some(cached) = self.check_idempotency(key).await?
        {
            debug!(counter_id = %tenant_id, key, "counter.decrement idempotent hit");
            return Ok(cached);
        }

        let mut retries = 0;
        let (value, version) = loop {
            let current = self
                .repo
                .load(tenant_id)
                .await
                .map_err(CounterError::Database)?;
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            match self.repo.decrement(tenant_id, expected_version, now).await {
                Ok(result) => break Ok(result),
                Err(_) if retries < 3 => {
                    retries += 1;
                    debug!(counter_id = %tenant_id, retries, "CAS conflict, retrying");
                    continue;
                }
                Err(_) => break Err(CounterError::CasConflict),
            }
        }?;

        let event = CounterChanged {
            tenant_id: tenant_id.as_str().to_string(),
            counter_key: tenant_id.as_str().to_string(),
            operation: "decrement".to_string(),
            new_value: value,
            delta: -1,
            version,
        };
        self.write_outbox_event(&event, idempotency_key, context)
            .await?;

        if let Some(key) = idempotency_key {
            self.cache_idempotency(key, value, version).await?;
        }

        debug!(counter_id = %tenant_id, value, version, "counter.decrement");
        Ok(value)
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
        let now = Utc::now();

        if let Some(key) = idempotency_key
            && let Some(cached) = self.check_idempotency(key).await?
        {
            debug!(counter_id = %tenant_id, key, "counter.reset idempotent hit");
            return Ok(cached);
        }

        let mut retries = 0;
        let (value, version) = loop {
            let current = self
                .repo
                .load(tenant_id)
                .await
                .map_err(CounterError::Database)?;
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            match self.repo.reset(tenant_id, expected_version, now).await {
                Ok(v) => break Ok((0, v)),
                Err(_) if retries < 3 => {
                    retries += 1;
                    debug!(counter_id = %tenant_id, retries, "CAS conflict, retrying");
                    continue;
                }
                Err(_) => break Err(CounterError::CasConflict),
            }
        }?;

        let event = CounterChanged {
            tenant_id: tenant_id.as_str().to_string(),
            counter_key: tenant_id.as_str().to_string(),
            operation: "reset".to_string(),
            new_value: 0,
            delta: -value,
            version,
        };
        self.write_outbox_event(&event, idempotency_key, context)
            .await?;

        if let Some(key) = idempotency_key {
            self.cache_idempotency(key, 0, version).await?;
        }

        debug!(counter_id = %tenant_id, version, "counter.reset");
        Ok(0)
    }
}

impl<R: CounterRepository> RepositoryBackedCounterService<R> {
    /// Write a counter-changed event to the outbox table.
    async fn write_outbox_event(
        &self,
        event: &CounterChanged,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<(), CounterError> {
        let mut envelope =
            EventEnvelope::new(AppEvent::CounterChanged(event.clone()), "counter-service");
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

        debug!(
            event_type = event_type_name(&envelope.event),
            correlation_id = %correlation_id,
            trace_id = ?envelope.metadata.trace_id,
            span_id = ?envelope.metadata.span_id,
            tenant_id = %event.tenant_id,
            "counter outbox event prepared"
        );

        let payload =
            serde_json::to_string(&envelope).map_err(|e| CounterError::Database(Box::new(e)))?;

        self.repo
            .write_outbox(
                &envelope.id.to_string(),
                event_type_name(&envelope.event),
                &payload,
                &envelope.source_service,
                Some(correlation_id.as_str()),
            )
            .await
            .map_err(CounterError::Database)
    }

    /// Check if an idempotency key was already processed.
    /// Returns Some(value) if the key exists, None otherwise.
    async fn check_idempotency(&self, key: &str) -> Result<Option<i64>, CounterError> {
        match self.repo.check_idempotency(key).await {
            Ok(Some((value, _version))) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(CounterError::Database(e)),
        }
    }

    /// Cache an idempotency result.
    async fn cache_idempotency(
        &self,
        key: &str,
        value: i64,
        version: i64,
    ) -> Result<(), CounterError> {
        self.repo
            .cache_idempotency(key, value, version)
            .await
            .map_err(CounterError::Database)
    }
}

/// Tenant-scoped CounterService — accepts TenantId, isolates by tenant.
///
/// This is the **primary** service used by the BFF layer.
/// Each tenant gets an independent counter with no cross-tenant leakage.
pub struct TenantScopedCounterService<R: CounterRepository> {
    repo: R,
}

impl<R: CounterRepository> TenantScopedCounterService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Get counter value for a specific tenant.
    pub async fn get_value(&self, tenant_id: &kernel::TenantId) -> Result<i64, CounterError> {
        let id = CounterId::new(tenant_id.as_str());
        let counter = self.repo.load(&id).await.map_err(CounterError::Database)?;

        let value = counter.map(|c| c.value).unwrap_or(0);
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
        let id = CounterId::new(tenant_id.as_str());
        let now = Utc::now();

        // Idempotency check
        if let Some(key) = idempotency_key
            && let Some(cached) = self.check_idempotency(key).await?
        {
            debug!(tenant_id = %tenant_id, key, "counter.increment_for_tenant idempotent hit");
            return Ok(cached);
        }

        // CAS mutation with retry
        let mut retries = 0;
        let (value, version) = loop {
            let current = self.repo.load(&id).await.map_err(CounterError::Database)?;
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            match self.repo.increment(&id, expected_version, now).await {
                Ok(result) => break Ok(result),
                Err(_) if retries < 3 => {
                    retries += 1;
                    debug!(tenant_id = %tenant_id, retries, "CAS conflict, retrying");
                    continue;
                }
                Err(_) => break Err(CounterError::CasConflict),
            }
        }?;

        // Write outbox event
        let event = CounterChanged {
            tenant_id: tenant_id.as_str().to_string(),
            counter_key: tenant_id.as_str().to_string(),
            operation: "increment".to_string(),
            new_value: value,
            delta: 1,
            version,
        };
        self.write_outbox_event(&event, idempotency_key, context)
            .await?;

        // Cache idempotency result
        if let Some(key) = idempotency_key {
            self.cache_idempotency(key, value, version).await?;
        }

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
        let id = CounterId::new(tenant_id.as_str());
        let now = Utc::now();

        if let Some(key) = idempotency_key
            && let Some(cached) = self.check_idempotency(key).await?
        {
            debug!(tenant_id = %tenant_id, key, "counter.decrement_for_tenant idempotent hit");
            return Ok(cached);
        }

        let mut retries = 0;
        let (value, version) = loop {
            let current = self.repo.load(&id).await.map_err(CounterError::Database)?;
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            match self.repo.decrement(&id, expected_version, now).await {
                Ok(result) => break Ok(result),
                Err(_) if retries < 3 => {
                    retries += 1;
                    debug!(tenant_id = %tenant_id, retries, "CAS conflict, retrying");
                    continue;
                }
                Err(_) => break Err(CounterError::CasConflict),
            }
        }?;

        let event = CounterChanged {
            tenant_id: tenant_id.as_str().to_string(),
            counter_key: tenant_id.as_str().to_string(),
            operation: "decrement".to_string(),
            new_value: value,
            delta: -1,
            version,
        };
        self.write_outbox_event(&event, idempotency_key, context)
            .await?;

        if let Some(key) = idempotency_key {
            self.cache_idempotency(key, value, version).await?;
        }

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
        let id = CounterId::new(tenant_id.as_str());
        let now = Utc::now();

        if let Some(key) = idempotency_key
            && let Some(cached) = self.check_idempotency(key).await?
        {
            debug!(tenant_id = %tenant_id, key, "counter.reset_for_tenant idempotent hit");
            return Ok(cached);
        }

        let mut retries = 0;
        let (value, version) = loop {
            let current = self.repo.load(&id).await.map_err(CounterError::Database)?;
            let expected_version = current.as_ref().map(|c| c.version).unwrap_or(0);

            match self.repo.reset(&id, expected_version, now).await {
                Ok(v) => break Ok((0, v)),
                Err(_) if retries < 3 => {
                    retries += 1;
                    debug!(tenant_id = %tenant_id, retries, "CAS conflict, retrying");
                    continue;
                }
                Err(_) => break Err(CounterError::CasConflict),
            }
        }?;

        let event = CounterChanged {
            tenant_id: tenant_id.as_str().to_string(),
            counter_key: tenant_id.as_str().to_string(),
            operation: "reset".to_string(),
            new_value: 0,
            delta: -value,
            version,
        };
        self.write_outbox_event(&event, idempotency_key, context)
            .await?;

        if let Some(key) = idempotency_key {
            self.cache_idempotency(key, 0, version).await?;
        }

        debug!(tenant_id = %tenant_id, "counter.reset_for_tenant");
        Ok(0)
    }

    /// Check if an idempotency key was already processed.
    async fn check_idempotency(&self, key: &str) -> Result<Option<i64>, CounterError> {
        match self.repo.check_idempotency(key).await {
            Ok(Some((value, _version))) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(CounterError::Database(e)),
        }
    }

    /// Cache an idempotency result.
    async fn cache_idempotency(
        &self,
        key: &str,
        value: i64,
        version: i64,
    ) -> Result<(), CounterError> {
        self.repo
            .cache_idempotency(key, value, version)
            .await
            .map_err(CounterError::Database)
    }

    /// Write a counter-changed event to the outbox table.
    async fn write_outbox_event(
        &self,
        event: &CounterChanged,
        idempotency_key: Option<&str>,
        context: &CounterCommandContext,
    ) -> Result<(), CounterError> {
        let mut envelope =
            EventEnvelope::new(AppEvent::CounterChanged(event.clone()), "counter-service");
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

        debug!(
            event_type = event_type_name(&envelope.event),
            correlation_id = %correlation_id,
            trace_id = ?envelope.metadata.trace_id,
            span_id = ?envelope.metadata.span_id,
            tenant_id = %event.tenant_id,
            "counter outbox event prepared"
        );

        let payload =
            serde_json::to_string(&envelope).map_err(|e| CounterError::Database(Box::new(e)))?;

        self.repo
            .write_outbox(
                &envelope.id.to_string(),
                event_type_name(&envelope.event),
                &payload,
                &envelope.source_service,
                Some(correlation_id.as_str()),
            )
            .await
            .map_err(CounterError::Database)
    }
}
