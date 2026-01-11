// Output formatting utilities

use crate::models::Task;
use crate::repo::ProjectRepo;
use chrono::Local;
use rusqlite::Connection;
use anyhow::Result;

/// Format timestamp for display
pub fn format_timestamp(ts: i64) -> String {
    use chrono::TimeZone;
    let dt = Local.timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(|| Local.timestamp_opt(0, 0).single().unwrap());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format date for display (date only, no time)
pub fn format_date(ts: i64) -> String {
    use chrono::TimeZone;
    let dt = Local.timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(|| Local.timestamp_opt(0, 0).single().unwrap());
    dt.format("%Y-%m-%d").to_string()
}

/// Format duration for display
pub fn format_duration(secs: i64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    
    if hours > 0 {
        format!("{}h{}m{}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m{}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format task list as a table
pub fn format_task_list_table(
    conn: &Connection,
    tasks: &[(Task, Vec<String>)],
) -> Result<String> {
    if tasks.is_empty() {
        return Ok("No tasks found.".to_string());
    }
    
    // Calculate column widths
    let mut id_width = 4;
    let mut desc_width = 20;
    let mut status_width = 10;
    let mut project_width = 15;
    let mut tags_width = 20;
    let mut due_width = 12;
    
    // First pass: calculate widths
    for (task, tags) in tasks {
        id_width = id_width.max(task.id.map(|id| id.to_string().len()).unwrap_or(0));
        desc_width = desc_width.max(task.description.len().min(50));
        status_width = status_width.max(task.status.as_str().len());
        
        if let Some(project_id) = task.project_id {
            if let Ok(Some(project)) = ProjectRepo::get_by_id(conn, project_id) {
                project_width = project_width.max(project.name.len().min(15));
            }
        }
        
        if !tags.is_empty() {
            let tag_str = tags.iter().map(|t| format!("+{}", t)).collect::<Vec<_>>().join(" ");
            tags_width = tags_width.max(tag_str.len().min(30));
        }
        
        if task.due_ts.is_some() {
            due_width = due_width.max(12);
        }
    }
    
    // Build header
    let mut output = String::new();
    output.push_str(&format!(
        "{:<id$} {:<desc$} {:<status$} {:<project$} {:<tags$} {:<due$}\n",
        "ID", "Description", "Status", "Project", "Tags", "Due",
        id = id_width,
        desc = desc_width,
        status = status_width,
        project = project_width,
        tags = tags_width,
        due = due_width
    ));
    
    // Separator line
    let total_width = id_width + desc_width + status_width + project_width + tags_width + due_width + 5;
    output.push_str(&format!("{}\n", "-".repeat(total_width)));
    
    // Build rows
    for (task, tags) in tasks {
        let id = task.id.map(|id| id.to_string()).unwrap_or_else(|| "?".to_string());
        
        let desc = if task.description.len() > desc_width {
            format!("{}..", &task.description[..desc_width.saturating_sub(2)])
        } else {
            task.description.clone()
        };
        
        let status = task.status.as_str();
        
        let project = if let Some(project_id) = task.project_id {
            if let Ok(Some(proj)) = ProjectRepo::get_by_id(conn, project_id) {
                if proj.name.len() > project_width {
                    format!("{}..", &proj.name[..project_width.saturating_sub(2)])
                } else {
                    proj.name
                }
            } else {
                format!("[{}]", project_id)
            }
        } else {
            String::new()
        };
        
        let tag_str = if !tags.is_empty() {
            let full = tags.iter().map(|t| format!("+{}", t)).collect::<Vec<_>>().join(" ");
            if full.len() > tags_width {
                format!("{}..", &full[..tags_width.saturating_sub(2)])
            } else {
                full
            }
        } else {
            String::new()
        };
        
        let due = if let Some(due_ts) = task.due_ts {
            format_date(due_ts)
        } else {
            String::new()
        };
        
        output.push_str(&format!(
            "{:<id$} {:<desc$} {:<status$} {:<project$} {:<tags$} {:<due$}\n",
            id, desc, status, project, tag_str, due,
            id = id_width,
            desc = desc_width,
            status = status_width,
            project = project_width,
            tags = tags_width,
            due = due_width
        ));
    }
    
    Ok(output)
}

/// Format stack display
pub fn format_stack_display(items: &[(i64, i32)]) -> String {
    if items.is_empty() {
        return "Stack is empty.".to_string();
    }
    
    let mut output = String::new();
    output.push_str("Stack:\n");
    
    for (idx, (task_id, _ordinal)) in items.iter().enumerate() {
        output.push_str(&format!("  [{}] Task {}\n", idx, task_id));
    }
    
    output
}

/// Format clock transition message
pub fn format_clock_transition(
    action: &str,
    task_id: Option<i64>,
    task_description: Option<&str>,
) -> String {
    match (action, task_id, task_description) {
        ("started", Some(id), Some(desc)) => {
            format!("Started timing task {}: {}", id, desc)
        }
        ("started", Some(id), None) => {
            format!("Started timing task {}", id)
        }
        ("stopped", Some(id), Some(desc)) => {
            format!("Stopped timing task {}: {}", id, desc)
        }
        ("stopped", Some(id), None) => {
            format!("Stopped timing task {}", id)
        }
        ("switched", Some(old_id), _) => {
            format!("Switched from task {} to task {}", old_id, task_id.unwrap_or(0))
        }
        _ => format!("Clock {}", action)
    }
}
