//! Reconciliation checkpoint — tracks the last sync state.
//!
/// Stores the timestamp and sequence of the last successful
/// reconciliation for each plan, enabling resumable sync operations.
use std::collections::HashMap;
use std::sync::RwLock;

/// Represents the state of a reconciliation checkpoint.
#[derive(Debug, Clone)]
pub struct CheckpointState {
    /// The plan ID this checkpoint belongs to.
    pub plan_id: String,
    /// Timestamp of the last successful reconciliation (Unix epoch seconds).
    pub last_sync_timestamp: u64,
    /// Sequence number or version of the last successful reconciliation.
    pub last_sync_sequence: u64,
    /// Number of conflicts resolved in the last sync.
    pub conflicts_resolved: u32,
}

/// Manages checkpoints for reconciliation plans.
pub struct ReconcileCheckpoint {
    checkpoints: RwLock<HashMap<String, CheckpointState>>,
}

impl ReconcileCheckpoint {
    pub fn new() -> Self {
        Self {
            checkpoints: RwLock::new(HashMap::new()),
        }
    }

    /// Get the checkpoint state for a plan, if one exists.
    pub fn get(&self, plan_id: &str) -> Option<CheckpointState> {
        let checkpoints = self.checkpoints.read().unwrap();
        checkpoints.get(plan_id).cloned()
    }

    /// Record a successful reconciliation checkpoint.
    pub fn record_success(
        &self,
        plan_id: &str,
        timestamp: u64,
        sequence: u64,
        conflicts_resolved: u32,
    ) {
        let mut checkpoints = self.checkpoints.write().unwrap();
        checkpoints.insert(
            plan_id.to_string(),
            CheckpointState {
                plan_id: plan_id.to_string(),
                last_sync_timestamp: timestamp,
                last_sync_sequence: sequence,
                conflicts_resolved,
            },
        );
    }

    /// Get the last sync sequence for a plan. Returns 0 if no checkpoint exists.
    pub fn last_sequence(&self, plan_id: &str) -> u64 {
        self.get(plan_id)
            .map(|cp| cp.last_sync_sequence)
            .unwrap_or(0)
    }

    /// Get the last sync timestamp for a plan. Returns 0 if no checkpoint exists.
    pub fn last_timestamp(&self, plan_id: &str) -> u64 {
        self.get(plan_id)
            .map(|cp| cp.last_sync_timestamp)
            .unwrap_or(0)
    }

    /// Remove the checkpoint for a plan.
    pub fn remove(&self, plan_id: &str) -> Option<CheckpointState> {
        let mut checkpoints = self.checkpoints.write().unwrap();
        checkpoints.remove(plan_id)
    }

    /// List all tracked checkpoints.
    pub fn list_checkpoints(&self) -> Vec<CheckpointState> {
        let checkpoints = self.checkpoints.read().unwrap();
        checkpoints.values().cloned().collect()
    }
}

impl Default for ReconcileCheckpoint {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_plan_has_no_checkpoint() {
        let cp = ReconcileCheckpoint::new();
        assert!(cp.get("plan-1").is_none());
        assert_eq!(cp.last_sequence("plan-1"), 0);
    }

    #[test]
    fn record_and_retrieve_checkpoint() {
        let cp = ReconcileCheckpoint::new();
        cp.record_success("plan-1", 1000, 5, 2);

        let state = cp.get("plan-1").unwrap();
        assert_eq!(state.plan_id, "plan-1");
        assert_eq!(state.last_sync_timestamp, 1000);
        assert_eq!(state.last_sync_sequence, 5);
        assert_eq!(state.conflicts_resolved, 2);
    }

    #[test]
    fn checkpoint_updates_on_new_success() {
        let cp = ReconcileCheckpoint::new();
        cp.record_success("plan-1", 1000, 5, 2);
        cp.record_success("plan-1", 2000, 10, 0);

        let state = cp.get("plan-1").unwrap();
        assert_eq!(state.last_sync_timestamp, 2000);
        assert_eq!(state.last_sync_sequence, 10);
    }

    #[test]
    fn list_checkpoints_returns_all() {
        let cp = ReconcileCheckpoint::new();
        cp.record_success("plan-1", 1000, 1, 0);
        cp.record_success("plan-2", 2000, 2, 1);

        let all = cp.list_checkpoints();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn remove_checkpoint() {
        let cp = ReconcileCheckpoint::new();
        cp.record_success("plan-1", 1000, 1, 0);
        let removed = cp.remove("plan-1");
        assert!(removed.is_some());
        assert!(cp.get("plan-1").is_none());
    }
}
