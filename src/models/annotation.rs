use serde::{Deserialize, Serialize};

/// Task annotation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Option<i64>,
    pub task_id: i64,
    pub session_id: Option<i64>,
    pub note: String,
    pub entry_ts: i64,
    pub created_ts: i64,
}

impl Annotation {
    /// Create a new annotation
    pub fn new(task_id: i64, note: String, session_id: Option<i64>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            task_id,
            session_id,
            note,
            entry_ts: now,
            created_ts: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_creation() {
        let annotation = Annotation::new(1, "Test note".to_string(), None);
        assert_eq!(annotation.task_id, 1);
        assert_eq!(annotation.note, "Test note");
        assert!(annotation.session_id.is_none());
    }

    #[test]
    fn test_annotation_with_session() {
        let annotation = Annotation::new(1, "Note during session".to_string(), Some(5));
        assert_eq!(annotation.session_id, Some(5));
    }
}
