# Build Team Handoff Package

This package is the implementation contract for building the Rust + SQLite CLI described in:

* **Task System Design & Syntax Specification**
* **Rust + SQLite Engineering Plan**

It resolves remaining ambiguities and defines:

1. CLI grammar + parsing rules
2. Filter semantics (AND/OR/NOT)
3. Stack+clock semantics (stack[0] always live)
4. Micro-session behavior (merge/purge rules)
5. Recurrence semantics (least confusing, simplest, idempotent)
6. Output + exit codes
7. Database DDL + invariants
8. Acceptance tests (Given/When/Then)

---

## 1. MVP Scope Summary

### Included

* Tasks: add/modify/list/done/annotate
* Projects: master data CRUD (add/list/rename/archive)
* Tags: `+tag` / `-tag`
* Scheduling: `due`, `scheduled`, `wait`, derived `waiting`
* UDAs: `uda.<key>:<value>`
* Stack: single revolver stack with indexed operations
* Sessions: task-tied only
* Clock: single running session; stack operations affect timing
* Recurrence: templates + recurrence rules + `recur run`

### Not included (MVP)

* multi-stack contexts
* sync
* rich report customization
* interactive TUI

---

## 2. Global Conventions

### 2.1 Exit codes

* `0` success
* `1` user error (invalid input, empty stack, missing project, etc.)
* `>1` internal error (db corruption, unexpected invariants breach)

### 2.1.1 Error message format

* **User errors (exit code 1):** Messages start with "Error: " followed by descriptive text
* **Internal errors (exit code >1):** Messages start with "Internal error: " followed by technical details
* All error messages go to **stderr**
* Success messages and normal output go to **stdout**
* Error messages should be clear, actionable, and avoid technical jargon when possible

Examples:
```
Error: No active task (stack is empty)
Error: Project 'work' already exists
Error: Task 10 not found
Internal error: Database constraint violation: unique constraint failed
```

### 2.2 Timezone behavior (match Taskwarrior)

* Parse date expressions in **local timezone**.
* Store timestamps as **UTC epoch seconds** in SQLite.
* Display timestamps in **local timezone**.
* No per-event timezone stored; interpretation follows current local TZ rules (including DST) at display time.

**DST (Daylight Saving Time) handling:**
* All timestamps stored as UTC epoch seconds eliminates DST ambiguity at storage time
* When displaying, conversion from UTC to local time uses current system timezone rules
* DST transitions are handled automatically by the system's timezone library
* **Fall back hour (2am → 1am):** If a time falls in the ambiguous hour, use the first occurrence (earlier timestamp)
* **Spring forward hour (2am → 3am):** The skipped hour doesn't exist; times in that range are invalid and should error
* Date parsing uses local timezone at parse time, then converts to UTC for storage
* This approach ensures consistency: same UTC timestamp always represents the same absolute moment, regardless of DST rules at display time

### 2.3 Database location and configuration

**Default database location:**
* Database file: `~/.taskninja/tasks.db`
* Configuration directory: `~/.taskninja/`
* Configuration file: `~/.taskninja/rc` (optional)

**Configuration file (`~/.taskninja/rc`):**
* Simple key-value format (one per line): `key=value`
* Supported keys:
  * `data.location` - Override database location (absolute or relative path)
  * Example: `data.location=/home/user/my-tasks.db`
* If `data.location` is set, use that path instead of default
* If path is relative, it's resolved relative to the configuration file's directory
* Configuration file is optional - if not present, defaults are used

**Database initialization:**
* Database is created automatically on first use if it doesn't exist
* Directory structure is created automatically if needed
* No explicit initialization command required

### 2.4 Description parsing

* `task add` and `task modify` parse descriptions from command arguments.
* Description is the first sequence of tokens that don't match field patterns (`field:value`), tag patterns (`+tag`/`-tag`), or flags (`--flag`).
* Field tokens, tag tokens, and flags can appear anywhere in the argument list.

Example:

```
task add fix the clock project:home
task add project:home fix the clock +urgent
```

---

## 3. CLI Grammar and Parsing Rules

### 3.1 Command skeleton

```
task <global-subcommand>

task <filter...> <task-subcommand>
```

* If the first token matches a known global subcommand (`projects`, `stack`, `clock`, `recur`, `templates`, `sessions`), treat as global.
* Otherwise parse leading tokens as `<filter...>` until a known task subcommand is found (`add`, `modify`, `list`, `done`, `annotate`, `clock`).
* For `stack` commands, see Section 5.3 - task ID (not filter) comes after `task`, position comes after `stack` and before the command.

### 3.2 Task add

```
task add [--template <name>] <description...> [field:value ...] [+tag ...] [-tag ...]
```

Rules:

* Description is the first sequence of tokens that don't match:
  * field patterns (`field:value`),
  * tag patterns (`+tag`/`-tag`), or
  * flags (`--flag`)
* Field tokens, tag tokens, and flags can appear anywhere in the argument list
* Description is required

Examples:
```
task add fix the clock project:home
task add project:home fix the clock +urgent
task add --template meeting schedule standup daily
```

### 3.3 Task modify

```
task <id|filter> modify [<description...>] [field:value ...] [+tag ...] [-tag ...]
```

Rules:

* Modifies one or more tasks matching the filter
* Description is optional - if provided, replaces the existing description
* Description parsing follows same rules as `task add` (Section 3.2)
* Field tokens, tag tokens can appear anywhere in the argument list
* **Multiple task modification:**
  * If filter matches multiple tasks, user is alerted with count
  * User can confirm:
    * `y` or `yes` - apply to all matching tasks
    * `n` or `no` - skip all
    * `i` or `interactive` - confirm one by one for each task
  * With `--yes` flag: applies to all without confirmation
  * With `--interactive` flag: forces one-by-one confirmation

Examples:
```
task 10 modify +urgent project:work
task project:home modify due:tomorrow
task +urgent modify --yes +important  # Apply to all without confirmation
task project:work modify --interactive description:updated  # One-by-one confirmation
```

### 3.4 Field tokens

Supported MVP fields:

* `project:<name>`
* `due:<date_expr>`
* `scheduled:<date_expr>`
* `wait:<date_expr>`
* `alloc:<duration>`
* `template:<name>`
* `recur:<rule>`
* `uda.<key>:<value>`

**UDA (User Defined Attribute) format:**
* Syntax: `uda.<key>:<value>` where `<key>` is the attribute name and `<value>` is the attribute value
* Key format: Must match tag charset `[A-Za-z0-9_\-\.]+` (same as tags)
* Value format: Any string (no restrictions)
* Storage: Stored in `tasks.udas_json` as JSON object: `{"key1": "value1", "key2": "value2", ...}`
* Key storage: Keys are stored **without** the `uda.` prefix (e.g., `uda.priority:high` stores as `{"priority": "high"}`)
* Clearing: Use `uda.<key>:none` to remove a UDA
* Multiple UDAs: Can have multiple UDAs per task, each with a unique key

Examples:
```
task add fix bug uda.priority:high uda.estimate:2h
task 10 modify uda.priority:low
task 10 modify uda.priority:none  # Remove priority UDA
```

Clearing fields:

* `<field>:none` clears (where applicable)

### 3.3 Task modify

```
task <id|filter> modify [<description...>] [field:value ...] [+tag ...] [-tag ...]
```

Rules:

* Modifies one or more tasks matching the filter
* Description is optional - if provided, replaces the existing description
* Description parsing follows same rules as `task add` (Section 3.2)
* Field tokens, tag tokens can appear anywhere in the argument list
* **Multiple task modification:**
  * If filter matches multiple tasks, user is alerted with count
  * User can confirm:
    * `y` or `yes` - apply to all matching tasks
    * `n` or `no` - skip all
    * `i` or `interactive` - confirm one by one for each task
  * With `--yes` flag: applies to all without confirmation
  * With `--interactive` flag: forces one-by-one confirmation

Examples:
```
task 10 modify +urgent project:work
task project:home modify due:tomorrow
task +urgent modify --yes +important  # Apply to all without confirmation
task project:work modify --interactive description:updated  # One-by-one confirmation
```

### 3.4 Tags

* `+tag` adds tag
* `-tag` removes tag

Tags are tokens with no spaces; allowed charset: `[A-Za-z0-9_\-\.]+`.

### 3.5 Date expressions

Accepted forms (examples):

* Absolute: `2026-01-10`, `2026-01-10T14:30`
* Relative: `today`, `tomorrow`, `+2d`, `+1w`, `eod`, `eow`, `eom`
* Time-only: `9am`, `14:30`, `noon`

**Time-only resolution:** Time-only expressions resolve to a specific instance using a 24-hour window:
* Consider a 24-hour window: 8 hours in the past, 16 hours in the future
* Within this window, if the future instance is no more than twice as far away as the past instance, resolve to the **future instance**
* Otherwise, resolve to the **nearest instance** (past or future)

Examples:
* `9am` at 08:00 → today 9am (1h future, 23h past; future is closer, choose future)
* `9am` at 10:00 → today 9am (1h past, 23h future; 23h > 2×1h, so choose nearest = past)
* `9am` at 07:00 → today 9am (2h future, 22h past; 2h <= 2×22h, so choose future)
* `2pm` at 13:00 → today 2pm (1h future, 23h past; 1h <= 2×23h, so choose future)
* `2pm` at 15:00 → today 2pm (1h past, 23h future; 23h > 2×1h, so choose nearest = past)

### 3.6 Duration

Duration format: one or more unit specifications in order from largest to smallest unit.

**Grammar:**
```
duration = unit_spec+
unit_spec = digits unit
digits = [0-9]+
unit = 's' | 'm' | 'h' | 'd'
```

**Rules:**
* Units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days)
* Ordering: units must appear in order from largest to smallest: `d`, `h`, `m`, `s`
* No spaces: units are concatenated without spaces (e.g., `1h30m`, not `1h 30m`)
* Each unit type may appear at most once
* At least one unit specification is required

**Examples:**
* `30s` - 30 seconds
* `10m` - 10 minutes
* `2h` - 2 hours
* `1h30m` - 1 hour 30 minutes
* `2d5h30m` - 2 days 5 hours 30 minutes
* `90m` - 90 minutes (valid, but `1h30m` preferred for readability)

**Invalid examples:**
* `10s30m` - wrong order (should be `30m10s`)
* `1h 30m` - spaces not allowed
* `2h2h` - duplicate unit
* `1h30` - missing unit

---

## 3.7 Project commands

Projects are master data used to organize tasks. Project commands are global subcommands.

**Nested Projects:** Projects support hierarchical organization using dot notation (e.g., `admin.email`, `sales.northamerica.texas`). The hierarchy is implicit in the project name - no explicit parent-child relationships are stored. Dots separate hierarchy levels.

### 3.7.1 `task projects add <name>`

Creates a new project with the specified name.

* `<name>` is the project name (required)
* Project names must be unique
* Errors if project with same name already exists
* Project is created as active (not archived)
* Supports nested projects using dot notation (e.g., `admin.email`, `sales.northamerica.texas`)

Examples:
```
task projects add work
task projects add home
task projects add admin.email
task projects add sales.northamerica.texas
```

### 3.7.2 `task projects list [--archived]`

Lists all projects.

* Without `--archived`: lists only active (non-archived) projects
* With `--archived`: lists only archived projects
* Supports `--json` output format (see Section 8.2)
* Lists all projects including nested projects (e.g., shows both `admin` and `admin.email` if both exist)
* Projects are listed in alphabetical order

Examples:
```
task projects list
task projects list --archived
task projects list --json
```

### 3.7.3 `task projects rename <old_name> <new_name> [--force]`

Renames an existing project.

* `<old_name>` is the current project name
* `<new_name>` is the new project name
* Errors if old project doesn't exist
* **Without `--force`:** Errors if new name already exists
* **With `--force`:** If new name already exists, merges projects:
  * Moves all tasks from `<old_name>` to `<new_name>`
  * Deletes the `<old_name>` project
  * If `<old_name>` is archived and `<new_name>` is active, merged project becomes active
  * If `<old_name>` is active and `<new_name>` is archived, merged project remains archived
* Updates all tasks referencing the project
* **Nested projects:** Renaming a parent project (e.g., `admin` to `administration`) does NOT automatically rename nested projects (e.g., `admin.email` remains unchanged). To rename nested projects, rename them explicitly.

Examples:
```
task projects rename work office
task projects rename admin.email admin.mail
task projects rename temp-project work --force  # Merge temp-project into work
```

### 3.7.4 `task projects archive <name>`

Archives a project.

* `<name>` is the project name to archive
* Errors if project doesn't exist
* Sets `is_archived=1` for the project
* Archived projects are excluded from default `list` output
* Tasks referencing archived projects remain unchanged

Example:
```
task projects archive old-project
```

### 3.7.5 `task projects unarchive <name>`

Unarchives a project (restores from archived state).

* `<name>` is the project name to unarchive
* Errors if project doesn't exist
* Sets `is_archived=0` for the project
* Project becomes active again

Example:
```
task projects unarchive old-project
```

### 3.8 Sessions commands

Sessions represent time spent working on tasks. Session commands allow viewing session history.

* `task [<id>] sessions list [--json]`

  * Lists session history
  * Without `<id>`: lists all sessions (all tasks)
  * With `<id>`: lists sessions for the specified task only
  * Sessions are ordered by start time (newest first)
  * Shows: task ID, description, start time, end time, duration
  * Supports `--json` output format (see Section 8.2)
  
  Examples:
  ```
  task sessions list
  task 10 sessions list
  task sessions list --json
  task 10 sessions list --json
  ```

* `task [<id>] sessions show`

  * Shows detailed session information
  * Without `<id>`: shows current running session (if any)
  * With `<id>`: shows most recent session for that task
  * Includes: task details, start/end times, duration, linked annotations
  
  Examples:
  ```
  task sessions show
  task 10 sessions show
  ```

---

## 4. Filter Semantics (AND/OR/NOT)

### 4.1 Token model

Filters form a boolean expression. MVP supports:

* Implicit **AND** between adjacent terms
* Explicit **OR** with keyword `or`
* Negation with keyword `not` applied to the following term

Examples:

```
task project:home +urgent list
# AND: project home AND tag urgent

task +urgent or +important list

task not +waiting list
```

**Complex filter examples:**

```
# Multiple AND conditions
task project:work +urgent due:tomorrow list
# Matches: project=work AND tag=urgent AND due=tomorrow

# OR with AND
task project:work +urgent or project:home +important list
# Matches: (project=work AND tag=urgent) OR (project=home AND tag=important)

# NOT with AND
task project:work not +waiting list
# Matches: project=work AND NOT tag=waiting

# Complex: NOT with OR
task not +urgent or not +important list
# Matches: (NOT tag=urgent) OR (NOT tag=important)
# Note: This matches tasks that don't have urgent OR don't have important
# (includes tasks with neither, tasks with only urgent, tasks with only important)

# Multiple OR conditions
task +urgent or +important or +critical list
# Matches: tag=urgent OR tag=important OR tag=critical

# Combining status with other filters
task status:pending project:work +urgent list
# Matches: status=pending AND project=work AND tag=urgent
```

Precedence:

* `not` binds to the next term
* `and` (implicit) binds tighter than `or`
* Evaluation order: `not` first, then implicit `and`, then `or`

No parentheses in MVP (to avoid complex parsing), but expression design should allow them later.

### 4.2 MVP filter terms

* `id:<n>` or bare numeric id
* `status:pending|completed|deleted`
* `project:<name>` - matches exact project name or any nested project under that name (prefix match)
  * `project:admin` matches `admin`, `admin.email`, `admin.other`, etc.
  * `project:admin.email` matches only `admin.email` and nested projects like `admin.email.inbox`
  * Use exact name for precise matching
* `+tag` / `-tag`
* `due:any|none|<expr>` and similarly for `scheduled`, `wait`
* `waiting` (derived: `wait > now`)

---

## 5. Stack + Clock Semantics (Slot 0 is Always Live)

### 5.1 Invariants

* Exactly one stack exists (single revolver).
* `stack[0]` is the **active slot**.
* If a session is running, it is always for `stack[0]`.
* At most one session is running.

### 5.1.1 Stack Initialization

* The default stack is **auto-created** on the first stack operation.
* The default stack has `name='default'`.
* No explicit initialization or migration step is required.
* All stack operations (show, set, pick, roll, drop, clear) will automatically create the default stack if it doesn't exist.
* The stack is created with `created_ts` and `modified_ts` set to the current time.

### 5.2 Stack operations affect timing by default

Default behavior: stack operations **preserve clock state**, and ensure slot 0 remains the live task.

* If clock is running and stack[0] changes, the current session is closed at `now` and a new session is started for the new `stack[0]` at the same timestamp.
* If clock is not running, stack operations do not create sessions.

Optional explicit flags:

* `--clock in`: ensure clock is running after the operation (starts `stack[0]` if needed)
* `--clock out`: ensure clock is stopped after the operation (closes running session if any)

### 5.3 Stack commands

**Primary commands (recommended):**
* `task <id> clock in` - **Do it now**: push task to stack[0] and start clock (see Section 5.4)
* `task <id> enqueue` - **Do it later**: add task to end of stack (doesn't start clock, doesn't affect running session)

**Enqueue command details:**
* `task <id> enqueue` adds the task to the end of the stack (position -1)
* Does not start the clock (no session created)
* Does not affect a running session (if clock is running, it continues)
* If task is already in stack, moves it to the end (repositioning)

**Stack management commands:**
* `task stack show` - show current stack
* `task stack <index> pick [--clock in|--clock out]` - move task at position to top
* `task stack roll [n] [--clock in|--clock out]` - rotate stack
  * `n` defaults to 1 if omitted (rotate once)
  * Rotates stack by `n` positions: `[a,b,c]` with `roll 1` becomes `[b,c,a]`
  * Negative `n` rotates in reverse direction
* `task stack <index> drop [--clock in|--clock out]` - remove task at position
* `task stack clear [--clock out]` - clear all tasks from stack

**Syntax rules:**
* Task ID comes after `task` (no filtering - only single task ID)
* Stack position/index comes after `stack` and before the command (for `pick`, `drop`)

**Indexing:**
* 0 = top
* -1 = end
* clamp out-of-range

Examples:
```
task stack show
task 10 clock in             # Do it now: push to top and start
task 11 enqueue              # Do it later: add to end
task stack 2 pick            # Move task at position 2 to top
task stack 1 drop            # Remove task at position 1
task stack roll              # Rotate once (default)
task stack roll 2            # Rotate twice
task stack clear             # Clear all
```

### 5.4 Clock commands

* `task clock in [<start>|<start..end>]`

  * Requires stack non-empty.
  * Errors if a session is already running.
  * **Default behavior:** If no start time is provided, it defaults to "now" (current time).
  * If `<start>` only: starts timing `stack[0]` at the specified start time.
  * If no arguments: starts timing `stack[0]` at now.
  * If `<start..end>` interval: creates a closed session for `stack[0]` with both start and end times specified.
  
  Interval syntax: `<start_expr>..<end_expr>` where both are date expressions (see Section 3.5).
  
  Examples:
  ```
  task clock in                    # Start now
  task clock in 2026-01-10T09:00  # Start at specified time
  task clock in 2026-01-10T09:00..2026-01-10T10:30  # Closed interval
  task clock in today..eod         # Closed interval (today to end of day)
  task clock in 09:00..11:00       # Closed interval (time-only)
  ```
  
  **Overlap prevention:** If another task session starts before the end time of a closed interval session, the end time of the interval session is automatically amended to the start time of the new session to prevent overlap.

* `task <id> clock in [<start>|<start..end>]`

  * **"Do it now" command**: Pushes task to stack[0] and starts clock.
  * Moves task to stack[0] (top of stack).
  * **Default behavior:** If no start time is provided, it defaults to "now" (current time).
  * If a session is running, closes it at the effective start time (now if start omitted, or the specified start time).
  * If interval provided: creates closed session for new `stack[0]` with specified start and end times.
  * If start only: starts timing new `stack[0]` at specified start time.
  * If no arguments: starts timing new `stack[0]` at now.
  * Same overlap prevention rule applies: if a subsequent session starts before the interval end, the interval end is amended.
  
  Examples:
  ```
  task 10 clock in                    # Push task 10 to top and start now
  task 10 clock in 2026-01-10T09:00  # Push to top and start at specified time
  task 10 clock in 09:00..11:00      # Push to top and create closed interval
  ```

* `task clock out [<end_expr>]`

  * Closes the running session at the specified end time.
  * **Default behavior:** If no end time is provided, it defaults to "now" (current time).

### 5.5 Done semantics

* `task done` is shorthand for `task <stack[0]> done`.
* `task done` **errors** if:

  * stack is empty, OR
  * no session is running.

Canonical:

```
task [<id|filter>] done [--at <end_expr>] [--next]
```

Note: `task done` (without ID/filter) is shorthand for `task <stack[0]> done`.

**Multiple task completion:**
* If filter matches multiple tasks, user is alerted with count
* User can confirm:
  * `y` or `yes` - complete all matching tasks
  * `n` or `no` - skip all
  * `i` or `interactive` - confirm one by one for each task
* With `--yes` flag: completes all without confirmation
* With `--interactive` flag: forces one-by-one confirmation
* **Important:** Only tasks with running sessions can be completed via `task done` (without ID/filter). Filter-based completion requires explicit task IDs or filters that match tasks with running sessions.

Behavior:

1. close running session at `--at` or now
2. mark task completed
3. remove from stack
4. if `--next` and stack non-empty: start session for new stack[0] at same timestamp

### 5.6 Task annotations

Annotations are timestamped notes attached to tasks, useful for tracking progress, adding context, or logging work. Annotations can be linked to the session during which they were created.

* `task [<id>] annotate <note...>`

  * Adds an annotation to a task
  * **If `<id>` is provided:** Adds annotation to the specified task
  * **If `<id>` is omitted and clock is running:** Adds annotation to the currently clocked-in task (stack[0]) and links it to the running session
  * **If `<id>` is omitted and clock is not running:** Errors (must specify task ID)
  * Annotation text is the remaining arguments (all tokens after `annotate`)
  * Timestamp is automatically set to current time (now)
  * **Session linking:** If created during a running session for the task, the annotation is linked to that session via `session_id`
  * Multiple annotations can be added to a task
  * Annotations are ordered by timestamp (oldest first)
  
  Examples:
  ```
  task 10 annotate Started working on this
  task 10 annotate Found the bug in the authentication module
  task annotate Waiting for API response from team  # If clocked in, uses current task
  ```

* `task <id> annotate --delete <annotation_id>`

  * Deletes a specific annotation by ID
  * Annotation IDs are shown when listing task details
  * Errors if annotation doesn't exist or doesn't belong to the task

* Annotations are displayed when showing task details (e.g., `task <id> list` with verbose output)
* Annotations are included in JSON output when using `--json` flag
* Session-linked annotations can be used to track work notes during specific time periods

### 5.7 Task Events (Immutable History)

The `task_events` table provides an **immutable audit log** of all changes to tasks. This enables:
* Reconstructing complete task history
* Analysis and reporting on task lifecycle
* Audit trails for compliance
* Debugging and troubleshooting

**Event recording:**
* Events are automatically recorded for all task state changes
* Events are **never modified or deleted** (immutable)
* Each event includes:
  * `task_id` - the task that changed
  * `ts` - timestamp of the event
  * `event_type` - type of event (see below)
  * `payload_json` - JSON object with event-specific data

**Event types:**
* `created` - task was created
* `modified` - task attributes were modified (description, project, due date, etc.)
* `status_changed` - task status changed (pending → completed, etc.)
* `tag_added` - tag was added to task
* `tag_removed` - tag was removed from task
* `annotation_added` - annotation was added
* `annotation_deleted` - annotation was deleted
* `stack_added` - task was added to stack
* `stack_removed` - task was removed from stack
* `session_started` - timing session started for task
* `session_ended` - timing session ended for task

**Event payload examples:**
* `created`: `{"description": "Fix bug", "project": "work"}`
* `modified`: `{"field": "due_ts", "old_value": null, "new_value": 1704067200}`
* `status_changed`: `{"old_status": "pending", "new_status": "completed"}`
* `tag_added`: `{"tag": "urgent"}`
* `stack_added`: `{"stack_id": 1, "position": 0}`

**Usage:**
* Events are recorded automatically by the system
* No direct user commands to create events
* Events can be queried for analysis (future feature)
* Events enable complete task history reconstruction

---

## 6. Micro-Session Policy (30-second rules)

Definitions:

* `MICRO = 30 seconds` (configurable in future; initially 30 seconds)

When a session is closed, if its duration `< MICRO`, it is a **micro-session**.

Rules:

1. **Merge**: if within `MICRO` seconds **of the micro-session's end time**, a session for the **same task** begins, merge the micro-session into the adjacent same-task session.

2. **Purge**: if within `MICRO` seconds **of the micro-session's end time**, a session for a **different task** begins, purge the micro-session.

3. **Otherwise keep**: if neither condition triggers, the micro-session remains.

**Timing clarification:** Both rules are evaluated relative to the micro-session's **end time**. A subsequent session that begins within `MICRO` seconds after the micro-session ends will trigger the appropriate rule.

Verbosity requirement:

* When a micro-session is created, print a warning that it may be merged/purged.
* When a micro-session is merged or purged, print what happened (task ids, timestamps, rule applied).

Implementation note:

* Apply merge/purge at the time the *subsequent* session begins or at the time the micro-session ends (whichever is simpler) as long as the semantics match.

---

## 7. Recurrence (Path of Least Resistance, Least Confusing)

Goal: simple, deterministic, and idempotent generation of instances.

### 7.1 Model

* A recurring **seed task** has:

  * `recur:<rule>`
  * `template:<name>` (optional)

* Instances generated are normal tasks with no `recur` field.

### 7.2 Idempotency approach (recommended)

Introduce a table to record generated occurrences:

* `recur_occurrences(seed_task_id, occurrence_ts, instance_task_id, PRIMARY KEY(seed_task_id, occurrence_ts))`

This avoids mutating seed state and prevents duplicates.

### 7.3 Recur command scope

The `recur` command manages recurring task generation. MVP scope matches Taskwarrior core capabilities:

* `task recur run [--until <date_expr>]` - Generate recurring task instances
* Future (not MVP): `task recur list` - List seed tasks, `task recur show <seed_id>` - Show seed details

**MVP scope:** Only `recur run` is supported. Other recur commands deferred to future versions.

### 7.3.1 `task recur run [--until <date_expr>]`

Default behavior:

* `--until` defaults to end of next week (configurable later); MVP default: `now + 14 days`.
* For each seed task with a `recur:` field (see Section 7.4 for grammar), generate occurrences > now and <= until.
* For each occurrence, if not present in `recur_occurrences`, create instance and insert row.

**Attribute precedence for instance creation (common sense approach):**

When creating an instance from a seed task, attributes are determined in this order:

1. **Template provides base/default attributes** (if template is specified)
   * Template acts as a starting point with default values
   * Example: Template "meeting" might have `project:work`, `+meeting`, `alloc:1h`

2. **Seed task attributes override template** (more specific wins)
   * Seed task attributes take precedence over template attributes
   * This allows customizing specific recurring tasks while keeping template defaults
   * Example: Seed has `project:home` → instance gets `project:home` (not template's `project:work`)

3. **Occurrence-specific dates are computed** (relative to occurrence time)
   * Date fields (`due`, `scheduled`, `wait`) are computed relative to the occurrence timestamp
   * If template or seed has relative date expressions, they're evaluated at materialization time
   * Example: Seed has `due:+1d` → instance gets `due` = occurrence_date + 1 day

**Precedence rule summary:**
* **Template** = defaults/base values
* **Seed** = overrides template (more specific)
* **Computed dates** = evaluated at materialization time relative to occurrence

**Example:**
```
Template "standup":
  project:work
  +meeting
  alloc:30m
  due:+1d

Seed task:
  template:standup
  recur:daily
  description:Daily standup
  due:+0d  # Override template's +1d

Instance generated for 2026-01-15:
  project:work          # from template
  +meeting              # from template
  alloc:30m             # from template
  description:Daily standup  # from seed (overrides template if it had one)
  due:2026-01-15        # from seed (+0d evaluated at occurrence time)
  (no recur field)       # instances never have recur
```

See Section 7.4 for the recurrence rule grammar specification.

### 7.4 Recurrence Rule Grammar

Recurrence rules specify how often a task should repeat. Rules are parsed from the `recur:` field value.

**Grammar:**
```
recur_rule = frequency [modifiers]
frequency = simple_freq | interval_freq
simple_freq = 'daily' | 'weekly' | 'monthly' | 'yearly'
interval_freq = 'every:' number unit
number = [0-9]+
unit = 'd' | 'w' | 'm' | 'y'  # day, week, month, year
modifiers = modifier [modifiers]
modifier = weekday_mod | day_mod
weekday_mod = 'byweekday:' weekday_list
weekday_list = weekday [',' weekday_list]
weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun'
day_mod = 'bymonthday:' day_list
day_list = day [',' day_list]
day = [1-9] | [1-2][0-9] | '3' [0-1]  # 1-31
```

**Rules:**
* Simple frequencies: `daily`, `weekly`, `monthly`, `yearly` - repeat every day/week/month/year
* Interval frequencies: `every:Nd`, `every:Nw`, `every:Nm`, `every:Ny` - repeat every N days/weeks/months/years
* Weekday modifier (`byweekday:`): Only for `weekly` or `every:Nw` - restrict to specific weekdays
* Day-of-month modifier (`bymonthday:`): Only for `monthly` or `every:Nm` - restrict to specific days of month
* Modifiers are space-separated or comma-separated
* Case-insensitive for keywords (`daily`, `weekly`, etc.) and weekdays

**Examples:**

Simple frequencies:
* `daily` - every day
* `weekly` - every week (same day of week)
* `monthly` - every month (same day of month)
* `yearly` - every year (same month and day)

Interval frequencies:
* `every:2d` - every 2 days
* `every:3w` - every 3 weeks
* `every:2m` - every 2 months
* `every:1y` - every year (equivalent to `yearly`)

Weekly with weekday filter:
* `weekly byweekday:mon,wed,fri` - every Monday, Wednesday, Friday
* `every:2w byweekday:tue,thu` - every 2 weeks on Tuesday and Thursday

Monthly with day filter:
* `monthly bymonthday:1` - first day of every month
* `monthly bymonthday:15,30` - 15th and 30th of every month
* `every:3m bymonthday:1` - first day of every 3 months

**Validation:**
* `number` in `every:N<unit>` must be > 0
* `byweekday:` modifier only valid with weekly frequencies
* `bymonthday:` modifier only valid with monthly frequencies
* Day values in `bymonthday:` must be 1-31 (validation against actual month length happens at generation time)

**MVP Constraint:**
* Support all simple frequencies and interval frequencies
* Support `byweekday:` for weekly patterns
* Support `bymonthday:` for monthly patterns
* More advanced patterns (e.g., "last Friday of month", "first Monday", "every 2nd Tuesday") deferred to future versions

---

## 8. Output Contract (Human + Optional JSON)

### 8.1 Human readable defaults

* `list` outputs a table with stable columns.
* `stack show` outputs indices 0..n-1.
* `clock` commands output explicit transitions.

### 8.2 JSON option

* `--json` supported on list-like commands:

  * `task <filter> list --json`
  * `task stack show --json`
  * `task projects list --json`

No guarantee of JSON schema stability until v1.0; still, keys should be obvious and consistent.

---

## 9. Database DDL (MVP + Recurrence)

SQLite DDL sketch (finalize in migrations):

```sql
PRAGMA foreign_keys=ON;

CREATE TABLE projects (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  is_archived INTEGER NOT NULL DEFAULT 0,
  created_ts INTEGER NOT NULL,
  modified_ts INTEGER NOT NULL
);
-- Note: Nested projects use dot notation in the name field (e.g., 'admin.email', 'sales.northamerica.texas').
-- The hierarchy is implicit - no explicit parent-child relationships are stored.

CREATE TABLE tasks (
  id INTEGER PRIMARY KEY,
  uuid TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending','completed','deleted')),
  project_id INTEGER NULL REFERENCES projects(id),
  due_ts INTEGER NULL,
  scheduled_ts INTEGER NULL,
  wait_ts INTEGER NULL,
  alloc_secs INTEGER NULL,
  template TEXT NULL,
  recur TEXT NULL,
  udas_json TEXT NULL,  -- JSON object: {"key": "value", ...} - keys stored without "uda." prefix
  created_ts INTEGER NOT NULL,
  modified_ts INTEGER NOT NULL
);

CREATE TABLE task_tags (
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  PRIMARY KEY(task_id, tag)
);
CREATE INDEX idx_task_tags_tag ON task_tags(tag);

-- Task annotations (timestamped notes)
-- session_id links annotation to the session during which it was created (if applicable)
CREATE TABLE task_annotations (
  id INTEGER PRIMARY KEY,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  session_id INTEGER NULL REFERENCES sessions(id) ON DELETE SET NULL,
  note TEXT NOT NULL,
  entry_ts INTEGER NOT NULL,
  created_ts INTEGER NOT NULL
);
CREATE INDEX idx_task_annotations_task_entry ON task_annotations(task_id, entry_ts);
CREATE INDEX idx_task_annotations_session ON task_annotations(session_id);

-- Single stack
-- Note: The default stack (name='default') is auto-created on first stack operation.
-- No explicit initialization or migration is required.
CREATE TABLE stacks (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  created_ts INTEGER NOT NULL,
  modified_ts INTEGER NOT NULL
);

CREATE TABLE stack_items (
  stack_id INTEGER NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL,
  added_ts INTEGER NOT NULL,
  PRIMARY KEY(stack_id, task_id),
  UNIQUE(stack_id, ordinal)
);
CREATE INDEX idx_stack_items_stack_ordinal ON stack_items(stack_id, ordinal);

-- Sessions (exactly one running session)
-- Note: Session notes are handled via task annotations linked to the session
CREATE TABLE sessions (
  id INTEGER PRIMARY KEY,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  start_ts INTEGER NOT NULL,
  end_ts INTEGER NULL,
  created_ts INTEGER NOT NULL
);
CREATE INDEX idx_sessions_task_start ON sessions(task_id, start_ts);
CREATE INDEX idx_sessions_open ON sessions(end_ts) WHERE end_ts IS NULL;

-- Enforce single open session via partial unique index (SQLite supports this)
CREATE UNIQUE INDEX ux_sessions_single_open ON sessions(1) WHERE end_ts IS NULL;

-- Task events: immutable audit log of all task changes
-- Used for reconstructing task history, analysis, and audit trails
CREATE TABLE task_events (
  id INTEGER PRIMARY KEY,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  ts INTEGER NOT NULL,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL
);
CREATE INDEX idx_task_events_task_ts ON task_events(task_id, ts);
CREATE INDEX idx_task_events_type ON task_events(event_type);

CREATE TABLE templates (
  name TEXT PRIMARY KEY,
  payload_json TEXT NOT NULL,
  created_ts INTEGER NOT NULL,
  modified_ts INTEGER NOT NULL
);

CREATE TABLE recur_occurrences (
  seed_task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  occurrence_ts INTEGER NOT NULL,
  instance_task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  PRIMARY KEY(seed_task_id, occurrence_ts)
);
```

Notes:

* `ux_sessions_single_open` is a pragmatic enforcement of "one running session."
* `stack_items.ordinal` may be renumbered; rebalancing is acceptable.
* **Stack initialization:** The default stack (name='default') is auto-created on the first stack operation. No explicit bootstrap or migration step is required.

---

## 10. Transactions and Atomicity

All state-mutating commands MUST run in a single transaction.

Critical atomic operations:

* stack operation while running: close session + reorder + open new session
* `done --next`: close + complete + pop + open next
* `clock in task`: close running at effective start + move-to-top + open

If any step fails, rollback entirely.

---

## 11. Acceptance Tests (Given/When/Then)

### 11.1 Stack basics

**Scenario: stack auto-initialization on first operation**

* Given no stack exists (fresh database)
* And tasks 1,2,3 exist
* When `task stack show` (first stack operation)
* Then a default stack exists with name='default'
* And stack is empty `[]`

**Scenario: enqueue adds to end**

* Given tasks 1,2,3 exist
* And stack is empty
* When `task 1 enqueue`
* And `task 2 enqueue`
* And `task 3 enqueue`
* Then stack order is `[1,2,3]` (tasks added to end in order)

**Scenario: clock in pushes to top**

* Given tasks 1,2,3 exist
* And stack is `[1,2]`
* When `task 3 clock in`
* Then stack order is `[3,1,2]` (task 3 pushed to top)

**Scenario: stack pick moves to top**

* Given stack `[10,11,12]`
* When `task stack 2 pick`
* Then stack is `[12,10,11]`

**Scenario: stack roll rotates**

* Given stack `[10,11,12]`
* When `task stack roll 1`
* Then stack is `[11,12,10]` (roll syntax unchanged - n comes after roll)

### 11.2 Clock and stack coupling

**Scenario: stack roll while clock running switches live task**

* Given stack `[10,11]`
* And clock is running on task 10 since 09:00
* When `task stack roll 1` at 09:10
* Then session for task 10 ends at 09:10
* And a new session for task 11 starts at 09:10

**Scenario: stack pick while stopped does not create sessions**

* Given stack `[10,11,12]`
* And no running session
* When `task stack 2 pick`
* Then stack is `[12,10,11]`
* And no sessions are created

**Scenario: `task clock in` starts stack[0] at now**

* Given stack `[10,11]`
* And no running session
* When `task clock in` (no arguments) at 09:00
* Then a running session exists for task 10 starting 09:00 (defaults to now)

**Scenario: `task clock in` errors on empty stack**

* Given stack is empty
* When `task clock in`
* Then exit code is 1
* And message contains "No active task"

**Scenario: clock in with interval creates closed session**

* Given stack `[10]`
* And no running session
* When `task clock in 2026-01-10T09:00..2026-01-10T10:30`
* Then a closed session exists for task 10 from 09:00 to 10:30
* And no running session exists

**Scenario: interval end time amended on overlap**

* Given stack `[10,11]`
* And a closed session for task 10 from 09:00 to 10:30
* When `task 11 clock in 09:45` (starts before task 10's end time)
* Then task 10's session end time is amended to 09:45
* And task 10 session is from 09:00 to 09:45
* And a new session for task 11 starts at 09:45

### 11.3 Done semantics

**Scenario: done errors if not running**

* Given stack `[10]`
* And no running session
* When `task done`
* Then exit code is 1

**Scenario: done completes and removes from stack**

* Given stack `[10,11]`
* And clock running on 10 since 09:00
* When `task done` at 09:30
* Then session for 10 ends 09:30
* And task 10 status is completed
* And stack is `[11]`

**Scenario: done --next starts next**

* Given stack `[10,11]`
* And clock running on 10 since 09:00
* When `task done --next` at 09:30
* Then session for 10 ends 09:30
* And session for 11 starts 09:30
* And stack is `[11]`

### 11.4 Micro-session behavior

**Scenario: micro purge on rapid switch**

* Given stack `[10,11]` and clock running on 10
* When user rolls stack to 11 at 09:00:00
* And task 11 session ends at 09:00:20 (20s duration, micro-session)
* And task 10 session begins at 09:00:25 (within 30s of micro-session end)
* Then task 11 micro-session is purged (different task, within MICRO of end)
* And stdout indicates purge rule applied

**Scenario: micro merge on bounce back to same task**

* Given a task 11 session of 20s ends at 09:00:20
* And within 30s of the end time (at 09:00:25), a new session for task 11 begins
* Then the 20s session is merged into the later 11 session
* And stdout indicates merge rule applied

**Scenario: micro session preserved if no rule triggers**

* Given a task 11 session of 20s ends at 09:00:20
* And next session starts at 09:01:05 (45s after end, beyond MICRO)
* Then the 20s session remains (no rule triggered)

### 11.5 Tags and filters

**Scenario: tag add/remove**

* Given task 10 exists
* When `task 10 modify +urgent +home`
* And `task 10 modify -home`
* Then task 10 has tag `urgent` and does not have tag `home`

**Scenario: filter AND/OR/NOT**

* Given tasks: A has +urgent, B has +important, C has both
* When `task +urgent or +important list`
* Then results include A,B,C
* When `task not +urgent list`
* Then results exclude A and C

### 11.6 Scheduling and waiting

**Scenario: waiting derived**

* Given task 10 has wait set to tomorrow
* When `task waiting list`
* Then task 10 appears
* When time passes beyond wait
* Then task 10 no longer matches waiting

### 11.7 Recurrence

**Scenario: recur run is idempotent**

* Given a seed task S with `recur:weekly byweekday:mon` and template T
* When `task recur run --until <date>` is run twice
* Then the same set of instances exist after both runs (no duplicates)

### 11.8 Projects

**Scenario: project rename errors if target exists**

* Given project `work` exists
* And project `office` exists
* When `task projects rename work office`
* Then exit code is 1
* And message indicates project already exists

**Scenario: project rename with force merges projects**

* Given project `temp` exists with tasks 10, 11
* And project `work` exists with task 12
* When `task projects rename temp work --force`
* Then project `temp` no longer exists
* And tasks 10, 11, 12 all reference project `work`
* And project `work` still exists

**Scenario: project merge archive status handling**

* Given project `temp` (archived) exists with task 10
* And project `work` (active) exists with task 11
* When `task projects rename temp work --force`
* Then project `temp` no longer exists
* And tasks 10, 11 both reference project `work`
* And project `work` is active (merged from archived becomes active)

**Scenario: nested project filtering**

* Given projects `admin`, `admin.email`, `admin.other` exist
* And tasks: A has project `admin`, B has project `admin.email`, C has project `admin.other`, D has project `work`
* When `task project:admin list`
* Then results include A, B, C
* And results exclude D
* When `task project:admin.email list`
* Then results include only B

---

## 12. Build Checklist

Before coding starts, engineers should confirm:

* migrations apply cleanly
* all commands map to a single transaction
* invariant checks exist in both constraints and runtime assertions
* acceptance tests are runnable against temp databases

---

## 13. Open Items (Explicit)

* Parentheses in filter grammar (deferred)
* Advanced recurrence (e.g., end-of-month rules)
* Export/import formats
