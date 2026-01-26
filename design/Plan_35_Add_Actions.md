# Plan 35: Extra Actions During Task Add

## Overview

This plan evaluates adding action flags to `tatl add` that would allow performing post-creation actions like finishing, closing, or sending tasks in a single command.

## Current State

### Current `tatl add` Flags

```
tatl add <description> [field:value...] [options]
```

**Options:**
- `--on[=TIME]` - Start timing immediately (pushes to queue[0], starts session)
- `--onoff=INTERVAL` - Add historical session (e.g., `09:00..12:00`)
- `--enqueue` - Add to end of queue without starting timing
- `-y, --yes` - Auto-confirm prompts

### Field:Value Syntax (Already Available)

The `field:value` syntax allows setting attributes during creation:
- `project:<name>` - Assign to project
- `due:<expr>` - Set due date
- `scheduled:<expr>` - Set scheduled date
- `wait:<expr>` - Set wait date
- `allocation:<dur>` - Set time allocation
- `template:<name>` - Use template
- `respawn:<pattern>` - Set respawn rule
- `uda.<key>:<value>` - Set user-defined attribute
- `+<tag>` / `-<tag>` - Add/remove tags

### Related Commands

| Command | Action | Respawn Triggered |
|---------|--------|-------------------|
| `tatl finish [target]` | Mark complete, stop session | Yes |
| `tatl close [target]` | Mark closed (cancelled/won't do) | Yes |
| `tatl send <id> <recipient> [request]` | Send to external party | No |

---

## Proposed Actions

### 1. `--finish` Flag

**Proposal:** Add `--finish` flag to create a task already marked as completed.

**Use Case:** Recording historical completed work (e.g., "I already did this yesterday").

**Syntax:**
```bash
tatl add "Fixed production bug" project:work +urgent --finish
tatl add "Fixed bug" --onoff 14:00..15:30 --finish  # Historical session + complete
```

**Behavior:**
1. Create task
2. If `--onoff` present: Add historical session
3. Mark task as completed (triggers respawn if applicable)

**Compatibility:**
- `--finish` + `--on`: Conflicting. `--on` starts timing, `--finish` ends it immediately. **Recommendation:** Error or warn, don't allow.
- `--finish` + `--onoff`: Compatible. Creates historical session then completes task.
- `--finish` + `--enqueue`: Conflicting. Can't enqueue a completed task. **Recommendation:** Error.

### 2. `--close` Flag

**Proposal:** Add `--close` flag to create a task already marked as closed.

**Use Case:** Recording tasks that were decided against or already closed.

**Syntax:**
```bash
tatl add "Cancelled feature request" project:work --close
```

**Behavior:**
1. Create task
2. Mark task as closed (triggers respawn if applicable)

**Compatibility:**
- `--close` + `--on`: Conflicting. **Recommendation:** Error.
- `--close` + `--onoff`: Questionable. Session on closed task? **Recommendation:** Error.
- `--close` + `--enqueue`: Conflicting. **Recommendation:** Error.
- `--close` + `--finish`: Conflicting. **Recommendation:** Error.

### 3. `--send <recipient>` Flag

**Proposal:** Add `--send <recipient>` flag to create a task and immediately send to external party.

**Use Case:** Creating tasks that are already waiting on external input.

**Syntax:**
```bash
tatl add "Review my PR" project:work --send colleague
tatl add "Needs supervisor approval" --send supervisor
```

**Behavior:**
1. Create task
2. Send to specified recipient (creates external record, removes from queue)

**Compatibility:**
- `--send` + `--on`: Conflicting. **Recommendation:** Error.
- `--send` + `--onoff`: Compatible? Could record time spent preparing before sending.
- `--send` + `--enqueue`: Conflicting. **Recommendation:** Error.
- `--send` + `--finish`: Conflicting. **Recommendation:** Error.
- `--send` + `--close`: Conflicting. **Recommendation:** Error.

---

## Analysis

### Can Any of This Be Done with `field:value` Syntax?

**No.** The `field:value` syntax sets *attributes*, not *statuses*. Status changes are intentionally command-based because they have side effects:
- Stopping active sessions
- Triggering respawn rules
- Removing from queue

Adding `status:completed` or `status:closed` as a settable field would bypass these side effects, which is why `status` is currently read-only.

### Precedence and Conflicts

If multiple action flags are provided, they conflict. Proposed resolution:

| Combination | Behavior |
|-------------|----------|
| `--on` + `--finish` | Error: Cannot start and finish simultaneously |
| `--on` + `--close` | Error: Cannot start and close simultaneously |
| `--on` + `--send` | Error: Cannot start and send simultaneously |
| `--onoff` + `--finish` | OK: Add session, then complete |
| `--onoff` + `--close` | OK: Add session, then close (to allow recording effort prior to close) |
| `--onoff` + `--send` | OK: Add session, then send |
| `--enqueue` + `--finish` | Error: Can't enqueue completed task |
| `--enqueue` + `--close` | Error: Can't enqueue closed task |
| `--enqueue` + `--send` | Error: Can't enqueue sent task |
| `--finish` + `--close` | Error: Mutually exclusive statuses |
| `--finish` + `--send` | Error: Can't send completed task |
| `--close` + `--send` | Error: Can't send closed task |

### Existing Workflow Comparison

**Creating and finishing a task (current):**
```bash
tatl add "Fixed bug" project:work
# Note the ID from output
tatl finish <id>
```

**With proposed `--finish`:**
```bash
tatl add "Fixed bug" project:work --finish
```

**Creating and sending (current):**
```bash
tatl add "Review PR" project:work
# Note the ID from output
tatl send <id> colleague "Please review"
```

**With proposed `--send`:**
```bash
tatl add "Review PR" project:work --send colleague
# Note: No request message in this form
```

---

## Open Questions

### 1. Is `--finish` Common Enough to Warrant a Flag?

**Consideration:** How often do users create tasks that are already complete?

Scenarios:
- Recording historical work for time tracking
- Importing tasks from another system
- Creating task + historical session with `--onoff`

**Decision Point:** [x] Yes, add `--finish` / [ ] No, use two-step workflow

### 2. Is `--close` Common Enough to Warrant a Flag?

**Consideration:** How often do users create tasks that are already closed?

Scenarios:
- Recording declined requests for audit trail
- Importing cancelled items from another system

**Decision Point:** [x] Yes, add `--close` / [ ] No, use two-step workflow

### 3. Should `--send` Accept a Request Message?

**Option A:** `--send <recipient>` (no message)
- Simpler
- Message can be added with `tatl annotate` afterward

**Option B:** `--send <recipient> <message>` (with message)
- Harder to parse with `trailing_var_arg`
- Potential confusion with description

**Option C:** `--send <recipient> --request "message"`
- Explicit but verbose
- Clearer separation

**Decision Point:** [ ] Option A / [ ] Option B / [x] Option C / [ ] Don't add `--send`

### 4. Should `--onoff` + `--finish` Auto-complete?

If a user creates a task with a historical session, should it auto-complete?

**Example:**
```bash
tatl add "Meeting" --onoff 14:00..15:00
# Currently: Task is pending, has historical session
# Proposal: Automatically complete since work is "done"?
```

**Decision Point:** [ ] Yes, auto-complete / [x] No, require explicit `--finish`

### 5. Action Flags vs. Separate Workflows

**Alternative:** Instead of adding flags, document the two-step workflow prominently.

```bash
# Create and finish:
tatl add "Task" && tatl finish $(tatl list --json | jq -r '.[-1].id')

# Or use shell alias:
alias tatl-add-done='tatl add "$@" && tatl finish $(tatl list --json | jq -r ".[-1].id")'
```

**Decision Point:** [x] Add action flags / [ ] Improve documentation / [ ] Both

---

## Recommendations

### Core Recommendations

1. **Add `--finish` flag** - The `--onoff` + finish workflow is common enough to warrant streamlining. Recording historical completed work is a valid use case.

2. **Add `--close` flag** - Lower priority, but consistent with `--finish`. Useful for audit trails.

3. **Skip `--send` flag for now** - The send workflow requires a recipient and often a request message, making it awkward as a flag. The two-step workflow (`add` then `send`) is clearer.

### Implementation Priority

| Flag | Priority | Rationale |
|------|----------|-----------|
| `--finish` | High | Enables one-command historical task+session logging |
| `--close` | Medium | Consistency, audit trails |
| `--send` | Skip | Complex args, rare use case |

### Flag Compatibility Matrix (Final)

| Flag 1 | Flag 2 | Result |
|--------|--------|--------|
| `--on` | `--finish` | Error |
| `--on` | `--close` | Error |
| `--onoff` | `--finish` | OK |
| `--onoff` | `--close` | OK |
| `--enqueue` | `--finish` | Error |
| `--enqueue` | `--close` | Error |
| `--finish` | `--close` | Error |

---

## Implementation Plan

**Decisions Made:**
- ✅ Add `--finish` flag
- ✅ Add `--close` flag  
- ❌ Skip `--send` flag (use two-step workflow)
- ✅ `--onoff` + `--close` is OK (record effort before close)
- ❌ No auto-complete for `--onoff` (require explicit `--finish`)

### Phase 1: Add `--finish` Flag

1. **CLI Definition** (`src/cli/commands.rs`)
   - Add `finish: bool` to `Add` struct
   - Add conflict validation in handler

2. **Handler Logic** (`handle_task_add`)
   - After task creation and optional session creation
   - Call task finish logic (set status, trigger respawn)

3. **Tests**
   - `tatl add "Task" --finish` → task created + completed
   - `tatl add "Task" --onoff 09:00..10:00 --finish` → session + completed
   - `tatl add "Task" --on --finish` → error
   - `tatl add "Task" --enqueue --finish` → error
   - Respawn triggered on finish

### Phase 2: Add `--close` Flag

1. **CLI Definition**
   - Add `close: bool` to `Add` struct
   - Add conflict validation

2. **Handler Logic**
   - Call task close logic (set status, trigger respawn)

3. **Tests**
   - `tatl add "Task" --close` → task created + closed
   - `tatl add "Task" --on --close` → error
   - `tatl add "Task" --finish --close` → error

### Phase 3: Documentation

- Update `tatl add --help` with new flags
- Update COMMAND_REFERENCE.md
- Update README.md examples

---

## Success Criteria

1. ✅ `tatl add "Task" --finish` creates completed task
2. ✅ `tatl add "Task" --onoff 09:00..10:00 --finish` creates task with session and completes
3. ✅ `tatl add "Task" --close` creates closed task
4. ✅ Conflicting flags produce clear error messages
5. ✅ Respawn rules triggered appropriately on finish/close
6. ✅ Documentation updated
7. ✅ All tests pass

---

## Appendix: Full Flag Interaction Table

| --on | --onoff | --enqueue | --finish | --close | Result |
|:----:|:-------:|:---------:|:--------:|:-------:|--------|
| - | - | - | - | - | Task created (pending) |
| ✓ | - | - | - | - | Task created, timing started |
| - | ✓ | - | - | - | Task created, historical session |
| - | - | ✓ | - | - | Task created, added to queue |
| - | - | - | ✓ | - | Task created, completed |
| - | - | - | - | ✓ | Task created, closed |
| ✓ | - | ✓ | - | - | `--on` takes precedence (per Plan_19) |
| - | ✓ | - | ✓ | - | Task + session, completed |
| ✓ | - | - | ✓ | - | **Error:** Cannot start and finish |
| ✓ | - | - | - | ✓ | **Error:** Cannot start and close |
| - | ✓ | - | - | ✓ | Task + session, closed |
| - | - | ✓ | ✓ | - | **Error:** Cannot enqueue completed |
| - | - | ✓ | - | ✓ | **Error:** Cannot enqueue closed |
| - | - | - | ✓ | ✓ | **Error:** Mutually exclusive |
