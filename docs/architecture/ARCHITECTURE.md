# Byte Architecture

Current system design and implementation details.

**Last Updated:** 2026-01-03
**Version:** 0.4.0

---

## Overview

Byte is a project orchestration CLI that helps developers:
- Initialize projects with consistent structure
- Auto-discover projects via `byte.toml` manifests
- Browse and manage projects in a TUI
- Execute commands with logging and state tracking
- Manage multiple workspace directories
- Track git status across projects

**Philosophy:** Simple over complex. Keyboard-driven, ISPF-inspired interface with minimal abstraction layers.

---

## System Architecture

```
+-------------------------------------------------------------+
|                         User                                |
+------------+----------------------------+-------------------+
             |                            |
             v                            v
      +-------------+             +-------------+
      |  CLI Entry  |             |     TUI     |
      |  (main.rs)  |             |  (tui/*)    |
      +------+------+             +------+------+
             |                            |
             v                            |
      +-------------+                     |
      | CLI Handler |<--------------------+
      |  (cli/*)    |
      +------+------+
             |
             +----------+----------+----------+----------+
             v          v          v          v          v
      +----------+ +----------+ +--------+ +--------+ +--------+
      |  Config  | | Projects | |  Exec  | |  Path  | |  State |
      |(config/) | |(projects)| |(exec/) | |(path/) | |(state/)|
      +----------+ +----------+ +--------+ +--------+ +--------+
             |          |           |          |          |
             v          v           v          v          v
      +------------------------------------------------------+
      |              Filesystem & Git                        |
      |  ~/.config/byte/    ~/projects/    .byte/logs/       |
      +------------------------------------------------------+
```

---

## Core Modules

### 1. CLI (`src/cli/`)

**Purpose:** Command-line interface and argument parsing

**Commands:**
```rust
byte init <ecosystem> <type> <name>  // Initialize new project
byte tui                              // Launch TUI
byte                                  // Default: launch TUI
```

### 2. Config (`src/config/`)

**Purpose:** Configuration loading, saving, and workspace management

**Config Hierarchy:**
```toml
# Global: ~/.config/byte/config.toml
[workspace]
path = "~/projects"
auto_scan = true
registered = []  # Additional workspace paths

[tui]
refresh_rate_ms = 16
animations = true
default_view = "browser"

# Project: <project>/byte.toml
[project]
name = "my-project"
type = "cli"
ecosystem = "rust"
description = "Optional description"

[build]
dev = "cargo run"
release = "cargo build --release"
```

### 3. Projects (`src/projects.rs`)

**Purpose:** Project discovery, initialization, and name validation

**Key Functions:**
- `discover_projects()` - Scan workspaces for byte.toml files
- `init_project()` - Create new project with ecosystem-specific setup
- `validate_project_name()` - Security validation for project names

**Supported Ecosystems:**
- `rust` - Cargo-based projects
- `go` - Go modules
- `bun` - Bun/TypeScript projects

### 4. Path (`src/path/`)

**Purpose:** Safe path handling with validation

**Key Type:** `SafePath` - Centralized path abstraction with:
- Tilde expansion
- Canonicalization
- Permission validation
- Path traversal protection

### 5. Exec (`src/exec/`)

**Purpose:** Safe command execution with validation and logging

**Key Type:** `CommandBuilder` - Fluent API for command execution:
- `execute()` - Capture stdout/stderr
- `execute_status()` - Check exit status only
- Command whitelist validation
- Working directory support

### 6. State (`src/state/`)

**Purpose:** Track project state (git, build)

**Files:**
- `git.rs` - Parse git status (branch, modified, staged, ahead/behind)
- `build.rs` - Track build state and timing

### 7. TUI (`src/tui/`)

**Purpose:** Terminal user interface

**Views:**
1. **ProjectBrowser (1)** - List all discovered projects
2. **CommandPalette (2)** - Execute commands with category filters
3. **Detail (3)** - Project details, git status, command logs
4. **WorkspaceManager (4)** - Add/remove workspace directories
5. **Form** - Modal input collection

**Key Features:**
- ISPF-inspired keyboard navigation
- Command execution with build animation
- File watching for auto-reload
- Fuzzy directory matching
- Full-screen log preview

---

## Data Flow

### Init Flow
```
User: byte init rust cli my-tool
  |
  v
cli::run() parses args
  |
  v
validate_project_name("my-tool")
  |
  v
SafePath::workspace("~/projects").validate_writable()
  |
  v
init_project(~/projects, rust, cli, my-tool)
  |
  +-> Create ~/projects/my-tool/.byte/{logs,state}/
  +-> Write byte.toml
  +-> init_rust_project() -> cargo init
  +-> add_to_gitignore() -> add .byte/ to .gitignore
  |
  v
Print success message
```

### Discovery Flow
```
User: byte tui
  |
  v
tui::run() -> setup_terminal()
  |
  v
App::new()
  |
  +-> Config::load() -> GlobalConfig
  +-> For each workspace path:
  |     +-> SafePath::workspace(path)
  |     +-> walkdir up to depth 3
  |     +-> Find byte.toml files
  |     +-> Parse ProjectConfig
  +-> Return Vec<DiscoveredProject>
  |
  v
run_app() -> event loop
```

---

## File Structure

### Project Directory
```
~/projects/my-project/
+-- .byte/                  # Byte runtime data (gitignored)
|   +-- logs/              # Command output logs
|   +-- state/             # Build state, metadata
+-- .git/                  # Git repository
+-- .gitignore             # Includes .byte/
+-- byte.toml              # Project manifest
+-- [ecosystem files]      # Cargo.toml, go.mod, etc.
```

### Source Code
```
src/
+-- cli/
|   +-- mod.rs            # CLI runner
+-- config/
|   +-- mod.rs            # Config loading/saving
|   +-- types.rs          # Data structures
+-- exec/
|   +-- mod.rs            # Command execution
+-- path/
|   +-- mod.rs            # SafePath abstraction
+-- state/
|   +-- git.rs            # Git status parsing
|   +-- build.rs          # Build state tracking
+-- tui/
|   +-- mod.rs            # TUI main, App struct, views
+-- forms/
|   +-- mod.rs            # Form input system
+-- fs/
|   +-- mod.rs            # Project filesystem operations
+-- log.rs                # File-based logging
+-- projects.rs           # Discovery & init
+-- lib.rs                # Library exports
+-- main.rs               # Binary entry
```

---

## Key Design Decisions

### 1. Trusted Config Model
byte.toml is treated as trusted developer input. Shell operators (`&&`, `|`, etc.) are allowed in commands because the config is written by the developer, not parsed from untrusted sources.

### 2. SafePath Centralization
All path operations go through `SafePath` to ensure consistent tilde expansion, canonicalization, and validation. This prevents path traversal attacks and permission issues.

### 3. Command Whitelist
The exec module maintains a whitelist of allowed commands for direct execution. This provides defense-in-depth without blocking legitimate developer workflows.

### 4. ISPF-Inspired Navigation
Keyboard-first design with numbered views (1-4), modal input, and dense information display. Optimized for developers familiar with mainframe-era interfaces.

### 5. File-based Logging
All logging goes to `.byte/logs/` instead of stderr because the TUI uses alternate screen mode where stderr would corrupt the display.

---

## Dependencies

### Core
- **clap** (4.5) - CLI argument parsing
- **serde** (1.0) - Serialization
- **toml** (0.8) - Config parsing
- **anyhow** (1.0) - Error handling

### TUI
- **ratatui** (0.29) - Terminal UI framework
- **crossterm** (0.28) - Terminal control

### Utilities
- **walkdir** (2.5) - Directory traversal
- **shellexpand** (3) - Tilde expansion
- **dirs** (5.0) - Standard directories
- **notify** (6.0) - File watching

---

## Security Model

1. **Path Traversal Prevention** - SafePath validates all path operations
2. **Project Name Validation** - Strict character and reserved name checks
3. **Command Whitelist** - Known-good commands for direct execution
4. **Permission Checks** - Validate write access before operations
5. **No Remote Execution** (yet) - Remote paths are detected and rejected with future support message

See [../audit/AUDIT_STATUS.md](../audit/AUDIT_STATUS.md) for detailed security audit status.

---

## Performance

**Discovery:**
- Scans up to 3 levels deep
- ~100ms for typical workspace with 20 projects
- File watcher for incremental updates

**TUI:**
- 60fps (16ms refresh rate)
- Stateful rendering (redraws on input only)
- Async command execution with progress animation
