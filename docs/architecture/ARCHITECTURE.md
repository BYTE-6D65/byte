# Byte Architecture

Current system design and implementation details.

**Last Updated:** 2025-12-30
**Version:** 0.1.0

---

## Overview

Byte is a simple project orchestration CLI that helps developers:
- Initialize projects with consistent structure
- Auto-discover projects via `byte.toml` manifests
- Browse projects in a clean TUI
- Organize runtime data in `.byte/` directories

**Philosophy:** Simple over complex. No driver abstractions, no command wrappers—just init, organize, and browse.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         User                                │
└────────────┬────────────────────────────────┬───────────────┘
             │                                │
             ▼                                ▼
      ┌─────────────┐                 ┌─────────────┐
      │  CLI Entry  │                 │     TUI     │
      │  (main.rs)  │                 │  (tui/*)    │
      └──────┬──────┘                 └──────┬──────┘
             │                                │
             ▼                                │
      ┌─────────────┐                        │
      │ CLI Handler │◄───────────────────────┘
      │  (cli/*)    │
      └──────┬──────┘
             │
             ├──────────┬──────────────┬─────────────┐
             ▼          ▼              ▼             ▼
      ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
      │  Config  │ │ Projects │ │  Logger  │ │   TUI    │
      │ (config/)│ │(projects)│ │ (logger) │ │  (tui/)  │
      └──────────┘ └──────────┘ └──────────┘ └──────────┘
             │          │             │             │
             ▼          ▼             ▼             ▼
      ┌──────────────────────────────────────────────────┐
      │              Filesystem & Git                    │
      │  ~/.config/byte/    ~/projects/    .byte/logs/   │
      └──────────────────────────────────────────────────┘
```

---

## Core Modules

### 1. CLI (`src/cli/`)

**Purpose:** Command-line interface and argument parsing

**Files:**
- `mod.rs` - Main CLI runner
- `args.rs` - Argument definitions (unused, uses clap derive)
- `commands.rs` - Command implementations (unused, inline in mod.rs)

**Commands:**
```rust
byte init <ecosystem> <type> <name>  // Initialize new project
byte tui                              // Launch TUI
byte                                  // Default: launch TUI
```

**Key Functions:**
- `cli::run()` - Parse args and route to handlers
- Calls `projects::init_project()` for init
- Calls `tui::run()` for TUI

### 2. Config (`src/config/`)

**Purpose:** Configuration loading and types

**Files:**
- `mod.rs` - Config loading logic
- `types.rs` - Data structures

**Config Hierarchy:**
```toml
# Global: ~/.config/byte/config.toml
[workspace]
path = "~/projects"
auto_scan = true
registered = []  # Manual project paths

[tui]
refresh_rate_ms = 16
animations = true
default_view = "browser"

# Project: <project>/byte.toml
[project]
name = "my-project"
type = "cli"
ecosystem = "rust"
description = "Optional description"  # Optional
```

**Key Types:**
- `GlobalConfig` - Workspace, TUI, logging settings
- `ProjectConfig` - Project metadata
- `Config` - Combines global + optional project config

**Loading:**
1. Check `~/.config/byte/config.toml`
2. Check `./byte.toml` (current dir)
3. Fall back to defaults

### 3. Projects (`src/projects.rs`)

**Purpose:** Project discovery and initialization

**Key Functions:**

```rust
// Discover all projects in workspace
pub fn discover_projects(global_config: &GlobalConfig)
    -> Result<Vec<DiscoveredProject>>

// Initialize new project
pub fn init_project(
    workspace_path: &str,
    ecosystem: &str,
    project_type: &str,
    name: &str,
) -> Result<PathBuf>
```

**Discovery Process:**
1. If `auto_scan` enabled: Walk workspace up to 3 levels deep
2. Look for `byte.toml` files
3. Load and parse each config
4. Add manually `registered` paths
5. Return list of `DiscoveredProject`

**Init Process:**
1. Create `~/projects/<name>/`
2. Create `.byte/logs/` and `.byte/state/`
3. Write `byte.toml` with project metadata
4. Call ecosystem-specific init:
   - `init_go_project()` - Creates cmd/pkg/internal, runs `go mod init`
   - `init_bun_project()` - Creates src/, runs `bun init`
   - `init_rust_project()` - Runs `cargo init`
5. Add `.byte/` to `.gitignore`

**Supported Ecosystems:**
- `rust` - Cargo-based projects
- `go` - Go modules
- `bun` - Bun/TypeScript projects

### 4. TUI (`src/tui/`)

**Purpose:** Terminal user interface

**Structure:**
```
tui/
├── mod.rs              # Main TUI logic, theme, App struct
├── app.rs              # Unused (moved to mod.rs)
└── views/
    ├── mod.rs          # View exports
    ├── project_browser.rs
    ├── command_palette.rs
    └── detail.rs
```

**App State:**
```rust
pub struct App {
    pub should_quit: bool,
    pub current_view: View,
    pub projects: Vec<Project>,        // Discovered projects
    pub commands: Vec<Command>,        // Init commands
    pub selected_project: usize,
    pub selected_command: usize,
    pub project_list_state: ListState,
    pub command_list_state: ListState,
    pub status_message: String,
}
```

**Views:**
1. **ProjectBrowser (tab 1)** - List all projects with descriptions and ecosystem tags
2. **CommandPalette (tab 2)** - Quick init commands
3. **Detail (tab 3)** - Selected project details (path, ecosystem, drivers)

**Theme:**
- OLED-optimized colors in `theme` module
- See `TUI_STYLING_GUIDE.md` for full design system

**Key Bindings:**
- `1/2/3` - Switch views
- `↑↓` - Navigate lists
- `Enter` - Select item
- `?` - Help
- `q` - Quit

**Initialization:**
- `App::new()` loads config and discovers projects
- Converts `DiscoveredProject` to TUI `Project` struct
- Displays count in status message

### 5. Logger (`src/logger.rs`)

**Purpose:** Simple file logging

**Functions:**
- `info(msg)` - Log info message
- `error(msg)` - Log error message
- `debug(msg)` - Log debug message

**Log Location:** `target/logs/byte.log` (relative to CWD)

**Format:** `LEVEL: message`

---

## Data Flow

### Init Flow

```
User: byte init rust cli my-tool
  │
  ▼
cli::run() parses args
  │
  ▼
Config::load() reads ~/.config/byte/config.toml
  │
  ▼
projects::init_project(~/projects, rust, cli, my-tool)
  │
  ├─► Create ~/projects/my-tool/.byte/{logs,state}/
  ├─► Write byte.toml
  ├─► init_rust_project() → cargo init
  └─► add_to_gitignore() → add .byte/ to .gitignore
  │
  ▼
Print success message
```

### Discovery Flow

```
User: byte tui
  │
  ▼
tui::run() → setup_terminal()
  │
  ▼
App::new()
  │
  ├─► Config::load() → GlobalConfig
  ├─► projects::discover_projects(&config.global)
  │     │
  │     ├─► If auto_scan: walkdir ~/projects/ for byte.toml
  │     ├─► For each registered path: load_project()
  │     └─► Return Vec<DiscoveredProject>
  │
  ├─► Convert to Vec<Project> for TUI
  └─► Set status message
  │
  ▼
run_app() → event loop
  │
  ├─► draw() renders current view
  └─► handle_key() processes input
```

---

## File Structure

### Project Directory

```
~/projects/my-project/
├── .byte/                  # Byte runtime data (gitignored)
│   ├── logs/              # Future: command outputs
│   └── state/             # Future: metadata, cache
├── .git/                  # Git repository
├── .gitignore             # Includes .byte/
├── byte.toml              # Project manifest
└── [ecosystem files]      # Cargo.toml, go.mod, package.json, etc.
```

### Byte Config

```
~/.config/byte/
└── config.toml            # Global configuration
```

### Source Code

```
src/
├── cli/
│   ├── mod.rs            # CLI runner
│   ├── args.rs           # Unused stub
│   └── commands.rs       # Unused stub
├── config/
│   ├── mod.rs            # Config loading
│   └── types.rs          # Data structures
├── tui/
│   ├── mod.rs            # TUI main, theme, App
│   ├── app.rs            # Unused (moved to mod.rs)
│   └── views/
│       ├── mod.rs
│       ├── project_browser.rs
│       ├── command_palette.rs
│       └── detail.rs
├── projects.rs           # Discovery & init
├── logger.rs             # Logging
├── lib.rs                # Library exports
└── main.rs               # Binary entry
```

---

## Key Design Decisions

### 1. No Driver System

**Decision:** Don't abstract ecosystem tools behind a driver interface

**Rationale:**
- User wants to run native commands (`go build`, `cargo run`)
- Byte only handles init and organization
- Simpler codebase, easier to maintain

**Alternative Considered:** Driver plugin system with capabilities (rejected as over-engineered)

### 2. Simple Config Format

**Decision:** Flat TOML with minimal nesting

**Rationale:**
- Easy to read and edit manually
- Just metadata for display, not execution
- No complex validation needed

**Example:**
```toml
[project]
name = "my-app"
type = "cli"
ecosystem = "rust"
```

### 3. .byte/ for Runtime Data

**Decision:** All Byte-managed data in `.byte/` directory

**Rationale:**
- Clean separation from project files
- Easy to gitignore entire directory
- Conventional (like `.git/`, `.vscode/`)
- Future-proof for logs, cache, metadata

### 4. Discovery via Scanning

**Decision:** Auto-scan workspace for `byte.toml` files

**Rationale:**
- No manual project registration required
- Just drop a project in `~/projects/` and it appears
- Supports manual registration too via `workspace.registered`

**Limitation:** Max depth of 3 levels to avoid slow scans

---

## Dependencies

### Core

- **clap** (4.5) - CLI argument parsing with derive macros
- **serde** (1.0) - Serialization for config
- **toml** (0.8) - TOML config parsing
- **anyhow** (1.0) - Error handling

### TUI

- **ratatui** (0.29) - Terminal UI framework
- **crossterm** (0.28) - Terminal control

### Utilities

- **walkdir** (2.5) - Directory traversal for discovery
- **shellexpand** (3) - Tilde expansion in paths
- **dirs** (5.0) - Standard directory locations

### All MIT-compatible (MIT, Apache-2.0, Unlicense)

---

## Limitations & Future Work

### Current Limitations

1. **No Command Execution** - Can't run `byte dev` or `byte build`
2. **No Git Integration** - Can't see status or perform git ops from TUI
3. **No Project Search** - Can't filter projects in TUI
4. **Limited Ecosystems** - Only rust, go, bun
5. **No Templates** - Can't customize init structure

### Planned Features (see FEATURES.md)

- Interactive directory scanner
- Git operations in TUI
- Project search/filter
- More ecosystems (Python, Node, Deno)
- Custom templates

---

## Testing

Currently: Manual testing only

**Future:**
- Unit tests for discovery logic
- Integration tests for init process
- TUI snapshot tests

---

## Performance

**Discovery:**
- Scans up to 3 levels deep
- ~100ms for typical ~/projects with 20 projects
- Could be optimized with parallel scanning

**TUI:**
- 60fps (16ms refresh rate)
- No lag with 100+ projects
- Stateful rendering (only redraws on input)

---

## Security Considerations

1. **Path Traversal** - shellexpand prevents `~/` exploitation
2. **Command Injection** - No shell=True, uses Command::new() directly
3. **Config Validation** - TOML parser validates structure
4. **Git Safety** - Not applicable yet (no git commands executed)

---

## Contributing

See [CARGO_GUIDE.md](CARGO_GUIDE.md) for build instructions.

**Code Style:**
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch issues
- Keep modules focused and small
- Document public APIs

---

**Next:** See [FEATURES.md](FEATURES.md) for planned features and roadmap.
