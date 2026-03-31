use serde::{Deserialize, Serialize};
use std::fmt;

/// Strategy for resolving conflicts when both local and remote have changes.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// The last push to the server wins. Remote changes overwrite local.
    #[default]
    LastPushWins,
    /// The last write by timestamp wins. Compares modification times.
    LastWriteWins,
    /// Conflict is logged and marked for manual resolution.
    Manual,
}

impl fmt::Display for ConflictStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConflictStrategy::LastPushWins => write!(f, "last_push_wins"),
            ConflictStrategy::LastWriteWins => write!(f, "last_write_wins"),
            ConflictStrategy::Manual => write!(f, "manual"),
        }
    }
}

/// Record of a detected conflict for audit/logging purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    /// Table or resource where the conflict occurred.
    pub table: String,
    /// Primary key or identifier of the conflicting row.
    pub row_id: String,
    /// Strategy used to resolve the conflict.
    pub strategy: ConflictStrategy,
    /// Resolution outcome: "local_won", "remote_won", "pending".
    pub resolution: String,
    /// Timestamp of the conflict (ISO 8601).
    pub timestamp: String,
}

impl ConflictRecord {
    pub fn new(table: &str, row_id: &str, strategy: ConflictStrategy, resolution: &str) -> Self {
        Self {
            table: table.to_string(),
            row_id: row_id.to_string(),
            strategy,
            resolution: resolution.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Resolve a conflict between local and remote frames using the given strategy.
///
/// Returns the resolution outcome: "local_won", "remote_won", or "pending".
pub fn resolve_conflict(
    strategy: ConflictStrategy,
    local_ts: Option<i64>,
    remote_ts: Option<i64>,
) -> &'static str {
    match strategy {
        ConflictStrategy::LastPushWins => {
            // Remote (last push to server) wins
            "remote_won"
        }
        ConflictStrategy::LastWriteWins => {
            // Compare timestamps; remote wins on tie or newer
            match (local_ts, remote_ts) {
                (Some(l), Some(r)) if l > r => "local_won",
                _ => "remote_won",
            }
        }
        ConflictStrategy::Manual => {
            // Log and defer to user
            "pending"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_push_wins_always_remote() {
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastPushWins, Some(100), Some(50)),
            "remote_won"
        );
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastPushWins, Some(50), Some(100)),
            "remote_won"
        );
    }

    #[test]
    fn test_last_write_wins_local_newer() {
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastWriteWins, Some(200), Some(100)),
            "local_won"
        );
    }

    #[test]
    fn test_last_write_wins_remote_newer_or_equal() {
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastWriteWins, Some(100), Some(200)),
            "remote_won"
        );
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastWriteWins, Some(100), Some(100)),
            "remote_won"
        );
    }

    #[test]
    fn test_last_write_wins_no_timestamps() {
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastWriteWins, None, None),
            "remote_won"
        );
        assert_eq!(
            resolve_conflict(ConflictStrategy::LastWriteWins, Some(100), None),
            "remote_won"
        );
    }

    #[test]
    fn test_manual_always_pending() {
        assert_eq!(
            resolve_conflict(ConflictStrategy::Manual, Some(100), Some(50)),
            "pending"
        );
    }

    #[test]
    fn test_conflict_record_serialization() {
        let record =
            ConflictRecord::new("users", "u-1", ConflictStrategy::LastPushWins, "remote_won");
        assert_eq!(record.table, "users");
        assert_eq!(record.row_id, "u-1");
        assert!(!record.timestamp.is_empty());
    }

    #[test]
    fn test_conflict_strategy_display() {
        assert_eq!(ConflictStrategy::LastPushWins.to_string(), "last_push_wins");
        assert_eq!(
            ConflictStrategy::LastWriteWins.to_string(),
            "last_write_wins"
        );
        assert_eq!(ConflictStrategy::Manual.to_string(), "manual");
    }

    #[test]
    fn test_conflict_strategy_default() {
        assert_eq!(ConflictStrategy::default(), ConflictStrategy::LastPushWins);
    }

    #[test]
    fn test_conflict_strategy_serde() {
        let strategy = ConflictStrategy::LastWriteWins;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, r#""last_write_wins""#);

        let deserialized: ConflictStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, strategy);
    }
}
