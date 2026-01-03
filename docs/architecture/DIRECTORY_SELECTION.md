# Directory Selection Features

## Overview

When adding workspace directories in the Workspace Manager (press `4`), you have three powerful ways to select directories:

1. **Manual typing** - Just type the path
2. **Tab completion** - Press Tab to auto-complete directory names
3. **Fuzzy finder** - Press Ctrl+D to launch an interactive picker

## Feature 1: Tab Completion

### How it works

While typing a directory path, press `Tab` to automatically complete the current directory name.

### Examples

```
Type: ~/Do[Tab]
Result: ~/Documents/

Type: ~/proj[Tab]
Result: ~/projects/

Type: ~/Library/App[Tab]
Result: Shows "2 matches: Application Support, ApplicationScripts"
```

### Behavior

- **Single match**: Automatically completes the path and adds a trailing `/`
- **Multiple matches**: Shows how many matches exist and lists them (up to 5)
- **No matches**: Shows "No matching directories" message
- **Tilde expansion**: Properly handles `~` for home directory
- **Only directories**: Only completes directory names, not files

## Feature 2: Fuzzy Finder (Ctrl+D)

### How it works

Press `Ctrl+D` to open an interactive fuzzy finder powered by [skim](https://github.com/lotabout/skim).

### What you'll see

The fuzzy finder shows:
- **Common directories**: ~, ~/Desktop, ~/Documents, ~/Downloads, ~/Music, ~/Pictures, ~/Videos, ~/projects
- **Registered workspaces**: All directories already added to Byte
- **Subdirectories**: If you've typed a partial path, shows subdirectories of that path

### Navigation

```
↑↓        Navigate through results
Type      Fuzzy search (e.g., "doc" matches "Documents")
Enter     Select the highlighted directory
Esc/^C    Cancel and return to TUI
```

### Installation

**No installation required!** The fuzzy finder is embedded in Byte using the skim library. Just press Ctrl+D and it works out of the box.

### Examples

**Scenario 1: Quick selection of common directory**
```
1. Press '4' to open Workspace Manager
2. Press 'a' to add directory
3. Press Ctrl+D
4. Type "docs" → filters to ~/Documents
5. Press Enter → fills input with ~/Documents
6. Press Enter again → adds ~/Documents to workspace list
```

**Scenario 2: Fuzzy search from partial path**
```
1. Press 'a' to add directory
2. Type: ~/Library/
3. Press Ctrl+D → shows subdirectories of ~/Library/
4. Type "app" → filters to "Application Support"
5. Select and add
```

## Feature 3: Manual Typing

### Basic usage

Simply type the full or partial path:

```
~/projects
~/Documents/workspace
/Users/yourname/code
```

### Supported path formats

- **Tilde expansion**: `~/path` → `/Users/yourname/path`
- **Absolute paths**: `/full/path/to/directory`
- **Relative paths**: Not supported (must be absolute or use ~)

## Combined Workflow

The three methods work together seamlessly:

```
Example: Adding ~/Documents/Projects/WebApps

Method 1 (Manual):
  Type: ~/Documents/Projects/WebApps

Method 2 (Tab completion):
  Type: ~/Do[Tab]cu[Tab]Proj[Tab]Web[Tab]

Method 3 (Fuzzy + Tab):
  Type: ~/
  Press Ctrl+D, select Documents
  Press Tab to complete Projects
  Press Tab to complete WebApps

Method 4 (Hybrid):
  Press Ctrl+D, fuzzy find to ~/Documents
  Then type: /Projects/WebApps
```

## UI Reference

### Normal Mode
```
Workspace Manager  3 directories

  ~/projects                    5 projects  [primary]
  ~/Documents/code             2 projects
  ~/work                       12 projects

─────────────────────────────────────────

  a add directory  d remove  1 back

Status: Managing workspace directories
```

### Input Mode (after pressing 'a')
```
Enter directory path: ~/Documents/_

[Tab] complete  [Ctrl+D] fuzzy find  [Enter] add  [Esc] cancel

Status: Type path, Tab to complete, Ctrl+D for fuzzy picker
```

### During Tab Completion
```
Enter directory path: ~/Docu_

Status: Completed: ~/Documents/
```

### After Fuzzy Picker
```
Enter directory path: ~/Documents/Projects_

Status: Selected: ~/Documents/Projects
```

## Error Handling

### Tab Completion Errors
- **Cannot read directory**: "Cannot read directory: /path"
- **No matches**: "No matching directories"
- **Multiple matches**: "3 matches: Desktop, Documents, Downloads"

### Fuzzy Picker Errors
- **No candidates**: Fuzzy picker won't launch if there are no directories to show
- **User cancelled**: "Cancelled" (press Esc or Ctrl+C in the picker)

## Implementation Details

### Tab Completion
- **File**: src/tui/mod.rs:282-362
- **Trigger**: KeyCode::Tab in AddingDirectory mode
- **Algorithm**:
  1. Parse input into directory + prefix
  2. Read directory contents
  3. Filter for directories matching prefix
  4. If 1 match: complete it
  5. If N matches: show count and list

### Fuzzy Picker
- **File**: src/tui/mod.rs:565-642
- **Library**: skim v0.15 (embedded, no external installation)
- **Trigger**: Ctrl+D (KeyCode::Char('d') + CONTROL modifier)
- **Flow**:
  1. Set `app.launch_fuzzy_picker = true`
  2. Event loop detects flag
  3. Suspend TUI (restore terminal)
  4. Run embedded skim picker with candidate list
  5. Resume TUI (re-setup terminal)
  6. Populate input buffer with selection

### Terminal Management
```rust
// Suspend TUI
restore_terminal(terminal)?;

// Run embedded skim library
let selected = run_fuzzy_picker(&app.input_buffer);

// Resume TUI
*terminal = setup_terminal()?;
```

The skim library is compiled directly into Byte, so there's no need for external dependencies or commands.

## Keyboard Reference

| Key | Context | Action |
|-----|---------|--------|
| `a` | Workspace Manager (Normal mode) | Start adding directory |
| `Tab` | Adding directory | Auto-complete path |
| `Ctrl+D` | Adding directory | Launch fuzzy finder |
| `Backspace` | Adding directory | Delete character |
| `Esc` | Adding directory | Cancel and clear input |
| `Enter` | Adding directory | Add the directory |
| `d` | Workspace Manager (Normal mode) | Remove selected directory |

## Performance Notes

- **Tab completion**: Fast (synchronous directory read)
- **Fuzzy finder**: Slight delay while building candidate list (~50-100ms)
- **TUI suspend/resume**: Seamless, no visible flicker

## Future Enhancements

Potential improvements (not yet implemented):
- Cycle through multiple tab completions
- Show completion preview before accepting
- Integrated fuzzy finder (no external skim dependency)
- Recent directories history
- Bookmarks/favorites
- Browse mode with arrow keys
