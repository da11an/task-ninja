// Filter expression evaluator
// Evaluates filter expressions against tasks

use crate::models::Task;
use crate::repo::TaskRepo;
use crate::filter::parser::FilterTerm;
use rusqlite::Connection;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum FilterExpr {
    All, // Match all
    Term(FilterTerm),
    And(Vec<FilterExpr>),
    Or(Vec<FilterExpr>),
    Not(Box<FilterExpr>),
}

impl FilterExpr {
    /// Evaluate filter against a task
    pub fn matches(&self, task: &Task, conn: &Connection) -> Result<bool> {
        match self {
            FilterExpr::All => Ok(true),
            FilterExpr::Term(term) => term.matches(task, conn),
            FilterExpr::And(exprs) => {
                for expr in exprs {
                    if !expr.matches(task, conn)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            FilterExpr::Or(exprs) => {
                for expr in exprs {
                    if expr.matches(task, conn)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            FilterExpr::Not(expr) => {
                Ok(!expr.matches(task, conn)?)
            }
        }
    }
}

impl FilterTerm {
    fn matches(&self, task: &Task, conn: &Connection) -> Result<bool> {
        match self {
            FilterTerm::Id(id) => {
                Ok(task.id == Some(*id))
            }
            FilterTerm::Status(status) => {
                Ok(task.status.as_str() == status)
            }
            FilterTerm::Project(project_name) => {
                // For now, simple exact match
                // TODO: Implement nested project prefix matching in Phase 3.2
                if let Some(project_id) = task.project_id {
                    // Get project name from database by ID
                    let mut stmt = conn.prepare("SELECT name FROM projects WHERE id = ?1")?;
                    let project_name_opt: Option<String> = stmt.query_row([project_id], |row| row.get(0)).ok();
                    if let Some(pname) = project_name_opt {
                        Ok(pname == *project_name || pname.starts_with(&format!("{}.", project_name)))
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            FilterTerm::Tag(tag, is_positive) => {
                let tags = TaskRepo::get_tags(conn, task.id.unwrap())?;
                let has_tag = tags.contains(tag);
                Ok(if *is_positive { has_tag } else { !has_tag })
            }
            FilterTerm::Due(_expr) => {
                // TODO: Implement date expression matching in Phase 3.2
                Ok(true) // Placeholder
            }
            FilterTerm::Scheduled(_expr) => {
                // TODO: Implement date expression matching in Phase 3.2
                Ok(true) // Placeholder
            }
            FilterTerm::Wait(_expr) => {
                // TODO: Implement date expression matching in Phase 3.2
                Ok(true) // Placeholder
            }
            FilterTerm::Waiting => {
                Ok(task.is_waiting())
            }
        }
    }
}

/// Get tasks matching a filter expression
pub fn filter_tasks(conn: &Connection, filter: &FilterExpr) -> Result<Vec<(Task, Vec<String>)>> {
    let all_tasks = TaskRepo::list_all(conn)?;
    let mut matching = Vec::new();
    
    for (task, tags) in all_tasks {
        if filter.matches(&task, conn)? {
            matching.push((task, tags));
        }
    }
    
    Ok(matching)
}
