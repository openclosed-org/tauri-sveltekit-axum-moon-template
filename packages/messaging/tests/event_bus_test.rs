//! Event bus tests — publish, subscribe, and handler invocation.

use contracts_events::{AppEvent, CounterChanged, CounterOperation, TenantCreated};
use event_bus::adapters::memory_bus::InMemoryEventBus;
use event_bus::ports::{EventBus, EventEnvelope};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn make_counter_event() -> AppEvent {
    AppEvent::CounterChanged(CounterChanged {
        tenant_id: "t1".into(),
        counter_key: "default".into(),
        operation: CounterOperation::Increment,
        new_value: 0,
        delta: 0,
        version: 0,
    })
}

#[tokio::test]
async fn publish_and_receive_via_broadcast() {
    let bus: InMemoryEventBus = InMemoryEventBus::new();
    let mut rx = bus.subscribe_receiver();

    let event = AppEvent::CounterChanged(CounterChanged {
        tenant_id: "t1".into(),
        counter_key: "default".into(),
        operation: CounterOperation::Increment,
        new_value: 42,
        delta: 1,
        version: 1,
    });

    bus.publish(EventEnvelope::new(event, "counter-service"))
        .await
        .unwrap();

    let received = rx.recv().await.unwrap();
    match received.event {
        AppEvent::CounterChanged(c) => {
            assert_eq!(c.tenant_id, "t1");
            assert_eq!(c.new_value, 42);
            assert_eq!(c.delta, 1);
        }
        _ => panic!("unexpected event type"),
    }
    assert_eq!(received.source_service, "counter-service");
}

#[tokio::test]
async fn handler_is_called_on_publish() {
    let bus: InMemoryEventBus = InMemoryEventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    bus.subscribe(
        "test-handler",
        Box::new(move |_envelope| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }),
    )
    .await
    .unwrap();

    let event = AppEvent::TenantCreated(TenantCreated {
        tenant_id: "t1".into(),
        owner_sub: "user-1".into(),
    });

    bus.publish(EventEnvelope::new(event, "tenant-service"))
        .await
        .unwrap();

    assert_eq!(
        count.load(Ordering::SeqCst),
        1,
        "handler should be called once"
    );
}

#[tokio::test]
async fn multiple_handlers_all_receive_event() {
    let bus: InMemoryEventBus = InMemoryEventBus::new();
    let count_a = Arc::new(AtomicUsize::new(0));
    let count_b = Arc::new(AtomicUsize::new(0));
    let count_a_clone = count_a.clone();
    let count_b_clone = count_b.clone();

    bus.subscribe(
        "handler-a",
        Box::new(move |_e| {
            count_a_clone.fetch_add(1, Ordering::SeqCst);
        }),
    )
    .await
    .unwrap();

    bus.subscribe(
        "handler-b",
        Box::new(move |_e| {
            count_b_clone.fetch_add(1, Ordering::SeqCst);
        }),
    )
    .await
    .unwrap();

    bus.publish(EventEnvelope::new(make_counter_event(), "counter-service"))
        .await
        .unwrap();

    assert_eq!(
        count_a.load(Ordering::SeqCst),
        1,
        "handler-a should be called"
    );
    assert_eq!(
        count_b.load(Ordering::SeqCst),
        1,
        "handler-b should be called"
    );
}

#[tokio::test]
async fn unsubscribe_stops_handler_calls() {
    let bus: InMemoryEventBus = InMemoryEventBus::new();
    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    bus.subscribe(
        "temp-handler",
        Box::new(move |_e| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }),
    )
    .await
    .unwrap();

    // Unsubscribe before publishing
    bus.unsubscribe("temp-handler").await.unwrap();

    bus.publish(EventEnvelope::new(make_counter_event(), "counter-service"))
        .await
        .unwrap();

    assert_eq!(
        count.load(Ordering::SeqCst),
        0,
        "unsubscribed handler should not be called"
    );
}

#[tokio::test]
async fn event_id_is_unique() {
    let e1 = EventEnvelope::new(make_counter_event(), "test");
    let e2 = EventEnvelope::new(make_counter_event(), "test");

    assert_ne!(e1.id, e2.id, "each envelope must have a unique ID");
}

#[tokio::test]
async fn correlation_id_can_be_set() {
    let envelope = EventEnvelope::new(make_counter_event(), "test").with_correlation_id("corr-123");

    assert_eq!(
        envelope.metadata.correlation_id,
        Some("corr-123".to_string())
    );
}
