# Keybinding Design

**Last Updated**: 2026-01-03

---

## Current Keybindings

### Global (All Views)
- `q` / `Q` - Quit application
- `r` / `R` - Reload/hotload projects
- `?` - Show help message
- `1` - Switch to Project Browser
- `2` - Switch to Command Palette
- `3` - Switch to Detail View
- `4` - Switch to Workspace Manager

### Project Browser (View 1)
- `n` - New project form
- `↑` / `↓` - Navigate projects
- `Enter` - Open project details

### Command Palette (View 2)
- `t` - Toggle command filter (Build/Lint/Git/Test/Other) ⚠️ **COLLISION**
- `↑` / `↓` - Navigate commands
- `Enter` - Execute command

### Detail View (View 3)
- `t` - Create git tag form ⚠️ **COLLISION**
- `l` - View logs
- `o` - Open project in editor
- `Esc` - Close log viewer (when viewing logs)
- `↑` / `↓` - Scroll logs (when viewing)

### Workspace Manager (View 4)
- `a` - Add new workspace
- `e` - Edit selected workspace
- `d` - Delete selected workspace
- `↑` / `↓` - Navigate workspaces

### Forms (Overlay)
- `Tab` - Next field
- `Shift+Tab` - Previous field
- `Space` - Toggle checkbox/multi-select
- `↑` / `↓` - Navigate dropdowns / increment numbers
- `Enter` - Submit form
- `Esc` - Cancel form
- `Backspace` - Delete character
- Any char - Input text

---

## Identified Issues

1. **Key Collision:** `t` used in both Detail View (git tag) AND Command Palette (toggle filter)
2. **Inconsistent mnemonics:** Some keys match action (n=new, a=add) but not all (t=tag not "create")
3. **No vim navigation:** Only arrows, no hjkl support
4. **Poor finger flow:** Random mix of left/right hand keys
5. **Limited scalability:** Number row only goes to 4, what about view 5+?

---

## Proposed Redesign

### Design Principles
1. **Home row priority** - Most common actions on home row (asdfghjkl;)
2. **Mnemonic consistency** - First letter of action (n=new, e=edit, d=delete)
3. **Vim-style navigation** - hjkl for movement (arrows still work)
4. **Modal thinking** - Each view has clear, distinct action set
5. **No collisions** - Each key has one meaning per context
6. **Finger flow** - Alternate hands for common sequences

### Global (All Views)

**View Switching (Number Row - Left Hand):**
- `1` - Project Browser
- `2` - Command Palette
- `3` - Detail View
- `4` - Workspace Manager
- `5-9` - (Future views)

**System Actions (Home Row Edges):**
- `q` - Quit (left pinky - easy to reach, hard to accident)
- `;` - Command mode (future: type commands like `:quit`)
- `/` - Search/filter (vim-style)
- `?` - Help

**Common Actions:**
- `r` - Reload/refresh
- `Esc` - Cancel/go back

### Navigation (Universal - Vim Style)

**Primary Navigation:**
- `j` / `↓` - Move down
- `k` / `↑` - Move up
- `h` / `←` - Move left / collapse (future: tree navigation)
- `l` / `→` - Move right / expand (future: tree navigation)

**Quick Navigation:**
- `g` - Go to top
- `G` (Shift+g) - Go to bottom
- `Ctrl+d` - Page down
- `Ctrl+u` - Page up

### Project Browser (View 1)

**Home Row Actions:**
- `n` - **N**ew project
- `e` - **E**dit project (edit byte.toml)
- `d` - **D**elete project (with confirmation)
- `f` - **F**ilter/search projects

**Secondary Actions:**
- `o` - **O**pen in editor
- `Enter` - View project details

### Command Palette (View 2)

**Home Row Actions:**
- `f` - **F**ilter by type (Build/Lint/Git/Test/Other) ✅ **FIX: was `t`**
- `e` - **E**dit command before running
- `s` - **S**earch commands

**Secondary Actions:**
- `Enter` - Execute command
- `Space` - Mark for batch execution (future)

### Detail View (View 3)

**Home Row Actions:**
- `e` - **E**dit project (byte.toml)
- `l` - View **L**ogs
- `o` - **O**pen in editor

**Secondary Actions:**
- `t` - Create git **T**ag
- `g` - **G**it operations menu (future: commit, push, pull)
- `b` - **B**uild operations menu (future: build, test, lint)

### Workspace Manager (View 4)

**CRUD Actions:**
- `a` - **A**dd workspace
- `e` - **E**dit workspace
- `d` - **D**elete workspace
- `s` - **S**can workspace manually

---

## Key Improvements

### 1. Eliminated Collisions
✅ Command Palette filter: `t` → `f`
✅ Detail git tag stays: `t`
✅ Each key has one meaning per view

### 2. Better Ergonomics

**Common sequences:**
- `1` → `n` (left→left) - Switch to projects, create new
- `3` → `l` (left→right) - Switch to detail, view logs
- `j/k` (right hand) - Navigation stays on home row

**Action grouping (home row):**
- CRUD: `a`dd, `e`dit, `d`elete
- View: `n`ew, `l`ogs, `o`pen
- Filter/search: `f`, `s`

### 3. Vim Compatibility
- `hjkl` for navigation (optional, arrows still work)
- `g/G` for top/bottom
- `/` for search
- `Esc` for cancel/back
- `:` for command mode (future)

### 4. Mnemonic Consistency
- `n` = New (everywhere)
- `e` = Edit (everywhere)
- `d` = Delete (everywhere)
- `f` = Filter (everywhere)
- `s` = Search/Scan (everywhere)
- `o` = Open (everywhere)
- `l` = Logs (Detail view only)
- `t` = Tag (Detail view only)
- `a` = Add (Workspace view only)

---

## Migration Path

**Phase 1:** Add vim navigation (hjkl) alongside arrows (non-breaking)
**Phase 2:** Add `f` for filter in Command Palette (keep `t` working)
**Phase 3:** Document new bindings, show deprecation warnings
**Phase 4:** Remove old bindings in v1.0

---

## Quick Reference

```
┌─────────────────────────────────────────────────────┐
│ GLOBAL                                              │
│ 1-4: Views  q: Quit  r: Reload  ?: Help  /: Search │
├─────────────────────────────────────────────────────┤
│ NAVIGATION                                          │
│ j/k or ↑/↓: Move  g/G: Top/Bottom  Enter: Select   │
├─────────────────────────────────────────────────────┤
│ ACTIONS (Context-Dependent)                         │
│ n: New  e: Edit  d: Delete  f: Filter  o: Open     │
│ a: Add  l: Logs  t: Tag  s: Search/Scan             │
└─────────────────────────────────────────────────────┘
```
