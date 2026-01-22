# Plan 25: Queue Position Column

**Status: ✅ Complete**

## Executive Summary

Add a "Q" column to the task list showing each task's position in the queue. This reinforces TATL's queue-centric workflow and helps users quickly see where tasks stand in their work order.

## Problem Statement

Currently, users can see if a task is in the queue through:
1. The Kanban status (queued, NEXT, LIVE)
2. Running `tatl list` which shows queue ordering

However, there's no direct indication of **which position** a task occupies. A user might want to know "Task 15 is 3rd in my queue" without counting rows.

## Proposed Solution

Add a new column "Q" (for Queue) that displays:
- The ordinal position (0, 1, 2, ...) if the task is in the queue
- Empty/blank if the task is not in the queue

### Example Output

```
ID  Q   Kanban   Description              Project    Due
──  ─   ──────   ───────────────────────  ─────────  ──────────
5       LIVE     Fix auth bug             work       today
8   1   NEXT     Review PR                work       tomorrow
12  2   queued   Write docs               docs       
3   3   queued   Update dependencies      infra      +3d
7       paused   Research caching         work       
15      proposed New feature request      work       
```

In this example:
- Task 5 is position 0 (LIVE, actively being worked on)
- Task 8 is position 1 (NEXT up)
- Tasks 12 and 3 are in positions 2 and 3
- Tasks 7 and 15 are not in the queue

**Note:** Position 0 shows as blank in the Q column because LIVE/NEXT already indicates "current task". The Q column is most useful for seeing where queued tasks fall in line.

## Design Options

### Option A: Always Show Position (Recommended)

Show the ordinal for all queued tasks including position 0:

```
ID  Q   Kanban   Description
──  ─   ──────   ───────────
5   0   LIVE     Fix auth bug
8   1   NEXT     Review PR
12  2   queued   Write docs
```

**Pros:**
- Consistent - every queued task shows its position
- No ambiguity
- Useful for commands like `tatl pick 2`

**Cons:**
- Slight redundancy with LIVE/NEXT status

### Option B: Hide Position 0

Only show position for tasks beyond the top of the queue:

```
ID  Q   Kanban   Description
──  ─   ──────   ───────────
5       LIVE     Fix auth bug
8   1   NEXT     Review PR
12  2   queued   Write docs
```

**Pros:**
- Less visual noise at the top
- LIVE already tells you it's the active task

**Cons:**
- Inconsistent - some queued tasks show position, some don't
- May confuse users about whether position 0 exists

### Option C: Use 1-Based Indexing

Show positions as 1, 2, 3... instead of 0, 1, 2...:

```
ID  Q   Kanban   Description
──  ─   ──────   ───────────
5   1   LIVE     Fix auth bug
8   2   NEXT     Review PR
12  3   queued   Write docs
```

**Pros:**
- More natural for non-programmers ("1st in line")

**Cons:**
- Inconsistent with CLI commands (`tatl pick 0` vs display showing "1")
- Could cause confusion and off-by-one errors

## Recommendation

**Option A: Always show 0-indexed position**

Rationale:
1. **Consistency with CLI**: Commands like `pick`, `roll`, `drop` use 0-based indexing
2. **No ambiguity**: Every queued task shows its position
3. **Compact**: Single digit in most cases
4. **Reinforces mental model**: "Queue position matches what I'd type in a command"

**Decision:** go with option a.

## Implementation Plan

### Phase 1: Add Queue Position to Task List

**Files to modify:**

1. `src/cli/output.rs` - Add Q column to output formatting
2. `src/cli/output.rs` - Modify `format_task_list_table` to include position

**Changes:**

```rust
// In TaskListOptions or similar, add queue_positions map
// Key: task_id, Value: ordinal position

// In format_task_list_table:
// - Add "Q" column header
// - For each task, look up position from map
// - Display position or empty string
```

### Phase 2: Column Configuration (Optional)

Allow users to show/hide the Q column:
- Add to column configuration system
- Default: shown

### Implementation Details

#### Data Flow

1. When listing tasks, fetch stack items: `StackRepo::get_items()`
2. Build a HashMap: `task_id → ordinal`
3. Pass this map to the formatting function
4. For each task row, look up and display position

#### Column Width

- Header: "Q" (1 character)
- Values: 0-9 for most cases, 10+ for large queues
- Suggested width: 2-3 characters, right-aligned

#### Sorting Considerations

- When sorted by queue position, tasks not in queue should appear last
- Default sort could be: queue position (if present), then by ID or other criteria

## Edge Cases

1. **Empty queue**: Q column shows all blanks
2. **Large queue (10+ items)**: Column width expands as needed
3. **Task in multiple stacks**: Currently only one "default" stack exists; if multiple stacks are added later, would need to specify which

## Testing

1. Test task list with empty queue
2. Test task list with 1 task in queue
3. Test task list with multiple tasks in queue
4. Verify position matches `tatl pick N` indexing
5. Test filtering (e.g., `tatl list project:work`) - positions should still be accurate

## Success Criteria

1. ✅ Q column appears in task list output
2. ✅ Position shows for all queued tasks
3. ✅ Position is blank for non-queued tasks
4. ✅ Position matches CLI command indexing (0-based)
5. ✅ Column width is appropriate
6. ✅ No performance regression for large task lists

## Implementation Notes

### Changes Made

1. **`src/cli/output.rs`**:
   - Added `Queue` variant to `TaskListColumn` enum
   - Updated `parse_task_column()` to recognize "q" and "queue"
   - Updated `column_label()` to return "Q"
   - Added queue position to task row values HashMap
   - Added queue position to sort_values (tasks not in queue sort to end)
   - Updated default columns to include Queue after ID
   - Changed separator line from continuous dashes to per-column underlines with gaps (using Unicode box-drawing character `─`)

2. **`tests/stack_clock_tests.rs`**:
   - Removed obsolete tests for non-existent `next` and `pick` commands
   - Added `test_queue_column_shows_position` to verify Q column functionality
   - Added `test_switch_task_with_on_command` to test new workflow

### Table Styling

The separator line now uses per-column underlines with gaps between columns:

```
ID  Q   Kanban    Description                   Project    Due
──  ─   ────────  ────────────────────────────  ─────────  ─────────
```

This creates a cleaner visual separation matching the column structure.

## Open Questions

### Question 1: Should Q column be shown by default?

**Options:**
- A. Always shown (like ID)
- B. Shown by default, can be hidden
- C. Hidden by default, can be shown

**Recommendation:** A - Always shown. The queue is central to TATL's workflow.

### Question 2: Position for LIVE task?

When a session is running, should the LIVE task show:
- A. Its queue position (0)
- B. A special indicator (e.g., "▶" or "*")
- C. Just the number like any other

**Recommendation:** C - Just show 0. Keep it simple and consistent.

**Decision:** Use "▶" indicator instead of 0 when live.

### Question 3: Column placement?

Where should Q appear in the column order?

**Options:**
- A. After ID: `ID Q Kanban Description...`
- B. Before Kanban: `ID Q Kanban Description...` (same as A)
- C. After Kanban: `ID Kanban Q Description...`

**Recommendation:** A/B - After ID, before Kanban. This groups "identity" columns (ID, position) together.

## Appendix: Current Column Structure

```
ID   Kanban    Description                     Project    Due        Logged    Alloc   Tags
───  ────────  ──────────────────────────────  ─────────  ─────────  ────────  ──────  ─────────
```

After adding Q:

```
ID  Q   Kanban    Description                   Project    Due        Logged    Alloc   Tags
──  ─   ────────  ────────────────────────────  ─────────  ─────────  ────────  ──────  ─────────
```
