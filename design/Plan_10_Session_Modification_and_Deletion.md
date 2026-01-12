## Session Modification and Deletion - Planning Document

This document specifies the addition of session modification and deletion functionality to Task Ninja.

---

### Current Behavior

- Sessions can be created (`task clock in`, `task <id> clock in`)
- Sessions can be closed (`task clock out`, `task done`)
- Sessions can be listed (`task sessions list`, `task <id> sessions list`)
- Sessions can be viewed (`task sessions show`, `task <id> sessions show`)
- **Sessions CANNOT be modified** (except automatic overlap prevention)
- **Sessions CANNOT be deleted** (except automatic micro-session purging)
- **Session IDs are NOT visible in `sessions list` table** (only in JSON output and `sessions show`)

**Overlap Prevention (Automatic):**
- When creating a new session, closed sessions that would overlap are automatically amended
- The `amend_end_time()` method exists but is only used internally
- No user control over overlap resolution

---

### Proposed Behavior

Add session modification and deletion commands with syntax consistent with the filter-before-verb pattern used throughout Task Ninja.

#### 1. Session Modification

**Command Syntax:**
```
task sessions <session_id> modify [start:<expr>] [end:<expr>] [--yes] [--force]
```

**Consistent with Filter-Before-Verb Pattern:**
- `task <id|filter> modify [attributes...]` → `task sessions <session_id> modify [start:<expr>] [end:<expr>]`
- Filter comes before verb: `task <filter> <verb> <details>`
- Both use field:value syntax
- Both support `--yes` flag for confirmation bypass

**Behavior:**
1. Modify start time: `task sessions 5 modify start:09:00`
2. Modify end time: `task sessions 5 modify end:17:00`
3. Modify both: `task sessions 5 modify start:09:00 end:17:00`
4. Clear end time (make session open): `task sessions 5 modify end:none` (only if currently closed)
5. Set end time (close session): `task sessions 5 modify end:now` (only if currently open)

**Overlap Detection:**
- Before applying modifications, check for conflicts with other sessions
- Report all conflicting sessions with details
- Prevent modification if conflicts exist (unless `--force` flag)
- Conflicts occur when:
  - Two sessions have overlapping time ranges
  - An open session exists when trying to create another open session (only one open session allowed)

**Conflict Reporting:**
```
Error: Session modification would create conflicts:

  Session 5 (Task 10): 2024-01-15 09:00 - 2024-01-15 11:00
  Conflicts with:
    - Session 3 (Task 8): 2024-01-15 10:00 - 2024-01-15 12:00
    - Session 7 (Task 12): 2024-01-15 08:30 - 2024-01-15 09:30

Use --force to override (may require resolving conflicts manually).
```

**Options:**
- `--yes` - Apply modification without confirmation
- `--force` - Allow modification even with conflicts (user must resolve manually)

#### 2. Session Deletion

**Command Syntax:**
```
task sessions <session_id> delete [--yes]
```

**Consistent with Filter-Before-Verb Pattern:**
- `task <id|filter> delete [--yes]` → `task sessions <session_id> delete [--yes]`
- Filter comes before verb: `task <filter> <verb> <details>`
- Both require explicit ID specification
- Both support `--yes` flag for confirmation bypass

**Behavior:**
1. Delete a specific session by ID
2. Show session details in confirmation prompt
3. Handle related data:
   - Annotations linked to session: Set `session_id` to NULL (via `ON DELETE SET NULL`)
   - Events: Keep events (they reference session_id, but deletion is allowed)
4. Cannot delete the currently running session (must clock out first)

**Confirmation Prompt:**
```
Delete session 5?
  Task: 10 (Fix bug in authentication)
  Start: 2024-01-15 09:00:00
  End: 2024-01-15 11:00:00
  Duration: 2h0m0s
  Linked annotations: 2

Are you sure? (y/n):
```

**Error Cases:**
- Session not found: `Error: Session <id> not found`
- Attempting to delete running session: `Error: Cannot delete running session. Please clock out first.`

#### 3. Expose Session IDs in List Output

**Current State:**
- Session IDs are shown in JSON output (`task sessions list --json`)
- Session IDs are shown in `sessions show` output
- Session IDs are **NOT** shown in `sessions list` table output

**Proposed Change:**
Add a Session ID column to the `sessions list` table output:

```
Session  Task  Description                              Start                End                  Duration
5        10    Fix bug in authentication                2024-01-15 09:00:00  2024-01-15 11:00:00  2h0m0s
3        8     Review PR                                2024-01-15 10:00:00  2024-01-15 12:00:00  2h0m0s
7        12    Write documentation                      2024-01-15 08:30:00  2024-01-15 09:30:00  1h0m0s
```

**Alternative (if space is tight):**
Show session ID in parentheses after task ID:
```
Task    Description                              Start                End                  Duration
10 (5)  Fix bug in authentication                2024-01-15 09:00:00  2024-01-15 11:00:00  2h0m0s
8 (3)   Review PR                                2024-01-15 10:00:00  2024-01-15 12:00:00  2h0m0s
12 (7)  Write documentation                      2024-01-15 08:30:00  2024-01-15 09:30:00  1h0m0s
```

---

### Decisions Made

1. **Command Syntax:**
   - Use `task sessions <session_id> modify` (filter before verb pattern)
   - Use `task sessions <session_id> delete` (filter before verb pattern)
   - Rationale: Consistent with `task <filter> <verb>` pattern used throughout Task Ninja
     - `task <id|filter> modify` → `task sessions <session_id> modify`
     - `task <id|filter> delete` → `task sessions <session_id> delete`
     - `task <filter> list` → `task sessions list` (or `task <task_id> sessions list`)
   - This maintains the natural language flow: "task sessions 5 modify" = "modify session 5"

2. **Field Syntax:**
   - Use `start:<expr>` and `end:<expr>` (parallel to task modification's `project:name`, `due:expr`, etc.)
   - Use `end:none` to clear end time (make open)
   - Use `end:now` to set end time to current time (close session)

3. **Overlap Detection:**
   - Check for conflicts before applying modifications
   - Report all conflicting sessions with full details
   - Prevent modification by default (require `--force` to override)
   - Rationale: Overlapping sessions violate the "one active session" constraint and create data integrity issues

4. **Session ID Display:**
   - Add Session ID column to `sessions list` table output
   - Keep JSON output unchanged (already includes session IDs)
   - Keep `sessions show` output unchanged (already shows session ID)

5. **Running Session Protection:**
   - Cannot delete running session (must clock out first)
   - Cannot modify running session's end time to `none` (it's already open)
   - Can modify running session's start time (but check for conflicts)

6. **Related Data Handling:**
   - Annotations: `ON DELETE SET NULL` (already in schema)
   - Events: Keep events (they reference session_id, but deletion is allowed)
   - No cascade deletion needed

---

### Implementation Considerations

#### 1. **Repository Layer (`src/repo/session.rs`)**

**New Methods:**
```rust
/// Modify session start time
pub fn modify_start_time(conn: &Connection, session_id: i64, new_start_ts: i64) -> Result<()>

/// Modify session end time
pub fn modify_end_time(conn: &Connection, session_id: i64, new_end_ts: Option<i64>) -> Result<()>
// Note: Existing amend_end_time only works for closed sessions, need new method

/// Delete a session
pub fn delete(conn: &Connection, session_id: i64) -> Result<()>

/// Check for overlapping sessions
pub fn find_overlapping_sessions(
    conn: &Connection, 
    task_id: i64, 
    start_ts: i64, 
    end_ts: Option<i64>,
    exclude_session_id: Option<i64>  // Exclude current session when modifying
) -> Result<Vec<Session>>

/// Get session by ID
pub fn get_by_id(conn: &Connection, session_id: i64) -> Result<Option<Session>>
```

**Overlap Detection Logic:**
- Two sessions overlap if:
  - Both are closed: `(start1 < end2) && (end1 > start2)`
  - One is open: `(start1 < start2) && (end1 is None)` (open session conflicts with any session that starts before it ends)
  - Both are open: Only one open session allowed at a time

#### 2. **CLI Layer (`src/cli/commands.rs`)**

**Update `SessionsCommands` enum:**
```rust
#[derive(Subcommand)]
pub enum SessionsCommands {
    /// List session history
    List { ... },
    /// Show detailed session information
    Show,
    /// Modify session start/end times
    /// Syntax: task sessions <session_id> modify [start:<expr>] [end:<expr>]
    Modify {
        /// Modification arguments (start:<expr>, end:<expr>)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
        /// Apply modification without confirmation
        #[arg(long)]
        yes: bool,
        /// Allow modification even with conflicts
        #[arg(long)]
        force: bool,
    },
    /// Delete a session
    /// Syntax: task sessions <session_id> delete
    Delete {
        /// Delete without confirmation
        #[arg(long)]
        yes: bool,
    },
}
```

**Note:** The session ID will be parsed from the pre-clap parsing layer, similar to how `task <id> modify` works. The session ID comes before the verb in the command line.

**Handler Functions:**
- `handle_sessions_modify(session_id, args, yes, force)` in `src/cli/commands_sessions.rs`
- `handle_sessions_delete(session_id, yes)` in `src/cli/commands_sessions.rs`

**Pre-Clap Parsing:**
- Add pattern matching for `task sessions <session_id> modify` in `src/cli/commands.rs`
- Add pattern matching for `task sessions <session_id> delete` in `src/cli/commands.rs`
- Extract session_id from args before passing to Clap, similar to `task <id> modify` pattern

#### 3. **Parser for Modification Arguments**

**Parse `start:<expr>` and `end:<expr>`:**
- Similar to task modification parser
- Support date expressions (same as `parse_date_expr`)
- Support `end:none` to clear end time
- Support `end:now` to set end time to current time

**Example:**
```rust
fn parse_session_modify_args(args: Vec<String>) -> Result<SessionModifyArgs> {
    struct SessionModifyArgs {
        start: Option<Option<i64>>,  // Some(None) = clear, Some(Some(ts)) = set, None = no change
        end: Option<Option<i64>>,
    }
    // Parse start:<expr> and end:<expr> tokens
}
```

#### 4. **Overlap Detection and Reporting**

**Function:**
```rust
fn check_session_overlaps(
    conn: &Connection,
    task_id: i64,
    start_ts: i64,
    end_ts: Option<i64>,
    exclude_session_id: Option<i64>,
) -> Result<Vec<Session>> {
    // Query for overlapping sessions
    // Return list of conflicting sessions
}
```

**Reporting Format:**
```
Error: Session modification would create conflicts:

  Session 5 (Task 10): 2024-01-15 09:00:00 - 2024-01-15 11:00:00
  Conflicts with:
    - Session 3 (Task 8): 2024-01-15 10:00:00 - 2024-01-15 12:00:00
    - Session 7 (Task 12): 2024-01-15 08:30:00 - 2024-01-15 09:30:00

Use --force to override (may require resolving conflicts manually).
```

#### 5. **Update Session List Display**

**File:** `src/cli/commands_sessions.rs`

**Change:** Add Session ID column to table output:
```rust
println!("{:<8} {:<6} {:<40} {:<20} {:<20} {:<12}", 
    "Session", "Task", "Description", "Start", "End", "Duration");
println!("{}", "-".repeat(106));

for session in &sessions {
    let session_id_str = session.id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "?".to_string());
    
    println!("{:<8} {:<6} {:<40} {:<20} {:<20} {:<12}", 
        session_id_str, session.task_id, description, start_str, end_str, duration_str);
}
```

#### 6. **Error Handling**

**Session Not Found:**
```rust
if session.is_none() {
    user_error(&format!("Session {} not found", session_id));
}
```

**Running Session Deletion:**
```rust
if session.is_open() {
    user_error("Cannot delete running session. Please clock out first.");
}
```

**Overlap Conflicts:**
```rust
let conflicts = check_session_overlaps(...)?;
if !conflicts.is_empty() && !force {
    // Format and display conflict error
    user_error(&format!("Session modification would create conflicts:\n\n{}", conflict_msg));
}
```

---

### Implementation Checklist

- [ ] **Repository Layer:**
  - [ ] Add `modify_start_time()` method to `SessionRepo`
  - [ ] Add `modify_end_time()` method to `SessionRepo` (enhance existing `amend_end_time`)
  - [ ] Add `delete()` method to `SessionRepo`
  - [ ] Add `find_overlapping_sessions()` method to `SessionRepo`
  - [ ] Add `get_by_id()` method to `SessionRepo`
  - [ ] Add unit tests for overlap detection logic
  - [ ] Add unit tests for modification methods
  - [ ] Add unit tests for deletion method

- [ ] **CLI Layer:**
  - [ ] Add `Modify` variant to `SessionsCommands` enum
  - [ ] Add `Delete` variant to `SessionsCommands` enum
  - [ ] Add handler for `SessionsCommands::Modify` in `handle_sessions()`
  - [ ] Add handler for `SessionsCommands::Delete` in `handle_sessions()`
  - [ ] Create `handle_sessions_modify()` function in `src/cli/commands_sessions.rs`
  - [ ] Create `handle_sessions_delete()` function in `src/cli/commands_sessions.rs`
  - [ ] Create `parse_session_modify_args()` function
  - [ ] Create `check_session_overlaps()` function
  - [ ] Create `format_conflict_error()` function

- [ ] **Display Updates:**
  - [ ] Add Session ID column to `sessions list` table output
  - [ ] Update table header and formatting
  - [ ] Ensure session IDs are visible and easy to reference

- [ ] **Error Handling:**
  - [ ] Handle session not found errors
  - [ ] Handle running session deletion attempts
  - [ ] Handle overlap conflicts with detailed reporting
  - [ ] Handle invalid date expressions
  - [ ] Handle invalid session IDs

- [ ] **Documentation:**
  - [ ] Update `docs/COMMAND_REFERENCE.md` with session modification command
  - [ ] Update `docs/COMMAND_REFERENCE.md` with session deletion command
  - [ ] Add examples for both commands
  - [ ] Document overlap detection behavior
  - [ ] Update `README.md` if needed

- [ ] **Testing:**
  - [ ] Integration tests for session modification
  - [ ] Integration tests for session deletion
  - [ ] Integration tests for overlap detection
  - [ ] Integration tests for conflict reporting
  - [ ] Edge case tests (running session, invalid IDs, etc.)

---

### Examples

#### Session Modification

```bash
# Modify start time
task sessions 5 modify start:09:00

# Modify end time
task sessions 5 modify end:17:00

# Modify both
task sessions 5 modify start:09:00 end:17:00

# Close an open session
task sessions 5 modify end:now

# Make a closed session open (clear end time)
task sessions 5 modify end:none

# Modify with confirmation bypass
task sessions 5 modify start:09:00 --yes

# Force modification despite conflicts
task sessions 5 modify start:09:00 --force
```

#### Session Deletion

```bash
# Delete session (with confirmation)
task sessions 5 delete

# Delete session (without confirmation)
task sessions 5 delete --yes
```

#### Overlap Conflict Example

```bash
$ task sessions 5 modify start:10:00
Error: Session modification would create conflicts:

  Session 5 (Task 10): 2024-01-15 10:00:00 - 2024-01-15 11:00:00
  Conflicts with:
    - Session 3 (Task 8): 2024-01-15 10:00:00 - 2024-01-15 12:00:00

Use --force to override (may require resolving conflicts manually).
```

#### Updated Session List Output

```bash
$ task sessions list
Session  Task  Description                              Start                End                  Duration
5        10    Fix bug in authentication                2024-01-15 09:00:00  2024-01-15 11:00:00  2h0m0s
3        8     Review PR                                2024-01-15 10:00:00  2024-01-15 12:00:00  2h0m0s
7        12    Write documentation                      2024-01-15 08:30:00  2024-01-15 09:30:00  1h0m0s
```

---

### Design Decisions Summary

1. **Syntax Consistency:** Session commands follow the filter-before-verb pattern (`task <filter> <verb> <details>`)
   - `task <id|filter> modify` → `task sessions <session_id> modify`
   - `task <id|filter> delete` → `task sessions <session_id> delete`
   - Maintains natural language flow and consistency with existing commands
2. **Session ID Exposure:** Session IDs are shown in list output for easy reference
3. **Overlap Prevention:** Strict overlap detection with detailed conflict reporting
4. **Safety:** Confirmation prompts and protection for running sessions
5. **Data Integrity:** Proper handling of related data (annotations, events)

---

### Future Considerations

1. **Bulk Operations:**
   - Could add `task sessions <id1> <id2> <id3> delete` for multiple deletions (following filter-before-verb pattern)
   - Could add filter support: `task sessions project:work delete` (delete all sessions for tasks in project)

2. **Advanced Overlap Resolution:**
   - Could add `--auto-resolve` flag to automatically adjust conflicting sessions
   - Could add interactive conflict resolution

3. **Session Splitting/Merging:**
   - Could add ability to split a session into two
   - Could add ability to merge two sessions

4. **Session Templates:**
   - Could add ability to duplicate a session with modifications

---

## Summary

This plan adds session modification and deletion capabilities with:
- **Consistent syntax pattern:** `task <filter> <verb> <details>` - filter before verb
  - `task sessions <session_id> modify` (consistent with `task <id> modify`)
  - `task sessions <session_id> delete` (consistent with `task <id> delete`)
- Overlap detection with detailed conflict reporting
- Session ID exposure in list output
- Safety features (confirmations, running session protection)
- Proper data integrity handling

The implementation follows the established filter-before-verb pattern used throughout Task Ninja, ensuring a consistent and learnable command syntax.
