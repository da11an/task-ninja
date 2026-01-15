# Plan 15 Implementation Notes

## Summary
Implemented monotonic (ordinal) sorting for categorical columns, negation prefix for descending sort, hide columns filter, and styling improvements.

## Changes Made

### 1. Ordinal Sort Functions
Added to `src/cli/output.rs`:
- `kanban_sort_order(kanban: &str) -> i64` - Maps kanban stages to ordinal values (proposed=0 through quit=7)
- `status_sort_order(status: &str) -> i64` - Maps task status to ordinal values (pending=0, completed=1, closed=2)

### 2. SortSpec with Negation Support
Added `SortSpec` struct and `parse_sort_spec()` function to parse sort column specifications:
- `sort:kanban` → ascending (proposed → quit)
- `sort:-kanban` → descending (quit → proposed)
- Works for all columns including ID, due, priority, etc.

### 3. Hide Columns Filter
- Added `hide_columns: Vec<String>` to `TaskListOptions`
- Added `hide:col1,col2,...` parsing to `parse_list_request()`
- Columns in hide list are filtered out before rendering

Example: `task list hide:kanban,clock,priority`

### 4. Status Column Added to Default
- `TaskListColumn::Status` now included in `default_columns` array
- Status appears after ID and Description

### 5. Group Header Styling
- Group headers now display as `[label]` only (no trailing dashes)
- Multiple group values separated by `:` (e.g., `[proposed:dogcare]`)

## Files Modified
1. `src/cli/output.rs`
   - Added `SortSpec` struct and `parse_sort_spec()` function
   - Added `kanban_sort_order()` and `status_sort_order()` functions
   - Updated sort value extraction to use ordinal values for Kanban and Status
   - Updated sorting logic to use `SortSpec` with descending support
   - Added Status to default columns
   - Added hide_columns filtering
   - Updated group header to remove dashes

2. `src/cli/commands.rs`
   - Added `hide_columns` to `ListRequest` struct
   - Added `hide:` parsing to `parse_list_request()`
   - Added `hide_columns` to `TaskListOptions` construction

## Testing Commands
```bash
# Ordinal kanban sorting (ascending)
task list sort:kanban

# Negated kanban sorting (descending)
task list sort:-kanban

# Hide columns
task list hide:kanban,clock,priority

# Group without divider lines
task list group:kanban

# Combined
task list sort:-kanban,-id hide:clock group:status
```

## Ordinal Mappings

### Kanban Stages
| Stage | Ordinal | Meaning |
|-------|---------|---------|
| proposed | 0 | Not started, not on stack |
| queued | 1 | On stack, no sessions yet |
| paused | 2 | Has sessions but off stack |
| working | 3 | On stack, has sessions |
| NEXT | 4 | Top of stack (or pos 1 when LIVE) |
| LIVE | 5 | Actively clocked in |
| done | 6 | Completed |
| quit | 7 | Closed |

### Task Status
| Status | Ordinal |
|--------|---------|
| pending | 0 |
| completed | 1 |
| closed | 2 |

## Not Implemented
- View storage for hide_columns (would require database migration)
- Sessions list sorting updates (left for future work)
