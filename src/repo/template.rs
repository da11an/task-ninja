use rusqlite::{Connection, OptionalExtension};
use serde_json;
use anyhow::Result;
use std::collections::HashMap;

/// Template repository for database operations
pub struct TemplateRepo;

/// Template model (stored in database)
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub payload: HashMap<String, serde_json::Value>,
    pub created_ts: i64,
    pub modified_ts: i64,
}

impl TemplateRepo {
    /// Get a template by name
    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<Template>> {
        let mut stmt = conn.prepare(
            "SELECT name, payload_json, created_ts, modified_ts FROM templates WHERE name = ?1"
        )?;
        
        let template_opt = stmt.query_row([name], |row| {
            let name: String = row.get(0)?;
            let payload_json: String = row.get(1)?;
            let created_ts: i64 = row.get(2)?;
            let modified_ts: i64 = row.get(3)?;
            
            // Parse payload JSON
            let payload: HashMap<String, serde_json::Value> = serde_json::from_str(&payload_json)
                .map_err(|e| rusqlite::Error::InvalidColumnType(1, format!("Invalid JSON: {}", e), rusqlite::types::Type::Text))?;
            
            Ok(Template {
                name,
                payload,
                created_ts,
                modified_ts,
            })
        }).optional()?;
        
        Ok(template_opt)
    }
    
    /// Create or update a template
    /// For MVP, templates are created implicitly when used with --template flag
    /// This method is for explicit template creation/update
    pub fn save(conn: &Connection, name: &str, payload: &HashMap<String, serde_json::Value>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let payload_json = serde_json::to_string(payload)?;
        
        // Check if template exists
        let exists = Self::get_by_name(conn, name)?.is_some();
        
        if exists {
            // Update
            conn.execute(
                "UPDATE templates SET payload_json = ?1, modified_ts = ?2 WHERE name = ?3",
                rusqlite::params![payload_json, now, name],
            )?;
        } else {
            // Create
            conn.execute(
                "INSERT INTO templates (name, payload_json, created_ts, modified_ts) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![name, payload_json, now, now],
            )?;
        }
        
        Ok(())
    }
    
    /// Create a template from task attributes
    /// Used when creating a task with --template flag
    pub fn create_from_task(
        conn: &Connection,
        name: &str,
        project_id: Option<i64>,
        due_ts: Option<i64>,
        scheduled_ts: Option<i64>,
        wait_ts: Option<i64>,
        alloc_secs: Option<i64>,
        udas: &HashMap<String, String>,
        tags: &[String],
    ) -> Result<()> {
        let mut payload = HashMap::new();
        
        if let Some(pid) = project_id {
            payload.insert("project_id".to_string(), serde_json::Value::Number(pid.into()));
        }
        if let Some(due) = due_ts {
            payload.insert("due_ts".to_string(), serde_json::Value::Number(due.into()));
        }
        if let Some(scheduled) = scheduled_ts {
            payload.insert("scheduled_ts".to_string(), serde_json::Value::Number(scheduled.into()));
        }
        if let Some(wait) = wait_ts {
            payload.insert("wait_ts".to_string(), serde_json::Value::Number(wait.into()));
        }
        if let Some(alloc) = alloc_secs {
            payload.insert("alloc_secs".to_string(), serde_json::Value::Number(alloc.into()));
        }
        if !udas.is_empty() {
            let udas_value: HashMap<String, serde_json::Value> = udas.iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            payload.insert("udas".to_string(), serde_json::json!(udas_value));
        }
        if !tags.is_empty() {
            payload.insert("tags".to_string(), serde_json::json!(tags));
        }
        
        Self::save(conn, name, &payload)
    }
    
    /// Merge template attributes with task attributes
    /// Template provides base, task overrides
    pub fn merge_attributes(
        template: &Template,
        task_project_id: Option<i64>,
        task_due_ts: Option<i64>,
        task_scheduled_ts: Option<i64>,
        task_wait_ts: Option<i64>,
        task_alloc_secs: Option<i64>,
        task_udas: &HashMap<String, String>,
        task_tags: &[String],
    ) -> (
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        HashMap<String, String>,
        Vec<String>,
    ) {
        // Start with template values
        let mut project_id = template.payload.get("project_id")
            .and_then(|v| v.as_i64());
        let mut due_ts = template.payload.get("due_ts")
            .and_then(|v| v.as_i64());
        let mut scheduled_ts = template.payload.get("scheduled_ts")
            .and_then(|v| v.as_i64());
        let mut wait_ts = template.payload.get("wait_ts")
            .and_then(|v| v.as_i64());
        let mut alloc_secs = template.payload.get("alloc_secs")
            .and_then(|v| v.as_i64());
        
        let mut udas: HashMap<String, String> = if let Some(udas_value) = template.payload.get("udas") {
            if let Some(udas_map) = udas_value.as_object() {
                udas_map.iter()
                    .filter_map(|(k, v)| {
                        v.as_str().map(|s| (k.clone(), s.to_string()))
                    })
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        
        let mut tags: Vec<String> = if let Some(tags_value) = template.payload.get("tags") {
            if let Some(tags_array) = tags_value.as_array() {
                tags_array.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        // Override with task values (task takes precedence)
        if task_project_id.is_some() {
            project_id = task_project_id;
        }
        if task_due_ts.is_some() {
            due_ts = task_due_ts;
        }
        if task_scheduled_ts.is_some() {
            scheduled_ts = task_scheduled_ts;
        }
        if task_wait_ts.is_some() {
            wait_ts = task_wait_ts;
        }
        if task_alloc_secs.is_some() {
            alloc_secs = task_alloc_secs;
        }
        
        // Merge UDAs (task overrides template)
        for (k, v) in task_udas {
            udas.insert(k.clone(), v.clone());
        }
        
        // Merge tags (task adds to template)
        for tag in task_tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
        
        (project_id, due_ts, scheduled_ts, wait_ts, alloc_secs, udas, tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbConnection;

    #[test]
    fn test_template_create_and_get() {
        let conn = DbConnection::connect_in_memory().unwrap();
        
        let mut payload = HashMap::new();
        payload.insert("project_id".to_string(), serde_json::json!(1));
        payload.insert("alloc_secs".to_string(), serde_json::json!(3600));
        
        TemplateRepo::save(&conn, "meeting", &payload).unwrap();
        
        let template = TemplateRepo::get_by_name(&conn, "meeting").unwrap().unwrap();
        assert_eq!(template.name, "meeting");
        assert_eq!(template.payload.get("project_id").unwrap().as_i64(), Some(1));
        assert_eq!(template.payload.get("alloc_secs").unwrap().as_i64(), Some(3600));
    }
    
    #[test]
    fn test_template_merge_attributes() {
        let conn = DbConnection::connect_in_memory().unwrap();
        
        // Create template
        let mut template_payload = HashMap::new();
        template_payload.insert("project_id".to_string(), serde_json::json!(1));
        template_payload.insert("alloc_secs".to_string(), serde_json::json!(1800));
        template_payload.insert("tags".to_string(), serde_json::json!(["meeting", "recurring"]));
        TemplateRepo::save(&conn, "standup", &template_payload).unwrap();
        
        let template = TemplateRepo::get_by_name(&conn, "standup").unwrap().unwrap();
        
        // Merge with task attributes (task overrides template)
        let (project_id, _due_ts, _scheduled_ts, _wait_ts, alloc_secs, udas, tags) = 
            TemplateRepo::merge_attributes(
                &template,
                Some(2), // Task overrides project_id
                None,
                None,
                None,
                None, // Task doesn't override alloc_secs
                &HashMap::new(),
                &["urgent".to_string()], // Task adds tag
            );
        
        assert_eq!(project_id, Some(2)); // Task value
        assert_eq!(alloc_secs, Some(1800)); // Template value
        assert!(tags.contains(&"meeting".to_string())); // From template
        assert!(tags.contains(&"urgent".to_string())); // From task
    }
}
