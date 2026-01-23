# Plan 28: Kanban System Updates

## Overview

This plan addresses several improvements to the kanban system:
1. Reorder and rename kanban stages for better workflow clarity
2. Simplify kanban by removing redundant stages (NEXT/LIVE)
3. Add multi-value filtering for kanban and status
4. Clarify handling of closed tasks
5. Add external review/workflow tracking

---

## 1. Reorder Kanban Stages and Rename "Paused" to "Stalled"

### Current State
- Kanban order: `proposed` → `queued` → `paused` → `NEXT` → `LIVE` → `done`
- "Paused" indicates a task that has sessions but is not in the queue

### Proposed Changes
- **Rename "paused" to "stalled"** - Better conveys that this is a state to avoid/recover from
- **Reorder to: `proposed` → `stalled` → `queued` → `done`**
  - This puts "stalled" earlier in the workflow, making it more visible as something to address
  - Logical flow: proposed (new) → stalled (needs attention) → queued (ready to work) → done (complete)

### Critique & Refinement
✅ **AGREED** - This makes semantic sense. "Stalled" is more descriptive and the reordering emphasizes it as a problem state.

**Implementation Notes:**
- Update `calculate_kanban_status()` in `src/cli/output.rs`
- Update `calculate_task_kanban()` in `src/filter/evaluator.rs`
- Update kanban sort order in `Plan_15_Monotonic_Sorting.md` ordinals
- Update all filter parsing/evaluation to recognize "stalled" instead of "paused"
- Update help text and documentation

**Migration:**
- No database migration needed (kanban is derived, not stored)
- Update any saved views/filters that reference "paused" → "stalled"

---

## 2. Drop NEXT and LIVE as Kanban Stages

### Current State
- `NEXT` = task at queue[0] but clock is not running
- `LIVE` = task at queue[0] and clock is running
- These were useful before the Q column existed

### Proposed Changes
- Remove `NEXT` and `LIVE` as distinct kanban stages
- Tasks at queue[0] should show as `queued` (with Q=0 or Q=▶ indicator)
- The Q column already shows queue position, making NEXT/LIVE redundant

### Critique & Refinement
✅ **AGREED** - The Q column provides better information:
- Q=0 or Q=▶ shows the active task
- Q=1,2,3... shows position in queue
- Clock status is already visible via the Clock column
- Simplifies the kanban model without losing information

**Edge Case Consideration:**
- What about position 1 when position 0 is LIVE? Currently it shows as NEXT.
- **Decision:** After removing NEXT/LIVE, position 1 should just be `queued` (Q=1)
- The distinction between "next up" and "actively working" is less important than queue position
- AGREED

**Implementation Notes:**
- Simplify `calculate_kanban_status()` logic:
  - Position 0 → `queued` (regardless of clock status)
  - Position > 0 → `queued`
  - Not in queue + has sessions → `stalled`
  - Not in queue + no sessions → `proposed`
- Update filter parsing to remove "next" and "live" as kanban values
- Update documentation

---

## 3. Multi-Value Filtering for Kanban and Status

### Current State
- Filters only accept single values: `kanban:queued`, `status:pending`
- Cannot filter for multiple values in one expression

### Proposed Changes
- Support comma-separated values: `kanban:queued,stalled`, `status:pending,closed`
- These should be treated as OR conditions (task matches if it has any of the values)

### Critique & Refinement
✅ **AGREED** - This is a useful enhancement. However, consider the syntax carefully:

**Syntax Options:**
1. **Comma-separated:** `kanban:queued,stalled` - Simple, intuitive
2. **Multiple filters:** `kanban:queued or kanban:stalled` - More explicit, uses existing OR syntax
3. **Both:** Support both for flexibility

**Recommendation:** Support comma-separated values as syntactic sugar for OR:
- `kanban:queued,stalled` → `kanban:queued or kanban:stalled`
- This is more concise and readable

**Implementation Notes:**
- Update `parse_filter_term()` in `src/filter/parser.rs` to split on commas
- Convert `kanban:queued,stalled` to `FilterExpr::Or([kanban:queued, kanban:stalled])`
- Apply same logic to `status:` filter
- Update help text to document this syntax
- Consider: Should this apply to all filters or just kanban/status?

**Edge Cases:**
- `kanban:queued,stalled or kanban:proposed` - How to parse?
  - **Decision:** Comma has higher precedence than OR
  - `kanban:queued,stalled or kanban:proposed` = `(queued OR stalled) OR proposed`
- `kanban:queued,stalled project:work` - Multiple filters still AND together
  - **Decision:** Comma only applies within a single filter term

---

## 4. Closed Tasks in Kanban View

### Current State
- Both `completed` and `closed` tasks show as kanban `done`
- No distinction in kanban view

### Question
Where should closed tasks appear in kanban view?

### Analysis & Recommendation

**Option A: Keep as "done"** (current behavior)
- Pros: Simple, both are terminal states
- Cons: Loses distinction between "completed" (success) and "closed" (cancelled/won't do)

**Option B: Separate kanban stage "closed"**
- Pros: Preserves distinction, useful for filtering
- Cons: Adds another stage, may clutter view

**Option C: Show as "done" but allow filtering by status**
- Pros: Clean kanban view, but can still filter `status:closed` when needed
- Cons: Requires using status filter instead of kanban filter

**Recommendation: Option C** - Keep kanban as "done" for both, but:
- Users can filter with `status:closed` or `status:completed` when distinction matters
- Multi-value filtering makes this easier: `status:completed,closed` or `kanban:done status:closed`
- Kanban represents workflow state, not lifecycle state
- "Done" is appropriate for both terminal states in workflow terms

**Alternative Consideration:**
- If there's strong need to distinguish in kanban view, could add "closed" as a separate stage
- But this should be driven by actual use cases, not theoretical purity

**Decision:** Option C.

---

## 5. External Review/Workflow Tracking

### Problem Statement
Current system doesn't handle workflows where:
- Task is "done" from your perspective, but waiting on external review/approval
- Task is blocked, but you've proactively engaged someone else (ball in their court)
- Task needs to wait for a release window or external event

These tasks shouldn't be lost, but they're not actively in your queue either.

### Proposed Solution: External Queue/Table

**Schema:**
```sql
CREATE TABLE externals (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    recipient TEXT NOT NULL,  -- e.g., "colleague", "supervisor", "Release_5.2", "Customer"
    request TEXT,              -- Optional: what was requested
    sent_ts INTEGER NOT NULL,  -- When sent
    returned_ts INTEGER NULL,   -- When returned (if applicable)
    created_ts INTEGER NOT NULL,
    modified_ts INTEGER NOT NULL
);
```

**Workflow:**
- `tatl send <task_id> <recipient> [request]` - Send task to external party
- Task shows as kanban "external" (or "stalled" with E notation in Q column)
- `tatl collect <task_id>` - Task returned, back to your control
- After collect, task can be: finished, closed, or re-queued

### Critique & Refinement

**Strengths:**
- Addresses real workflow need
- Keeps tasks visible but out of active queue
- Simple model (send/collect)

**Concerns & Questions:**

1. **Overlap with Projects?**
   - Recipients like "Release_5.2" could be projects
   - **Decision:** Recipients are ad-hoc strings, not structured. Projects are for organization, externals are for workflow state. They serve different purposes.

2. **Multiple Recipients?**
   - Can a task be sent to multiple recipients simultaneously?
   - **Recommendation:** Yes, support multiple externals per task. Each external is independent.

3. **Q Column Notation:**
   - Proposal: Show "E" in Q column for external tasks
   - **Refinement:** Could show "E" or "E1,E2" if multiple externals
   - Alternative: Show as kanban "external" stage

4. **Kanban Stage vs Queue Notation:**
   - Option A: New kanban stage "external"
   - Option B: Keep as "stalled" but show "E" in Q column
   - **Recommendation:** Option A - "external" is semantically different from "stalled"
   - Stalled = you're blocked, external = someone else is handling it
   - **Decision:** Option A

5. **Mutual Exclusivity:**
   - Proposal: Task cannot be in queue AND external simultaneously
   - **Agreement:** Makes sense - if it's external, it's not in your queue
   - Implementation: `send` should dequeue if task is in queue
   - `collect` should not auto-enqueue (user decides next action)

6. **Return Handling:**
   - What happens when `collect` is called?
   - **Recommendation:** Task returns to "proposed" or "stalled" (depending on sessions)
   - User explicitly decides: `tatl collect 5` then `tatl enqueue 5` or `tatl finish 5`
   - **Decision:** Yes, this recommmendation works, but cause enqueue, close, finish commands to automatically collect (this would be implied).

7. **Recipient Tracking:**
   - Should we track who has it? Or just that it's external?
   - **Recommendation:** Track recipient for clarity, but keep it simple (free text)
   - Useful for filtering: `external:colleague`, `external:Release_5.2`

8. **Filtering:**
   - Add `external:<recipient>` filter
   - Add `external:*` to show all external tasks
   - Kanban filter: `kanban:external`

### Refined Proposal

**Kanban Stages (Final):**
- `proposed` - New task, not started
- `stalled` - Has sessions but not in queue (needs attention)
- `queued` - In queue (any position, Q shows exact position)
- `external` - Sent to external party (not in your control)
- `done` - Completed or closed

**Commands:**
```bash
# Send task to external party
tatl send <task_id> <recipient> [request]

# List external tasks
tatl externals [filter]

# Collect task back
tatl collect <task_id>

# Filter external tasks
tatl list external:colleague
tatl list kanban:external
tatl list external:Release_5.2
```

**Q Column Display:**
- Normal queue: Q=0, Q=1, Q=2, etc.
- External: Q=E (don't bother noting multiples)
- Could also show recipient: Q=E(colleague) -- no, keep it simple

**Database Schema:**
```sql
CREATE TABLE externals (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    recipient TEXT NOT NULL,
    request TEXT NULL,
    sent_ts INTEGER NOT NULL,
    returned_ts INTEGER NULL,
    created_ts INTEGER NOT NULL,
    modified_ts INTEGER NOT NULL,
    UNIQUE(task_id, recipient)  -- Prevent duplicate sends to same recipient
);

CREATE INDEX idx_externals_task ON externals(task_id);
CREATE INDEX idx_externals_recipient ON externals(recipient);
CREATE INDEX idx_externals_returned ON externals(returned_ts) WHERE returned_ts IS NULL;
```

**Implementation Considerations:**
- `send` command should:
  - Remove task from queue if present
  - Create external record
  - Update kanban calculation
- `collect` command should:
  - Mark external as returned
  - Task returns to normal kanban flow (proposed/stalled)
  - Does NOT auto-enqueue (user decides)
- Other commands that logically exclude from task being external should mark external as returned and into queue or done
- Kanban calculation:
  - If task has any unreturned externals → `external`
  - Otherwise, use normal logic

**Alternative Simpler Approach:**
- Instead of separate table, could use UDA: `uda.external:colleague`
- **Rejection:** Externals need timestamps, return tracking, and are core workflow state
- Better as first-class feature

---

## Implementation Priority

### Phase 1: Simple Changes (High Priority)
1. ✅ Rename "paused" to "stalled"
2. ✅ Reorder kanban stages
3. ✅ Remove NEXT/LIVE stages
4. ✅ Multi-value filtering for kanban and status

### Phase 2: External Workflow (Medium Priority)
5. External queue/table implementation
6. Send/collect commands
7. External kanban stage
8. Filtering support

### Phase 3: Polish (Lower Priority)
9. Q column notation for externals
10. External reporting/analytics
11. Bulk operations (send multiple, collect multiple)

---

## Open Questions

1. **Should "external" be a kanban stage or just a Q notation?**
   - **Decision:** Kanban stage - it's a distinct workflow state

2. **Can a task have multiple externals simultaneously?**
   - **Decision:** Yes - e.g., "colleague review" AND "supervisor approval"

3. **What happens to external tasks when they're completed/closed?**
   - **Decision:** Mark all externals as returned automatically, task shows as "done"

4. **Should `collect` auto-enqueue?**
   - **Decision:** No - user explicitly decides next action

5. **Should externals support expiration/reminders?**
   - **Future consideration:** Could add `wait_ts` or reminder system later

---

## Success Criteria

1. ✅ Kanban stages are semantically clear and ordered logically
2. ✅ Q column provides all queue position information (no need for NEXT/LIVE)
3. ✅ Users can filter by multiple kanban/status values easily
4. ✅ External workflow is tracked and visible
5. ✅ Closed vs completed distinction is available when needed
6. ✅ System remains simple and doesn't add unnecessary complexity

---

## Next Steps

1. Review and approve this formalized plan
2. Implement Phase 1 changes (simple renames and removals)
3. Test multi-value filtering
4. Design external workflow in more detail if proceeding
5. Implement Phase 2 if external workflow is approved
