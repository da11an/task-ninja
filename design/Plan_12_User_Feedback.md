

### 1. Implement tab completion rather than allowing abbreviations

Rationale: less mental work, more natural commandline expectations
Applies to: commands, subcommands, project names, filters, ...

### 2. `task clock list`

Drop `task clock show` alias. We want to have a tight syntax with one right way to do things (pythonic)
`task clock list` should include the same columns as the regular task list, but leading with the clock stack position column and sorted accordingly. It is too hard to remember what the items are without details.

### 3. `task list` should include more columns, like allocation


### 4. Apply filtering to `task sessions list <arguments>` similar to `task list <arguments>`


### 5. Allow project creation during task creation with confirmation

- Consider the same for tags
- This is a new [project|tag] <similar to ...>. Add new [project|tag]? y=Add <name> [project|tag] and apply, n=No but create task without, [c]=No and cancel task creation (cancel is default)

### 6. `task done <id>` if not running throws error. Need to be able to check off task even if not clocked in

### 7. `task clock in --task <id>` is verbose. Can we just have `task clock in <id>` Implications about ambiguity with clock stack ids?

### 8. Need syntax to clock in task on adding it in the same one liner command

### 9. Drop status lines from individual commands and provide a single dashboard or status command that provides all the most actionable information

- Clock status
- Top up to 3 tasks from clock stack
- Top priority up to 3 tasks from task list NOT on clock stack
- Session summary for day
- Number of tasks overdue, if none, when tasks will become overdue based on ...
- Etc.

### 10. `task list` provide multiple views as arguments

- Project organized view
- Priority score organized view (develop priority score)
- Sort flags (choose a column)
