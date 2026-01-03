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

### Project List Missing Location
- Projects from different workspaces look identical
- Can't tell which workspace a project is from
- **Solution**: Show subdued path or workspace badge

### Lower Section Not Cleared on Tab Switch
- Old command preview persists when switching views
- Should clear when changing tabs

### Drivers Section Still Showing
- Detail view shows "DRIVERS: rust active"
- Drivers concept was removed
- Should show ecosystem instead or remove

---

## Low Priority

### Debug Logging Enabled
- Verbose [DISCOVERY], [SCAN] logs still active
- Should be configurable or disabled by default

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

---

## Technical Debt

### TOML Field Naming
Config uses `#[serde(rename = "type")]` which is confusing. Consider using `type` directly or improving error messages.
