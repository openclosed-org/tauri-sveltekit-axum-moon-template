//! Session entity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session entity — represents an authenticated user session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub user_sub: String,
    pub tenant_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    /// Check if the session has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update the last accessed timestamp.
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }
}
