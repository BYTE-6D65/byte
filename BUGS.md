# Known Issues & Feature Requests

## 🐛 Bugs

### High Priority

- **Text Overrun Everywhere** (screenshot evidence)
  - Detail view PATH overruns: `/Users/liam/Library/Mobile Documents/.../Byte_t` gets cut off
  - Need global solution for text truncation instead of fixing per-widget
  - Already fixed in: workspace manager (view 4), but needs to be applied globally
  - Affected views: Detail (3), potentially others
  - **Solution ideas:**
    - Use ratatui's built-in `Paragraph::wrap()` for multi-line content
    - Create `truncate_text()` helper for single-line displays
    - Hybrid: wrap descriptions, truncate paths/titles

- **Command Palette Needs Edit Stage**
  - Selecting a command immediately executes it
  - Should enter edit mode first to customize args/names
  - Flow: Select cmd → [Enter] → Edit mode → [Enter] → Execute
  - Allows changing project names, args, paths before running

- **Command Execution Issues**
  - Init commands show "✗ Error: No such file or directory (os error 2)"
  - Unknown target directory for project creation
  - Commands need working directory context

### Medium Priority

- **Project List Doesn't Show Location** (image 13)
  - Projects from different workspaces look identical
  - "meow" in ~/projects and "byte" in iCloud path - can't tell which is where
  - Should show workspace/path indicator
  - Ideas: subdued path below name, or badge with workspace name

- **Lower Section Not Cleared on Tab Switch** (image 13)
  - Switch from Commands → Projects still shows old command preview
  - "go: creating new go.mod..." text persists from previous view
  - Should clear/reset lower section when changing views
  - Follows "upper = display, lower = edit" philosophy

- **Drivers Section Still Showing**
  - Detail view shows "DRIVERS: rust active"
  - Drivers concept was removed from architecture
  - Should probably show ecosystem instead, or remove section entirely

### Low Priority

- **Debug Logging Still Enabled**
  - Verbose [DISCOVERY], [SCAN], [COUNT], [RELOAD] logs
  - Should clean up or make configurable
  - Good for troubleshooting but noisy for production

## 🎯 Feature Requests

### UI/UX Improvements

**Design Philosophy: Consistent Layout Language**

Byte uses a consistent 2-axis layout pattern for ALL views:

**Vertical Axis (Top/Bottom):**
- **Top:** Display/communicate (read-only, lists, status)
- **Bottom:** Edit/specify (input, editing, configuration)
- Clear "viewing" vs "doing" separation

**Horizontal Axis (Left/Right) - Table Pattern:**
- **Left column:** Primary content (what)
- **Right column:** Context/location (where)
- Applies to ALL list views consistently

**Implementation plan:**

1. **Project Browser (View 1): Add Location Column**
   ```
   Projects  2
   ┌─────────────────────────────────┬──────────────────┐
   │ Project                         │ Location         │
   ├─────────────────────────────────┼──────────────────┤
   │ meow                            │ ~/projects       │
   │ cli project  #go                │                  │
   │                                 │                  │
   │ byte                            │ .../iCloud/...   │
   │ Project orchestration... #rust  │                  │
   └─────────────────────────────────┴──────────────────┘
   ```

2. **Command Palette (View 2): Add Target Column**
   ```
   Commands  3
   ┌─────────────────────────────────┬──────────────────┐
   │ Command                         │ Target           │
   ├─────────────────────────────────┼──────────────────┤
   │ init go cli <name>              │ ~/projects       │
   │ Initialize Go CLI project       │                  │
   └─────────────────────────────────┴──────────────────┘
   ```

3. **Apply Everywhere:**
   - Workspace Manager (already has path display)
   - Detail view (could show path prominently)
   - Consistent visual language across all views

From FEATURE_REQUESTS.md:
- Arrow key path navigation (Fish/zsh-style)
  - Right arrow (→) to accept inline completion
  - Left arrow (←) to navigate backwards in path
  - Ghost text preview

## ✅ Recently Fixed

- ✅ Project discovery in registered paths (TOML field name mismatch)
- ✅ Command execution (was only showing message, not running)
- ✅ Text truncation in workspace manager
- ✅ Edit workspace path functionality (press 'e' in view 4)
- ✅ Centralized logging to .byte/logs/
- ✅ Input mode number key conflict (typing numbers in paths)
- ✅ Inconsistent trailing slash in tab completion

## 💡 Technical Debt

- **Global Text Truncation Solution**
  - Consider creating a helper function/widget wrapper
  - Should handle: paths, descriptions, any long text
  - Maybe: `truncate_for_width(text, available_width, strategy)`
  - Strategies: end (show end), start (show start), middle (ellipsis in middle)

- **TOML Field Naming Consistency**
  - Config uses `#[serde(rename = "type")]` for project_type
  - Confusing - maybe just use `type` in Rust too?
  - Or better error messages when TOML parsing fails

## 📝 Notes

- Keep this doc updated as we find/fix issues
- Screenshot evidence helpful for UI bugs
- Tag priority: high (blocking), medium (annoying), low (nice to have)
