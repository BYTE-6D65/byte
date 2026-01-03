# Byte Features & Roadmap

Planned features with implementation specifications.

**Status:** Planning
**Target Version:** 0.2.0

---

## Feature 1: Interactive Workspace Manager

**Priority:** HIGH
**Complexity:** Medium
**Status:** 📋 Planned

### Overview

Allow users to interactively add/remove workspace directories that Byte scans for projects. Currently, users must manually edit `~/.config/byte/config.toml`. This feature adds a TUI view for managing scan directories.

### User Story

```
As a developer with projects in multiple directories,
I want to easily add/remove scan paths from the TUI,
So that I don't have to manually edit config files.
```

### Current Behavior

```toml
# ~/.config/byte/config.toml
[workspace]
path = "~/projects"           # Primary workspace (auto-scanned)
auto_scan = true
registered = []               # Additional paths (must edit manually)
```

### Proposed Behavior

**New TUI View: Tab 4 - Workspace Manager**

```
┌─ Workspace Manager ─────────────────────────────────────────┐
│                                                              │
│ Scanned Directories                                       3  │
│                                                              │
│   ~/projects                      [auto-scan: ON ] [primary]│
│   ~/work/clients                  [remove]                  │
│   ~/github/forks                  [remove]                  │
│                                                              │
│ ────────────────────────────────────────────────────────────│
│                                                              │
│ Press 'a' to add directory                                  │
│ Press 'd' to remove selected                                │
│ Press 's' to toggle auto-scan                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### UI Flow

**1. Add Directory (Press 'a')**

```
┌─ Add Workspace Directory ───────────────────────────────────┐
│                                                              │
│ Enter directory path:                                        │
│ ┌──────────────────────────────────────────────────────────┐│
│ │ ~/                                                       ││
│ └──────────────────────────────────────────────────────────┘│
│                                                              │
│ [Tab] Complete  [Enter] Add  [Esc] Cancel                   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**2. After Adding**

- Validate path exists
- Expand tilde (`~/` → `/Users/liam/`)
- Add to `workspace.registered` array
- Save to `~/.config/byte/config.toml`
- Re-scan for projects
- Update status: "Added ~/work/clients - found 5 projects"

**3. Remove Directory (Press 'd' on selected)**

```
┌─ Remove Directory ──────────────────────────────────────────┐
│                                                              │
│ Remove ~/work/clients from workspace?                       │
│                                                              │
│ This will stop scanning this directory.                     │
│ Projects will disappear from the browser.                   │
│                                                              │
│ [Enter] Confirm  [Esc] Cancel                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Implementation Plan

#### Phase 1: Data Model

**Add to App state:**
```rust
pub struct App {
    // ... existing fields
    pub workspace_directories: Vec<WorkspaceDir>,
    pub selected_workspace: usize,
    pub workspace_list_state: ListState,
    pub input_mode: InputMode,
    pub input_buffer: String,
}

pub struct WorkspaceDir {
    pub path: PathBuf,
    pub is_primary: bool,
    pub auto_scan: bool,
    pub project_count: usize,
}

pub enum InputMode {
    Normal,
    AddingDirectory,
}
```

#### Phase 2: Config Management

**Add functions to `config/mod.rs`:**
```rust
impl Config {
    pub fn add_workspace_path(&mut self, path: &str) -> Result<()> {
        // Expand tilde
        // Validate path exists
        // Add to workspace.registered
        // Save to file
    }

    pub fn remove_workspace_path(&mut self, path: &str) -> Result<()> {
        // Remove from workspace.registered
        // Save to file
    }

    pub fn save(&self) -> Result<()> {
        // Serialize to TOML
        // Write to ~/.config/byte/config.toml
    }
}
```

#### Phase 3: TUI View

**Create `src/tui/views/workspace_manager.rs`:**
```rust
pub fn render_workspace_manager(f: &mut Frame, area: Rect, app: &App) {
    // Title: "Workspace Manager"
    // List of directories with stats
    // Help text at bottom
}
```

**Add to View enum:**
```rust
pub enum View {
    ProjectBrowser,
    CommandPalette,
    Detail,
    WorkspaceManager,  // NEW
}
```

#### Phase 4: Input Handling

**Update `App::handle_key()`:**
```rust
KeyCode::Char('4') => {
    self.current_view = View::WorkspaceManager;
}

// In WorkspaceManager view:
KeyCode::Char('a') => {
    self.input_mode = InputMode::AddingDirectory;
    self.input_buffer.clear();
}

KeyCode::Char('d') => {
    // Remove selected directory
}

KeyCode::Enter if self.input_mode == InputMode::AddingDirectory => {
    // Add directory from input_buffer
    config.add_workspace_path(&self.input_buffer)?;
    // Re-discover projects
    // Update app.projects
}
```

#### Phase 5: Path Completion (Optional)

**Tab completion for directory input:**
- Use `std::fs::read_dir()` to list directories
- Filter by current input prefix
- Cycle through matches on Tab

### Testing

**Manual Test Cases:**

1. ✅ Add valid directory → appears in list, projects discovered
2. ✅ Add invalid directory → error message shown
3. ✅ Add duplicate directory → show "already exists" message
4. ✅ Remove directory → disappears from list, config updated
5. ✅ Remove primary workspace → prevent (show warning)
6. ✅ Add directory with tilde → expands correctly
7. ✅ Add directory with spaces → handles correctly

### Files to Modify

```
src/config/mod.rs           # Add save(), add_workspace_path(), remove_workspace_path()
src/tui/mod.rs              # Add View::WorkspaceManager, InputMode, input_buffer
src/tui/views/workspace_manager.rs  # NEW FILE
src/tui/views/mod.rs        # Export workspace_manager
```

### Future Enhancements

- Drag-and-drop directory reordering
- Per-directory auto-scan toggle
- Exclude patterns (`.git/`, `node_modules/`)
- Directory aliases ("Work", "Personal")

---

## Feature 2: Git Integration

**Priority:** HIGH
**Complexity:** High
**Status:** 📋 Planned

### Overview

Add git operations to the TUI, allowing users to check status, pull, push, manage branches, etc. without leaving Byte. All git commands execute in the project directory and log output to `.byte/logs/git.log`.

### User Story

```
As a developer managing multiple projects,
I want to perform common git operations from the Byte TUI,
So that I can sync and manage repos without switching contexts.
```

### Current Behavior

- TUI shows project details (path, ecosystem, description)
- No git information or actions available
- User must `cd` to project and run git manually

### Proposed Behavior

**Enhanced Detail View (Tab 3)**

```
┌─ my-project ────────────────────────────────────────────────┐
│                                                              │
│ rust cli project                                             │
│                                                              │
│ PATH                                                         │
│ ~/projects/my-project                                        │
│                                                              │
│ ECOSYSTEM                                                    │
│   ● rust  active                                             │
│                                                              │
│ GIT STATUS                                                   │
│   ✓ On branch main                                           │
│   ✓ Up to date with origin/main                              │
│   ⚠ 2 files modified, 1 file untracked                       │
│                                                              │
│ ────────────────────────────────────────────────────────────│
│                                                              │
│ Git Actions:                                                 │
│   g status  p pull  P push  b branches  c checkout  l log   │
│                                                              │
│ Press 1 to return to projects                                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Git Commands

#### 1. Status (Press 'g')

**Command:** `git status --short --branch`

**Display:**
```
┌─ Git Status: my-project ────────────────────────────────────┐
│ ## main...origin/main [ahead 1]                              │
│  M src/main.rs                                               │
│  M Cargo.toml                                                │
│ ?? new-file.txt                                              │
│                                                              │
│ Press any key to continue                                    │
└──────────────────────────────────────────────────────────────┘
```

**Log:** Append to `.byte/logs/git.log`

#### 2. Pull (Press 'p')

**Command:** `git pull --rebase`

**Flow:**
1. Show "Pulling from origin..." status
2. Execute `git pull --rebase` in project dir
3. Stream output to `.byte/logs/git.log`
4. Show success/failure message
5. Update git status display

**Success:**
```
✓ Pulled from origin/main - 3 commits ahead
```

**Failure:**
```
✗ Pull failed: merge conflict in src/main.rs
  See ~/.byte/logs/git.log for details
```

#### 3. Push (Press 'P' - capital)

**Command:** `git push`

**Confirmation:**
```
┌─ Push to Remote ────────────────────────────────────────────┐
│                                                              │
│ Push 1 commit to origin/main?                               │
│                                                              │
│ [Enter] Confirm  [Esc] Cancel                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### 4. Branches (Press 'b')

**Command:** `git branch -a`

**Display:**
```
┌─ Branches: my-project ──────────────────────────────────────┐
│                                                              │
│ Local Branches:                                              │
│ ▸ main                              [current]                │
│   feature/new-ui                    [2 commits ahead]        │
│   bugfix/memory-leak                [stale]                  │
│                                                              │
│ Remote Branches:                                             │
│   origin/main                                                │
│   origin/develop                                             │
│                                                              │
│ Press 'c' on branch to checkout  [Esc] Close                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### 5. Checkout (Press 'c' in branch list)

**Command:** `git checkout <branch>`

**Confirmation:**
```
┌─ Checkout Branch ───────────────────────────────────────────┐
│                                                              │
│ Switch to branch 'feature/new-ui'?                          │
│                                                              │
│ Current branch: main                                         │
│ Target branch: feature/new-ui                               │
│                                                              │
│ [Enter] Confirm  [Esc] Cancel                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

#### 6. Log (Press 'l')

**Command:** `git log --oneline -10`

**Display:**
```
┌─ Recent Commits: my-project ────────────────────────────────┐
│ a3d9f2c Add new feature                                      │
│ 8b4e1a7 Fix bug in parser                                    │
│ c5f6d3e Update dependencies                                  │
│ 2a1b9c4 Refactor main module                                 │
│ 7e8f3d2 Initial commit                                       │
│                                                              │
│ Press any key to continue                                    │
└──────────────────────────────────────────────────────────────┘
```

### Implementation Plan

#### Phase 1: Git Status Module

**Create `src/git.rs`:**
```rust
use std::path::Path;
use std::process::Command;
use anyhow::Result;

pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub modified: usize,
    pub untracked: usize,
    pub staged: usize,
}

pub fn get_status(project_path: &Path) -> Result<GitStatus> {
    let output = Command::new("git")
        .args(&["status", "--short", "--branch"])
        .current_dir(project_path)
        .output()?;

    // Parse output
    // Return GitStatus
}

pub fn git_pull(project_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(&["pull", "--rebase"])
        .current_dir(project_path)
        .output()?;

    // Log to .byte/logs/git.log
    // Return output
}

pub fn git_push(project_path: &Path) -> Result<String> {
    // Similar to pull
}

pub fn git_branches(project_path: &Path) -> Result<Vec<Branch>> {
    // Parse git branch -a output
}

pub fn git_checkout(project_path: &Path, branch: &str) -> Result<()> {
    // git checkout <branch>
}

pub fn git_log(project_path: &Path, count: usize) -> Result<Vec<Commit>> {
    // Parse git log --oneline output
}
```

#### Phase 2: Update App State

**Add to `App` in `src/tui/mod.rs`:**
```rust
pub struct App {
    // ... existing fields
    pub git_status: Option<GitStatus>,
    pub git_view_mode: GitViewMode,
    pub selected_branch: usize,
    pub branches: Vec<Branch>,
}

pub enum GitViewMode {
    Normal,
    Branches,
    Log,
    ConfirmPush,
    ConfirmCheckout(String),
}
```

#### Phase 3: Update Detail View

**Modify `src/tui/views/detail.rs`:**
```rust
pub fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    // ... existing project details

    // Add git status section
    if let Some(project) = app.get_selected_project() {
        if let Ok(status) = git::get_status(&project.path) {
            render_git_status(f, area, &status);
            render_git_actions(f, area);
        }
    }
}

fn render_git_status(f: &mut Frame, area: Rect, status: &GitStatus) {
    // Display branch, ahead/behind, file changes
}

fn render_git_actions(f: &mut Frame, area: Rect) {
    // Display "g status  p pull  P push  b branches  c checkout  l log"
}
```

#### Phase 4: Input Handling

**Update `App::handle_key()` in Detail view:**
```rust
View::Detail => match key {
    KeyCode::Char('g') => {
        // Run git status, show in modal
    }
    KeyCode::Char('p') => {
        // Run git pull, update status
        self.status_message = "Pulling from origin...".to_string();
        // Async or blocking?
    }
    KeyCode::Char('P') => {
        // Show confirm push modal
        self.git_view_mode = GitViewMode::ConfirmPush;
    }
    KeyCode::Char('b') => {
        // Load branches, show branch list
        self.git_view_mode = GitViewMode::Branches;
    }
    KeyCode::Char('l') => {
        // Show git log
        self.git_view_mode = GitViewMode::Log;
    }
    // ... existing keys
}
```

#### Phase 5: Logging

**Update `src/logger.rs`:**
```rust
pub fn git_log(project_name: &str, command: &str, output: &str) -> Result<()> {
    let log_path = project_path.join(".byte/logs/git.log");
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!("[{}] {} > {}\n{}\n\n", timestamp, project_name, command, output);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    file.write_all(entry.as_bytes())?;
    Ok(())
}
```

**Log Format:**
```
[2025-12-30 14:30:15] my-project > git pull --rebase
Already up to date.

[2025-12-30 14:31:42] my-project > git push
Enumerating objects: 5, done.
...
```

### Safety Considerations

1. **No Force Operations** - Never run `git push --force` without explicit confirmation
2. **Clean Working Directory** - Warn before destructive operations if uncommitted changes
3. **Branch Protection** - Prevent accidental deletion of main/master
4. **Command Validation** - Sanitize any user input used in git commands
5. **Error Handling** - Always show clear error messages, never fail silently

### Files to Create/Modify

```
src/git.rs                  # NEW - Git operations module
src/lib.rs                  # Export git module
src/main.rs                 # Import git module
src/logger.rs               # Add git_log() function
src/tui/mod.rs              # Add GitStatus, GitViewMode to App
src/tui/views/detail.rs     # Add git status display and actions
src/tui/views/git_branches.rs  # NEW - Branch list view
src/tui/views/git_log.rs    # NEW - Log view
```

### Testing

**Manual Test Cases:**

1. ✅ Git status shows correct branch and file counts
2. ✅ Pull with fast-forward → updates cleanly
3. ✅ Pull with conflicts → shows error, doesn't corrupt repo
4. ✅ Push with upstream set → pushes correctly
5. ✅ Push without upstream → shows error or prompts to set
6. ✅ Checkout existing branch → switches correctly
7. ✅ Checkout with uncommitted changes → warns user
8. ✅ Branch list shows local and remote branches
9. ✅ Git log shows recent commits
10. ✅ All operations log to .byte/logs/git.log

### Future Enhancements

- Commit from TUI (with message input)
- Diff viewer
- Stash management
- Remote management
- Cherry-pick operations
- Interactive rebase (advanced)

---

## Feature 3: Project Search/Filter

**Priority:** Medium
**Complexity:** Low
**Status:** 📋 Planned

### Overview

Add search/filter to project browser to quickly find projects by name, ecosystem, or path.

**Key Binding:** `/` to enter search mode

**Implementation:** Filter `app.projects` based on input, update displayed list.

---

## Feature 4: More Ecosystems

**Priority:** Medium
**Complexity:** Low
**Status:** 📋 Planned

### Planned Ecosystems

- **Python** - `byte init python cli my-script` → creates venv, setup.py
- **Node** - `byte init node web my-app` → npm init
- **Deno** - `byte init deno cli my-tool` → deno.json
- **Wails** - `byte init wails desktop my-app` → wails init

**Implementation:** Add `init_<ecosystem>_project()` functions to `src/projects.rs`.

---

## Feature 5: Project Templates

**Priority:** Low
**Complexity:** Medium
**Status:** 💭 Idea

### Overview

Allow custom project templates stored in `~/.config/byte/templates/`.

**Usage:**
```bash
byte init --template=my-go-api go api my-service
```

**Template Structure:**
```
~/.config/byte/templates/my-go-api/
├── template.toml       # Template metadata
├── cmd/
├── internal/
└── README.md
```

---

## Implementation Priority

**Phase 1 (v0.2.0):**
1. ✅ Workspace Manager (high value, medium complexity)
2. ✅ Git Integration - Status & Pull (high value, foundational)

**Phase 2 (v0.3.0):**
3. Git Integration - Push, Branches, Checkout (builds on Phase 1)
4. Project Search/Filter (quick win)

**Phase 3 (v0.4.0):**
5. More Ecosystems (Python, Node, Deno)
6. Project Templates (low priority, nice-to-have)

---

## Success Criteria

**Workspace Manager:**
- ✅ Can add directory from TUI in < 10 seconds
- ✅ Config saved correctly without manual editing
- ✅ Projects from new directory appear immediately

**Git Integration:**
- ✅ Git status shown for all projects with repos
- ✅ Pull/push work without leaving TUI
- ✅ All operations logged to .byte/logs/git.log
- ✅ No repo corruption from failed operations

---

**Next Steps:**

1. Review this spec
2. Start with Workspace Manager (simpler, foundational)
3. Implement Git Integration in phases (status → pull → push → branches)
4. User testing at each phase
