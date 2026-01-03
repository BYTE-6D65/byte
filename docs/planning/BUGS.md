# Known Issues

**Last Updated**: 2026-01-03

---

## High Priority

### Command Palette Needs Edit Stage
- Selecting a command immediately executes it
- Should enter edit mode first to customize args/names
- Flow: Select -> [Enter] -> Edit mode -> [Enter] -> Execute

---

## Medium Priority

_(No active medium-priority bugs)_

---

## Low Priority

_(No active low-priority bugs)_

---

## Feature Requests

See [ROADMAP.md](./ROADMAP.md) for planned features including:
- Arrow key path navigation (Fish-style)
- Project search/filter
- Batch command execution
- Command history

---

## Recently Fixed

### Text Overrun in Views (Fixed 2026-01-03)
- ✅ Created `truncate_path()` helper in `src/tui/mod.rs`
- ✅ Fixed detail view PATH text overflow
- ✅ Fixed log viewer PATH text overflow
- ✅ Fixed project browser description overflow
- ✅ Fixed command palette description overflow
- Paths truncate intelligently showing start and end with "..." in middle

### Lower Section Not Cleared on Tab Switch (Fixed 2026-01-03)
- ✅ `command_result_display` now cleared when switching views (menus 1-4)
- Command success/failure progress bar dismisses immediately on tab switch
- Previously persisted until 3-second auto-dismiss timeout

### Project List Missing Location (Already Fixed)
- ✅ Workspace paths displayed in project browser (line 2162: `get_workspace_for_project()`)
- Projects show workspace location in right column (60/40 layout)
- Path truncates with "…" prefix if too long

### Drivers Section Still Showing (Obsolete/Already Fixed)
- ✅ No "DRIVERS:" label found in detail view
- Ecosystem tags shown in project browser as "#rust", "#node", etc.
- Field name still "drivers" in Project struct but displays correctly

### Debug Logging Enabled (Fixed 2026-01-03)
- ✅ Added log level filtering to Logger (src/log.rs)
- ✅ Default log level set to INFO (skips DEBUG messages)
- ✅ Changed all [DISCOVERY] and [SCAN] logs from INFO to DEBUG
- Verbose discovery/scan logs no longer clutter log files by default
- Can be enabled in future by adding config option for log level

---

## Technical Debt

### TOML Field Naming
Config uses `#[serde(rename = "type")]` which is confusing. Consider using `type` directly or improving error messages.
