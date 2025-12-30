# Byte - Rust Project

Welcome to Byte! This is a Rust project orchestration CLI.

## Quick Start

### Install Rust (if needed)
```bash
# Check if installed
cargo --version

# If not, install via rustup (already installed on this system)
# ~/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo is available
```

### Build and Run
```bash
# Build
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH" cargo build

# Run
./target/debug/byte

# Run directly via cargo
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH" cargo run
```

### Dev Tools
```bash
./clean.py              # Clean build artifacts
./dev.py --tui          # Run TUI in dev mode
./build.py --run        # Build and run
./format.py             # Format code
./lint.py               # Run clippy
./setup.py              # First-time setup
```

## Learn Rust

All Rust examples are organized in `learn_rust/` for Go developers learning Rust.

### Quick Reference

| File | Topic |
|------|--------|
| `unwrap_examples.rs` | Error handling (Option/Result) |
| `match_examples.rs` | Control flow (like Go's switch) |
| `mutability_examples.rs` | Variables, shadowing |
| `loop_examples.rs` | Loops and iterators |
| `statemachine_examples.rs` | State machines with enums |
| `datastruct_examples.rs` | Vec, HashMap, collections |
| `stdlib_examples.rs` | File I/O, time, strings |
| `ownership_examples.rs` | Ownership, borrowing |
| `no_mem_math.rs` | No manual memory management |
| `stack_heap.rs` | Stack vs heap (types decide) |
| `enforce_stack_heap.rs` | Control allocation patterns |
| `clock_examples.rs` | Monotonic vs system clocks |

### Compile All Examples
```bash
cd learn_rust
python3 compile_all.py
```

### Read Examples
```bash
cd learn_rust
# Open README.md for full guide
open README.md
```

## Project Structure

```
byte/
├── learn_rust/              # Rust learning examples
│   ├── README.md            # Complete guide
│   ├── compile_all.py       # Compile all examples
│   └── *.rs               # Example files
├── src/                    # Byte source code
├── drivers/                 # Ecosystem drivers
├── tests/                  # Unit and integration tests
├── Cargo.toml              # Dependencies
└── *.py                   # Dev tools
```

## Dependencies

All dependencies already in `Cargo.toml`:
- `clap` - CLI argument parsing
- `ratatui` - Terminal UI
- `tokio` - Async runtime
- `serde` - Serialization
- `tracing` - Logging
- And more...

See `RUST_SPECS.md` for full details.

## Documentation

- **README.md** - Project overview
- **PRD.md** - Product requirements
- **WBS.md** - Work breakdown structure
- **RUST_SPECS.md** - Rust needs and wants
- **DEV_TOOLS.md** - Dev tools documentation

## Rust Learning Path for Go Developers

1. **Basics** → `unwrap_examples.rs`, `mutability_examples.rs`, `match_examples.rs`
2. **Control Flow** → `loop_examples.rs`, `statemachine_examples.rs`
3. **Data Structures** → `datastruct_examples.rs`, `stdlib_examples.rs`
4. **Memory** → `ownership_examples.rs`, `no_mem_math.rs`, `stack_heap.rs`
5. **Advanced** → `enforce_stack_heap.rs`, `clock_examples.rs`

Each example has Go comparisons and explains the concept clearly.

## Next Steps

1. Explore `learn_rust/` to get comfortable with Rust
2. Review project docs (README.md, PRD.md, WBS.md)
3. Start implementing Byte drivers and routing
4. Build TUI with ratatui

Happy coding! 🚀
