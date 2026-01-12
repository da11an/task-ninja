## Outstanding Items

This document tracks outstanding tasks and improvements across all plans.

---

### Plan 02: Revision 1

#### Item 3: Status Lines for Commands Without Arguments
- [ ] Performance test status queries on large datasets
- [ ] For status as leading line, format as header, not addendum to help docs

#### Item 2: Command Truncation/Abbreviation Support
- [ ] Add configuration option for expansion verbosity (`~/.taskninja/rc`)
- [ ] Document abbreviation feature

#### Item 4: Filter-Before-Command Pattern
- [ ] Fix remaining test issues (2 tests failing - likely test setup issue, command works in real environment)

---

### Plan 03: Task Deletion

- [ ] Write tests for single task deletion
- [ ] Write tests for bulk deletion
- [ ] Write tests for confirmation logic
- [ ] Write tests for related data cleanup

---

### Plan 04: Task ID Ranges and Lists

- [x] Update `handle_task_list()` to use ID spec parsing (for bare numeric IDs) - **COMPLETED**
- [ ] Update `handle_annotation_add()` to use ID spec parsing
- [ ] Update `handle_task_sessions_*()` to use ID spec parsing
- [ ] Write integration tests for commands
- [ ] Update command reference documentation

---

### Plan 10: Session Modification and Deletion

**Status:** Planning complete, implementation pending

**Summary:** Add session modification and deletion commands with syntax parallel to task modification. Includes overlap detection, conflict reporting, and session ID exposure in list output.

**See:** `design/Plan_10_Session_Modification_and_Deletion.md`

**Key Features:**
- `task sessions modify <session_id> [start:<expr>] [end:<expr>]`
- `task sessions delete <session_id>`
- Overlap detection with detailed conflict reporting
- Session IDs exposed in `sessions list` output

**Implementation Checklist:**
- [ ] Repository layer methods (modify, delete, overlap detection)
- [ ] CLI layer (commands, handlers, parsers)
- [ ] Display updates (session IDs in list output)
- [ ] Error handling and validation
- [ ] Documentation updates
- [ ] Integration tests

---

### General

- [ ] Fix compiler warnings (11 unused variable warnings)
- [ ] Add comprehensive integration tests for all new features
- [ ] Update user documentation with new features

---

## Notes

- Items marked with [x] are completed but may need verification
- Items marked with [ ] are pending
- Priority should be given to testing and documentation
