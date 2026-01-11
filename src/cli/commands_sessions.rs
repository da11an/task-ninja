// Sessions command handlers

use crate::db::DbConnection;
use crate::repo::{SessionRepo, TaskRepo, AnnotationRepo};
use crate::cli::error::{user_error, validate_task_id};
use crate::filter::{parse_filter, filter_tasks};
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use serde_json;

/// Format timestamp for display
fn format_timestamp(ts: i64) -> String {
    let dt = Local.timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(|| Local.timestamp_opt(0, 0).single().unwrap());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format duration for display
fn format_duration(secs: i64) -> String {
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

/// Handle `task [<id>] sessions list [--json]`
pub fn handle_task_sessions_list(task_id_opt: Option<String>, json: bool) -> Result<()> {
    let conn = DbConnection::connect()
        .context("Failed to connect to database")?;
    
    let sessions = if let Some(ref task_id_str) = task_id_opt {
        // List sessions for specific task
        let task_id = match validate_task_id(task_id_str) {
            Ok(id) => id,
            Err(e) => user_error(&e),
        };
        
        // Verify task exists
        if TaskRepo::get_by_id(&conn, task_id)?.is_none() {
            user_error(&format!("Task {} not found", task_id));
        }
        
        SessionRepo::get_by_task(&conn, task_id)?
    } else {
        // List all sessions
        SessionRepo::list_all(&conn)?
    };
    
    if json {
        // JSON output
        let mut json_sessions = Vec::new();
        for session in &sessions {
            let task = TaskRepo::get_by_id(&conn, session.task_id)?
                .ok_or_else(|| anyhow::anyhow!("Task {} not found", session.task_id))?;
            
            let json_session = serde_json::json!({
                "id": session.id,
                "task_id": session.task_id,
                "task_description": task.description,
                "start_ts": session.start_ts,
                "end_ts": session.end_ts,
                "duration_secs": session.duration_secs(),
                "is_open": session.is_open(),
            });
            
            json_sessions.push(json_session);
        }
        println!("{}", serde_json::to_string_pretty(&json_sessions)?);
    } else {
        // Human-readable output
        if sessions.is_empty() {
            println!("No sessions found.");
            return Ok(());
        }
        
        println!("{:<6} {:<40} {:<20} {:<20} {:<12}", "Task", "Description", "Start", "End", "Duration");
        println!("{}", "-".repeat(98));
        
        for session in &sessions {
            let task = TaskRepo::get_by_id(&conn, session.task_id)?
                .ok_or_else(|| anyhow::anyhow!("Task {} not found", session.task_id))?;
            
            let description = if task.description.len() > 38 {
                format!("{}..", &task.description[..38])
            } else {
                task.description.clone()
            };
            
            let start_str = format_timestamp(session.start_ts);
            let end_str = if let Some(end_ts) = session.end_ts {
                format_timestamp(end_ts)
            } else {
                "(running)".to_string()
            };
            
            let duration_str = if let Some(duration) = session.duration_secs() {
                format_duration(duration)
            } else {
                format_duration(chrono::Utc::now().timestamp() - session.start_ts)
            };
            
            println!("{:<6} {:<40} {:<20} {:<20} {:<12}", 
                session.task_id, description, start_str, end_str, duration_str);
        }
    }
    
    Ok(())
}

/// Handle `task [<id>] sessions show`
pub fn handle_task_sessions_show(task_id_opt: Option<String>) -> Result<()> {
    let conn = DbConnection::connect()
        .context("Failed to connect to database")?;
    
    let session = if let Some(ref task_id_str) = task_id_opt {
        // Show most recent session for specific task
        let task_id = match validate_task_id(task_id_str) {
            Ok(id) => id,
            Err(e) => user_error(&e),
        };
        
        // Verify task exists
        if TaskRepo::get_by_id(&conn, task_id)?.is_none() {
            user_error(&format!("Task {} not found", task_id));
        }
        
        SessionRepo::get_most_recent_for_task(&conn, task_id)?
    } else {
        // Show current running session
        SessionRepo::get_open(&conn)?
    };
    
    if let Some(session) = session {
        let task = TaskRepo::get_by_id(&conn, session.task_id)?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", session.task_id))?;
        
        // Get linked annotations
        let annotations = if let Some(session_id) = session.id {
            AnnotationRepo::get_by_session(&conn, session_id)?
        } else {
            Vec::new()
        };
        
        println!("Session {} (Task {})", 
            session.id.map(|id| id.to_string()).unwrap_or_else(|| "?".to_string()),
            session.task_id);
        println!("Description: {}", task.description);
        println!("Start: {}", format_timestamp(session.start_ts));
        
        if let Some(end_ts) = session.end_ts {
            println!("End: {}", format_timestamp(end_ts));
            if let Some(duration) = session.duration_secs() {
                println!("Duration: {}", format_duration(duration));
            }
        } else {
            let current_duration = chrono::Utc::now().timestamp() - session.start_ts;
            println!("End: (running)");
            println!("Duration: {} (running)", format_duration(current_duration));
        }
        
        if !annotations.is_empty() {
            println!("\nLinked Annotations:");
            for annotation in &annotations {
                println!("  [{}] {}", 
                    annotation.id.map(|id| id.to_string()).unwrap_or_else(|| "?".to_string()),
                    annotation.note);
            }
        }
    } else {
        if task_id_opt.is_some() {
            println!("No sessions found for this task.");
        } else {
            println!("No session is currently running.");
        }
    }
    
    Ok(())
}

/// Handle `task [<id|filter>] sessions list [--json]` with filter support
pub fn handle_task_sessions_list_with_filter(id_or_filter_opt: Option<String>, json: bool) -> Result<()> {
    let conn = DbConnection::connect()
        .context("Failed to connect to database")?;
    
    let sessions = if let Some(ref id_or_filter) = id_or_filter_opt {
        // Try to parse as task ID first, otherwise treat as filter
        match validate_task_id(id_or_filter) {
            Ok(task_id) => {
                // Single task ID
                if TaskRepo::get_by_id(&conn, task_id)?.is_none() {
                    user_error(&format!("Task {} not found", task_id));
                }
                SessionRepo::get_by_task(&conn, task_id)?
            }
            Err(_) => {
                // Treat as filter - aggregate sessions across all matching tasks
                let filter_expr = match parse_filter(vec![id_or_filter.clone()]) {
                    Ok(expr) => expr,
                    Err(e) => user_error(&format!("Filter parse error: {}", e)),
                };
                let matching_tasks = filter_tasks(&conn, &filter_expr)
                    .context("Failed to filter tasks")?;
                
                if matching_tasks.is_empty() {
                    return Ok(()); // No tasks, no sessions
                }
                
                let task_ids: Vec<i64> = matching_tasks.iter()
                    .filter_map(|(task, _)| task.id)
                    .collect();
                
                // Aggregate sessions from all matching tasks
                let mut all_sessions = Vec::new();
                for task_id in task_ids {
                    let mut task_sessions = SessionRepo::get_by_task(&conn, task_id)?;
                    all_sessions.append(&mut task_sessions);
                }
                
                // Sort by start time (newest first)
                all_sessions.sort_by(|a, b| b.start_ts.cmp(&a.start_ts));
                all_sessions
            }
        }
    } else {
        // List all sessions
        SessionRepo::list_all(&conn)?
    };
    
    if json {
        // JSON output
        let mut json_sessions = Vec::new();
        for session in &sessions {
            let task = TaskRepo::get_by_id(&conn, session.task_id)?
                .ok_or_else(|| anyhow::anyhow!("Task {} not found", session.task_id))?;
            
            let json_session = serde_json::json!({
                "id": session.id,
                "task_id": session.task_id,
                "task_description": task.description,
                "start_ts": session.start_ts,
                "end_ts": session.end_ts,
                "duration_secs": session.duration_secs(),
                "is_open": session.is_open(),
            });
            
            json_sessions.push(json_session);
        }
        println!("{}", serde_json::to_string_pretty(&json_sessions)?);
    } else {
        // Human-readable output
        if sessions.is_empty() {
            println!("No sessions found.");
            return Ok(());
        }
        
        println!("{:<6} {:<40} {:<20} {:<20} {:<12}", "Task", "Description", "Start", "End", "Duration");
        println!("{}", "-".repeat(98));
        
        for session in &sessions {
            let task = TaskRepo::get_by_id(&conn, session.task_id)?
                .ok_or_else(|| anyhow::anyhow!("Task {} not found", session.task_id))?;
            
            let description = if task.description.len() > 38 {
                format!("{}..", &task.description[..38])
            } else {
                task.description.clone()
            };
            
            let start_str = format_timestamp(session.start_ts);
            let end_str = if let Some(end_ts) = session.end_ts {
                format_timestamp(end_ts)
            } else {
                "(running)".to_string()
            };
            
            let duration_str = if let Some(duration) = session.duration_secs() {
                format_duration(duration)
            } else {
                format_duration(chrono::Utc::now().timestamp() - session.start_ts)
            };
            
            println!("{:<6} {:<40} {:<20} {:<20} {:<12}", 
                session.task_id, description, start_str, end_str, duration_str);
        }
    }
    
    Ok(())
}

/// Handle `task [<id|filter>] sessions show` with filter support
pub fn handle_task_sessions_show_with_filter(id_or_filter_opt: Option<String>) -> Result<()> {
    let conn = DbConnection::connect()
        .context("Failed to connect to database")?;
    
    let session = if let Some(ref id_or_filter) = id_or_filter_opt {
        // Try to parse as task ID first, otherwise treat as filter
        match validate_task_id(id_or_filter) {
            Ok(task_id) => {
                // Single task ID - show most recent session for this task
                if TaskRepo::get_by_id(&conn, task_id)?.is_none() {
                    user_error(&format!("Task {} not found", task_id));
                }
                SessionRepo::get_most_recent_for_task(&conn, task_id)?
            }
            Err(_) => {
                // Treat as filter - show most recent session from all matching tasks
                let filter_expr = match parse_filter(vec![id_or_filter.clone()]) {
                    Ok(expr) => expr,
                    Err(e) => user_error(&format!("Filter parse error: {}", e)),
                };
                let matching_tasks = filter_tasks(&conn, &filter_expr)
                    .context("Failed to filter tasks")?;
                
                if matching_tasks.is_empty() {
                    println!("No tasks found matching filter.");
                    return Ok(());
                }
                
                let task_ids: Vec<i64> = matching_tasks.iter()
                    .filter_map(|(task, _)| task.id)
                    .collect();
                
                // Find most recent session across all matching tasks
                let mut all_sessions = Vec::new();
                for task_id in task_ids {
                    if let Some(session) = SessionRepo::get_most_recent_for_task(&conn, task_id)? {
                        all_sessions.push(session);
                    }
                }
                
                // Get the most recent session overall
                all_sessions.sort_by(|a, b| b.start_ts.cmp(&a.start_ts));
                all_sessions.first().cloned()
            }
        }
    } else {
        // Show current running session
        SessionRepo::get_open(&conn)?
    };
    
    if let Some(session) = session {
        let task = TaskRepo::get_by_id(&conn, session.task_id)?
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", session.task_id))?;
        
        // Get linked annotations
        let annotations = if let Some(session_id) = session.id {
            AnnotationRepo::get_by_session(&conn, session_id)?
        } else {
            Vec::new()
        };
        
        println!("Session {} (Task {})", 
            session.id.map(|id| id.to_string()).unwrap_or_else(|| "?".to_string()),
            session.task_id);
        println!("Description: {}", task.description);
        println!("Start: {}", format_timestamp(session.start_ts));
        
        if let Some(end_ts) = session.end_ts {
            println!("End: {}", format_timestamp(end_ts));
            if let Some(duration) = session.duration_secs() {
                println!("Duration: {}", format_duration(duration));
            }
        } else {
            let current_duration = chrono::Utc::now().timestamp() - session.start_ts;
            println!("End: (running)");
            println!("Duration: {} (running)", format_duration(current_duration));
        }
        
        if !annotations.is_empty() {
            println!("\nLinked Annotations:");
            for annotation in &annotations {
                println!("  [{}] {}", 
                    annotation.id.map(|id| id.to_string()).unwrap_or_else(|| "?".to_string()),
                    annotation.note);
            }
        }
    } else {
        if id_or_filter_opt.is_some() {
            println!("No sessions found for this task/filter.");
        } else {
            println!("No session is currently running.");
        }
    }
    
    Ok(())
}
