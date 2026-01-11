use serde::{Deserialize, Serialize};

/// Session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Option<i64>,
    pub task_id: i64,
    pub start_ts: i64,
    pub end_ts: Option<i64>,
    pub created_ts: i64,
}

impl Session {
    /// Create a new session
    pub fn new(task_id: i64, start_ts: i64) -> Self {
        Self {
            id: None,
            task_id,
            start_ts,
            end_ts: None,
            created_ts: chrono::Utc::now().timestamp(),
        }
    }

    /// Check if session is open (not ended)
    pub fn is_open(&self) -> bool {
        self.end_ts.is_none()
    }

    /// Get session duration in seconds
    pub fn duration_secs(&self) -> Option<i64> {
        self.end_ts.map(|end| end - self.start_ts)
    }

    /// Close the session
    pub fn close(&mut self, end_ts: i64) {
        self.end_ts = Some(end_ts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(1, 1000);
        assert_eq!(session.task_id, 1);
        assert_eq!(session.start_ts, 1000);
        assert!(session.is_open());
    }

    #[test]
    fn test_session_close() {
        let mut session = Session::new(1, 1000);
        session.close(2000);
        assert!(!session.is_open());
        assert_eq!(session.duration_secs(), Some(1000));
    }
}
