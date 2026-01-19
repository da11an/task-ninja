# Plan 19: Add --enqueue Option to task add

## Goals
- Add `--enqueue` flag to `task add` command to automatically enqueue newly created tasks to the clock stack.
- Provide symmetry with existing `--clock-in` option.
- Enable single-command workflow for creating and queuing tasks.

## Non-Goals
- No changes to enqueue behavior itself.
- No changes to clock stack management.
- No changes to other task add functionality.

## Assumptions
- CLI uses `clap` with `--enqueue` as a long flag option.
- Existing `handle_task_enqueue` function can be reused.
- Clock stack operations are handled by `StackRepo::enqueue()`.
- `--enqueue` and `--clock-in` are mutually exclusive (or can be used together?).

## Open Questions
- Should `--enqueue` and `--clock-in` be mutually exclusive, or can both be used? (If both used, what's the order: enqueue then clock-in, or clock-in then enqueue?)
- **Decision**: They are mutually exclusive. `--clock-in` already pushes to stack[0] and starts timing, so `--enqueue` would be redundant. If both are specified, `--clock-in` takes precedence (or show error).

## Design Summary
Add `--enqueue` flag to `task add` command that:
1. Creates the task (normal behavior)
2. Adds the task to the end of the clock stack (via `StackRepo::enqueue()`)
3. Does NOT start timing (unlike `--clock-in`)

This mirrors the existing `--clock-in` behavior but for enqueueing instead of immediately starting work.

---

## Current Behavior

### `task add` (no flags)
- Creates task
- Task is not added to clock stack
- User must manually enqueue: `task enqueue <id>` or `task clock enqueue <id>`

### `task add --clock-in`
- Creates task
- Pushes task to clock[0] (top of stack)
- Starts timing session
- Equivalent to: `task add` + `task clock in --task <id>`

### Desired: `task add --enqueue`
- Creates task
- Adds task to end of clock stack
- Does NOT start timing
- Equivalent to: `task add` + `task enqueue <id>`

---

## Implementation

### 1. Add `--enqueue` Flag to CLI

**File**: `src/cli/commands.rs`

**Change**: Add `enqueue` field to `Add` command variant:

```rust
Add {
    /// Automatically clock in after creating task
    #[arg(long = "clock-in")]
    clock_in: bool,
    /// Automatically enqueue task to clock stack after creating
    #[arg(long = "enqueue")]
    enqueue: bool,
    /// Automatically create project if it doesn't exist (non-interactive)
    #[arg(long = "auto-create-project")]
    auto_create_project: bool,
    /// Task description and fields
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
},
```

### 2. Update `handle_task_add` Function

**File**: `src/cli/commands.rs`

**Change**: Add `enqueue` parameter and implement enqueue logic:

```rust
fn handle_task_add(
    mut args: Vec<String>, 
    mut clock_in: bool, 
    mut enqueue: bool,  // Add this parameter
    auto_create_project: bool
) -> Result<()> {
    // Extract --clock-in and --enqueue flags from args if they appear after description
    let mut filtered_args = Vec::new();
    for arg in args.iter() {
        if arg == "--clock-in" {
            clock_in = true;
        } else if arg == "--enqueue" {
            enqueue = true;
        } else {
            filtered_args.push(arg.clone());
        }
    }
    args = filtered_args;
    
    // ... existing task creation logic ...
    
    let task_id = task.id.unwrap();
    println!("Created task {}: {}", task_id, description);
    
    // Handle --clock-in (takes precedence over --enqueue)
    if clock_in {
        handle_task_clock_in(task_id.to_string(), Vec::new())
            .context("Failed to clock in task")?;
    } else if enqueue {
        // Enqueue to clock stack
        let stack = StackRepo::get_or_create_default(&conn)?;
        StackRepo::enqueue(&conn, stack.id.unwrap(), task_id)
            .context("Failed to enqueue task")?;
        println!("Enqueued task {}", task_id);
    }
    
    Ok(())
}
```

### 3. Update Command Handler

**File**: `src/cli/commands.rs`

**Change**: Pass `enqueue` parameter to `handle_task_add`:

```rust
Commands::Add { args, clock_in, enqueue, auto_create_project } => 
    handle_task_add(args, clock_in, enqueue, auto_create_project),
```

### 4. Update Command Reference Documentation

**File**: `docs/COMMAND_REFERENCE.md`

**Change**: Add `--enqueue` to options list:

```markdown
### `task add [--clock-in] [--enqueue] [--auto-create-project] <description> [attributes...]`

**Options:**
- `--clock-in` - Automatically clock in after creating task (pushes to clock[0] and starts timing)
- `--enqueue` - Automatically enqueue task to clock stack after creating (adds to end, does not start timing)
- `--auto-create-project` - Automatically create project if it doesn't exist (non-interactive mode)
```

Add example:
```markdown
# Task with --enqueue (adds to clock stack without starting timing)
task add --enqueue "Review documentation" project:docs
```

---

## Behavior Details

### Mutually Exclusive Flags
- If both `--clock-in` and `--enqueue` are specified, `--clock-in` takes precedence.
- Rationale: `--clock-in` already adds task to stack (at position 0) and starts timing, so `--enqueue` would be redundant.

### Enqueue Behavior
- Uses existing `StackRepo::enqueue()` function
- Adds task to end of clock stack (same as `task enqueue <id>` command)
- Does NOT start a timing session
- Does NOT push task to clock[0]

### Error Handling
- If task creation fails, enqueue is not attempted
- If enqueue fails, task is still created (enqueue is a post-creation operation)
- Error messages should be clear about what succeeded and what failed

---

## Tests

### Unit Tests
1. Test `task add --enqueue` creates task and enqueues it
2. Test `task add --enqueue` does NOT start timing
3. Test `task add --clock-in --enqueue` (clock-in takes precedence)
4. Test `task add --enqueue` with invalid task data (task creation fails, no enqueue attempted)

### Integration Tests
1. Create task with `--enqueue`, verify it appears at end of clock stack
2. Create task with `--enqueue`, verify no session is started
3. Create multiple tasks with `--enqueue`, verify order in stack
4. Create task with `--enqueue`, then verify `task clock list` shows it

### Acceptance Tests
1. `task add --enqueue "Test task"` → task created, enqueued, no session
2. `task add --enqueue "Test" project:work` → task created with project, enqueued
3. `task add --clock-in --enqueue "Test"` → task created, clocked in (enqueue ignored)

---

## Examples

```bash
# Create and enqueue a task (adds to end of clock stack)
task add --enqueue "Review PR" project:work +code-review

# Create and enqueue with multiple attributes
task add --enqueue "Write tests" project:work due:tomorrow allocation:2h

# Create task normally (not enqueued)
task add "Fix bug" project:work

# Create and immediately start working (pushes to clock[0] and starts timing)
task add --clock-in "Urgent fix" project:work +urgent

# Both flags specified: clock-in takes precedence
task add --clock-in --enqueue "Task"  # Same as just --clock-in
```

---

## Implementation Order

1. **Add flag to CLI definition** - Update `Commands::Add` enum variant
2. **Update function signature** - Add `enqueue` parameter to `handle_task_add`
3. **Implement enqueue logic** - Add enqueue call after task creation
4. **Handle flag extraction** - Extract `--enqueue` from args (like `--clock-in`)
5. **Update command handler** - Pass `enqueue` parameter
6. **Add tests** - Unit, integration, and acceptance tests
7. **Update documentation** - Command reference and examples

---

## Risks and Mitigations

### Risk 1: Confusion between --clock-in and --enqueue
- **Mitigation**: Clear documentation explaining the difference. `--clock-in` pushes to top and starts timing; `--enqueue` adds to end without timing.

### Risk 2: Both flags specified
- **Mitigation**: `--clock-in` takes precedence. Document this behavior clearly.

### Risk 3: Enqueue fails after task creation
- **Mitigation**: Task is still created. Error message indicates task was created but enqueue failed. User can manually enqueue.

---

## Success Criteria

1. ✅ `task add --enqueue` creates task and adds it to end of clock stack
2. ✅ `task add --enqueue` does NOT start a timing session
3. ✅ `task add --clock-in --enqueue` respects `--clock-in` (enqueue ignored)
4. ✅ Documentation updated with new option
5. ✅ Tests pass (unit, integration, acceptance)
6. ✅ Behavior matches `task add` + `task enqueue <id>` workflow

---

## Related Commands

- `task add --clock-in` - Creates task and immediately starts working
- `task enqueue <id>` - Enqueues existing task to clock stack
- `task clock enqueue <id>` - Same as `task enqueue <id>`
- `task clock in --task <id>` - Pushes task to clock[0] and starts timing

---

## Future Enhancements (Out of Scope)

- `--enqueue-at <position>` - Enqueue at specific position in stack
- `--enqueue-top` - Alias for `--clock-in` (for symmetry)
- Config option to always enqueue new tasks by default
