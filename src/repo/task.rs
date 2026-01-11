use rusqlite::{Connection, OptionalExtension};
use crate::models::Task;
use anyhow::{Context, Result};

/// Task repository for database operations
pub struct TaskRepo;

impl TaskRepo {
    /// Create a new task
    pub fn create(conn: &Connection, description: &str, project_id: Option<i64>) -> Result<Task> {
        let mut task = Task::new(description.to_string());
        task.project_id = project_id;
        
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO tasks (uuid, description, status, project_id, created_ts, modified_ts) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                task.uuid,
                task.description,
                task.status.as_str(),
                task.project_id,
                now,
                now
            ],
        )
        .with_context(|| format!("Failed to create task: {}", description))?;
        
        let id = conn.last_insert_rowid();
        Ok(Task {
            id: Some(id),
            ..task
        })
    }

    /// Get task by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Task>> {
        let mut stmt = conn.prepare(
            "SELECT id, uuid, description, status, project_id, due_ts, scheduled_ts, 
                    wait_ts, alloc_secs, template, recur, udas_json, created_ts, modified_ts 
             FROM tasks WHERE id = ?1"
        )?;
        
        let task = stmt.query_row([id], |row| {
            let udas_json: Option<String> = row.get(11)?;
            let mut udas = std::collections::HashMap::new();
            if let Some(json) = udas_json {
                if let Ok(parsed) = serde_json::from_str::<std::collections::HashMap<String, String>>(&json) {
                    udas = parsed;
                }
            }
            
            Ok(Task {
                id: Some(row.get(0)?),
                uuid: row.get(1)?,
                description: row.get(2)?,
                status: crate::models::TaskStatus::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(crate::models::TaskStatus::Pending),
                project_id: row.get(4)?,
                due_ts: row.get(5)?,
                scheduled_ts: row.get(6)?,
                wait_ts: row.get(7)?,
                alloc_secs: row.get(8)?,
                template: row.get(9)?,
                recur: row.get(10)?,
                udas,
                created_ts: row.get(12)?,
                modified_ts: row.get(13)?,
            })
        }).optional()?;
        
        Ok(task)
    }
}
