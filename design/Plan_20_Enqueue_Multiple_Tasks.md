# Plan 20: Make task enqueue Accept Comma-Separated List of IDs

## Goals
- Update `task enqueue` command to accept comma-separated list of task IDs and ranges
- Enqueue tasks in the order specified (preserve order, don't sort)
- Support both single ID (backward compatible), multiple IDs, and ranges

## Non-Goals
- No changes to other enqueue behavior
- No changes to clock stack management

## Assumptions
- CLI uses `clap` with `task_id` as a string parameter (not `i64`)
- Existing `parse_task_id_spec()` function exists but sorts/deduplicates
- Need to preserve order for enqueue operations
- Backward compatibility: single ID should still work

## Open Questions
- Should we support ranges (e.g., `2-4`) in addition to comma-separated lists?
  - **Decision**: Yes, ranges are supported. Ranges preserve direction (forward or reverse).
- Should duplicate IDs be allowed or filtered?
  - **Decision**: Filter duplicates but preserve order of first occurrence.

## Design Summary
Update `task enqueue` command to:
1. Accept comma-separated list of task IDs and ranges (e.g., `1,3,5` or `1,3-5,10`)
2. Parse the list preserving order (ranges preserve direction)
3. Validate all task IDs exist
4. Enqueue tasks in the specified order (first ID enqueued first, appears at end of stack first)
5. Maintain backward compatibility with single ID

---

## Current Behavior

### `task enqueue <id>`
- Accepts single task ID as `i64`
- Validates task exists
- Enqueues task to end of clock stack
- Prints: "Enqueued task {id}"

### Current Implementation
```rust
Enqueue {
    /// Task ID to enqueue
    task_id: i64,  // Single ID only
},
```

```rust
fn handle_task_enqueue(task_id_str: String) -> Result<()> {
    let task_id = match validate_task_id(&task_id_str) {
        Ok(id) => id,
        Err(e) => user_error(&e),
    };
    // ... enqueue single task
}
```

---

## Proposed Behavior

### `task enqueue <id|id,id,...|range|mixed>`
- Accepts single task ID: `task enqueue 5`
- Accepts comma-separated list: `task enqueue 1,3,5`
- Accepts ranges: `task enqueue 30-31` (expands to 30, 31)
- Accepts mixed: `task enqueue 1,3-5,10` (expands to 1, 3, 4, 5, 10)
- Enqueues tasks in listed order (ranges preserve direction)
- Validates all tasks exist before enqueueing any
- Prints: "Enqueued task 1", "Enqueued task 3", etc. (one per task)

### Examples
```bash
# Single task (backward compatible)
task enqueue 5

# Multiple tasks
task enqueue 1,3,5

# Range
task enqueue 30-31

# Mixed (ranges and individual IDs)
task enqueue 1,3-5,10

# With spaces (should work)
task enqueue 1, 3, 5
task enqueue 30 - 31
```

---

## Implementation

### 1. Update Command Definition

**File**: `src/cli/commands.rs`

**Change**: Change `task_id` from `i64` to `String`:

```rust
Enqueue {
    /// Task ID(s) to enqueue (comma-separated list)
    task_id: String,  // Changed from i64 to String
},
```

### 2. Create Order-Preserving Parser

**File**: `src/cli/error.rs` (or `src/cli/commands.rs`)

**Change**: Create a new function that parses comma-separated IDs and ranges preserving order:

```rust
/// Parse comma-separated task IDs and ranges preserving order
/// Unlike parse_task_id_spec, this does NOT sort or deduplicate
/// Returns IDs in the order specified (ranges preserve direction)
pub fn parse_task_id_list(spec: &str) -> Result<Vec<i64>, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err("Empty task ID list".to_string());
    }
    
    let mut ids = Vec::new();
    let mut seen = std::collections::HashSet::new();
    
    // Split by comma
    let parts: Vec<&str> = spec.split(',').map(|s| s.trim()).collect();
    
    for part in parts {
        if part.is_empty() {
            continue;
        }
        
        // Check if this is a range (contains '-')
        if part.contains('-') {
            // Parse range and add IDs in range order
            // ... (range parsing logic)
        } else {
            // Single ID
            let id = part.parse::<i64>()?;
            if !seen.contains(&id) {
                ids.push(id);
                seen.insert(id);
            }
        }
    }
    
    Ok(ids)
}
```

### 3. Update `handle_task_enqueue` Function

**File**: `src/cli/commands.rs`

**Change**: Update to handle multiple IDs:

```rust
fn handle_task_enqueue(task_id_str: String) -> Result<()> {
    let conn = DbConnection::connect()
        .context("Failed to connect to database")?;
    
    // Parse comma-separated list of IDs (preserves order)
    let task_ids = match parse_task_id_list(&task_id_str) {
        Ok(ids) => ids,
        Err(e) => user_error(&e),
    };
    
    // Validate all tasks exist before enqueueing any
    let mut valid_ids = Vec::new();
    let mut missing_ids = Vec::new();
    
    for task_id in &task_ids {
        if TaskRepo::get_by_id(&conn, *task_id)?.is_some() {
            valid_ids.push(*task_id);
        } else {
            missing_ids.push(*task_id);
        }
    }
    
    if !missing_ids.is_empty() {
        user_error(&format!("Task(s) not found: {}", 
            missing_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ")));
    }
    
    if valid_ids.is_empty() {
        user_error("No valid tasks to enqueue");
    }
    
    // Enqueue all tasks in order
    let stack = StackRepo::get_or_create_default(&conn)?;
    let stack_id = stack.id.unwrap();
    
    for task_id in valid_ids {
        StackRepo::enqueue(&conn, stack_id, task_id)
            .context(format!("Failed to enqueue task {}", task_id))?;
        println!("Enqueued task {}", task_id);
    }
    
    Ok(())
}
```

### 4. Update Command Handler

**File**: `src/cli/commands.rs`

**Change**: Update to pass String instead of converting to_string():

```rust
Commands::Enqueue { task_id } => {
    handle_task_enqueue(task_id)  // task_id is now String, not i64
}
```

### 5. Update Documentation

**File**: `docs/COMMAND_REFERENCE.md`

**Change**: Update enqueue command documentation:

```markdown
### `task enqueue <id|id,id,...>`

Add task(s) to end of clock stack.

**Arguments:**
- `<id>` - Single task ID
- `<id,id,...>` - Comma-separated list of task IDs (enqueued in listed order)

**Examples:**
```bash
# Enqueue single task
task enqueue 5

# Enqueue multiple tasks
task enqueue 1,3,5

# Enqueue with spaces
task enqueue 1, 3, 5
```
```

---

## Behavior Details

### Order Preservation
- Tasks are enqueued in the exact order specified
- First ID in list is enqueued first (appears at end of stack first)
- Last ID in list is enqueued last (appears at end of stack last)

### Duplicate Handling
- Duplicate IDs are filtered (only first occurrence is enqueued)
- Order of first occurrence is preserved
- Example: `1,3,1,5` → enqueues `1,3,5` (in that order)

### Validation
- All task IDs must exist before any are enqueued
- If any task is missing, error is shown with list of missing IDs
- No partial enqueueing (all-or-nothing validation)

### Error Messages
- Clear error for invalid ID format: "Invalid task ID 'abc': must be a number"
- Clear error for missing tasks: "Task(s) not found: 5, 7"
- Clear error for empty list: "No valid task IDs found"

---

## Tests

### Unit Tests
1. Test `parse_task_id_list` with single ID
2. Test `parse_task_id_list` with comma-separated list
3. Test `parse_task_id_list` with duplicates (should filter)
4. Test `parse_task_id_list` with invalid IDs
5. Test `parse_task_id_list` with empty string

### Integration Tests
1. Enqueue single task (backward compatibility)
2. Enqueue multiple tasks, verify order in stack
3. Enqueue with duplicate IDs, verify only one instance
4. Enqueue with non-existent task, verify error
5. Enqueue with mix of existing and non-existent, verify error

### Acceptance Tests
1. `task enqueue 5` → single task enqueued
2. `task enqueue 1,3,5` → three tasks enqueued in order
3. `task enqueue 1,3,1,5` → three tasks enqueued (duplicate filtered)
4. `task enqueue 99` (non-existent) → error message
5. `task enqueue 1,99,3` (one non-existent) → error message with missing ID

---

## Examples

```bash
# Single task (backward compatible)
task enqueue 5

# Multiple tasks
task enqueue 1,3,5

# With spaces
task enqueue 1, 3, 5

# Duplicates (filtered, order preserved)
task enqueue 1,3,1,5  # Enqueues 1,3,5 in that order

# Error cases
task enqueue 99  # Error: Task 99 not found
task enqueue 1,99,3  # Error: Task(s) not found: 99
```

---

## Implementation Order

1. **Create order-preserving parser** - Add `parse_task_id_list()` function
2. **Update command definition** - Change `task_id: i64` to `task_id: String`
3. **Update handler function** - Modify `handle_task_enqueue()` to handle multiple IDs
4. **Update command handler** - Remove `.to_string()` conversion
5. **Add tests** - Unit, integration, and acceptance tests
6. **Update documentation** - Command reference with examples

---

## Risks and Mitigations

### Risk 1: Breaking backward compatibility
- **Mitigation**: Single ID still works (just parsed as a list of one). Test thoroughly.

### Risk 2: Performance with large lists
- **Mitigation**: Validate all IDs exist before enqueueing. If performance becomes an issue, can optimize later.

### Risk 3: Confusion about order
- **Mitigation**: Clear documentation that tasks are enqueued in listed order. Examples show this behavior.

### Risk 4: Duplicate handling unclear
- **Mitigation**: Document that duplicates are filtered but order is preserved. Show example.

---

## Success Criteria

1. ✅ `task enqueue 5` still works (backward compatible)
2. ✅ `task enqueue 1,3,5` enqueues three tasks
3. ✅ Tasks are enqueued in listed order
4. ✅ Duplicate IDs are filtered (first occurrence preserved)
5. ✅ Clear error messages for invalid/missing tasks
6. ✅ Documentation updated with examples
7. ✅ Tests pass (unit, integration, acceptance)

---

## Related Commands

- `task add --enqueue` - Creates and enqueues a single task
- `task clock enqueue <id>` - Same as `task enqueue <id>` (may need similar update)
- `task <id> enqueue` - Alternative syntax (may need similar update)

---

## Future Enhancements (Out of Scope)

- Support ranges: `task enqueue 2-4` (expands to 2,3,4)
- Support mixed syntax: `task enqueue 1,3-5,7` (expands to 1,3,4,5,7)
- Batch operations for other commands (delete, modify, etc.)
