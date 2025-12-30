# Byte

**Simple project orchestration for developers who just want organized workspaces.**

Byte helps you initialize projects with consistent structure, discover them automatically, and browse them in a clean TUI. No complexity, no wrappers—just init, organize, and get to work.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)

## What It Does

- **Init Projects**: `byte init rust cli my-tool` creates projects with proper structure
- **Auto-Discovery**: Scans `~/projects` for `byte.toml` files
- **Clean TUI**: Browse all your projects in one place
- **Organized Logs**: All runtime data goes in `.byte/` (gitignored)

That's it. Byte doesn't wrap commands or replace your tools—it just helps you stay organized.

## Installation

### From Source

```bash
git clone https://github.com/BYTE-6D65/byte.git
cd byte
cargo build --release
sudo cp target/release/byte /usr/local/bin/
```

### Configuration

Create `~/.config/byte/config.toml`:

```toml
[workspace]
path = "~/projects"
auto_scan = true
registered = []

[tui]
refresh_rate_ms = 16
animations = true
default_view = "browser"
```

## Usage

### Initialize a New Project

```bash
byte init rust cli my-tool
byte init go cli my-service
byte init bun web my-app
```

Creates:
```
~/projects/my-tool/
├── .byte/              # Logs and state (gitignored)
│   ├── logs/
│   └── state/
├── .gitignore          # Includes .byte/
├── byte.toml           # Project metadata
└── [ecosystem files]   # Cargo.toml, go.mod, etc.
```

### Browse Projects

```bash
byte tui
```

**Keyboard Shortcuts:**
- `1` - Projects tab
- `2` - Commands tab
- `3` - Details tab
- `↑↓` - Navigate
- `Enter` - Select
- `?` - Help
- `q` - Quit

### Project Structure

Every Byte project has:

```toml
# byte.toml
[project]
name = "my-project"
type = "cli"
ecosystem = "rust"
```

Simple. Just metadata for the TUI to display.

## Supported Ecosystems

| Ecosystem | Types | Init Creates |
|-----------|-------|-------------|
| **Rust** | cli, lib | `Cargo.toml`, `src/main.rs` |
| **Go** | cli, api | `go.mod`, `cmd/`, `pkg/`, `internal/` |
| **Bun** | web, api | `package.json`, `src/index.ts` |

More coming soon.

## Philosophy

Byte follows these principles:

1. **Simple Over Complex** - No driver system, no plugins, no abstractions
2. **Organize, Don't Wrap** - Use native tools (`go build`, `cargo run`), Byte just organizes
3. **Consistent Structure** - All projects follow the same `.byte/` pattern
4. **Discovery Over Config** - Scan for `byte.toml`, don't manage registries

## Project Layout

```
byte/
├── src/
│   ├── cli/          # Command handling
│   ├── config/       # Config loading
│   ├── projects.rs   # Discovery & init
│   ├── tui/          # Terminal UI
│   └── logger.rs
├── CARGO_GUIDE.md
├── TUI_STYLING_GUIDE.md
└── PROJECT.md
```

Clean, focused, no cruft.

## Roadmap

- [ ] Add more ecosystems (Python, Node, Deno)
- [ ] Project templates
- [ ] Command history in `.byte/logs/`
- [ ] Search/filter in TUI
- [ ] Project tags

## Contributing

PRs welcome! Keep it simple:

1. Fork it
2. Create your feature branch (`git checkout -b feat/cool-thing`)
3. Commit with good messages
4. Push and open a PR

Read [CARGO_GUIDE.md](CARGO_GUIDE.md) for dev setup.

## License

MIT - See [LICENSE](LICENSE)

## Credits

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [ratatui](https://github.com/ratatui/ratatui) - Terminal UI library
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing

---

**Made with Claude Code** - https://claude.com/claude-code
