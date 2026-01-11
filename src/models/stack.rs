use serde::{Deserialize, Serialize};

/// Stack model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub id: Option<i64>,
    pub name: String,
    pub created_ts: i64,
    pub modified_ts: i64,
}

impl Stack {
    /// Create a new stack
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            name,
            created_ts: now,
            modified_ts: now,
        }
    }

    /// Create the default stack
    pub fn default() -> Self {
        Self::new("default".to_string())
    }
}

/// Stack item (task in stack with position)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackItem {
    pub stack_id: i64,
    pub task_id: i64,
    pub ordinal: i32,
    pub added_ts: i64,
}

impl StackItem {
    /// Create a new stack item
    pub fn new(stack_id: i64, task_id: i64, ordinal: i32) -> Self {
        Self {
            stack_id,
            task_id,
            ordinal,
            added_ts: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_creation() {
        let stack = Stack::new("work".to_string());
        assert_eq!(stack.name, "work");
        assert!(stack.id.is_none());
    }

    #[test]
    fn test_default_stack() {
        let stack = Stack::default();
        assert_eq!(stack.name, "default");
    }

    #[test]
    fn test_stack_item_creation() {
        let item = StackItem::new(1, 10, 0);
        assert_eq!(item.stack_id, 1);
        assert_eq!(item.task_id, 10);
        assert_eq!(item.ordinal, 0);
    }
}
