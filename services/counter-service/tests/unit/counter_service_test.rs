//! Unit tests for the counter application layer.
//!
//! These tests use an in-memory mock repository to verify that
//! the application service correctly orchestrates repository calls.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use counter_service::application::{RepositoryBackedCounterService, TenantScopedCounterService};
use counter_service::contracts::service::CounterService;
use counter_service::domain::{Counter, CounterId};
use counter_service::ports::{CounterRepository, RepositoryError};
use kernel::TenantId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// In-memory mock repository for testing with CAS and outbox support.
struct MockCounterRepository {
    counters: Arc<Mutex<HashMap<String, Counter>>>,
    outbox: Arc<Mutex<Vec<String>>>,
    idempotency: Arc<Mutex<HashMap<String, (i64, i64)>>>,
}

impl MockCounterRepository {
    fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            outbox: Arc::new(Mutex::new(Vec::new())),
            idempotency: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CounterRepository for MockCounterRepository {
    async fn load(&self, id: &CounterId) -> Result<Option<Counter>, RepositoryError> {
        let map = self.counters.lock().await;
        Ok(map.get(id.as_str()).cloned())
    }

    async fn increment(
        &self,
        id: &CounterId,
        _expected_version: i64,
        now: DateTime<Utc>,
    ) -> Result<(i64, i64), RepositoryError> {
        let mut map = self.counters.lock().await;
        let counter = map
            .entry(id.as_str().to_string())
            .or_insert_with(|| Counter::new(id.clone(), now));
        let updated = counter.clone().increment();
        *map.get_mut(id.as_str()).unwrap() = updated.clone();
        Ok((updated.value, updated.version))
    }

    async fn decrement(
        &self,
        id: &CounterId,
        _expected_version: i64,
        now: DateTime<Utc>,
    ) -> Result<(i64, i64), RepositoryError> {
        let mut map = self.counters.lock().await;
        let counter = map
            .entry(id.as_str().to_string())
            .or_insert_with(|| Counter::new(id.clone(), now));
        let updated = counter.clone().decrement();
        *map.get_mut(id.as_str()).unwrap() = updated.clone();
        Ok((updated.value, updated.version))
    }

    async fn reset(
        &self,
        id: &CounterId,
        _expected_version: i64,
        now: DateTime<Utc>,
    ) -> Result<i64, RepositoryError> {
        let mut map = self.counters.lock().await;
        let counter = Counter::new(id.clone(), now);
        map.insert(id.as_str().to_string(), counter);
        Ok(0)
    }

    async fn upsert(&self, counter: &Counter) -> Result<(), RepositoryError> {
        let mut map = self.counters.lock().await;
        map.insert(counter.id.as_str().to_string(), counter.clone());
        Ok(())
    }

    async fn write_outbox(
        &self,
        event_type: &str,
        payload: &str,
        _source_service: &str,
    ) -> Result<(), RepositoryError> {
        let mut outbox = self.outbox.lock().await;
        outbox.push(format!("{}:{}", event_type, payload));
        Ok(())
    }

    async fn check_idempotency(&self, key: &str) -> Result<Option<(i64, i64)>, RepositoryError> {
        let map = self.idempotency.lock().await;
        Ok(map.get(key).cloned())
    }

    async fn cache_idempotency(
        &self,
        key: &str,
        value: i64,
        version: i64,
    ) -> Result<(), RepositoryError> {
        let mut map = self.idempotency.lock().await;
        map.insert(key.to_string(), (value, version));
        Ok(())
    }
}

#[tokio::test]
async fn increment_creates_counter_at_one() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    let value = service.increment(&tenant, None).await.unwrap();
    assert_eq!(value, 1, "first increment should produce value 1");
}

#[tokio::test]
async fn increment_is_idempotent_per_tenant() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    let v1 = service.increment(&tenant, None).await.unwrap();
    let v2 = service.increment(&tenant, None).await.unwrap();
    let v3 = service.increment(&tenant, None).await.unwrap();

    assert_eq!(v1, 1);
    assert_eq!(v2, 2);
    assert_eq!(v3, 3);
}

#[tokio::test]
async fn tenant_a_increment_does_not_affect_tenant_b() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant_a = TenantId("tenant-a".into());
    let tenant_b = TenantId("tenant-b".into());

    service.increment(&tenant_a, None).await.unwrap();
    let b_value = service.get_value(&tenant_b).await.unwrap();

    assert_eq!(b_value, 0, "tenant-b should not see tenant-a's counter");
}

#[tokio::test]
async fn decrement_can_go_negative() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    let value = service.decrement(&tenant, None).await.unwrap();
    assert_eq!(value, -1, "decrement on nonexistent counter should be -1");
}

#[tokio::test]
async fn reset_returns_zero() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    service.increment(&tenant, None).await.unwrap();
    service.increment(&tenant, None).await.unwrap();
    let result = service.reset(&tenant, None).await.unwrap();

    assert_eq!(result, 0, "reset must always return 0");

    let value = service.get_value(&tenant).await.unwrap();
    assert_eq!(value, 0, "counter must be zero after reset");
}

#[tokio::test]
async fn multi_tenant_isolation_after_reset() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant_a = TenantId("tenant-a".into());
    let tenant_b = TenantId("tenant-b".into());

    service.increment(&tenant_a, None).await.unwrap();
    service.increment(&tenant_a, None).await.unwrap();
    service.increment(&tenant_b, None).await.unwrap();

    service.reset(&tenant_a, None).await.unwrap();

    let a_val = service.get_value(&tenant_a).await.unwrap();
    let b_val = service.get_value(&tenant_b).await.unwrap();

    assert_eq!(a_val, 0, "tenant-a must be zero after reset");
    assert_eq!(b_val, 1, "tenant-b must remain unaffected");
}

#[tokio::test]
async fn get_value_returns_zero_for_missing_counter() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("unknown-tenant".into());

    let value = service.get_value(&tenant).await.unwrap();
    assert_eq!(value, 0, "missing counter should default to zero");
}

#[tokio::test]
async fn repository_backed_service_implements_feature_trait() {
    let repo = MockCounterRepository::new();
    let service: RepositoryBackedCounterService<MockCounterRepository> =
        RepositoryBackedCounterService::new(repo);

    // Verify it implements the contracts CounterService trait
    let _: &dyn counter_service::contracts::service::CounterService = &service;

    let id = CounterId::new("test-tenant");
    let v = service.increment(&id, None).await.unwrap();
    assert_eq!(v, 1);
}

#[tokio::test]
async fn increment_with_idempotency_key_returns_same_value_on_repeat() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());
    let idem_key = "req-123";

    let v1 = service.increment(&tenant, Some(idem_key)).await.unwrap();
    let v2 = service.increment(&tenant, Some(idem_key)).await.unwrap();

    assert_eq!(v1, 1);
    assert_eq!(v2, 1, "idempotency key must return cached result");
}

#[tokio::test]
async fn decrement_with_idempotency_key_returns_same_value_on_repeat() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());
    let idem_key = "req-456";

    let v1 = service.decrement(&tenant, Some(idem_key)).await.unwrap();
    let v2 = service.decrement(&tenant, Some(idem_key)).await.unwrap();

    assert_eq!(v1, -1);
    assert_eq!(v2, -1, "idempotency key must return cached result");
}

#[tokio::test]
async fn reset_with_idempotency_key_returns_same_value_on_repeat() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    service.increment(&tenant, None).await.unwrap();
    service.increment(&tenant, None).await.unwrap();

    let idem_key = "req-789";
    let v1 = service.reset(&tenant, Some(idem_key)).await.unwrap();
    let v2 = service.reset(&tenant, Some(idem_key)).await.unwrap();

    assert_eq!(v1, 0);
    assert_eq!(v2, 0, "idempotency key must return cached result");
}

#[tokio::test]
async fn different_idempotency_keys_produce_different_results() {
    let repo = MockCounterRepository::new();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    let v1 = service.increment(&tenant, Some("key-1")).await.unwrap();
    let v2 = service.increment(&tenant, Some("key-2")).await.unwrap();

    assert_eq!(v1, 1);
    assert_eq!(v2, 2, "different keys must produce different results");
}

#[tokio::test]
async fn outbox_events_are_written_on_increment() {
    let repo = MockCounterRepository::new();
    let outbox_clone = repo.outbox.clone();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());

    service.increment(&tenant, None).await.unwrap();

    let outbox = outbox_clone.lock().await;
    assert_eq!(outbox.len(), 1, "one outbox event must be written");
    assert!(outbox[0].contains("counter.changed"));
}

#[tokio::test]
async fn outbox_events_are_not_written_on_idempotent_hit() {
    let repo = MockCounterRepository::new();
    let outbox_clone = repo.outbox.clone();
    let service: TenantScopedCounterService<MockCounterRepository> =
        TenantScopedCounterService::new(repo);
    let tenant = TenantId("tenant-a".into());
    let idem_key = "req-no-dup";

    service.increment(&tenant, Some(idem_key)).await.unwrap();
    service.increment(&tenant, Some(idem_key)).await.unwrap();

    let outbox = outbox_clone.lock().await;
    assert_eq!(
        outbox.len(),
        1,
        "idempotent hit must not duplicate outbox event"
    );
}
