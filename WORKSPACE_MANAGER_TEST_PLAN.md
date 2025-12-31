# Workspace Manager - Test Plan

## Implementation Status: COMPLETE ✓

All 4 phases of Feature 1 (Interactive Workspace Manager) have been implemented:

- ✅ **Phase 1**: Data model (WorkspaceDir, InputMode, workspace fields in App)
- ✅ **Phase 2**: Config functions (save, add_workspace_path, remove_workspace_path)
- ✅ **Phase 3**: TUI view (render_workspace_manager with input field)
- ✅ **Phase 4**: Keyboard handlers (a/d/Esc/Backspace/Enter all wired)

## Manual Test Procedure

### Test 1: Add a Workspace Directory

1. **Launch TUI**
   ```bash
   cargo run -- tui
   ```

2. **Navigate to Workspace Manager**
   - Press `4` key
   - Verify: Title shows "Workspace Manager" with count
   - Verify: Primary workspace (~/projects) is listed with [primary] tag
   - Verify: Help text shows: "a add directory  d remove"

3. **Add a New Directory**
   - Press `a` key
   - Verify: Input field appears: "Enter directory path: _"
   - Verify: Help text changes to show input mode
   - Type: `~/Documents` (or any valid directory)
   - Verify: Characters appear as you type
   - Press `Enter`
   - Verify: Status message shows "✓ Added ~/Documents"
   - Verify: Directory appears in the workspace list
   - Verify: Project count is displayed (may be 0 if no projects)

4. **Verify Persistence**
   - Press `q` to quit
   - Check config file:
     ```bash
     cat ~/.config/byte/config.toml | grep -A 2 registered
     ```
   - Verify: `registered = ["~/Documents"]` is in the config
   - Re-launch TUI and press `4`
   - Verify: ~/Documents is still in the list

### Test 2: Remove a Workspace Directory

1. **Select the Added Directory**
   - Press `4` to go to Workspace Manager
   - Use `↑` `↓` arrow keys to select ~/Documents
   - Verify: Selected item is highlighted in cyan + bold

2. **Remove the Directory**
   - Press `d` key
   - Verify: Status message shows "✓ Removed ~/Documents"
   - Verify: Directory is removed from the list
   - Verify: Only primary workspace remains

3. **Verify Persistence**
   - Press `q` to quit
   - Check config:
     ```bash
     cat ~/.config/byte/config.toml | grep -A 2 registered
     ```
   - Verify: `registered = []` (empty array)

### Test 3: Error Handling

1. **Try to Add Invalid Path**
   - Press `4`, then `a`
   - Type: `~/this_does_not_exist_12345`
   - Press `Enter`
   - Verify: Error message shown: "✗ Error: Path does not exist: ..."

2. **Try to Add Duplicate**
   - Add ~/Documents successfully
   - Try to add ~/Documents again
   - Verify: Error message: "✗ Error: Path is already registered"

3. **Try to Remove Primary Workspace**
   - Select the primary workspace (~/projects)
   - Press `d`
   - Verify: Error message: "✗ Cannot remove primary workspace"

4. **Cancel Adding**
   - Press `a` to start adding
   - Type some text
   - Press `Esc`
   - Verify: Input mode cancelled
   - Verify: Status shows "Cancelled"
   - Verify: Returns to normal mode

### Test 4: Project Count Updates

1. **Add Directory with Byte Projects**
   - If you have projects in ~/projects/test-project
   - Press `4` to view Workspace Manager
   - Verify: Project count shows correct number next to ~/projects

2. **Add Another Directory**
   - Add a directory that contains byte.toml projects
   - Verify: Project count updates for that workspace
   - Press `1` to go back to Project Browser
   - Verify: Projects from both directories are listed

## Implementation Files Modified

1. **src/config/mod.rs** (lines 55-127)
   - `Config::save()` - Writes to ~/.config/byte/config.toml
   - `Config::add_workspace_path()` - Validates and adds to registered list
   - `Config::remove_workspace_path()` - Removes from registered list

2. **src/tui/mod.rs** (1070 lines total)
   - Lines 35-42: WorkspaceDir struct
   - Lines 44-47: InputMode enum
   - Lines 54-58: Workspace fields in App
   - Lines 62-64: View::WorkspaceManager enum variant
   - Lines 135-194: App::new() workspace loading
   - Lines 201-211: App::add_workspace()
   - Lines 214-225: App::remove_workspace()
   - Lines 227-284: App::reload_workspaces()
   - Lines 293-332: Keyboard handlers (a/d keys)
   - Lines 333-343: Input mode handlers (Esc/Backspace/Char)
   - Lines 404-420: Enter key handler for workspace manager
   - Lines 826-1069: render_workspace_manager() UI rendering

## Success Criteria

All of the following must pass:

- [x] Code compiles without errors ✓
- [ ] Can navigate to Workspace Manager with '4' key
- [ ] Can add a valid directory and it persists to config
- [ ] Can remove a non-primary directory
- [ ] Cannot remove primary workspace (error shown)
- [ ] Cannot add duplicate paths (error shown)
- [ ] Cannot add non-existent paths (error shown)
- [ ] Can cancel adding with Esc key
- [ ] Input field works (type, backspace, enter)
- [ ] Project counts are accurate for each workspace
- [ ] Config file is properly formatted TOML
- [ ] All changes persist across TUI restarts

## Known Limitations

- Selection state resets to index 0 after add/remove (acceptable)
- No confirmation dialog for remove (will add in future if needed)
- No way to change primary workspace yet (future feature)
- Project count is recalculated on every reload (could be optimized)

## Next Steps (Future Features)

After this feature is verified:
- Feature 2: Git Integration (status/pull/push/branch/checkout/log)
- Feature 3: Command execution from TUI
- Feature 4: Set primary workspace from TUI
