use serde::{Deserialize, Serialize};

/// External workflow model
/// Represents a task that has been sent to an external party for review/approval/etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct External {
    pub id: Option<i64>,
    pub task_id: i64,
    pub recipient: String,
    pub request: Option<String>,
    pub sent_ts: i64,
    pub returned_ts: Option<i64>,
    pub created_ts: i64,
    pub modified_ts: i64,
}

impl External {
    /// Create a new external record
    pub fn new(task_id: i64, recipient: String, request: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            task_id,
            recipient,
            request,
            sent_ts: now,
            returned_ts: None,
            created_ts: now,
            modified_ts: now,
        }
    }
    
    /// Check if the external is still active (not returned)
    pub fn is_active(&self) -> bool {
        self.returned_ts.is_none()
    }
}
