# Design Decisions Log

This document captures design decisions made during implementation that clarify, extend, or modify the original design specification.

**Last Updated:** 2025-01-XX

---

## Decision Format

Each decision entry includes:
- **Date:** When the decision was made
- **Issue:** What ambiguity or question was being resolved
- **Decision:** What was decided
- **Rationale:** Why this decision was made
- **Impact:** What parts of the codebase/design are affected

---

## Decisions

### DD-001: Template vs Class Terminology
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Inconsistency between `--class`, `template:<name>`, and `template:<class>` terminology  
**Decision:** Consolidate on "template" terminology throughout. "Class" and "template" refer to the same concept. Use "template" consistently:
- CLI flag: `--template <name>`
- Field token: `template:<name>`
- Database table: `templates` (renamed from `classes`)
- Global subcommand: `templates` (renamed from `classes`)
**Rationale:** "Template" is more descriptive and consistent with the field token syntax. Single terminology reduces confusion.  
**Impact:** CLI grammar, database schema, recurrence system - all updated to use "template" terminology

---

### DD-002: Clock Interval Syntax
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Undefined interval syntax for `clock in` command  
**Decision:** 
- Interval syntax: `<start_expr>..<end_expr>` where both are date expressions (Section 3.5)
- If interval provided, session is created as closed with both start and end times
- Examples: `2026-01-10T09:00..2026-01-10T10:30`, `today..eod`, `09:00..11:00`
- **Overlap prevention:** If another task session starts before the end time of a closed interval session, the end time is automatically amended to the start time of the new session to prevent overlap
- Applies to both `task clock in` and `task <id|filter> clock in` commands
**Rationale:** Explicit interval syntax allows users to record past time blocks. Overlap prevention ensures data integrity and prevents conflicting time records.  
**Impact:** Clock command implementation - must handle interval parsing, closed session creation, and automatic end time amendment on overlap

---

### DD-003: Stack Initialization
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** How is the default stack created?  
**Decision:** The default stack is **auto-created** on the first stack operation. The default stack has `name='default'`. No explicit initialization or migration step is required. All stack operations (show, set, pick, roll, drop, clear) will automatically create the default stack if it doesn't exist. The stack is created with `created_ts` and `modified_ts` set to the current time.  
**Rationale:** Auto-creation provides the best user experience - no setup required, works immediately. Lazy initialization is simpler than requiring an explicit bootstrap step.  
**Impact:** Database migrations (no migration needed), stack operations (must check for stack existence and create if missing), implementation (all stack commands need initialization check)

---

### DD-004: Micro-Session Timing Semantics
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Clarify "within MICRO seconds" timing for merge/purge rules  
**Decision:** 
- MICRO = 30 seconds (initially, configurable in future)
- **Both rules** (merge and purge) are evaluated relative to the **micro-session's end time**
- Rule 1 (Merge): If within MICRO seconds of the micro-session's end time, a session for the same task begins, merge the micro-session
- Rule 2 (Purge): If within MICRO seconds of the micro-session's end time, a session for a different task begins, purge the micro-session
- A subsequent session that begins within MICRO seconds after the micro-session ends will trigger the appropriate rule
**Rationale:** Consistent timing reference point (end time) for both rules simplifies implementation and makes behavior predictable. Using end time makes sense because we're looking at what happens after the micro-session completes.  
**Impact:** Session management logic - must track micro-session end times and check subsequent session start times relative to those end times

---

### DD-005: Duration Format Specification
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Duration format only had examples, no formal grammar or validation rules  
**Decision:** 
- Format: `duration = unit_spec+` where `unit_spec = digits unit`
- Units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days)
- Ordering: units must appear largest to smallest: `d`, `h`, `m`, `s`
- No spaces between units (e.g., `1h30m`, not `1h 30m`)
- Each unit type may appear at most once
- Examples: `30s`, `10m`, `2h`, `1h30m`, `2d5h30m`, `90m` (valid but `1h30m` preferred)
**Rationale:** Largest-to-smallest ordering is intuitive and matches common time notation. No spaces keeps parsing simple and format compact. Single occurrence per unit prevents ambiguity.  
**Impact:** Duration parsing implementation - must validate format, enforce ordering, and parse into total seconds

---

### DD-006: Recurrence Rule Grammar
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Recurrence rule format undefined - only examples mentioned  
**Decision:** 
- Formal grammar defined with BNF notation
- Simple frequencies: `daily`, `weekly`, `monthly`, `yearly`
- Interval frequencies: `every:Nd`, `every:Nw`, `every:Nm`, `every:Ny` where N > 0
- Modifiers:
  - `byweekday:mon,tue,wed,thu,fri,sat,sun` (comma-separated) - only for weekly patterns
  - `bymonthday:1,2,...,31` (comma-separated) - only for monthly patterns
- Modifiers are space or comma-separated
- Case-insensitive for keywords and weekdays
- Examples: `weekly byweekday:mon,wed,fri`, `monthly bymonthday:1`, `every:2w byweekday:tue,thu`
**Rationale:** Covers common business recurrence needs (daily standups, weekly meetings, monthly reports, etc.). Grammar is simple enough for MVP but extensible. Weekday and day-of-month filters handle most common patterns. More advanced patterns (last Friday, first Monday) deferred to keep MVP focused.  
**Impact:** Recurrence rule parsing and validation - must parse grammar, validate modifier compatibility, and generate occurrences based on rules

---

### DD-007: Nested Projects Support
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Specification didn't mention support for hierarchical/nested projects like Taskwarrior  
**Decision:** 
- Support nested projects using dot notation (e.g., `admin.email`, `sales.northamerica.texas`)
- Hierarchy is implicit in the project name - no explicit parent-child relationships stored in database
- Project filtering uses prefix matching: `project:admin` matches `admin`, `admin.email`, `admin.other`, etc.
- `project:admin.email` matches only `admin.email` and nested projects like `admin.email.inbox`
- Renaming a parent project does NOT automatically rename nested projects
- Database schema unchanged - just store full dotted name in `name` field
**Rationale:** Matches Taskwarrior's approach which is familiar to users. Simple implementation - no complex parent-child relationships to manage. Prefix matching provides intuitive filtering behavior.  
**Impact:** Project name parsing (must handle dots), project filtering (prefix matching logic), project commands (examples updated to show nested projects)

---

### DD-008: Project Merge on Rename
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Should renaming to an existing project name be allowed?  
**Decision:** 
- Renaming to an existing name is allowed with `--force` flag
- Without `--force`: errors if new name already exists (safe default)
- With `--force`: merges projects:
  - Moves all tasks from old project to new project
  - Deletes the old project
  - Archive status: if old is archived and new is active, merged becomes active; if old is active and new is archived, merged remains archived
- Command: `task projects rename <old_name> <new_name> [--force]`
**Rationale:** Allows consolidating projects safely. Force flag prevents accidental merges. Archive status handling ensures merged project state is predictable.  
**Impact:** Project rename implementation - must handle merge logic, task migration, project deletion, archive status resolution

---

### DD-009: Remove `--` Delimiter and Add Multi-Task Confirmation
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** `--` delimiter requirement and behavior when modifying multiple tasks  
**Decision:** 
- **Removed `--` delimiter requirement** from `task add` and `task modify`
- Description parsing is automatic: first sequence of tokens that don't match field patterns, tag patterns, or flags
- Field tokens, tag tokens, and flags can appear anywhere in argument list
- **Multi-task modification confirmation:**
  - If filter matches multiple tasks, alert user with count
  - User confirmation options: `y`/`yes` (all), `n`/`no` (skip all), `i`/`interactive` (one-by-one)
  - `--yes` flag: apply to all without confirmation
  - `--interactive` flag: force one-by-one confirmation
- Command: `task <id|filter> modify [<description...>] [field:value ...] [+tag ...] [-tag ...]`
**Rationale:** Removing `--` simplifies command syntax while still allowing unambiguous parsing. Multi-task confirmation prevents accidental bulk modifications and gives user control.  
**Impact:** Command parsing (must handle description extraction without delimiter), modify command (must implement confirmation flow), user interaction (must support interactive prompts)

---

### DD-010: Stack Command Syntax Simplification
**Date:** 2025-01-XX  
**Status:** Decided  
**Issue:** Stack command syntax was inconsistent and allowed filtering which added complexity  
**Decision:** 
- **Removed stack filtering** - only single task ID allowed (no filters)
- **Simplified syntax** - position/index comes after `stack` and before the command for consistency
- Commands:
  - `task stack show` - show stack
  - `task <id> stack set <index>` - add task to stack at position (task ID first, then position)
  - `task stack <index> pick` - move task at position to top (position before command)
  - `task stack roll [n]` - rotate stack (n after roll, defaults to 1)
  - `task stack <index> drop` - remove task at position (position before command)
  - `task stack clear` - clear all
- **Removed `stack:any` filter** - no longer needed
- Rule: Task ID comes after `task`, stack position comes after `stack` and before command (except `set` where position comes after command)
**Rationale:** Consistent syntax makes commands easier to learn and remember. Removing filtering simplifies implementation and reduces ambiguity. Position before command matches pattern of other commands.  
**Impact:** Stack command parsing (must distinguish task ID vs position), filter system (remove stack:any), command syntax (update all stack command examples and tests)

---

### DD-005: Date Parsing Library
**Date:** TBD  
**Status:** Pending  
**Issue:** Which Rust library to use for date parsing?  
**Decision:** TBD  
**Rationale:** TBD  
**Impact:** Date handling throughout codebase

---

### DD-006: Database Location
**Date:** TBD  
**Status:** Pending  
**Issue:** Where should the SQLite database be stored?  
**Decision:** TBD  
**Rationale:** TBD  
**Impact:** Configuration, initialization

---

### DD-007: Error Message Format
**Date:** TBD  
**Status:** Pending  
**Issue:** Standard format for error messages  
**Decision:** TBD  
**Rationale:** TBD  
**Impact:** All command implementations

---

### DD-008: Filter Multiple Match Behavior
**Date:** TBD  
**Status:** Pending  
**Issue:** What happens when filter matches multiple tasks for modify/done?  
**Decision:** TBD  
**Rationale:** TBD  
**Impact:** Modify and done command implementations

---

## Template for New Decisions

```markdown
### DD-XXX: [Short Title]
**Date:** YYYY-MM-DD  
**Status:** Pending | Decided | Superseded  
**Issue:** [What ambiguity or question]  
**Decision:** [What was decided]  
**Rationale:** [Why this decision]  
**Impact:** [What parts affected]  
**References:** [Links to related issues/PRs]
```

---

## Superseded Decisions

Decisions that were later changed or reversed are moved here with explanation.

---

## Notes

- Decisions should be made before or during implementation
- Update status as decisions are finalized
- Reference this document in code comments where decisions affect implementation
- Review decisions periodically for consistency
