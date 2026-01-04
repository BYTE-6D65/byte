# Byte Roadmap

**Last Updated**: 2026-01-03

---

## Current Features (v0.4.0)

- Multi-workspace project discovery and management
- Command execution with build animation and logging
- Git status tracking (branch, modified/staged/untracked, ahead/behind)
- Build state persistence and display
- Full-screen log preview with word wrapping and scrolling
- File watching and auto-reload
- Command log categorization (Build/Lint/Git/Test/Other tabs)
- Fuzzy directory matching with tab completion
- Forms system for user input
- ISPF-inspired keyboard navigation

---

## Planned Features

### Phase 1: Core Workflow (v0.5.0)

#### New Project Creation Flow
**Priority**: CRITICAL
**Status**: Missing fundamental workflow

Currently byte has:
- ✅ Tools to initialize projects (`byte init`, `init_project()` API)
- ✅ Forms system for input collection
- ✅ Workspace management
- ❌ **No TUI flow to create new projects interactively**

Proposed flow:
1. Press `n` in project browser → Enter "Create New Project" mode
2. Multi-step form using existing forms system:
   - **Step 1**: Select target workspace (from registered workspaces)
   - **Step 2**: Choose ecosystem (Rust/Go/Bun/Node)
   - **Step 3**: Choose project type (CLI/Web/Library)
   - **Step 4**: Enter project name (with validation)
   - **Step 5**: Optional description
3. Show confirmation with full path preview
4. Execute creation using existing `init_project()` function
5. Auto-reload projects and select newly created project

Implementation notes:
- Reuse `forms/mod.rs` for multi-step input
- Reuse `projects::init_project()` for actual creation
- Reuse `validate_project_name()` for name validation
- Add workspace picker (similar to fuzzy path picker)
- All infrastructure exists - just needs the flow wired up

---

#### Real-time Command Output
**Priority**: HIGH
- Stream stdout/stderr as commands execute
- Color-coded output in split panel
- Auto-scroll with pause option

#### Search/Filter Projects
**Priority**: HIGH
- Press `/` to enter search mode
- Filter by name, path, ecosystem, or tags
- Incremental filtering

#### Custom Commands Per Project
**Priority**: HIGH
- Define in byte.toml: `deploy`, `db:migrate`, etc.
- Merge with global commands in Command Palette
- Visual indicator for project-specific commands

---

### Phase 2: Power Features (v0.6.0)

#### Batch Command Execution
**Priority**: HIGH
- Mark projects with Space
- Run same command across selected projects
- Sequential or parallel execution
- Progress indicator

#### Command History
**Priority**: MEDIUM
- Track executed commands with timestamps
- Press `h` to view history
- Re-run or edit before running
- Persist to `.byte/history.json`

#### Project Bookmarks
**Priority**: MEDIUM
- Press `*` to star/unstar
- Starred projects at top of list

---

### Phase 3: Quality of Life (v0.7.0)

#### Better Help System
**Priority**: MEDIUM
- `?` for contextual help per view
- Searchable keybinding reference

#### Command Templates
**Priority**: MEDIUM
- Variable substitution: `{project_name}`, `{ecosystem}`
- Reusable command patterns

#### Arrow Key Path Navigation
**Priority**: LOW
- Fish/zsh-style inline completion
- Right arrow accepts ghost text
- Ghost text preview

---

### Phase 4: Polish (v0.8.0+)

#### Input Focus Management API
- Centralized focus system for modals
- Replace scattered input guards
- Focus stack for overlays

#### Color Themes
- Multiple schemes (default, gruvbox, solarized)
- Configurable in config.toml

#### More Ecosystems
- Python (venv, setup.py)
- Node (npm init)
- Deno (deno.json)

---

## Technical Debt

See [../audit/AUDIT_STATUS.md](../audit/AUDIT_STATUS.md) for:
- Remaining API abstractions
- Memory safety improvements
- TUI refactoring needs

---

## Design Principles

1. **ISPF Philosophy**: Keyboard-first, dense info, minimal mouse
2. **Reduce Keystrokes**: Every feature should speed up common workflows
3. **Scale to Monorepos**: Must work well with 50+ projects
4. **Trust the Developer**: byte.toml is trusted config, not user input
