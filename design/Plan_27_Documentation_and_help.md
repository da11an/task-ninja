# Plan 27: Documentation and Help System

- The current tatl --help do not fully document the available features and syntax.
- Options that would help with discoverability are spotty at best
- `man tatl` doesn't work

The entire CLI should be learnable through either the man page or the various command help pages accessible from the CLI.
If a non techie can't navigate using the help commands, it's not documented well enough.

## Current State Analysis

### Issues Identified

1. **Minimal Help Text**: Current `--help` output provides only brief command descriptions and argument names. No examples, no syntax explanations, no usage patterns.

2. **Missing Documentation in Help**:
   - Filter syntax (e.g., `project:work +urgent`, `desc:bug`, date ranges)
   - Date expressions (e.g., `tomorrow`, `+3d`, `2024-01-15`, `09:00`)
   - Respawn patterns (e.g., `daily`, `weekdays:mon,wed,fri`, `nth:1:day`)
   - Duration format (e.g., `2h`, `30m`, `1d`)
   - Task field syntax (e.g., `project:name`, `due:tomorrow`, `+tag`, `uda.key:value`)
   - Interval syntax (e.g., `09:00..12:00`, `start:09:00..end:12:00`)

3. **No Man Page**: `man tatl` doesn't work. Users must rely on external documentation (README.md, COMMAND_REFERENCE.md) which may not be installed or accessible.

4. **Poor Discoverability**:
   - No examples in help output
   - No cross-references between related commands
   - Complex features (filters, respawn, UDAs) are not explained
   - No "see also" sections

5. **Inconsistent Documentation**:
   - Some commands have better help than others
   - Subcommands have minimal documentation
   - Options lack detailed explanations

6. **Non-Technical User Barrier**: The help system assumes users understand:
   - Filter syntax
   - Date expressions
   - Field tokens
   - Command patterns

## Goals

1. **Complete CLI Help**: Every command should have comprehensive help accessible via `--help` that includes:
   - Clear description
   - Syntax examples
   - Common usage patterns
   - Related commands
   - Field/option explanations

2. **Man Page Support**: Generate and install a man page so `man tatl` works.

3. **Discoverability**: Users should be able to learn the entire CLI through help commands alone.

4. **Non-Technical Accessibility**: Help should be understandable by non-technical users with clear examples and explanations.

## Solution Approach

### Phase 1: Enhance Clap Help Text

#### 1.1 Add `long_about` to All Commands
- Replace or supplement short `///` doc comments with detailed `#[command(long_about = "...")]` attributes
- Include syntax examples in long_about
- Add "Examples:" sections where appropriate

#### 1.2 Enhance Argument Documentation
- Add detailed help text for all arguments using `#[arg(help = "...")]`
- Include examples in argument help (e.g., `help = "Date expression (e.g., 'tomorrow', '+3d', '2024-01-15')"`)
- Document filter syntax in `list`, `modify`, `show` commands
- Document date expressions where used
- Document respawn patterns in `add` and `modify`

#### 1.3 Add Examples Sections
- Use Clap's `#[command(example = "...")]` attribute for common usage patterns
- Add 2-3 examples per command showing typical use cases

#### 1.4 Document Complex Features
Create help sections for:
- **Filter Syntax**: Document in `list`, `modify`, `show`, `sessions list`, `sessions report`
  - Field filters: `project:`, `tag:`, `status:`, `desc:`, `due:`, etc.
  - Operators: `AND`, `OR`, `NOT`
  - Date ranges: `start:..end:`, `-7d..now`
- **Date Expressions**: Document in commands that accept dates
  - Relative: `tomorrow`, `+3d`, `-1w`
  - Absolute: `2024-01-15`, `2024-01-15 14:30`
  - Time-only: `09:00`, `14:30`
- **Respawn Patterns**: Document in `add` and `modify`
  - Simple: `daily`, `weekly`, `monthly`, `yearly`
  - Advanced: `weekdays:mon,wed,fri`, `monthdays:1,15`, `nth:1:day`, `every:2w`
- **Task Fields**: Document in `add` and `modify`
  - Standard: `project:`, `due:`, `scheduled:`, `wait:`, `allocation:`, `template:`, `respawn:`
  - Tags: `+tag`, `-tag`
  - UDAs: `uda.key:value`
- **Interval Syntax**: Document in `onoff`, `offon`, `sessions modify`
  - Time intervals: `09:00..12:00`
  - With dates: `2024-01-15 09:00..12:00`
  - Session modification: `start:09:00..end:12:00`

#### 1.5 Add Cross-References
- Add "See also:" sections in help text pointing to related commands
- Use `#[command(visible_alias = "...")]` where appropriate for discoverability

### Phase 2: Create Man Page

#### 2.1 Choose Man Page Generation Tool
Options:
- **`clap_mangen`**: Official Clap crate for generating man pages from Clap definitions
- **Manual**: Write man page manually in troff format
- **`mdman`**: Convert markdown to man page

**Recommendation**: Use `clap_mangen` as it automatically generates man pages from Clap definitions, ensuring consistency with `--help` output. AGREED

#### 2.2 Generate Man Page
- Add `clap_mangen` as a dev dependency
- Create build script or command to generate `tatl.1` man page
- Include all commands, subcommands, and examples

#### 2.3 Install Man Page
- Add installation step to build process
- Install to standard location (e.g., `/usr/local/share/man/man1/` or `~/.local/share/man/man1/`)
- Update `INSTALL.md` with man page installation instructions

#### 2.4 Man Page Structure
Standard sections:
- NAME
- SYNOPSIS
- DESCRIPTION
- COMMANDS (with subsections for each command)
- EXAMPLES
- SEE ALSO
- AUTHOR

### Phase 3: Improve Help Organization

#### 3.1 Add Help Topics
Create special help topics for complex features:
- `tatl help filters` - Filter syntax reference
- `tatl help dates` - Date expression reference
- `tatl help respawn` - Respawn pattern reference
- `tatl help fields` - Task field reference

These can be implemented as:
- Special subcommands that print formatted help
- Or sections in the main help output

#### 3.2 Group Related Commands
- Use Clap's command grouping to organize commands logically
- Add "Command Groups" section to main help

#### 3.3 Add Quick Reference
- Add a "Quick Reference" section to main `--help` showing common patterns
- Include one-line examples for frequent operations

### Phase 4: Testing and Validation

#### 4.1 Test Help Completeness
- Verify every command has comprehensive help
- Check all arguments have descriptions
- Ensure examples are present and correct

#### 4.2 Test Discoverability
- Have a non-technical user try to learn the CLI using only help
- Document gaps and address them

#### 4.3 Test Man Page
- Verify `man tatl` works after installation
- Check man page formatting is correct
- Ensure all commands are documented

## Implementation Priority

### High Priority (Phase 1)
1. Add `long_about` to all top-level commands with examples
2. Enhance argument help text with examples
3. Document filter syntax in relevant commands
4. Document date expressions in relevant commands
5. Document respawn patterns in `add` and `modify`

### Medium Priority (Phase 2)
1. Set up `clap_mangen` for man page generation
2. Generate initial man page
3. Add man page installation to build process

### Lower Priority (Phase 3)
1. Add help topics for complex features
2. Improve help organization
3. Add quick reference section

## Detailed Command Help Requirements

### Commands Needing Enhanced Help

#### `tatl add`
- Document all field syntax (project, due, scheduled, wait, allocation, template, respawn)
- Document tag syntax (`+tag`, `-tag`)
- Document UDA syntax (`uda.key:value`)
- Examples for common patterns
- Explain `--on`, `--onoff`, `--enqueue` options

#### `tatl list`
- Document filter syntax comprehensively
- Explain all filter fields
- Document operators (AND, OR, NOT)
- Examples of common filters

#### `tatl modify`
- Document field modification syntax
- Document filter syntax for target selection
- Document respawn pattern validation
- Examples of common modifications

#### `tatl show`
- Document ID spec syntax (single, range, list)
- Document filter syntax
- Explain output format

#### `tatl on` / `tatl off` / `tatl offon` / `tatl onoff`
- Document time expression syntax
- Document interval syntax
- Explain queue behavior
- Examples of break capture and historical sessions

#### `tatl finish` / `tatl close` / `tatl reopen`
- Document target selection (ID, filter, queue[0])
- Document time expression syntax
- Explain respawn behavior

#### `tatl sessions list`
- Document filter syntax
- Document date filtering (start:, end:)
- Explain output format

#### `tatl sessions modify`
- Document interval syntax (`start:..end:`)
- Document time expressions
- Explain conflict handling

#### `tatl sessions report`
- Document filter syntax
- Document date range syntax
- Explain output format

#### `tatl projects *`
- Document nested project syntax
- Explain project hierarchy

#### `tatl queue sort`
- Document sort field syntax
- Explain ascending/descending (prefix with `-`)

## Examples of Enhanced Help

### Before (Current):
```
Add a new task

Usage: tatl add [OPTIONS] [ARGS]...

Arguments:
  [ARGS]...  Task description and fields (e.g., "fix bug project:work +urgent")
```

### After (Proposed):
```
Add a new task

Create a new task with optional attributes, timing, and queue placement.

Usage: tatl add [OPTIONS] [ARGS]...

Arguments:
  [ARGS]...  Task description and fields. The description is all text not matching field patterns.
             
             Field syntax:
               project:<name>     - Assign to project (creates if new with -y)
               due:<expr>         - Set due date (see DATE EXPRESSIONS)
               scheduled:<expr>   - Set scheduled date
               wait:<expr>        - Set wait date
               allocation:<dur>   - Set time allocation (e.g., "2h", "30m")
               template:<name>    - Use template
               respawn:<pattern>  - Set respawn rule (see RESPAWN PATTERNS)
               +<tag>             - Add tag
               -<tag>             - Remove tag
               uda.<key>:<value>  - Set user-defined attribute
             
             Examples:
               tatl add "Fix bug" project:work +urgent
               tatl add "Review PR" due:tomorrow allocation:1h
               tatl add "Daily standup" respawn:daily due:09:00

Options:
      --on[=<TIME>]     Start timing immediately after creation. If TIME is provided (e.g., --on=14:00),
                        the session starts at that time instead of now. Pushes task to queue[0].
      --onoff <INTERVAL> Add historical session for the task (e.g., "09:00..12:00"). Takes precedence
                        over --on and --enqueue.
      --enqueue         Add task to end of queue without starting timing.
  -y, --yes             Auto-confirm prompts (create new projects, modify overlapping sessions).

Examples:
  # Simple task
  tatl add "Fix authentication bug"
  
  # Task with project and tags
  tatl add "Review PR" project:work +code-review +urgent
  
  # Task with due date and time allocation
  tatl add "Write docs" project:docs due:tomorrow allocation:2h
  
  # Task with respawn rule
  tatl add "Daily standup" respawn:daily due:09:00
  
  # Task that starts timing immediately
  tatl add --on "Start working on feature"
  
  # Task with historical session
  tatl add "Forgot to track meeting" --onoff 14:00..15:00 project:meetings

See also:
  tatl modify - Modify existing tasks
  tatl list - List tasks
  tatl help dates - Date expression reference
  tatl help respawn - Respawn pattern reference
```

## Technical Implementation Notes

### Clap Help Enhancement
- Use `#[command(long_about = "...")]` for detailed descriptions
- Use `#[arg(help = "...")]` for argument descriptions
- Use `#[command(example = "...")]` for examples
- Use `#[command(visible_alias = "...")]` for discoverability
- Consider using `#[command(next_help_heading = "...")]` for grouping

### Man Page Generation
- Add to `Cargo.toml`: `clap-mangen = { version = "4.5", optional = true }`
- Create `build.rs` or script to generate man page
- Use `clap_mangen::Man` to generate from `Cli` struct
- Install man page in build/install process

### Help Topics
- Can be implemented as special subcommands or as sections in main help
- Consider using Clap's `#[command(subcommand)]` with a `Help` enum for topics
- Or use a simple function that prints formatted help text

## Success Criteria

1. ✅ Every command has comprehensive help with examples
2. ✅ All complex features (filters, dates, respawn) are documented in help
3. ✅ `man tatl` works and is comprehensive
4. ✅ A non-technical user can learn the CLI using only help commands
5. ✅ Help is discoverable and well-organized
6. ✅ Examples are present and correct for all common use cases

## Open Questions

1. **Help Topics Implementation**: Should help topics be subcommands (`tatl help filters`) or sections in main help?
   - **Decision**: Implement as subcommands for better discoverability

2. **Man Page Location**: Where should man page be installed?
   - **Decision**: Follow standard locations: `/usr/local/share/man/man1/` for system install, `~/.local/share/man/man1/` for user install

3. **Help Text Length**: How detailed should help be? Risk of overwhelming users.
   - **Decision**: Use `long_about` for detailed help, keep short descriptions for quick reference. Users can use `--help` for details.

4. **Examples in Help**: How many examples per command?
   - **Decision**: 2-3 examples showing common patterns, with more in man page

5. **Filter Syntax Documentation**: Should it be in every command that uses filters, or in a central location?
   - **Decision**: Include brief syntax in each command, with reference to `tatl help filters` for details

## Next Steps

1. Review and approve this plan -- APPROVED DRP
2. Begin Phase 1: Enhance Clap help text (start with high-traffic commands: `add`, `list`, `modify`)
3. Test help completeness as we go
4. Once Phase 1 is complete, move to Phase 2 (man page)
5. Iterate based on user feedback
6. Test app behavior against documentation. Identify implementation issues for future work.
