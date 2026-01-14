

### 1. Expected `task annotate <note>` to default to live session

- Also consider the terminology 'live' instead of 'active' for clocked in tasks

### 2. `task session add <task id> start:<time or datetime> end:<time or datetime>`

For entering work session OUTSIDE of clock, for example after the fact.


### 3. Make yes default for task session modify:

$ task sessions modify 10 start:10:15
Modify session 10?
  Start: 2026-01-13 11:42:06 -> 2026-01-13 10:15:00
Are you sure? ([y]/n):

### 4. Add task info e.g. Modify session 9 (task 7: Group meeting)?

Modify session 9?
  Start: 2026-01-13 11:15:00 -> 2026-01-13 09:09:00
  End: 2026-01-13 11:40:00 -> 2026-01-13 10:15:00
 
### 5. task list sort:priority, group by, create views with filter, sort and group by setting

### 6. token abbreviation in filtering

(base) [princdr@brtuxwrkst03 ~]$ task list st:pending
Error: Filter parse error: Invalid filter token: st:pending
(base) [princdr@brtuxwrkst03 ~]$ task list stat:pending
Error: Filter parse error: Invalid filter token: stat:pending
(base) [princdr@brtuxwrkst03 ~]$ task list status:pending
[worked]

### 7. Parse more due dates formats (relative dates):

$ task add project:admin.networking Weekly catchup connection alloc:30m due:1week
Internal error: Failed to parse due date

### 8. Make done a subset of modify that also sets the status to completed

- Also allow status to be set in modify using status:blah if a valid status option -- all tokens should be settable here.
- Unrecognized tokens (limit syntax to no space token:value):
    - If not in quotes, and token fitting strict no-space syntax isn't a recognized token, give user dialog to cancel, or include token as (part of) description.

### 9. Allow new project entry in modify like in add with user dialog

### 10. alloc -> Alloc in task list table column title.

### 11. Priority -> Prior and before Alloc column in task list

### 12. Clock status include task description in Clock Status.

### 13. Modify Kanban to apply NEXT stage to task that is in position 1 if task pos 0 is LIVE.

### 14. Change task clock roll to task clock next syntax.

### 15. `task list sort:colA,colB,... group:Kanban,tag,...`

- Sort columns become like multi-indexes in pandas. Sorted by first first, second second, etc.
- Sort columns put first in the table.
- ID, Description is always after sort columns
- Group by columns directly follow Description and overlap the description field as follows, if ColA and ColB are sort fields and ColC and ColD are group fields:

ColA ColB ID Description ColC ColD ...
---------------------------------- ...
-------------------------Val1:Val1 ...
Val1 Val1 XX Description can split over labeling grouping divider line
Val2 Val2 XX Description for the next item
-----------------------------:Val2 ...
Val3 Val4 XX Description of the next item
-------------------------Val2:Val1 ...
...

### 16. Close vs Complete task completion statuses.

Enable `task close <id>` -- this assigns a closed status instead of completed
Update syntax of `task done <id>` to `task finish <id>` to make it a verb, status still becomes completed


