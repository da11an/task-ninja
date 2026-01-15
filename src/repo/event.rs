use rusqlite::Connection;
use serde_json;
use anyhow::Result;

/// Event repository for recording immutable task events
pub struct EventRepo;

/// Event types
#[derive(Debug, Clone)]
pub enum EventType {
    Created,
    Modified,
    StatusChanged,
    TagAdded,
    TagRemoved,
    AnnotationAdded,
    AnnotationDeleted,
    StackAdded,
    StackRemoved,
    SessionStarted,
    SessionEnded,
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::Created => "created",
            EventType::Modified => "modified",
            EventType::StatusChanged => "status_changed",
            EventType::TagAdded => "tag_added",
            EventType::TagRemoved => "tag_removed",
            EventType::AnnotationAdded => "annotation_added",
            EventType::AnnotationDeleted => "annotation_deleted",
            EventType::StackAdded => "stack_added",
            EventType::StackRemoved => "stack_removed",
            EventType::SessionStarted => "session_started",
            EventType::SessionEnded => "session_ended",
        }
    }
}

impl EventRepo {
    /// Record an event (immutable - never modified or deleted)
    pub fn record(
        conn: &Connection,
        task_id: i64,
        event_type: EventType,
        payload: serde_json::Value,
    ) -> Result<()> {
        let ts = chrono::Utc::now().timestamp();
        let payload_json = serde_json::to_string(&payload)?;
        
        conn.execute(
            "INSERT INTO task_events (task_id, ts, event_type, payload_json) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, ts, event_type.as_str(), payload_json],
        )?;
        
        Ok(())
    }

    /// Record task created event
    pub fn record_created(
        conn: &Connection,
        task_id: i64,
        description: &str,
        project_id: Option<i64>,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "description": description,
            "project_id": project_id,
        });
        Self::record(conn, task_id, EventType::Created, payload)
    }

    /// Record task modified event
    pub fn record_modified(
        conn: &Connection,
        task_id: i64,
        field: &str,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "field": field,
            "old_value": old_value,
            "new_value": new_value,
        });
        Self::record(conn, task_id, EventType::Modified, payload)
    }

    /// Record status changed event
    pub fn record_status_changed(
        conn: &Connection,
        task_id: i64,
        old_status: &str,
        new_status: &str,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "old_status": old_status,
            "new_status": new_status,
        });
        Self::record(conn, task_id, EventType::StatusChanged, payload)
    }

    /// Record tag added event
    pub fn record_tag_added(
        conn: &Connection,
        task_id: i64,
        tag: &str,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "tag": tag,
        });
        Self::record(conn, task_id, EventType::TagAdded, payload)
    }

    /// Record tag removed event
    pub fn record_tag_removed(
        conn: &Connection,
        task_id: i64,
        tag: &str,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "tag": tag,
        });
        Self::record(conn, task_id, EventType::TagRemoved, payload)
    }

    /// Record annotation added event
    pub fn record_annotation_added(
        conn: &Connection,
        task_id: i64,
        annotation_id: i64,
        session_id: Option<i64>,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "annotation_id": annotation_id,
            "session_id": session_id,
        });
        Self::record(conn, task_id, EventType::AnnotationAdded, payload)
    }

    /// Record annotation deleted event
    pub fn record_annotation_deleted(
        conn: &Connection,
        task_id: i64,
        annotation_id: i64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "annotation_id": annotation_id,
        });
        Self::record(conn, task_id, EventType::AnnotationDeleted, payload)
    }

    /// Record stack added event
    pub fn record_stack_added(
        conn: &Connection,
        task_id: i64,
        stack_id: i64,
        position: i32,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "stack_id": stack_id,
            "position": position,
        });
        Self::record(conn, task_id, EventType::StackAdded, payload)
    }

    /// Record stack removed event
    pub fn record_stack_removed(
        conn: &Connection,
        task_id: i64,
        stack_id: i64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "stack_id": stack_id,
        });
        Self::record(conn, task_id, EventType::StackRemoved, payload)
    }

    /// Record session started event
    pub fn record_session_started(
        conn: &Connection,
        task_id: i64,
        session_id: i64,
        start_ts: i64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "session_id": session_id,
            "start_ts": start_ts,
        });
        Self::record(conn, task_id, EventType::SessionStarted, payload)
    }

    /// Record session ended event
    pub fn record_session_ended(
        conn: &Connection,
        task_id: i64,
        session_id: i64,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<()> {
        let payload = serde_json::json!({
            "session_id": session_id,
            "start_ts": start_ts,
            "end_ts": end_ts,
        });
        Self::record(conn, task_id, EventType::SessionEnded, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbConnection;
    use crate::repo::TaskRepo;

    #[test]
    fn test_record_created() {
        let conn = DbConnection::connect_in_memory().unwrap();
        let task = TaskRepo::create(&conn, "Test task", None).unwrap();
        let task_id = task.id.unwrap();
        
        // Verify event was recorded (TaskRepo::create should have recorded it)
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM task_events WHERE task_id = ?1 AND event_type = 'created'").unwrap();
        let count: i64 = stmt.query_row([task_id], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_record_status_changed() {
        let conn = DbConnection::connect_in_memory().unwrap();
        let task = TaskRepo::create(&conn, "Test task", None).unwrap();
        let task_id = task.id.unwrap();
        
        EventRepo::record_status_changed(&conn, task_id, "pending", "completed").unwrap();
        
        // Verify event was recorded
        let mut stmt = conn.prepare("SELECT payload_json FROM task_events WHERE task_id = ?1 AND event_type = 'status_changed'").unwrap();
        let payload: String = stmt.query_row([task_id], |row| row.get(0)).unwrap();
        let payload_value: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(payload_value["old_status"], "pending");
        assert_eq!(payload_value["new_status"], "completed");
    }
}
