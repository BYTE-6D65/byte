# How to Use Cargo

No scripts needed - just use cargo directly. It's faster, simpler, and actually works.

## Quick Reference

```bash
# Development
cargo run                    # Build and run (debug)
cargo run -- tui             # Run with args
cargo watch -q -x run        # Hot reload on file changes

# Building
cargo build                  # Debug build (fast)
cargo build --release        # Release build (optimized)

# Code Quality
cargo fmt                    # Auto-format code
cargo fmt --check            # Check formatting
cargo clippy                 # Lint code
cargo clippy --fix           # Auto-fix lint issues

# Testing
cargo test                   # Run all tests
cargo test --lib             # Test library only
cargo watch -q -x test       # Hot reload tests

# Cleanup
cargo clean                  # Delete target directory
```

---

## Common Workflows

### 1. Development with Hot Reload

```bash
# TUI development
cargo watch -q -x 'run -- tui'
```

Now edit any file in `src/` and it auto-rebuilds and restarts. Press `Ctrl+C` to stop.

### 2. Test-Driven Development

```bash
# Watch tests
cargo watch -q -x test
```

Tests run automatically when you save a file.

### 3. Pre-Commit Quality Check

```bash
# Format, lint, test
cargo fmt && cargo clippy && cargo test
```

Run this before committing to ensure everything passes.

### 4. Release Build

```bash
# Optimized production build
cargo build --release

# Binary at: target/release/byte
./target/release/byte --help
```

---

## cargo-watch Options

If you don't have `cargo-watch` installed:

```bash
cargo install cargo-watch
```

### Useful Flags

```bash
-q, --quiet              # Suppress cargo output
-x, --exec <cmd>         # Execute cargo command
-c, --clear              # Clear screen before each run
-w, --watch <path>       # Watch specific paths
-i, --ignore <pattern>   # Ignore patterns

# Examples
cargo watch -q -x check                    # Just check compilation
cargo watch -q -c -x 'run -- tui'         # Clear screen each run
cargo watch -q -x 'clippy -- -D warnings'  # Strict linting
```

---

## Optimization Levels

```bash
# Debug (default) - fast compile, slow runtime
cargo build

# Release - slow compile, fast runtime
cargo build --release

# Custom profile in Cargo.toml:
[profile.dev]
opt-level = 0        # No optimization

[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower compile
strip = true         # Strip debug symbols
```

---

## Cargo Environment Variables

```bash
# Skip checking dependencies (faster rebuilds)
CARGO_INCREMENTAL=1 cargo build

# Use more CPU cores
CARGO_BUILD_JOBS=8 cargo build

# Verbose output
CARGO_LOG=cargo::core=debug cargo build

# Target directory (useful for multiple builds)
CARGO_TARGET_DIR=/tmp/byte-target cargo build
```

---

## Checking Without Building

```bash
cargo check              # Fast syntax/type checking (no binary)
cargo watch -q -x check  # Watch mode for quick feedback
```

Use `cargo check` during development - it's 10x faster than `cargo build`.

---

## Dependency Management

```bash
# Update dependencies
cargo update

# Check for outdated deps
cargo outdated           # (requires: cargo install cargo-outdated)

# Audit for security issues
cargo audit              # (requires: cargo install cargo-audit)

# Show dependency tree
cargo tree
```

---

## Aliases

Add to `~/.cargo/config.toml`:

```toml
[alias]
w = "watch -q -x"
r = "run"
b = "build --release"
t = "test"
c = "clippy"
f = "fmt"
```

Then use:
```bash
cargo w run              # watch run
cargo w test             # watch test
cargo b                  # release build
```

---

## Pro Tips

### 1. Use `cargo check` for quick feedback
```bash
# Instead of cargo build during development
cargo watch -q -x check
```

### 2. Parallel test execution
```bash
cargo test -- --test-threads=8
```

### 3. Show test output even on success
```bash
cargo test -- --nocapture
```

### 4. Run specific test
```bash
cargo test test_name
cargo test module::test_name
```

### 5. Benchmarking
```bash
cargo bench
```

### 6. Documentation
```bash
cargo doc --open          # Build and open docs in browser
```

---

## Troubleshooting

### "command not found: cargo"

Your PATH is missing cargo. Add to `~/.zshrc`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload:
```bash
source ~/.zshrc
```

### Slow compilation

```bash
# Use cargo-watch with check instead of build
cargo watch -q -x check

# Or use mold linker (faster linking)
brew install mold
```

Add to `~/.cargo/config.toml`:
```toml
[target.aarch64-apple-darwin]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=/opt/homebrew/bin/mold"]
```

### Out of disk space

```bash
# Clean all cargo caches
cargo cache --autoclean
cargo clean
```

---

## That's It

No Python scripts, no PATH issues, no complexity. Just cargo.

**Most common commands you'll use:**
- `cargo run` - run your app
- `cargo watch -q -x run` - hot reload
- `cargo fmt` - format code
- `cargo test` - run tests
- `cargo build --release` - production build

Done.
