# Plan 14 User Feedback Design Plan

## Goals
- Translate Plan 14 feedback into a concrete, testable design plan.
- Challenge dubious choices and identify implementation barriers.
- Suggest improvements where the feedback is ambiguous or risky.

## Non-Goals
- No implementation in this document.
- No behavior changes outside the listed feedback items.

## Assumptions
- CLI uses `clap` with subcommands for task/sessions/clock.
- Output formatting lives in `src/cli/output.rs`.
- Filters are parsed in `src/filter/*`.
- Session data exists and can be queried by task id.

## Open Questions
- Should “live” be user-facing terminology only, or a new status/state? LIVE is already a computed kanban stage. It shouldn't also be a regular status.
- Should `task done` be removed immediately or kept as a deprecated alias? Remove task done. Only keep finish.
- Should list view aliases be stored on disk or per-session only? I think they should be in a new database table. They need to be persistent.

## Design Summary
- Improve session UX (default yes + richer prompts).
- Add optional id inference for annotate when clocked in.
- Expand list sorting/grouping and introduce reusable view aliases.
- Allow unambiguous filter token abbreviations.
- Expand relative date parsing.
- Make modify accept status changes and handle unknown tokens safely.
- Update list headers and priority placement.
- Refine kanban mapping for LIVE/NEXT and align clock command naming.
- Introduce `closed` status and `finish/close` verbs.

---

## 1) `task annotate` optional id when clocked in
**Design**
- If a clocked-in task exists, `task annotate <note>` targets that task by default.
- If not clocked in, require explicit id.
- Add `--task <id>` to override inference.

**Challenge**
- Risk of ambiguity between a numeric note and an id. Prefer explicit `--task` over positional parsing when ambiguous.

**Implementation Barrier**
- Current annotate signature expects `<id> <note>`. Needs parsing that can distinguish note-only calls.

**Tests**
- Annotate uses live task when clocked in.
- Annotate without clock errors with clear message.
- `--task` overrides live task.

**Decision**
Default behavior is to parse assuming `task annotate <id> <note>`. (No change)
If <id> fails to match a valid task id AND a task is clocked in, attach annotation to clocked in task.
Error if no match and not clocked in.

---

## 3) Default yes for `task sessions modify`
**Design**
- Keep confirmation but default to yes: `Are you sure? ([y]/n):`
- Empty input proceeds.

**Challenge**
- “Trust by default” is good UX but increases risk. Consider a config flag: `confirm.default_yes = true`.

**Implementation Barrier**
- Prompt utility might be shared; ensure default does not leak to other commands.

**Tests**
- Empty input applies changes.
- `n` aborts.

**Decision**
- OK if default leaks. We'll deal with that if it happens.

---

## 4) Add task info to session modify prompt
**Design**
- Prompt includes task id and description if available: `Modify session 9 (task 7: Group meeting)?`
- If task missing, fall back to session-only prompt.

**Implementation Barrier**
- Requires fetching the task by id while modifying a session.

**Tests**
- Task info shown when available.
- Missing task handled gracefully.

---

## 5a) `task list` sorting + grouping
**Design**
- Accept `sort:colA,colB` and `group:Kanban,tag` tokens, where colA, colB are arbitrary columns or fields, as are Kanban and tag.
- Sorting columns appear first, then `ID`, `Description`, then group columns.
- Group headers appear as divider lines per group value.

**Challenge**
- Output format changes can break scripts. Consider gating under `--format table-v2` or `--layout grouped`.

**Implementation Barrier**
- Table renderer needs column reordering and group header rows.

**Tests**
- Single and multi-column sort order.
- Grouped output contains headers.

---

## 5b) Aliases for custom list views
**Design**
- Allow saving sort/filter/group as a named view.
- Suggested syntax: `task list <args> --add-alias myview` or `task list <args> alias:myview`.
- Usage: `task list myview`.

**Challenge**
- Storage location unclear. Prefer a simple config file (`~/.config/task-ninja/views.toml`).

**Implementation Barrier**
- Need serialization for views and validation when invoked.

**Tests**
- Save a view, then use it.
- Invalid view name errors with suggestions.

---

## 5c) Extend list views to `task sessions list`
**Design**
- Reuse the same alias system for sessions list.
- Allow sort/group tokens relevant to sessions (start/end/task/elapsed).

**Implementation Barrier**
- Sessions list output may need separate column map for sorting.

**Tests**
- Sessions view alias works.

---

## 6) Token abbreviation in filtering
**Design**
- Allow unambiguous prefixes for filter tokens.
- If prefix matches multiple tokens, error with suggestions.

**Challenge**
- Prefix rules can change as new tokens are added. Maintain a registry and keep ambiguity errors clear.

**Implementation Barrier**
- Parser currently expects exact tokens; needs prefix lookup.

**Tests**
- `st:pending` resolves to `status`.
- Ambiguous prefix yields error with choices.

---

## 7) Relative due date parsing
**Design**
- Support `1week`, `2weeks`, `1w`, `+1w`, `in 1 week`, `tomorrow`, `next week`.
- Apply same parser to `due:` and `start:`.

**Challenge**
- Natural-language parsing is fuzzy. Limit to a documented subset.

**Implementation Barrier**
- Existing date parser likely expects absolute dates. Extend safely without breaking current inputs.

**Tests**
- `due:1week` parses.
- Absolute date still parses.

---

DO NOT IMPLEMENT 8 AT THIS TIME
## 8) `done` as subset of `modify` and `status:` in modify
**Design**
- `task done <id>` becomes `task modify <id> status:completed`.
- Allow `status:<value>` in modify if value is in the known set.
- For unknown `token:value`:
  - Interactive: prompt to include in description or cancel.
  - Non-interactive: error out with suggestion.

**Challenge**
- Prompting can break scripts; detect TTY and fail fast in non-interactive mode.

**Implementation Barrier**
- Need a unified token parser to decide what becomes description vs recognized fields.

**Tests**
- `modify <id> status:completed` updates status.
- Unknown token prompts in TTY, errors without TTY.

---

## 9) Allow new project entry in modify
**Design**
- If `project:<name>` doesn’t exist, prompt to create like `add`.
- Provide `--yes` for non-interactive auto-create.

**Implementation Barrier**
- Share project-creation logic between add/modify.

**Tests**
- Modify with new project prompts and creates.
- `--yes` auto-creates.

---

## 10) `alloc` -> `Alloc` in list header
**Design**
- Title-case the header to `Alloc`.

**Challenge**
- This reverses the previous shortening. Confirm desired casing for all headers to keep output consistent.
- Title case should be the default for all headers.

**Tests**
- Header is `Alloc`.

---

DO NOT IMPLEMENT 11 AT THIS TIME.
## 11) Priority header/location
**Design**
- Rename to `Prior` and place before `Alloc`.

**Challenge**
- `Prior` is ambiguous. Suggest `Prio` or `Pri` for clarity if space is tight.

**Tests**
- Column order matches spec.

---

## 12) `task status` includes description in clock status
**Design**
- Show `Clock: <task id> <description>` in status output.

**Implementation Barrier**
- Need to fetch task description for the live task.

**Tests**
- Status includes description when clocked in.

---

## 13) Kanban mapping for LIVE/NEXT
**Design**
- Use the provided mapping; explicitly handle:
  - Position 0 + clock In => LIVE
  - Position 1 + clock In => NEXT (if pos 0 is LIVE)
  - Position 0 + clock Out => NEXT
- Ensure only one task is NEXT at a time.

**Challenge**
- Mapping uses both sessions list and clock status; must define precedence when data conflicts.

**Implementation Barrier**
- Current kanban calculation may not consider clock direction for positions 0/1.

**Tests**
- LIVE/NEXT mapping verified for position 0/1 with clock in/out.

---

## 14) Rename `task clock roll` → `task clock next`
**Design**
- Add `clock next` as new primary command.
- Do NOT keep `clock roll` as deprecated alias.

**Implementation Barrier**
- Update abbreviations, docs, and tests.

**Tests**
- `clock next` works.

---

## 15) Close vs Complete statuses
**Design**
- Add `closed` status.
- Add `task close <id>` to set `closed`.
- Rename `task done` to `task finish`.

**Challenge**
- Be harsh. I'm currently the only user. Don't deprecate done, remove it.

**Implementation Barrier**
- Update status enum, filters, reports, dashboard, and any migrations.

**Tests**
- `close` sets closed status.
- `finish` sets completed.
- `done` no longer exists.
- Filters support `status:closed`.

---

## Suggested Implementation Order
1) Session UX improvements (3–4).
2) Annotate default to live session (1).
3) Filter token abbreviations (6).
4) Relative date parsing (7).
5) Modify/done/status updates (9, 15). (Skip 8 for now)
6) Status output + kanban tweak (12–13).
7) List header + priority placement (10). (Skip 11 for now)
8) Sort/group output + view aliases (5a–5c).
9) Clock command rename (14).

## Risks & Mitigations
- **Output breaking scripts**: gate new layouts with a format flag.
- **Ambiguous abbreviations**: require unique prefix and list candidates.
- **Relative date ambiguity**: limit to a documented subset.
- **Interactive prompts in scripts**: detect TTY and error out.

## Documentation Updates
- Update `docs/COMMAND_REFERENCE.md` for new tokens, aliases, and date formats.
- Add examples for list views and grouping output.

## Execution Notes / Deviations
- Group headers in list output are rendered as a separator line plus a group-value row; the description-overlap layout in the sketch is not implemented.
- `task close` closes any running session for the task and removes the task from the clock stack to keep stack/session state consistent.
- Most items in this plan were already implemented in the current codebase; execution required only the annotate fallback for non-existent IDs and adding `task_description` to `task status --json` clock output.
- Tests now serialize access to `HOME` via a shared lock to avoid cross-test interference with per-test temp config paths.