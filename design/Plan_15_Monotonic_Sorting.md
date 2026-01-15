# Plan 15: Monotonic (Ordinal) Sorting for Categorical Columns

## Goal
Make sorting on categorical columns respect their semantic/workflow order rather than alphabetical order. Support reverse sorting via `-column` prefix.

---

## 1. Define Ordinal Rankings

### Kanban stages (workflow progression, low → high)

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

### Task status (lifecycle progression)

| Status | Ordinal |
|--------|---------|
| pending | 0 |
| completed | 1 |
| closed | 2 |

---

## 2. Implement Ordinal Mapping Functions

Add to `src/cli/output.rs`:

```rust
fn kanban_sort_order(kanban: &str) -> i64 {
    match kanban.to_lowercase().as_str() {
        "proposed" => 0,
        "queued" => 1,
        "paused" => 2,
        "working" => 3,
        "next" => 4,
        "live" => 5,
        "done" => 6,
        "quit" => 7,
        _ => 99,
    }
}

fn status_sort_order(status: &str) -> i64 {
    match status.to_lowercase().as_str() {
        "pending" => 0,
        "completed" => 1,
        "closed" => 2,
        _ => 99,
    }
}
```

Update `sort_values` population for `Kanban` and `Status` columns to use `SortValue::Int(ordinal)` instead of `SortValue::Str(...)`.

---

## 3. Parse Negation Prefix

Modify sort column parsing to detect leading `-`:

```rust
struct SortSpec {
    column: String,
    descending: bool,
}

fn parse_sort_spec(spec: &str) -> SortSpec {
    if let Some(col) = spec.strip_prefix('-') {
        SortSpec { column: col.to_string(), descending: true }
    } else {
        SortSpec { column: spec.to_string(), descending: false }
    }
}
```

In the sort comparator, flip the `Ordering` when `descending` is true:

```rust
let ordering = compare_sort_values(...);
if sort_spec.descending {
    ordering.reverse()
} else {
    ordering
}
```

---

## 4. Files to Modify

| File | Changes |
|------|---------|
| `src/cli/output.rs` | Add ordinal functions, update sort value extraction, parse `-` prefix, apply reverse |
| `src/cli/commands_sessions.rs` | Same changes if session list supports sorting |

---

## 5. Tests

1. **Kanban ordinal sort**: `sort:kanban` → proposed, queued, paused, working, NEXT, LIVE, done, quit
2. **Status ordinal sort**: `sort:status` → pending, completed, closed
3. **Negation**: `sort:-id` → 10, 9, 8...
4. **Negated ordinal**: `sort:-kanban` → quit, done, LIVE, NEXT, working, paused, queued, proposed
5. **Combined**: `sort:kanban,-id` → kanban groups ascending, ids descending within groups

---

## 6. Estimate

- Implementation: ~30 min
- Testing: ~15 min

---

## 7. Risks

- Breaking change for users relying on current alphabetical sort behavior (low risk since you're the only user)
- Need to keep ordinal mappings in sync if new statuses/kanban stages are added

---

## 8. Open Questions

- Should priority column also support negation? (Currently numeric, so `-priority` would show lowest priority first): yes
- Should due date negation show furthest dates first or closest? earliest to latest if normal, latest to earliest if negated

## 9. Styling
style group in square brackets (as it currently is) but drop the grouping dividing lines.

## 10. Build `hide` filter, to hide columns from view

- `task list hide:kanban,due`

## 11. Include Status column by default, as now these can be hidden if desired.