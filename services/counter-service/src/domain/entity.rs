//! Counter entity and value objects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque identifier for a Counter aggregate.
///
/// In the current implementation this is the tenant_id, but
/// future versions may introduce independent counter aggregates
/// (e.g. per-resource, per-feature counters).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CounterId(pub String);

impl CounterId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CounterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Counter aggregate — tracks a monotonically adjustable integer per tenant.
///
/// ## Invariants
/// - `value` starts at 0 for new counters
/// - `version` starts at 0 and increments on every mutation (CAS optimistic locking)
/// - `updated_at` is set on every mutation
/// - Counter is scoped to a single tenant (no cross-tenant operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    pub id: CounterId,
    pub value: i64,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

impl Counter {
    /// Create a new counter at zero.
    pub fn new(id: CounterId, now: DateTime<Utc>) -> Self {
        Self {
            id,
            value: 0,
            version: 0,
            updated_at: now,
        }
    }

    /// Return a new Counter with value incremented.
    pub fn increment(self) -> Self {
        Self {
            value: self.value + 1,
            version: self.version + 1,
            ..self
        }
    }

    /// Return a new Counter with value decremented.
    pub fn decrement(self) -> Self {
        Self {
            value: self.value - 1,
            version: self.version + 1,
            ..self
        }
    }

    /// Return a new Counter with value reset to zero.
    pub fn reset(self) -> Self {
        Self {
            value: 0,
            version: self.version + 1,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn new_counter_starts_at_zero() {
        let now = Utc::now();
        let c = Counter::new(CounterId::new("tenant-1"), now);
        assert_eq!(c.value, 0);
        assert_eq!(c.version, 0);
        assert_eq!(c.updated_at, now);
    }

    #[test]
    fn increment_adds_one() {
        let now = Utc::now();
        let c = Counter::new(CounterId::new("t1"), now).increment();
        assert_eq!(c.value, 1);
        assert_eq!(c.version, 1);
    }

    #[test]
    fn decrement_subtracts_one() {
        let now = Utc::now();
        let c = Counter::new(CounterId::new("t1"), now).decrement();
        assert_eq!(c.value, -1);
        assert_eq!(c.version, 1);
    }

    #[test]
    fn reset_returns_zero() {
        let now = Utc::now();
        let c = Counter::new(CounterId::new("t1"), now)
            .increment()
            .increment()
            .reset();
        assert_eq!(c.value, 0);
        assert_eq!(c.version, 3);
    }
}
