# Byte Roadmap

## Current Features (v0.4.0)
- ✅ Multi-workspace project discovery and management
- ✅ Command execution with build animation
- ✅ Git status tracking (branch, modified/staged/untracked counts, ahead/behind)
- ✅ Build state persistence and display
- ✅ Full-screen log preview with word wrapping and scrolling
- ✅ File watching and auto-reload
- ✅ Command log categorization (build/lint/git/other)
- ✅ Fuzzy directory matching
- ✅ ISPF-inspired keyboard-driven navigation
- ✅ **Command type filtering** (horizontal tabs: All/Build/Lint/Git/Test/Other)
- ✅ **Forms system** for user input collection (text, textarea, email, number, select, multiselect, checkbox, path)

## Proposed Features

### High Impact - Core Workflow

#### Real-time Command Output
**Priority: High**
- Stream stdout/stderr as commands execute (like tail -f)
- Show live output in a split panel or dedicated view
- Color-coded output (stdout vs stderr)
- Auto-scroll to bottom with option to pause
- **Value:** Immediate feedback, no waiting for command completion

#### Search/Filter Projects
**Priority: High**
- Press `/` to enter search mode
- Filter projects by name, path, ecosystem, or tags
- Incremental search with live filtering
- Clear with Esc, navigate filtered results with arrows
- **Value:** Quick access to projects in large workspaces

#### Command Type Filtering (Menu 2 Reorganization)
**Priority: High**
- Horizontal tab bar at top: `[All] [Build] [Lint] [Git] [Test] [Other]`
- Arrow keys or numbers to switch filter
- Command list updates to show only selected category
- Visual indicator for active filter
- **Value:** Reduces cognitive load, faster command discovery

#### Quick Jump / Direct Access
**Priority: Medium**
- Type project number or first few letters to jump directly
- Command line at bottom: `:goto byte` or `:3` to jump to project 3
- ISPF-style line commands
- **Value:** Power users can navigate without arrow key spam

### Quality of Life

#### Command History
**Priority: Medium**
- Track all executed commands with timestamps
- Press `h` to view history panel
- Re-run with Enter, edit before running
- Persist history to `.byte/history.json`
- Show last run time and exit code in list
- **Value:** Quick re-execution, troubleshooting reference

#### Project Bookmarks/Favorites
**Priority: Medium**
- Press `*` to star/unstar current project
- Starred projects appear at top of list with ⭐ indicator
- Persist to `.byte/favorites.json`
- **Value:** Faster access to frequently used projects

#### Custom Commands Per Project
**Priority: High**
- Define project-specific commands in `byte.toml`
- Example: `deploy`, `db:migrate`, `docker:up`
- Merge with global commands in menu 2
- Visual indicator for project-specific vs global commands
- **Value:** Contextual tooling, less mental overhead

#### Command Templates
**Priority: Medium**
- Variable substitution in commands
- `{project_name}`, `{project_path}`, `{ecosystem}`
- Example: `docker build -t {project_name}:latest .`
- **Value:** DRY, reusable command patterns

#### Open in Editor
**Priority: Low**
- Press `e` to open project root in $EDITOR
- Press `E` to open specific file (fuzzy search)
- Integrates with $EDITOR, $VISUAL env vars
- **Value:** Quick context switching

### Power User Features

#### Batch Command Execution
**Priority: High**
- Run same command across multiple projects
- Mark projects with Space, execute with `b`
- Confirmation prompt showing selected projects
- Execute sequentially or in parallel (configurable)
- Real-time progress indicator (3/10 projects completed)
- **Use cases:**
  - Update dependencies across all Rust projects
  - Run tests on all changed projects
  - Deploy multiple services
- **Remote execution:** Could extend to SSH targets in future
- **Value:** Massive time saver for monorepo workflows

#### Remote/SSH Execution
**Priority: Low (Future)**
- Execute commands on remote targets
- Configure SSH targets in byte.toml
- Batch operations across multiple servers
- **Value:** Infrastructure management, multi-environment deployments

#### Command Chaining
**Priority: Medium**
- Define command sequences: `build → test → deploy`
- Stop on first failure or continue
- Visual pipeline progress indicator
- Save common chains as named workflows
- **Value:** One-button complex operations

#### Process Viewer/Manager
**Priority: Low**
- View running processes spawned by Byte
- Kill hung processes from TUI
- See process tree and resource usage
- Detect orphaned processes from previous sessions
- **Value:** Process lifecycle management

#### Environment Variable Management
**Priority: Low**
- Load `.env` files automatically
- View/edit project environment variables
- Override global env vars per project
- Visual indicator when env vars are active
- **Value:** Simplified environment configuration

### Polish & UX

#### Better Help System
**Priority: Medium**
- Press `?` for contextual help in each view
- Show available keybindings for current context
- Quick command reference
- Searchable help text
- **Value:** Discoverability, reduced learning curve

#### Color Themes
**Priority: Low**
- Multiple color schemes (default, gruvbox, solarized, **IBM green screen**)
- Configure in `~/.config/byte/config.toml`
- Live theme switching with `:theme <name>`
- **Value:** Personalization, accessibility

#### Command Output Search
**Priority: Low**
- Grep through log files from TUI
- Press `/` in log preview to search within file
- Highlight matches, jump to next/previous
- **Value:** Faster debugging, less tool switching

#### Log Retention Management
**Priority: Low**
- Configure retention policy (days, count, size)
- Manual cleanup command
- Show total log disk usage
- **Value:** Prevent disk bloat

### Technical Debt & Improvements

#### Consistent Keybindings
**Priority: Medium**
- Audit and standardize key mappings across views
- Create keybinding reference (`docs/KEYBINDINGS.md`)
- Make keybindings configurable
- **Value:** Consistent UX, less confusion

#### Input Focus Management API
**Priority: Medium**
- Centralized focus system for modals and input contexts
- Replace scattered input guards (~30+ locations) with focus-aware routing
- Focus stack for overlay management (forms, dialogs, popups)
- Prevents input crosstalk between surfaces (e.g., forms vs view switching)
- **Implementation:**
  - `FocusContext` enum (Normal, Form, Dialog, InputBuffer)
  - `has_focus()` / `take_focus()` / `release_focus()` methods
  - Event routing: `route_input(key) -> bool` (returns true if consumed)
  - Focus stack for nested modals
- **Value:** Cleaner code, prevents input bugs, easier to add new modals
- **Effort:** High (architectural change, touches key handler logic)
- **Related:** Aligns with audit recommendation for UI component abstraction

#### Error Handling
**Priority: Medium**
- Better error messages for config parsing
- Graceful degradation when git/commands unavailable
- Recovery suggestions in error messages
- **Value:** Better DX, fewer frustrating moments

#### Performance Optimization
**Priority: Low**
- Profile git status checks
- Cache project discovery results
- Lazy-load command logs
- **Value:** Snappier feel, scales to large workspaces

## Implementation Priority

**Phase 1 - Core Workflow (v0.4.0)**
1. ✅ Command Type Filtering (Menu 2 tabs) - COMPLETED
2. Real-time Command Output
3. Custom Commands Per Project

**Phase 2 - Power Features (v0.5.0)**
4. Batch Command Execution
5. Search/Filter Projects
6. Command History

**Phase 3 - Quality of Life (v0.6.0)**
7. Project Bookmarks
8. Command Templates
9. Better Help System

**Phase 4 - Polish (v0.7.0+)**
10. Input Focus Management API
11. Color Themes (including IBM green!)
12. Consistent Keybindings Overhaul
13. Command Output Search

## Open Questions

- **Batching scope:** Local-only first, or design for eventual remote execution?
- **Command categories:** Hard-coded (build/lint/git/test) or user-definable?
- **Real-time output:** Replace current view or split screen?
- **Project selection:** Multi-select with Space bar or Ctrl modifier?

## Notes

- Keep ISPF philosophy: keyboard-first, dense info, minimal mouse
- Every feature should reduce keystrokes for common workflows
- Prioritize features that scale to large monorepos (50+ projects)
