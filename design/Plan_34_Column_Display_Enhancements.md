# Plan 34: Column Display Enhancements

## Overview

This plan addresses three related improvements to the `tatl list` output:

1. **Adjusted column hiding priority** - Reorder which columns are hidden first when the terminal is too narrow
2. **Q column indicators** - Add kanban stage indicators (P, D, E) to the Q column for non-queued tasks
3. **Dynamic column sizing** - Ensure column widths are recalculated on every invocation

---

## 1. Column Hiding Priority

### Current Order (first hidden → last hidden)

| Priority | Column(s)       | Behavior       |
|----------|-----------------|----------------|
| 8        | Alloc, Clock    | Hidden first   |
| 7        | Tags            |                |
| 6        | Priority        |                |
| 5        | Due             |                |
| 4        | Status, Kanban  |                |
| 3        | Project         | Truncated only |
| 2        | Description     | Truncated only |
| 1        | ID, Queue       | Never hidden   |

### Proposed Order (first hidden → last hidden)

| Priority | Column(s)   | Behavior       |
|----------|-------------|----------------|
| 10       | Status      | Hidden first   |
| 9        | Tags        |                |
| 8        | Priority    |                |
| 7        | Alloc       |                |
| 6        | Clock       |                |
| 5        | Kanban      |                |
| 4        | Due         | Hidden last    |
| 3        | Project     | Truncated only |
| 2        | Description | Truncated only |
| 1        | ID, Queue   | Never hidden   |

### Rationale

- **Status** hidden first: Kanban provides similar information; if both are shown, Status is redundant
- **Tags** hidden early: Often empty or less critical for quick scanning
- **Priority** hidden before Alloc/Clock: Priority is derived and can be inferred from other context
- **Due** hidden last (before truncation): Due dates are critical for task prioritization
- **Kanban** preserved longer: Provides workflow context that helps with task management

### Implementation

Update `column_priority()` function in `src/cli/output.rs`:

```rust
fn column_priority(column: TaskListColumn) -> u8 {
    match column {
        TaskListColumn::Id => 1,
        TaskListColumn::Queue => 1,
        TaskListColumn::Description => 2,
        TaskListColumn::Project => 3,
        TaskListColumn::Due => 4,
        TaskListColumn::Kanban => 5,
        TaskListColumn::Clock => 6,
        TaskListColumn::Alloc => 7,
        TaskListColumn::Priority => 8,
        TaskListColumn::Status => 9,
        TaskListColumn::Tags => 10,
    }
}
```

---

## 2. Q Column Kanban Indicators

### Current Behavior

The Q column currently shows:
- Numeric position (1, 2, 3, ...) for queued tasks
- `▶` for the active/live task (position 0)
- Empty for non-queued tasks

### Proposed Enhancement

Add kanban/status indicators for non-queued tasks using symbols:

| Kanban/Status    | Q Column Display | Notes                          |
|------------------|------------------|--------------------------------|
| `queued`         | 1, 2, 3, ...     | Numeric position (unchanged)   |
| `queued` (live)  | ▶                | Active task (unchanged)        |
| `proposed`       | ?                | Not yet started                |
| `stalled`        | !                | Has sessions but not in queue  |
| `external`       | @                | Awaiting external response     |
| `completed`      | ✓                | Completed (status)             |
| `closed`         | x                | Closed (status)                |

### Display Priority

When a task could have multiple indicators (e.g., external + stalled), use this priority:

1. Numeric position or ▶ (if in queue)
2. @ (if has active externals)
3. ✓ (if completed status)
4. x (if closed status)
5. ! (if stalled)
6. ? (if proposed)

### Implementation

Update `format_task_list_table()` in `src/cli/output.rs` where the Q column value is calculated:

```rust
// In the row building section where Q column is populated
let q_value = if let Some(pos) = stack_position {
    if pos == 0 && has_open_session {
        "▶".to_string()
    } else {
        pos.to_string()
    }
} else {
    // Not in queue - show status/kanban indicator
    // Priority: external > completed > closed > stalled > proposed
    if kanban_status == "external" {
        "@".to_string()
    } else if task_status == "completed" {
        "✓".to_string()
    } else if task_status == "closed" {
        "x".to_string()
    } else if kanban_status == "stalled" {
        "!".to_string()
    } else if kanban_status == "proposed" {
        "?".to_string()
    } else {
        String::new()
    }
};
```

### Example Output

```
ID   Q  Description                Status    Kanban   Project
──── ── ────────────────────────── ───────── ──────── ──────────
12   ▶  Active task                pending   queued   work
15   1  Next in queue              pending   queued   work
18   2  After that                 pending   queued   work
22   ?  New idea                   pending   proposed personal
25   !  Was working on this        pending   stalled  work
30   @  Waiting for review         pending   external work
35   ✓  Finished yesterday         completed done     home
36   x  Cancelled task             closed    done     home
```

---

## 3. Dynamic Column Sizing

### Current Behavior

The adaptive width logic runs on each invocation. However, there may be concerns about:
- Cached state affecting column visibility
- Interaction with manual `--hide` options

### Verification Points

1. **No cached state**: Confirm `column_widths` is calculated fresh in `format_task_list_table()`
2. **Manual hide respected**: Ensure `options.hide_columns` is applied before adaptive width logic
3. **Width detection**: Confirm `get_terminal_width()` is called each time

### Current Implementation Review

```rust
// In format_task_list_table():

// 1. Manual hide is applied first
let hidden_columns: Vec<TaskListColumn> = options.hide_columns.iter()
    .filter_map(|name| parse_task_column(name))
    .collect();
columns.retain(|col| !hidden_columns.contains(col));

// 2. Column widths calculated fresh
let mut column_widths: HashMap<TaskListColumn, usize> = HashMap::new();
// ... calculates based on actual data

// 3. Adaptive width runs if not full_width mode
if !options.full_width {
    let terminal_width = get_terminal_width();  // Fresh detection
    // ... hiding/truncation logic
}
```

### Confirmed Behavior

The current implementation already:
- ✅ Respects manual `--hide` options (applied before adaptive logic)
- ✅ Calculates column widths fresh on each invocation
- ✅ Detects terminal width on each run via `get_terminal_width()`

### Potential Issue

The `get_terminal_width()` function reads from `COLUMNS` environment variable, which:
- Is set by the shell when the terminal is resized
- May not update in all environments (e.g., piped output, some shells)

### Enhancement (Optional)

Consider adding `terminal_size` crate for more reliable detection:

```toml
[dependencies]
terminal_size = "0.2"
```

```rust
pub fn get_terminal_width() -> usize {
    // Try terminal_size crate first (more reliable)
    if let Some((terminal_size::Width(w), _)) = terminal_size::terminal_size() {
        return w as usize;
    }
    
    // Fallback to COLUMNS env var
    if let Ok(cols) = std::env::var("COLUMNS") {
        if let Ok(width) = cols.parse::<usize>() {
            if width > 0 && width < 10000 {
                return width;
            }
        }
    }
    
    // Default fallback
    120
}
```

**Decision Point**: Is the `terminal_size` crate worth adding as a dependency for more reliable width detection? Yes

---

## Implementation Checklist

### Phase 1: Column Priority Reorder
- [x] Update `column_priority()` function with new priority values
- [x] Update comments to reflect new order
- [x] Test at various terminal widths

### Phase 2: Q Column Indicators
- [x] Modify Q column value calculation in `format_task_list_table()`
- [x] Add indicator logic for ?, !, @, ✓, x
- [x] Ensure priority order (queue position > @ > ✓ > x > ! > ?)
- [x] Q column minimum width unchanged (single char fits)

### Phase 3: Dynamic Sizing Verification
- [x] Verify no cached state between invocations
- [x] Test manual `--hide` with adaptive width
- [x] Add `terminal_size` crate for reliable detection

### Testing
- [x] Test column hiding at widths: 60, 80, 100, 120, 160
- [x] Test Q column indicators for each kanban stage
- [x] Test combination of `--hide` with adaptive width
- [x] Test after terminal resize

## Implementation Complete

All phases implemented successfully on 2026-01-25.

---

## Decisions Made

1. **Q Column Width**: No change needed - single character indicators fit within existing width.

2. **Indicator Styling**: Use symbols instead of letters:
   - `?` = Proposed
   - `!` = Stalled
   - `@` = External
   - `✓` = Completed (Status)
   - `x` = Closed (Status)
   - `▶` = Active/live (queue position 0)
   - `1`, `2`, `3`... = Queue position

3. **`terminal_size` Dependency**: Added for reliable terminal width detection.
