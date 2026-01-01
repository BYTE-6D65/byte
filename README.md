# Byte

**ISPF-inspired project orchestration for developers who value keyboard-driven workflows.**

Byte helps you initialize projects with consistent structure, discover them automatically across multiple workspaces, and manage them through a powerful terminal UI. Execute commands, track builds, monitor git status, and view logs—all without leaving your terminal.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)
![Version](https://img.shields.io/badge/version-0.4.0-green.svg)

## Features

### Core Functionality
- **Multi-Workspace Discovery** - Scan multiple directories for projects with `byte.toml`
- **Project Initialization** - Bootstrap projects with proper ecosystem structure
- **Command Execution** - Run builds, tests, and custom commands with real-time feedback
- **Git Status Tracking** - See branch, modified files, staged changes, ahead/behind counts
- **Build State Persistence** - Track last build status and timestamp in `.byte/state/`
- **Log Management** - Categorized command logs (build/lint/git/other) with auto-cleanup

### Terminal UI (TUI)
- **4-Tab Interface** - Projects, Commands, Details, Workspace Manager
- **Command Filtering** - Filter by type: All/Build/Lint/Git/Test/Other
- **Live Log Preview** - Full-screen scrollable output with word wrapping
- **Build Animations** - Scanner/spotlight progress bar during command execution
- **Forms System** - Interactive forms for git tags, configuration, and more
- **File Watcher** - Auto-reload on config changes

### ISPF-Inspired Design
- **Keyboard-First Navigation** - No mouse required
- **Dense Information Display** - See all projects and metadata at a glance
- **Consistent Keybindings** - Number keys for views, arrows for navigation
- **Status Messages** - Clear feedback for every action
- **Fuzzy Directory Matching** - Quick workspace path entry

## Installation

### From Source

```bash
git clone https://github.com/BYTE-6D65/byte.git
cd byte
cargo build --release
sudo cp target/release/byte /usr/local/bin/
```

### First Run

```bash
# Initialize Byte (creates ~/.config/byte/config.toml)
byte init

# Launch TUI
byte tui
```

## Configuration

**Location:** `~/.config/byte/config.toml`

```toml
[workspace]
# Primary workspace directory
path = "~/projects"

# Auto-scan this workspace on startup
auto_scan = true

# Additional workspace directories (managed via TUI)
registered = [
    "~/work/clients",
    "~/experiments"
]

[tui]
# Refresh rate in milliseconds
refresh_rate_ms = 16

# Enable build animations
animations = true

# Default view on startup (browser, commands, detail, workspaces)
default_view = "browser"
```

## Usage

### Initialize a New Project

```bash
# Rust projects
byte init rust cli my-cli-tool
byte init rust lib my-library

# Go projects
byte init go cli my-service
byte init go api my-api

# Bun/TypeScript projects
byte init bun web my-webapp
byte init bun api my-api
```

**Creates:**
```
~/projects/my-cli-tool/
├── .byte/              # Runtime data (gitignored)
│   ├── logs/
│   │   └── commands/   # build/, lint/, git/, other/
│   └── state/
│       └── build.json  # Last build status
├── .gitignore          # Includes .byte/
├── byte.toml           # Project metadata
└── [ecosystem files]   # Cargo.toml, go.mod, package.json
```

### Project Metadata

**`byte.toml`:**
```toml
[project]
name = "my-cli-tool"
description = "A CLI tool that does things"
type = "cli"
ecosystem = "rust"
drivers = ["#rust", "#cli"]
```

### Browse and Manage Projects

```bash
byte tui
```

## TUI Keyboard Shortcuts

### Global Navigation
- `1` - Projects view
- `2` - Commands view
- `3` - Details view
- `4` - Workspace Manager
- `r` - Reload all state from disk
- `q` - Quit
- `?` - Help (future)

### Projects View (Tab 1)
- `↑↓` - Navigate project list
- `Enter` - View project details
- `f` - Open form (example: git tag creation)

### Commands View (Tab 2)
- `Left/Right` - Switch command filter (All/Build/Lint/Git/Test/Other)
- `↑↓` - Navigate command list
- `Enter` - Execute selected command
- **Preview Panel** - Shows command description and syntax

### Details View (Tab 3)
- **Git Status** - Branch, file counts, ahead/behind
- **Build State** - Last build time and status
- **Recent Logs** - Navigate with `↑↓`
- `l` - Open log preview in split view
- `o` - Open log in external editor ($EDITOR)
- `Esc` - Close log preview

### Workspace Manager (Tab 4)
- `a` - Add new workspace directory (with fuzzy path matching)
- `e` - Edit selected workspace path
- `d` - Remove workspace from config
- `Tab` - Autocomplete directory path

### Log Preview
- `↑↓` - Scroll up/down
- `PgUp/PgDn` - Scroll by page
- `Esc` - Close preview

### Forms
- `Tab/Shift+Tab` - Navigate fields
- `↑↓` - Select options (for select/multiselect)
- `Space` - Toggle checkboxes
- `Enter` - Submit form
- `Esc` - Cancel form

## Supported Ecosystems

| Ecosystem | Types | Init Creates | Build Command |
|-----------|-------|--------------|---------------|
| **Rust** | cli, lib | `Cargo.toml`, `src/main.rs`, `.gitignore` | `cargo build` |
| **Go** | cli, api | `go.mod`, `cmd/`, `pkg/`, `internal/` | `go build` |
| **Bun** | web, api | `package.json`, `src/index.ts`, `tsconfig.json` | `bun build` |

**Future:** Python, Node, Deno, Zig

## Project Discovery

Byte discovers projects by scanning workspace directories for `byte.toml` files:

1. **Primary Workspace** - Set in `config.toml` (`workspace.path`)
2. **Registered Workspaces** - Additional directories (managed in TUI)
3. **Auto-Scan** - Recursively finds all Byte projects

**Example:**
```
~/projects/
├── rust-cli/byte.toml      ✓ Discovered
├── go-api/byte.toml        ✓ Discovered
└── experiments/
    └── test-app/byte.toml  ✓ Discovered

~/work/clients/
└── client-a/byte.toml      ✓ Discovered (if registered)
```

## Command Execution

Commands are defined globally in config or per-project. Byte executes them in the project directory and captures:

- **stdout/stderr** - Logged to `.byte/logs/commands/{category}/{timestamp}-{command}.log`
- **Exit code** - Success/failure tracking
- **Execution time** - Displayed in UI
- **Build state** - Persisted to `.byte/state/build.json` for build commands

**Log Categories:**
- `build/` - `cargo build`, `go build`, `bun build`, `make`
- `lint/` - `cargo clippy`, `cargo fmt`, `eslint`, `prettier`
- `git/` - All `git` commands
- `test/` - `cargo test`, `go test`
- `other/` - Everything else

**Log Retention:** Last 20 logs per category (configurable)

## Git Integration

Byte tracks git status for each project:

- **Branch** - Current branch or detached HEAD
- **Modified Files** - Count of unstaged changes
- **Staged Files** - Count of staged changes
- **Untracked Files** - Count of untracked files
- **Ahead/Behind** - Commits ahead/behind tracking remote

**Display:**
```
Branch: main                    ✓ Clean
  3 modified, 1 untracked
  ↑2 ↓1
```

## Build State Tracking

Build commands (detected by name) save state to `.byte/state/build.json`:

```json
{
  "timestamp": 1704457800,
  "status": "Success",
  "task": "release"
}
```

**Status Types:** Success, Failed, Running

## Forms System

Interactive forms for user input:

**Field Types:**
- `TextInput` - Single-line text
- `TextArea` - Multi-line text
- `Email` - Email validation
- `Number` - Integer with min/max
- `Select` - Single choice (radio)
- `MultiSelect` - Multiple choices (checkboxes)
- `Checkbox` - Boolean toggle
- `Path` - File/directory path

**Example Use Cases:**
- Git tag creation
- Project configuration
- Batch command parameters
- Environment variable setup

## Philosophy

1. **Keyboard-First** - No mouse, no clicking, pure efficiency
2. **ISPF-Inspired** - Dense information, number-key navigation, status messages
3. **Organize, Don't Wrap** - Use native tools, Byte just orchestrates
4. **Consistent Structure** - All projects follow `.byte/` pattern
5. **Discovery Over Registry** - Scan for projects, don't maintain lists
6. **Zero Configuration** - Works out of the box, customize if needed

## Architecture

```
byte/
├── src/
│   ├── cli/          # Command-line interface
│   ├── config/       # Config loading and management
│   │   ├── mod.rs
│   │   └── types.rs
│   ├── forms/        # Interactive form system
│   ├── projects.rs   # Project discovery and initialization
│   ├── state/        # Git status and build state tracking
│   │   ├── mod.rs
│   │   ├── git.rs
│   │   └── build.rs
│   ├── tui/          # Terminal UI (ratatui)
│   ├── logger.rs     # Logging and command output capture
│   └── lib.rs
├── ROADMAP.md        # Feature roadmap
├── AUDIT_REPORT.md   # Security and architecture audit
└── Cargo.toml
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed feature planning.

**Phase 1 (v0.4.0 - Current):**
- ✅ Command type filtering
- ✅ Forms system
- Real-time command output
- Custom commands per project

**Phase 2 (v0.5.0):**
- Batch command execution across multiple projects
- Search/filter projects
- Command history

**Phase 3 (v0.6.0):**
- Project bookmarks/favorites
- Command templates with variables
- Better help system

**Phase 4 (v0.7.0+):**
- Input focus management API
- Color themes (including IBM green screen!)
- Command output search

## Development

### Build

```bash
cargo build
cargo run -- tui
```

### Test

```bash
cargo test
cargo clippy
cargo fmt
```

### Project Files

- [ROADMAP.md](ROADMAP.md) - Feature planning and priorities
- [AUDIT_REPORT.md](AUDIT_REPORT.md) - Security and architecture audit findings
- [CARGO_GUIDE.md](CARGO_GUIDE.md) - Cargo configuration guide
- [TUI_STYLING_GUIDE.md](TUI_STYLING_GUIDE.md) - UI styling conventions

## Contributing

PRs welcome! Keep it focused:

1. Fork the repo
2. Create feature branch (`git checkout -b feat/cool-thing`)
3. Follow existing code style (run `cargo fmt`)
4. Write clear commit messages
5. Push and open a PR

**Code Standards:**
- Use `anyhow::Result` for error handling
- Follow ISPF design principles (keyboard-first, dense UI)
- Add tests for new functionality
- Document public APIs

## Security

See [AUDIT_REPORT.md](AUDIT_REPORT.md) for security findings and recommendations.

**Known Issues:**
- Command execution uses `sh -c` - validate user input in configs
- Path inputs need validation to prevent traversal attacks
- No authentication/authorization (local tool only)

## License

MIT - See [LICENSE](LICENSE)

## Credits

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [ratatui](https://github.com/ratatui/ratatui) - Terminal UI framework (v0.29)
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
- [serde](https://serde.rs/) - Serialization framework
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [notify](https://github.com/notify-rs/notify) - File system watching

---

**Made with Claude Code** - https://claude.com/claude-code
