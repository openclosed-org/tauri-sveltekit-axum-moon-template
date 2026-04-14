//! Replay strategy — controls how the projector replays events from the event store.
//!
//! Supports different strategies for replaying events:
//! - Beginning: Replay all events from the start
//! - Checkpoint: Resume from the last processed checkpoint
//! - Sequence: Replay from a specific event sequence number

/// Defines the strategy for replaying events.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ReplayStrategy {
    /// Replay all events from the beginning of the event store.
    Beginning,
    /// Resume replaying from the last processed checkpoint.
    #[default]
    Checkpoint,
    /// Replay from a specific sequence number (exclusive).
    Sequence(u64),
}

/// Manages the replay strategy configuration and determines the starting point.
pub struct ReplayManager {
    strategy: ReplayStrategy,
    fallback_checkpoint: u64,
}

impl ReplayManager {
    /// Create a new replay manager with the given strategy.
    pub fn new(strategy: ReplayStrategy) -> Self {
        Self {
            strategy,
            fallback_checkpoint: 0,
        }
    }

    /// Set the fallback checkpoint value for FromCheckpoint strategy.
    pub fn with_fallback_checkpoint(mut self, checkpoint: u64) -> Self {
        self.fallback_checkpoint = checkpoint;
        self
    }

    /// Determine the starting sequence number for replay.
    pub fn start_sequence(&self) -> u64 {
        match &self.strategy {
            ReplayStrategy::Beginning => 0,
            ReplayStrategy::Checkpoint => self.fallback_checkpoint,
            ReplayStrategy::Sequence(seq) => *seq,
        }
    }

    /// Get the current replay strategy.
    pub fn strategy(&self) -> &ReplayStrategy {
        &self.strategy
    }

    /// Update the replay strategy.
    pub fn set_strategy(&mut self, strategy: ReplayStrategy) {
        self.strategy = strategy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_beginning_starts_at_zero() {
        let manager = ReplayManager::new(ReplayStrategy::Beginning);
        assert_eq!(manager.start_sequence(), 0);
    }

    #[test]
    fn from_checkpoint_uses_fallback() {
        let manager = ReplayManager::new(ReplayStrategy::Checkpoint).with_fallback_checkpoint(100);
        assert_eq!(manager.start_sequence(), 100);
    }

    #[test]
    fn from_sequence_uses_specified_value() {
        let manager = ReplayManager::new(ReplayStrategy::Sequence(500));
        assert_eq!(manager.start_sequence(), 500);
    }

    #[test]
    fn default_strategy_is_from_checkpoint() {
        let strategy = ReplayStrategy::default();
        assert_eq!(strategy, ReplayStrategy::Checkpoint);
    }

    #[test]
    fn strategy_can_be_updated() {
        let mut manager = ReplayManager::new(ReplayStrategy::Beginning);
        assert_eq!(manager.start_sequence(), 0);

        manager.set_strategy(ReplayStrategy::Sequence(200));
        assert_eq!(manager.start_sequence(), 200);
    }
}
