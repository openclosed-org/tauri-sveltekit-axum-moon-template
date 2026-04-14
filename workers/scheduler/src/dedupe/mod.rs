//! Job deduplication — prevents duplicate job executions.
//!
//! Tracks recently dispatched job instances to avoid running
//! the same job multiple times within a short window.

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

/// Tracks dispatched job IDs to prevent duplicate executions.
pub struct JobDedup {
    /// Set of recently dispatched job instance IDs.
    dispatched: RwLock<HashSet<String>>,
    /// Optional metadata about dispatch times.
    dispatch_times: RwLock<HashMap<String, u64>>,
    max_size: usize,
}

impl JobDedup {
    /// Create a new dedup tracker with the given capacity.
    pub fn new(max_size: usize) -> Self {
        Self {
            dispatched: RwLock::new(HashSet::with_capacity(max_size)),
            dispatch_times: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
        }
    }

    /// Check if a job instance has already been dispatched.
    pub fn is_duplicate(&self, job_instance_id: &str) -> bool {
        let dispatched = self.dispatched.read().unwrap();
        dispatched.contains(job_instance_id)
    }

    /// Mark a job instance as dispatched.
    /// Returns true if this is the first dispatch of this instance.
    pub fn mark_dispatched(&self, job_instance_id: &str, timestamp: u64) -> bool {
        let mut dispatched = self.dispatched.write().unwrap();
        let mut times = self.dispatch_times.write().unwrap();

        // Evict oldest entries if at capacity
        if dispatched.len() >= self.max_size {
            let to_remove: Vec<String> = dispatched
                .iter()
                .take(dispatched.len() / 4)
                .cloned()
                .collect();
            for key in to_remove {
                dispatched.remove(&key);
                times.remove(&key);
            }
        }

        let is_first = dispatched.insert(job_instance_id.to_string());
        if is_first {
            times.insert(job_instance_id.to_string(), timestamp);
        }
        is_first
    }

    /// Get the dispatch timestamp for a job instance, if recorded.
    pub fn dispatch_time(&self, job_instance_id: &str) -> Option<u64> {
        let times = self.dispatch_times.read().unwrap();
        times.get(job_instance_id).copied()
    }

    /// Clear all tracked job instances.
    pub fn clear(&self) {
        self.dispatched.write().unwrap().clear();
        self.dispatch_times.write().unwrap().clear();
    }
}

impl Default for JobDedup {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_dispatch_is_not_duplicate() {
        let dedup = JobDedup::new(100);
        assert!(!dedup.is_duplicate("job-1-instance-1"));
        assert!(dedup.mark_dispatched("job-1-instance-1", 1000));
    }

    #[test]
    fn second_dispatch_is_duplicate() {
        let dedup = JobDedup::new(100);
        dedup.mark_dispatched("job-1-instance-1", 1000);
        assert!(dedup.is_duplicate("job-1-instance-1"));
    }

    #[test]
    fn different_instances_not_duplicate() {
        let dedup = JobDedup::new(100);
        dedup.mark_dispatched("job-1-instance-1", 1000);
        assert!(!dedup.is_duplicate("job-1-instance-2"));
    }

    #[test]
    fn dispatch_time_is_recorded() {
        let dedup = JobDedup::new(100);
        dedup.mark_dispatched("job-1-instance-1", 12345);
        assert_eq!(dedup.dispatch_time("job-1-instance-1"), Some(12345));
    }

    #[test]
    fn clear_removes_all_entries() {
        let dedup = JobDedup::new(100);
        dedup.mark_dispatched("job-1", 1000);
        dedup.mark_dispatched("job-2", 2000);
        dedup.clear();
        assert!(!dedup.is_duplicate("job-1"));
        assert!(!dedup.is_duplicate("job-2"));
    }
}
