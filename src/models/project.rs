use serde::{Deserialize, Serialize};

/// Project model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub is_archived: bool,
    pub created_ts: i64,
    pub modified_ts: i64,
}

impl Project {
    /// Create a new project
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            name,
            is_archived: false,
            created_ts: now,
            modified_ts: now,
        }
    }

    /// Check if this is a nested project (contains dots)
    pub fn is_nested(&self) -> bool {
        self.name.contains('.')
    }

    /// Get the parent project name (if nested)
    pub fn parent_name(&self) -> Option<String> {
        if let Some(last_dot) = self.name.rfind('.') {
            Some(self.name[..last_dot].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("work".to_string());
        assert_eq!(project.name, "work");
        assert!(!project.is_archived);
        assert!(project.id.is_none());
    }

    #[test]
    fn test_nested_project() {
        let project = Project::new("admin.email".to_string());
        assert!(project.is_nested());
        assert_eq!(project.parent_name(), Some("admin".to_string()));
    }

    #[test]
    fn test_deeply_nested_project() {
        let project = Project::new("sales.northamerica.texas".to_string());
        assert!(project.is_nested());
        assert_eq!(project.parent_name(), Some("sales.northamerica".to_string()));
    }
}
