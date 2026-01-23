# User Feedback 2026-01-22

## Issues
### 1. No `tatl project report` exists
### 2. Add trouble

- Forgot to use add command. Should that be implicit? Probably not, but worth discussion.
- No add: `tatl project:bdsf.meeting "Board meeting" --on start:16:00
- Need: single line syntax (flags??) to add task, start recording, and set the session start time to earlier
    - `tatl add project:bdsf.meeting "Board meeting" --on start:16:00
        - Error: Unrecognized field name 'start' Did you mean 'wait'?
    - `tatl add project:bdsf.meeting "Board meeting" --on 16:00
        - Error: Unrecognized field name '16' Did you mean 'due'?

### 3. Report syntax spotty, redundent with status
- We don't probably need status and report commands. Drop status command.
- `tatl sessions report <start>..<stop>` syntax doesn't work, but would make sense (it's expecting only a start date)
- `tatl project report <start>..<stop>` syntax doesn't work, but would make sense

### 4. `tatl onoff <task_id> <start>..<stop>` has unexpected behavior when splitting a live session
- The first half makes sense, but the live session get's a start of era value
- Session 71 (task 45): Run validation data
    2026-01-22 14:29 - running (1h 8m)
    -> SPLIT into 14:29..15:10 and 15:33..9223372036854775807

### 5. `tatl sessions modify 69 end:14:30`
This works, but can we lean into interval syntax and accept for the time argument only:
..<end>, <start>.., and <start>..<end> which correspond to modifying only the end, only the start, and both

### 6. `tatl 15 modify respawn:every:fridayy` is accepted. We need to reject unparsable respawn arguments and also give a message about what the respawn will do. E.g. A new task will be created when task 15 is completed for the following Friday.

### 7. `tatl add Plan pr:miproject` -- this got confirmation about creating a new project, but didn't suggest the near match of `myproject`. When creating a new project during add, we should check for near matches (capitalization, 1-2 letters different)

### 8. It would be nice to be able to sort the queue (not things into or out of the queue) based on a column, like Priority, due, alloc. Syntax unclear, maybe `tatl queue:sort:priority`

### 9. `tatl list pr:admin.email` works to filter list by project, but `tatl list Words` doesn't search descriptions for the words. This seems like and oversight

---

## Analysis and Proposed Solutions

### Overview

The feedback reveals several themes:
1. **Inconsistent interval syntax** across commands (issues #3, #4, #5)
2. **Missing validation/feedback** at input time (issues #6, #7)
3. **Feature gaps** that could be addressed by extending existing patterns (issues #1, #8, #9)
4. **Redundant commands** (issue #3)
5. **Edge case bugs** (issue #4)

### Guiding Principles

- **Consistency over features**: A smaller, consistent command surface is better than many inconsistent conveniences
- **Validate early, fail loud**: Bad input should be rejected with helpful messages, never silently accepted
- **Extend existing patterns**: Reuse existing syntax rather than inventing new paradigms

---

### Issue #1: No `tatl project report`

**Current State:**
- `tatl sessions report [start] [end]` - shows hours by project for a time range
- No way to see project-specific time summaries

**Analysis:**
The existing `sessions report` already groups by project. A "project report" would essentially be a filtered version of this.

**Proposed Solution:**
Rather than adding a new command, extend `sessions report` to accept filters:

```bash
tatl sessions report -7d                    # All projects, last 7 days
tatl sessions report -7d project:work       # Filter by project
tatl sessions report -7d +urgent            # Filter by tag
```

This is consistent with how `sessions list` now accepts filters.

**Decision Point:** 
1. Add filtering to tatl sessions report. 
2. The sessions report is about times, organized by project. This is good. The projects report should be about tasks status (Kanban and/or Status).
Projects Report:
================
Project       Proposed Queued ... Completed Closed Total
-------       -------- ------     --------- ------ -----
admin                5     4              3      1    13
  email             ...
  meeting
research
  project_a
  project_b
TOTAL              ...
---

### Issue #2: Add with backdated session start

**Current State:**
- `--on` starts timing at "now"
- `--onoff <start>..<end>` creates a closed historical session
- No way to create a task, start timing, and set the start time to earlier in a single command

**Analysis:**
The desired workflow is: "I started working on something 30 minutes ago, want to add a task and continue timing from when I actually started."

**Proposed Solution A (Minimal - extend `--on`):**
Allow `--on` to accept an optional time argument:

```bash
tatl add "Board meeting" project:meetings --on=16:00
# Creates task, starts timing, session start_ts is 16:00, session is open (still running)
```

This is consistent with `tatl on 09:00` which starts timing at a specific time.

**Proposed Solution B (Alternative - new flag):**
Add `--on-at <time>` flag:

```bash
tatl add "Board meeting" project:meetings --on-at 16:00
```

**Recommendation:** Solution A. The `--on <time>` pattern mirrors existing `on` command behavior. The current error ("Unrecognized field name 'start'") happens because `start:16:00` looks like a task field, not a flag argument.

**Decision Point:** Should `--on` optionally accept a time? -- Yes Solution A.

---

### Issue #3: Report syntax and status redundancy

**Current State:**
- `tatl status` - dashboard showing current state
- `tatl sessions report <start> [end]` - time summary by project
- Report takes positional args, not interval syntax

**Analysis:**
1. **status vs report**: These serve different purposes:
   - `status` = "What am I doing right now?" (live session, queue state)
   - `report` = "How did I spend my time in this period?"
   
   The question is whether `status` provides enough value to justify a separate command.

2. **Interval syntax consistency**: The `<start>..<end>` pattern is used in:
   - `onoff 09:00..12:00`
   - `offon 14:00..15:00`
   - `sessions modify <id> start:09:00 end:12:00` (different pattern)

**Proposed Solutions:**

**3a. Keep `status`, unify interval syntax:**
Allow `sessions report` to accept either format:

```bash
tatl sessions report -7d                  # Current: works
tatl sessions report -7d..now             # New: interval syntax
tatl sessions report 2026-01-01..2026-01-15  # New: explicit range
```

**3b. Drop `status`, expand `sessions report`:**
If `status` is removed, its functionality could be absorbed:
- `tatl` (no args) could show current status
- `tatl report` could be an alias for `tatl sessions report`

**Recommendation:** Keep `status` for now (it's quick to see "am I timing?"). Add interval syntax to `sessions report` for consistency.

**Decision Point:** Drop `status`. I don't use status because task list gives is a better status. If we find a good reason to add it back we will.

---

### Issue #4: onoff splitting live session incorrectly

**Current State:**
When `onoff` splits a running session, the second part gets `end_ts = i64::MAX` (sentinel for "running").

**Analysis:**
This is a bug. The code in `modify_session_for_removal` correctly handles splits, but the display is showing the raw sentinel value:

```rust
// In modify_session_for_removal:
SessionRepo::create_closed(conn, session.task_id, remove_end, s_end)?;
// s_end is i64::MAX when session is open
```

The split creates a "closed" session from `remove_end` to `i64::MAX`, but it should remain open.

**Proposed Fix:**
When splitting a live session, the second half should be created as an open session:

```rust
if s_end == i64::MAX || session.end_ts.is_none() {
    // Original session was open - second part should be open too
    SessionRepo::create_open(conn, session.task_id, remove_end)?;
} else {
    SessionRepo::create_closed(conn, session.task_id, remove_end, s_end)?;
}
```

**Decision Point:** None - this is a bug fix.

---

### Issue #5: Interval syntax for sessions modify

**Current State:**
```bash
tatl sessions modify 69 start:09:00 end:17:00
```

**Proposal:**
Also accept interval syntax:
```bash
tatl sessions modify 69 09:00..17:00    # Both
tatl sessions modify 69 09:00..         # Start only (end unchanged)
tatl sessions modify 69 ..17:00         # End only (start unchanged)
```

**Analysis:**
This is a nice ergonomic improvement and consistent with the interval pattern used elsewhere. However, it adds parsing complexity.

**Consideration:** The current `start:` and `end:` syntax is explicit and unambiguous. The interval syntax is terser but could be confusing:
- What does `..17:00` mean without context? 
- Is this worth the implementation and documentation burden?

**Recommendation:** Implement as a convenience, but keep `start:`/`end:` as the canonical documented form.

**Decision Point:** Is the terse syntax worth the added complexity? Could skip this for now. Yes, the terse syntax is worth it. Because then you use the same interval syntax. No more start and end. Can we entirely drop the start: end: syntax if we go this direction?

---

### Issue #6: Respawn validation

**Current State:**
- `respawn:every:fridayy` is accepted silently
- The invalid rule is stored, then fails at completion time

**Analysis:**
This violates "validate early, fail loud." The respawn parser exists (`RespawnRule::parse`), but it's not called during `modify`.

**Proposed Fix:**

1. **Validate on modify:** In `handle_task_modify`, if a `respawn` field is provided, parse it immediately:

```rust
if let Some(respawn_str) = &parsed.respawn {
    if let Some(r) = respawn_str {
        RespawnRule::parse(r).map_err(|e| 
            user_error(&format!("Invalid respawn rule '{}': {}", r, e))
        )?;
    }
}
```

2. **Preview message:** After successful validation, show what will happen:

```
Modified task 15: respawn rule set to 'weekly'
↻ When completed, a new task will be created for the next Friday.
```

**Decision Point:** Should the preview show the actual calculated next date (requires knowing completion date) or just describe the pattern? Just describe the pattern.

---

### Issue #7: Near-match project suggestions

**Current State:**
- When creating a new project, user is prompted "Add new project? [y/n/c]"
- No suggestion of similar existing projects
- Fuzzy matching code exists (`utils/fuzzy.rs`)

**Analysis:**
The `fuzzy::find_near_project_matches` function already exists and is used in one place. It should also be used during project creation prompts.

**Proposed Fix:**
In `prompt_create_project`, check for near matches first:

```rust
fn prompt_create_project(project_name: &str, conn: &Connection) -> Result<Option<bool>> {
    let all_projects = ProjectRepo::list_all(conn)?;
    let matches = fuzzy::find_near_project_matches(project_name, &all_projects, 2);
    
    if !matches.is_empty() {
        eprintln!("Similar existing projects: {}", matches.join(", "));
    }
    
    eprint!("This is a new project '{}'. Add new project? [y/n/c] (default: y): ", project_name);
    // ... rest of prompt
}
```

**Decision Point:** None - straightforward improvement.

---

### Issue #8: Queue sorting

**Current State:**
- Queue order is determined by `enqueue` order and `on <id>` movements
- No way to reorder queue by task attributes

**Analysis:**
This is a feature request. Options:

**Option A: Explicit sort command**
```bash
tatl queue sort priority    # Reorder queue by priority
tatl queue sort due         # Reorder queue by due date
tatl queue sort alloc       # Reorder queue by allocation
```

**Option B: Temporary sort for display only**
```bash
tatl list sort:priority     # Already works - just displays sorted, doesn't change queue
```

**Option C: No queue sorting**
The philosophy of TATL is "what's next is queue[0]." If you want to change priority, you manually reorder with `enqueue`/`dequeue`/`on`. Automatic sorting undermines this intentionality.

**Recommendation:** Option C (no change). Queue order should be explicit user intent. Use `tatl list sort:priority` to visualize, then manually `on <id>` to bring the right task to the top.

**Decision Point:** The point is to keep things simple for the user. In order of queueing is simple. But to resort to priority or other considerations reduces mental load. It allows the user to change from FIFO to "what's due" mode or "what's big" or "what's important". Let's add Option A with negation options.

---

### Issue #9: Description search in filters

**Current State:**
- `tatl list project:work` - works (filters by project)
- `tatl list +urgent` - works (filters by tag)
- `tatl list Words` - fails ("Invalid filter token: Words")

**Analysis:**
The filter parser doesn't recognize bare words as description searches. This is intentional - bare numbers are IDs, bare words are ambiguous.

**Proposed Solutions:**

**Option A: Explicit desc: prefix**
```bash
tatl list desc:Words       # Search descriptions containing "Words"
tatl list desc:"multi word" # With quotes
```

**Option B: Bare words become description search**
```bash
tatl list Words            # Searches descriptions
tatl list project:work bug # "project:work AND description contains 'bug'"
```

**Considerations:**
- Option A is explicit and unambiguous
- Option B is convenient but could conflict with future filter keywords
- What about partial matches? Case sensitivity?

**Recommendation:** Option A. Add `desc:<pattern>` filter term. Keep parsing unambiguous.

**Decision Point:** Go with Option A, but handle substring and regex searches.

---

## Summary: Recommended Actions

| Issue | Action | Complexity | Priority |
|-------|--------|------------|----------|
| #1 | Extend `sessions report` to accept filters | Low | Medium |
| #2 | Allow `--on [time]` in add command | Low | High |
| #3 | Add interval syntax to `sessions report`; keep `status` | Medium | Low |
| #4 | Fix session split bug for open sessions | Low | High (bug) |
| #5 | Optional interval syntax for `sessions modify` | Medium | Low |
| #6 | Validate respawn rules on modify + preview | Low | High |
| #7 | Add near-match suggestions on project create | Low | Medium |
| #8 | No change (queue order is explicit) | N/A | N/A |
| #9 | Add `desc:<pattern>` filter term | Medium | Medium |

**High priority items:** #2, #4, #6 (usability gaps and bugs)
**Medium priority:** #1, #7, #9 (reasonable extensions)
**Low priority / skip:** #3, #5, #8 (nice-to-have, adds complexity)

---

## Implementation Plan

Based on the decisions above, here is the prioritized implementation plan.

### Phase 1: Bug Fixes and Validation (High Priority)

#### 1.1 Fix Session Split Bug (#4)
**File:** `src/cli/commands.rs`

**Current behavior:** When `onoff` splits a running session, the second part incorrectly gets `end_ts = i64::MAX` and is created as a closed session.

**Change:** In `modify_session_for_removal`, detect when splitting an open session and create the second part as open:

```rust
// In the split case (remove_start > s_start && remove_end < s_end):
if session.end_ts.is_none() {
    // Original session was open - second part should be open too
    SessionRepo::create_open(conn, session.task_id, remove_end)?;
} else {
    SessionRepo::create_closed(conn, session.task_id, remove_end, s_end)?;
}
```

**Testing:** Add test case that splits a live session and verifies the second part remains open.

---

#### 1.2 Validate Respawn Rules on Modify (#6)
**Files:** `src/cli/commands.rs`, `src/cli/parser.rs`

**Current behavior:** Invalid respawn rules like `respawn:every:fridayy` are silently accepted.

**Changes:**

1. In `handle_task_modify` (or in `modify_single_task`), after parsing, validate the respawn field:

```rust
if let Some(Some(respawn_str)) = &parsed.respawn {
    use crate::respawn::parser::RespawnRule;
    RespawnRule::parse(respawn_str).map_err(|e| {
        anyhow::anyhow!("Invalid respawn rule '{}': {}", respawn_str, e)
    })?;
}
```

2. After successful modification, if respawn was set, print a description:

```rust
if let Some(Some(respawn_str)) = &parsed.respawn {
    let rule = RespawnRule::parse(respawn_str)?;
    println!("↻ {}", describe_respawn_pattern(&rule));
}
```

3. Add `describe_respawn_pattern` function that returns human-readable description:
   - `daily` → "When completed, a new task will be created for the next day"
   - `weekly` → "When completed, a new task will be created for the next week"
   - `weekdays:mon,wed,fri` → "When completed, a new task will be created for the next Mon, Wed, or Fri"
   - etc.

**Testing:** Test that invalid rules are rejected; test that valid rules show description.

---

#### 1.3 Extend `--on` to Accept Optional Time (#2)
**File:** `src/cli/commands.rs`, `src/cli/mod.rs`

**Current behavior:** `--on` is a boolean flag that starts timing at "now".

**Changes:**

1. Change clap definition from:
```rust
#[arg(long = "on")]
on: bool,
```
to:
```rust
#[arg(long = "on")]
on: Option<Option<String>>,  // None = not specified, Some(None) = --on alone, Some(Some(x)) = --on <time>
```

Or simpler approach using `num_args`:
```rust
#[arg(long = "on", num_args = 0..=1, default_missing_value = "")]
on: Option<String>,  // None = not specified, Some("") = --on alone, Some(x) = --on <time>
```

2. In `handle_task_add`, after creating the task, if `--on` was specified:
```rust
if let Some(on_time) = on {
    let start_ts = if on_time.is_empty() {
        chrono::Utc::now().timestamp()
    } else {
        parse_date_expr(&on_time)?
    };
    // Enqueue to position 0 and start session at start_ts
    StackRepo::push_to_top(&conn, stack_id, task_id)?;
    SessionRepo::create_open_at(&conn, task_id, start_ts)?;
}
```

3. May need to add `SessionRepo::create_open_at` if it doesn't exist (or use existing method with timestamp).

**Testing:** Test `tatl add "task" --on`, `tatl add "task" --on 14:00`.

---

### Phase 2: Syntax Consistency

#### 2.1 Drop `status` Command (#3)
**Files:** `src/cli/mod.rs`, `src/cli/commands.rs`, `src/cli/abbrev.rs`

**Changes:**
1. Remove `Commands::Status` variant from enum
2. Remove `handle_status` function
3. Remove "status" from `TOP_LEVEL_COMMANDS` in abbrev.rs
4. Update documentation

**Testing:** Verify `tatl status` gives "unknown command" error.

---

#### 2.2 Interval Syntax for `sessions report` (#3)
**File:** `src/cli/commands_sessions.rs`

**Current:** `tatl sessions report <start> [end]` with two positional args.

**Change:** Accept single interval arg `<start>..<end>`:

```rust
// Parse first arg
if let Some(interval_arg) = args.first() {
    if interval_arg.contains("..") {
        // Parse as interval
        let (start_str, end_str) = interval_arg.split_once("..").unwrap();
        start_ts = if start_str.is_empty() { None } else { Some(parse_date_expr(start_str)?) };
        end_ts = if end_str.is_empty() || end_str == "now" { 
            Some(chrono::Utc::now().timestamp()) 
        } else { 
            Some(parse_date_expr(end_str)?) 
        };
    } else {
        // Existing positional arg parsing
    }
}
```

**Examples that should work:**
- `tatl sessions report -7d` (existing)
- `tatl sessions report -7d..` (last 7 days to now)
- `tatl sessions report -7d..now` (same)
- `tatl sessions report 2026-01-01..2026-01-15` (explicit range)

---

#### 2.3 Interval Syntax for `sessions modify` - Replace `start:`/`end:` (#5)
**File:** `src/cli/commands_sessions.rs`

**Current:** `tatl sessions modify <id> start:<time> end:<time>`

**Change:** Replace with interval-only syntax:
- `tatl sessions modify <id> <start>..<end>` - modify both
- `tatl sessions modify <id> <start>..` - modify start only
- `tatl sessions modify <id> ..<end>` - modify end only

**Implementation:**
```rust
fn parse_session_modify_interval(arg: &str) -> Result<(Option<i64>, Option<i64>)> {
    if let Some((start_str, end_str)) = arg.split_once("..") {
        let start = if start_str.is_empty() { None } else { Some(parse_date_expr(start_str)?) };
        let end = if end_str.is_empty() { None } else { Some(parse_date_expr(end_str)?) };
        Ok((start, end))
    } else {
        Err(anyhow::anyhow!("Expected interval format: <start>..<end>, <start>.., or ..<end>"))
    }
}
```

**Deprecation:** Keep `start:`/`end:` working for one version with a deprecation warning, then remove.

---

### Phase 3: Feature Additions

#### 3.1 Add Filtering to `sessions report` (#1)
**File:** `src/cli/commands_sessions.rs`

**Current:** Report accepts only time range.

**Change:** Accept filter tokens after the time range:
```bash
tatl sessions report -7d project:work +urgent
```

**Implementation:**
1. Parse first arg(s) as time range
2. Remaining args go through filter parser
3. Filter sessions by task before aggregating

---

#### 3.2 Add `projects report` Command (#1)
**Files:** `src/cli/mod.rs`, `src/cli/commands.rs` (or new file)

**New command:** `tatl projects report`

**Output format:**
```
Projects Report
===============
Project          Proposed  Queued  Paused  NEXT  LIVE  Done  Total
───────          ────────  ──────  ──────  ────  ────  ────  ─────
admin                   2       3       1     0     0     5     11
  email                 1       2       0     0     0     3      6
  meeting               1       1       1     0     0     2      5
research                3       0       0     1     0     2      6
  project_a             2       0       0     0     0     1      3
  project_b             1       0       0     1     0     1      3
(no project)            1       0       0     0     0     0      1
───────          ────────  ──────  ──────  ────  ────  ────  ─────
TOTAL                   6       3       1     1     0     7     18
```

**Implementation:**
1. Fetch all tasks
2. Compute kanban status for each
3. Group by project (hierarchical)
4. Count by status
5. Format table

---

#### 3.3 Near-Match Project Suggestions (#7)
**File:** `src/cli/commands.rs`

**Current:** `prompt_create_project` doesn't show similar existing projects.

**Change:** Before prompting, check for near matches:

```rust
fn prompt_create_project(project_name: &str, conn: &Connection) -> Result<Option<bool>> {
    let all_projects = ProjectRepo::list_all(conn)?;
    let project_names: Vec<&str> = all_projects.iter().map(|p| p.name.as_str()).collect();
    let matches = fuzzy::find_near_project_matches(project_name, &project_names, 2);
    
    if !matches.is_empty() {
        eprintln!("Note: Similar existing projects: {}", matches.join(", "));
    }
    
    eprint!("'{}' is a new project. Create it? [y/n/c] (default: y): ", project_name);
    // ... existing logic
}
```

**Note:** Function signature needs to accept `&Connection` parameter.

---

#### 3.4 Queue Sorting (#8)
**Files:** `src/cli/mod.rs`, `src/cli/commands.rs`, `src/repo/stack.rs`

**New command:** `tatl queue sort <field> [--desc]`

**Supported fields:** `priority`, `due`, `scheduled`, `alloc`, `id`, `description`

**Syntax:**
```bash
tatl queue sort priority        # Sort ascending (lowest priority first? or highest?)
tatl queue sort priority --desc # Descending (highest first)
tatl queue sort -priority       # Alternative: - prefix for descending
tatl queue sort due             # Sort by due date (soonest first)
tatl queue sort -due            # Latest due first
```

**Implementation:**
1. Add `QueueCommands` subcommand enum (or add to existing stack handling)
2. Fetch all queued task IDs with their attributes
3. Sort in memory by specified field
4. Update `stack_items` table with new positions

```rust
fn handle_queue_sort(field: &str, descending: bool) -> Result<()> {
    let conn = DbConnection::connect()?;
    let stack = StackRepo::get_or_create_default(&conn)?;
    let items = StackRepo::get_items(&conn, stack.id.unwrap())?;
    
    // Fetch task details for each item
    let mut task_items: Vec<(i64, Task)> = items.iter()
        .filter_map(|item| {
            TaskRepo::get_by_id(&conn, item.task_id).ok().flatten()
                .map(|t| (item.task_id, t))
        })
        .collect();
    
    // Sort by field
    match field {
        "priority" => task_items.sort_by(|a, b| /* compare priority */),
        "due" => task_items.sort_by_key(|(_id, t)| t.due_ts),
        // ... etc
    }
    
    if descending {
        task_items.reverse();
    }
    
    // Update positions
    StackRepo::reorder(&conn, stack.id.unwrap(), task_items.iter().map(|(id, _)| *id).collect())?;
    
    println!("Queue sorted by {}", field);
    Ok(())
}
```

**Note:** Need to add `StackRepo::reorder` function.

---

#### 3.5 Description Filter (`desc:`) (#9)
**File:** `src/filter/parser.rs`, `src/filter/evaluator.rs`

**New filter term:** `desc:<pattern>`

**Behavior:**
- `desc:foo` - substring match (case-insensitive)
- `desc:/foo.*/` - regex match (if surrounded by `/`)

**Implementation:**

1. In `parser.rs`, add to `FilterTerm` enum:
```rust
Desc(String, bool),  // (pattern, is_regex)
```

2. In `parse_filter_term`:
```rust
"desc" | "description" => {
    let is_regex = value.starts_with('/') && value.ends_with('/');
    let pattern = if is_regex {
        value[1..value.len()-1].to_string()
    } else {
        value.to_string()
    };
    Ok(Some(FilterTerm::Desc(pattern, is_regex)))
}
```

3. In `evaluator.rs`, add matching:
```rust
FilterTerm::Desc(pattern, is_regex) => {
    if *is_regex {
        let re = regex::Regex::new(pattern)?;
        Ok(re.is_match(&task.description))
    } else {
        Ok(task.description.to_lowercase().contains(&pattern.to_lowercase()))
    }
}
```

4. Add `regex` crate dependency if not already present.

**Examples:**
```bash
tatl list desc:meeting           # Tasks with "meeting" in description
tatl list desc:"project review"  # With quotes for spaces
tatl list desc:/^Daily/          # Regex: starts with "Daily"
```

---

### Phase 4: Documentation

Update the following after implementation:

1. **README.md** - Update examples and command reference
2. **docs/COMMAND_REFERENCE.md** - Full documentation of all changes
3. **Remove** references to `status` command
4. **Add** documentation for:
   - `--on [time]` in add
   - Interval syntax for sessions modify
   - `projects report` command
   - `queue sort` command
   - `desc:` filter

---

## Implementation Order

| Order | Issue | Description | Est. Effort |
|-------|-------|-------------|-------------|
| 1 | #4 | Fix session split bug | 30 min |
| 2 | #6 | Validate respawn on modify | 1 hr |
| 3 | #2 | `--on [time]` for add | 1 hr |
| 4 | #7 | Near-match project suggestions | 30 min |
| 5 | #3a | Drop `status` command | 30 min |
| 6 | #5 | Interval syntax for `sessions modify` | 1.5 hr |
| 7 | #3b | Interval syntax for `sessions report` | 1 hr |
| 8 | #1a | Add filtering to `sessions report` | 1 hr |
| 9 | #9 | `desc:` filter | 1.5 hr |
| 10 | #1b | `projects report` command | 2 hr |
| 11 | #8 | `queue sort` command | 2 hr |
| 12 | - | Documentation updates | 1 hr |

**Total estimated effort:** ~13 hours

---

## Open Questions

1. **Queue sort default direction:** For `tatl queue sort priority`, should ascending mean "lowest priority number first" or "highest"? Suggest: lowest first (P1 before P3), use `--desc` or `-priority` for highest first.

Go with same sorting conventions as used in task list sort:

2. **Deprecation period for `start:`/`end:`:** Should we immediately remove or keep for backward compatibility? Suggest: Keep for one version with deprecation warning.

Immediately remove. No backwards compatibility.

3. **Regex dependency:** Adding regex crate increases binary size. Is this acceptable? Alternative: Only support substring matching.

Only support substring matching for now.

## Completed Changes
### High Priority (Bugs + Usability Gaps)
1. Fix session split bug (#4) - Open sessions are now correctly preserved when splitting; the second part remains open instead of being created with i64::MAX end time.
2. Validate respawn rules on modify (#6) - Respawn rules are now validated when using modify, with a helpful preview message showing what will happen on completion.
3. Extend --on to accept optional time (#2) - You can now use --on=14:00 to start timing at an earlier time when adding tasks.
### Medium Priority (Reasonable Extensions)
4. Near-match project suggestions (#7) - When creating a new project, similar existing projects are now shown to help catch typos.
5. Add filtering to sessions report (#1a) - tatl sessions report now accepts task filters like project:work +urgent.
6. Add desc: filter (#9) - New case-insensitive description search: tatl list desc:meeting.
7. Add projects report command (#1b) - New command showing task counts by project and kanban status.
8. Add queue sort command (#8) - New command to sort the queue: tatl queue sort due, tatl queue sort -priority.
### Other Changes
9. Dropped status command (#3a) - The dashboard command was removed as decided.
10. Interval syntax for sessions modify (#5) - Now supports 09:00..17:00 syntax.
11. Interval syntax for sessions report (#3b) - Now supports -7d..now syntax.
12. Documentation updates - README.md and COMMAND_REFERENCE.md updated with all new features.