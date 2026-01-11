# Design Review: Issues, Corrections, and Recommendations

**Review Date:** 2025-01-XX  
**Document Reviewed:** `Plan_01_Build_Team_Handoff_Package.md`

## Summary

This document captures issues, ambiguities, corrections, and recommendations identified during the review of the Build Team Handoff Package. Items are categorized by severity and type.

---

## 🔴 Critical Issues (Must Fix Before Implementation)

### 1. Template/Class Terminology Inconsistency

**Location:** Sections 3.2, 3.3, 7.1

**Status:** ✅ **RESOLVED** - Consolidated on "template" terminology

**Issue:**
- Section 3.2 uses `--class <name>` flag
- Section 3.3 lists `template:<name>` as a field token
- Section 7.1 mentions `template:<class>` (optional)
- Database schema has `tasks.template` field and `classes` table

**Resolution:**
- "Class" and "template" are the same concept
- Standardized on "template" terminology throughout
- Updated grammar: `--template <name>` flag and `template:<name>` field token (both supported)
- Database table renamed from `classes` to `templates`
- All references updated in Plan_01_Build_Team_Handoff_Package.md

**Action:** ✅ Completed

---

### 2. Clock Interval Syntax Undefined

**Location:** Section 5.4

**Status:** ✅ **RESOLVED** - Interval syntax defined with overlap prevention

**Issue:**
- `task clock in [<start>|<start..end>]` mentions interval syntax `start..end`
- No definition of how this interval syntax works
- Unclear if this creates a closed session immediately or what behavior is expected

**Resolution:**
- Interval syntax defined: `<start_expr>..<end_expr>` where both are date expressions
- Behavior clarified: if interval provided, session is created as closed with both start and end times
- Examples added: `2026-01-10T09:00..2026-01-10T10:30`, `today..eod`, `09:00..11:00`
- **Overlap prevention rule added:** If another task session starts before the end time of a closed interval session, the end time is automatically amended to the start time of the new session to prevent overlap
- Applied to both `task clock in` and `task <id|filter> clock in` commands

**Action:** ✅ Completed

---

### 3. Stack Initialization Missing

**Location:** Section 9 (DDL), Section 5.1

**Status:** ✅ **RESOLVED** - Auto-creation on first stack operation

**Issue:**
- DDL creates `stacks` table but doesn't specify how the single stack is initialized
- No migration or bootstrap logic defined
- Stack operations assume a stack exists

**Resolution:**
- Default stack is **auto-created** on the first stack operation
- Default stack has `name='default'`
- No explicit initialization or migration step required
- All stack operations (show, set, pick, roll, drop, clear) automatically create the default stack if it doesn't exist
- Stack is created with `created_ts` and `modified_ts` set to current time
- Documented in Section 5.1.1 (Stack Initialization) and Section 9 (Database DDL notes)

**Action:** ✅ Completed

---

### 4. Micro-Session Merge/Purge Timing Ambiguity

**Location:** Section 6

**Status:** ✅ **RESOLVED** - Both rules relative to micro-session end time

**Issue:**
- Rule 1: "if within `MICRO` seconds a session for the **same task** begins"
- Rule 2: "if the subsequent session is for a **different task** and begins within `MICRO` seconds of the micro-session start"
- Unclear: "within MICRO seconds" of what? The micro-session end time or start time?
- Implementation note says "whichever is simpler" but semantics must match

**Resolution:**
- **MICRO = 30 seconds** (initially, configurable in future)
- **Both rules** are now relative to the **micro-session's end time**
- Rule 1 (Merge): "if within `MICRO` seconds **of the micro-session's end time**, a session for the **same task** begins"
- Rule 2 (Purge): "if within `MICRO` seconds **of the micro-session's end time**, a session for a **different task** begins"
- Added explicit timing clarification in Section 6
- Updated acceptance tests with specific timestamps to demonstrate the behavior

**Action:** ✅ Completed

---

## 🟡 Important Issues (Should Fix)

### 5. Filter Grammar Typo

**Location:** Section 4.1, Line 154

**Status:** ✅ **RESOLVED** - Formatting corrected

**Issue:**
- Line 154 had leading space: ` task +urgent or +important list`
- Inconsistent formatting

**Resolution:**
- Removed leading space from filter examples
- All examples now consistently formatted without leading spaces
- Verified all filter examples in Section 4.1 are properly formatted

**Action:** ✅ Completed

---

### 6. Date Expression Parsing Ambiguity

**Location:** Section 3.5

**Status:** ✅ **RESOLVED** - Time-only resolution with 24-hour window rule

**Issue:**
- "Time-only resolves to the **nearest instance** relative to now"
- Ambiguous: nearest in past or future? Or always future?

**Resolution:**
- Time-only expressions use a 24-hour window: 8 hours in the past, 16 hours in the future
- Within this window, if the future instance is no more than twice as far away as the past instance, resolve to the future instance
- Otherwise, resolve to the nearest instance (past or future)
- Added multiple examples demonstrating the behavior:
  - `9am` at 08:00 → today 9am (future chosen)
  - `9am` at 10:00 → today 9am (past chosen, as future is >2× past distance)
  - Additional examples for various scenarios

**Action:** ✅ Completed

---

### 7. Duration Format Specification Incomplete

**Location:** Section 3.6

**Status:** ✅ **RESOLVED** - Duration format fully specified

**Issue:**
- Only examples provided: `30s`, `10m`, `2h`, `1h30m`
- No formal grammar or validation rules
- Unclear if `1h30m` is required format or if `90m` is also valid

**Resolution:**
- Defined formal grammar: `duration = unit_spec+` where `unit_spec = digits unit`
- Units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days)
- Ordering: units must appear largest to smallest: `d`, `h`, `m`, `s`
- No spaces: concatenated format (e.g., `1h30m`, not `1h 30m`)
- Each unit type may appear at most once
- Added valid and invalid examples
- Clarified that `90m` is valid but `1h30m` is preferred for readability

**Action:** ✅ Completed

---

### 8. Recurrence Rule Format Undefined

**Location:** Section 7.3

**Status:** ✅ **RESOLVED** - Recurrence rule grammar fully specified

**Issue:**
- Mentions `daily`, `weekly`, `monthly`, `every:<n><unit>`, `byweekday:`
- No formal grammar or complete examples
- Unclear how to combine: `weekly byweekday:mon,wed,fri`?

**Resolution:**
- Defined formal grammar with BNF notation
- Simple frequencies: `daily`, `weekly`, `monthly`, `yearly`
- Interval frequencies: `every:Nd`, `every:Nw`, `every:Nm`, `every:Ny`
- Modifiers:
  - `byweekday:mon,tue,wed,thu,fri,sat,sun` - for weekly patterns
  - `bymonthday:1,2,...,31` - for monthly patterns
- Modifiers are space or comma-separated
- Case-insensitive for keywords and weekdays
- Added comprehensive examples for all patterns
- Specified validation rules (number > 0, modifier compatibility)
- Documented MVP constraints and future enhancements

**Action:** ✅ Completed

---

### 9. Project CRUD Commands Missing

**Location:** Section 1 (MVP Scope), Section 3 (CLI Grammar)

**Issue:**
- MVP scope lists "Projects: master data CRUD (add/list/rename/archive)"
- No CLI grammar defined for project commands
- Section 3.1 mentions `projects` as global subcommand but no details

**Status:** ✅ **RESOLVED** - Project command grammar fully specified

**Recommendation:**
- Add grammar for:
  - `task projects add <name>`
  - `task projects list [--archived]`
  - `task projects rename <old> <new>`
  - `task projects archive <name>`
- Add to acceptance tests

**Resolution:**
- Added Section 3.7 with complete project command grammar:
  - `task projects add <name>` - create new project
  - `task projects list [--archived]` - list projects (active or archived)
  - `task projects rename <old_name> <new_name>` - rename project
  - `task projects archive <name>` - archive project
  - `task projects unarchive <name>` - unarchive project (bonus)
- Each command includes:
  - Syntax specification
  - Behavior description
  - Error conditions
  - Examples
- Commands support `--json` output where applicable
- Project names must be unique
- Archived projects excluded from default list

**Action:** ✅ Completed

---

### 10. Modify Command Grammar Missing

**Location:** Section 1 (MVP Scope), Section 3 (CLI Grammar)

**Status:** ✅ **RESOLVED** - Modify command grammar added with multi-task confirmation

**Issue:**
- MVP scope lists "Tasks: add/modify/list/done"
- Section 3.1 mentions `modify` as task subcommand
- No grammar defined for `modify` command

**Resolution:**
- Added Section 3.3 with complete modify command grammar: `task <id|filter> modify [<description...>] [field:value ...] [+tag ...] [-tag ...]`
- **Removed `--` delimiter requirement** - description parsing is automatic (same as `task add`)
- **Multiple task modification with confirmation:**
  - If filter matches multiple tasks, user is alerted with count
  - User can confirm: `y`/`yes` (all), `n`/`no` (skip all), `i`/`interactive` (one-by-one)
  - `--yes` flag: applies to all without confirmation
  - `--interactive` flag: forces one-by-one confirmation
- Description is optional - if provided, replaces existing description
- Examples added showing single task, filter-based, and confirmation scenarios

**Action:** ✅ Completed

---

### 11. Stack Commands Ambiguity

**Location:** Section 5.3

**Status:** ✅ **RESOLVED** - Stack syntax simplified and clarified

**Issue:**
- `task <id|filter> stack set <index>` - what if filter matches multiple tasks?
- `task stack drop <index|id>` - mixing index and id is confusing
- Unclear behavior when dropping by id: does it remove from stack or just remove the item?

**Resolution:**
- **Removed stack filtering** - only single task ID allowed (no filters)
- **Simplified syntax** - position comes after `stack` and before command for consistency:
  - `task <id> stack set <index>` - add task to stack (task ID first, then position)
  - `task stack <index> pick` - move task at position to top (position before command)
  - `task stack <index> drop` - remove task at position (position before command)
  - `task stack roll [n]` - rotate (n after roll, defaults to 1)
- **Removed `stack:any` filter** - no longer needed
- Rule: Task ID comes after `task`, stack position comes after `stack` and before command
- Added examples and clarified indexing rules

**Action:** ✅ Completed

---

### 12. UDA Storage Format

**Location:** Section 9 (DDL), Section 3.3

**Status:** ✅ **RESOLVED** - UDA storage format specified (Taskwarrior-style)

**Issue:**
- `tasks.udas_json TEXT NULL` - JSON format not specified
- No validation rules
- Unclear if keys must match `uda.<key>` pattern or stored differently

**Resolution:**
- JSON schema: `{"key1": "value1", "key2": "value2", ...}`
- Key format: Keys stored **without** `uda.` prefix (e.g., `uda.priority:high` → `{"priority": "high"}`)
- Key validation: Keys must match tag charset `[A-Za-z0-9_\-\.]+` (same as tags)
- Value format: Any string (no restrictions)
- Clearing: Use `uda.<key>:none` to remove a UDA
- Multiple UDAs: Can have multiple UDAs per task
- Examples added showing usage
- Database comment added to schema

**Action:** ✅ Completed

---

### 13. Task Events Table Purpose Undefined

**Location:** Section 9 (DDL)

**Status:** ✅ **RESOLVED** - Task events documented as immutable history/audit log

**Issue:**
- `task_events` table defined but never mentioned in functional spec
- Purpose unclear
- No examples of what events are recorded

**Resolution:**
- **Purpose:** Immutable audit log of all task changes
- **Use cases:** Reconstructing task history, analysis, reporting, audit trails, debugging
- **Event types documented:**
  - `created`, `modified`, `status_changed`
  - `tag_added`, `tag_removed`
  - `annotation_added`, `annotation_deleted`
  - `stack_added`, `stack_removed`
  - `session_started`, `session_ended`
- **Event structure:** Each event has `task_id`, `ts`, `event_type`, and `payload_json`
- **Immutability:** Events are never modified or deleted
- **Automatic recording:** Events recorded automatically by system (no user commands)
- Added Section 5.7 with complete documentation
- Added indexes for efficient querying
- Added payload examples for each event type

**Action:** ✅ Completed

---

### 14. Sessions Table Note Field

**Location:** Section 9 (DDL)

**Status:** ✅ **RESOLVED** - Note field removed, annotations linked to sessions instead

**Issue:**
- `sessions.note TEXT NULL` - no mention in functional spec
- Unclear if notes are supported in MVP

**Resolution:**
- **Removed `note` field from sessions table**
- **Annotations now link to sessions:** Added `session_id` to `task_annotations` table (nullable)
- **Improved annotate command:** `task annotate` (without ID) works when clocked in, automatically associates with current task and session
- **Session-linked annotations:** If annotation is created during a running session, it's linked via `session_id`
- This provides better functionality - annotations can be tied to specific work sessions
- Added index on `session_id` for efficient querying
- Updated documentation to explain session linking

**Action:** ✅ Completed

---

## 🟢 Minor Issues / Recommendations

### 15. Recurrence Attribute Precedence Unclear

**Location:** Section 7.3

**Status:** ✅ **RESOLVED** - Precedence clarified with common sense approach and examples

**Issue:**
- Precedence listed as: 1) Template, 2) Seed task, 3) Materialization-time computed dates
- Unclear what happens when both template and seed have same field
- "Materialization-time computed dates" - what does this mean?

**Resolution:**
- **Clarified precedence with common sense approach:**
  1. Template provides base/default attributes (starting point)
  2. Seed task attributes override template (more specific wins)
  3. Occurrence-specific dates computed relative to occurrence time
- **Rule:** Seed overrides template (not the other way around) - more specific wins
- **Computed dates:** Date fields (`due`, `scheduled`, `wait`) with relative expressions are evaluated at materialization time relative to occurrence timestamp
- **Added comprehensive example** showing:
  - Template with defaults (project, tags, alloc, due)
  - Seed task overriding template (description, due date)
  - How instance is created with combined attributes
  - How relative dates are computed
- **Summary rule:** Template = defaults, Seed = overrides, Computed dates = evaluated at occurrence time

**Action:** ✅ Completed

---

### 16. Filter Precedence Examples Needed

**Location:** Section 4.1

**Status:** ✅ **RESOLVED** - Complex filter examples added

**Issue:**
- Precedence rules stated but no complex examples
- Unclear how `not` interacts with `or`: `not +urgent or +important`

**Resolution:**
- Added comprehensive complex filter examples:
  - Multiple AND conditions
  - OR with AND combinations
  - NOT with AND
  - Complex NOT with OR
  - Multiple OR conditions
  - Combining status with other filters
- Clarified evaluation order: `not` first, then implicit `and`, then `or`
- Examples demonstrate precedence in action
- Clarified: `not +urgent or +important` = `(not +urgent) or +important`

**Action:** ✅ Completed

---

### 17. Stack Roll Default Behavior

**Location:** Section 5.3

**Status:** ✅ **RESOLVED** - Default roll behavior clarified

**Issue:**
- `task stack roll [n]` - what is default `n` if omitted?
- Common pattern: `roll` = roll 1, `roll 2` = roll 2

**Resolution:**
- Specified: `n` defaults to 1 if omitted (rotate once)
- Clarified rotation behavior: `[a,b,c]` with `roll 1` becomes `[b,c,a]`
- Documented that negative `n` rotates in reverse direction
- Updated command documentation in Section 5.3

**Action:** ✅ Completed

---

### 18. Done Command Filter Behavior

**Location:** Section 5.5

**Status:** ✅ **RESOLVED** - Done command supports filtering with confirmation

**Issue:**
- `task <id|filter> done` - what if filter matches multiple tasks?
- Should it error or complete all?

**Resolution:**
- Updated to: `task [<id|filter>] done` - supports filtering
- **Multiple task completion with confirmation:**
  - If filter matches multiple tasks, user is alerted with count
  - User can confirm: `y`/`yes` (all), `n`/`no` (skip all), `i`/`interactive` (one-by-one)
  - `--yes` flag: completes all without confirmation
  - `--interactive` flag: forces one-by-one confirmation
- Same confirmation pattern as `modify` command for consistency
- Important note: Only tasks with running sessions can be completed via `task done` (without ID/filter)

**Action:** ✅ Completed

---

### 19. Database Migration Strategy

**Location:** Section 9, Section 12

**Issue:**
- Mentions "finalize in migrations" but no migration strategy defined
- No versioning scheme
- No rollback strategy

**Recommendation:**
- Define migration approach (e.g., numbered SQL files, Rust migration crate)
- Specify version tracking mechanism
- Add to build checklist

**Action:** Add migration strategy section.

---

### 20. Error Message Standards

**Location:** Throughout

**Status:** ✅ **RESOLVED** - Error message format standardized

**Issue:**
- Exit codes defined but no error message format specified
- Acceptance tests reference messages but no standard

**Resolution:**
- Added Section 2.1.1 with error message format standards:
  - User errors (exit code 1): "Error: " prefix, messages to stderr
  - Internal errors (exit code >1): "Internal error: " prefix, messages to stderr
  - Success messages and normal output go to stdout
  - Error messages should be clear, actionable, and avoid technical jargon when possible
- Added examples showing proper format
- Consistent with common CLI conventions

**Action:** ✅ Completed

---

### 21. JSON Output Schema Examples

**Location:** Section 8.2

**Issue:**
- JSON output mentioned but no schema examples
- "keys should be obvious" is vague

**Recommendation:**
- Provide example JSON outputs for each command
- Define at least basic schema (even if unstable)

**Action:** Add JSON schema examples.

---

### 22. Timezone Edge Cases

**Location:** Section 2.2

**Status:** ✅ **RESOLVED** - DST handling fully specified

**Issue:**
- DST transitions can cause ambiguity
- What happens during "fall back" hour?

**Resolution:**
- **Enhanced Section 2.2 with DST handling specification:**
  - All timestamps stored as UTC epoch seconds eliminates DST ambiguity at storage time
  - Conversion from UTC to local time uses current system timezone rules
  - DST transitions handled automatically by system's timezone library
  - **Fall back hour (2am → 1am):** Use first occurrence (earlier timestamp)
  - **Spring forward hour (2am → 3am):** Skipped hour doesn't exist; times in that range are invalid and should error
  - Date parsing uses local timezone at parse time, then converts to UTC for storage
  - This approach ensures consistency: same UTC timestamp always represents the same absolute moment
- Documented that UTC storage + local display approach helps with DST handling

**Action:** ✅ Completed

---

## 📋 Missing Specifications

### 23. Templates CRUD

**Location:** Section 1 (MVP Scope mentions templates)

**Issue:**
- Templates mentioned but no CRUD commands defined
- How are templates created, listed, modified?

**Recommendation:**
- Add commands: `task templates add <name>`, `task templates list`, `task templates show <name>`
- Or integrate into task commands: `task add --template <name> -- ...`

**Action:** Define template management commands.

---

### 24. Sessions Command

**Location:** Section 3.1, Section 1

**Status:** ✅ **RESOLVED** - Sessions command grammar added with consistent syntax

**Issue:**
- `sessions` listed as global subcommand but no grammar defined
- Unclear what `task sessions` should do

**Resolution:**
- Added Section 3.8 with complete sessions command grammar:
  - `task [<id>] sessions list [--json]` - list session history
    - Without ID: lists all sessions (all tasks)
    - With ID: lists sessions for specified task only
    - Sessions ordered by start time (newest first)
    - Shows: task ID, description, start time, end time, duration
  - `task [<id>] sessions show` - show detailed session information
    - Without ID: shows current running session (if any)
    - With ID: shows most recent session for that task
    - Includes: task details, start/end times, duration, linked annotations
- **Consistent syntax:** Task ID comes after `task` and before `sessions` (matches other commands)
- Supports `--json` output format
- Examples added

**Action:** ✅ Completed

---

### 25. Recur Command Details

**Location:** Section 7.3

**Status:** ✅ **RESOLVED** - Recur command scope clarified (Taskwarrior core capabilities)

**Issue:**
- Only `task recur run` defined
- No other recur subcommands mentioned
- Unclear if `task recur list` or `task recur show <seed>` needed

**Resolution:**
- Added Section 7.3 "Recur command scope" clarifying MVP scope
- **MVP scope:** Only `recur run` is supported (matches Taskwarrior core capabilities)
- Future commands (`recur list`, `recur show`) explicitly deferred to future versions
- Documented that MVP focuses on core recurrence generation functionality
- Section 7.3.1 contains the `recur run` command details

**Action:** ✅ Completed

---

## 🔧 Implementation Recommendations

### 26. Database Connection Management

**Location:** Section 2.3

**Status:** ✅ **RESOLVED** - Database location and configuration specified

**Issue:**
- Database location not specified
- No configuration mechanism defined

**Resolution:**
- Added Section 2.3 "Database location and configuration":
  - **Default location:** `~/.taskninja/tasks.db`
  - **Configuration directory:** `~/.taskninja/`
  - **Configuration file:** `~/.taskninja/rc` (optional, key-value format)
  - **Override support:** `data.location` key in rc file can override database path
  - **Auto-creation:** Database and directory structure created automatically on first use
  - No explicit initialization command required
- Configuration file format: simple `key=value` (one per line)
- Relative paths resolved relative to configuration file's directory

**Action:** ✅ Completed

---

### 27. Transaction Isolation

**Location:** Section 10

**Issue:**
- Mentions transactions but not isolation level

**Recommendation:**
- Specify: SQLite default (SERIALIZABLE) is sufficient
- Document any explicit isolation needs

**Action:** Add isolation level specification.

---

### 28. Concurrent Access Handling

**Location:** Section 10

**Issue:**
- No mention of concurrent access (multiple processes)

**Recommendation:**
- Document: SQLite handles this via locking
- Specify error handling for lock timeouts
- Consider advisory locks for critical sections

**Action:** Add concurrency handling section.

---

### 29. Date Parsing Library

**Recommendation:**
- Recommend Rust date parsing library (e.g., `chrono` or `time`)
- Specify which relative date expressions are supported
- Define parsing error handling

**Action:** Add date parsing specification.

---

### 30. Testing Strategy

**Location:** Section 11

**Issue:**
- Acceptance tests defined but no unit/integration test strategy

**Recommendation:**
- Define test structure (unit, integration, acceptance)
- Specify test database setup/teardown
- Recommend test framework

**Action:** Add testing strategy section.

---

## 📝 Documentation Recommendations

### 31. Cross-Reference Missing Documents

**Location:** Lines 5-6

**Issue:**
- References "Task System Design & Syntax Specification" and "Rust + SQLite Engineering Plan"
- These documents not present in repo

**Recommendation:**
- Either include these documents or remove references
- Or create placeholder documents

**Action:** Resolve document references.

---

### 32. README Mismatch

**Location:** Project root README.md

**Issue:**
- README mentions Python/pip installation
- Design document specifies Rust + SQLite

**Recommendation:**
- Update README to match Rust implementation
- Or clarify if this is a rewrite/migration

**Action:** Update README to match design.

---

## ✅ Positive Aspects

The design document is generally well-structured and comprehensive. Good aspects:

- Clear MVP scope definition
- Well-defined stack/clock semantics
- Thoughtful micro-session policy
- Good acceptance test coverage
- Transaction atomicity considerations
- Timezone handling approach

---

## Next Steps

1. **Immediate:** Address all 🔴 Critical Issues
2. **Before Implementation:** Resolve 🟡 Important Issues
3. **During Implementation:** Track 🟢 Minor Issues and implement recommendations
4. **Ongoing:** Update design documents as decisions are made

---

## Document Maintenance

As we build and make decisions, we should:

1. Create `Design_Decisions.md` to capture implementation choices
2. Update this review document as issues are resolved
3. Create `API_Reference.md` once CLI is stable
4. Create `Database_Schema.md` with detailed schema documentation
