# Implementation Plan: execute_interactive()

**Status**: Planned
**Priority**: High
**Related**: Audit API #1 completion

---

## Goal

Add `execute_interactive()` to `CommandBuilder` for interactive command execution (editors, prompts).

---

## Current State

```rust
// Current API (src/exec/mod.rs)
pub fn execute(&self) -> Result<CommandResult>   // Captures output
pub fn execute_status(&self) -> Result<bool>     // Checks status only

// Current workaround (src/tui/mod.rs)
fn run_interactive_command(terminal, editor, file_path) {
    restore_terminal(terminal)?;
    Command::new(editor).arg(file_path).status()?;  // Bypasses exec API
    *terminal = setup_terminal()?;
}
```

---

## Proposed API

```rust
/// Execute interactively (inherits stdin/stdout/stderr)
pub fn execute_interactive(&self) -> Result<()> {
    self.validate()?;

    let mut cmd = Command::new(&self.command);
    cmd.args(&self.args);

    if let Some(dir) = &self.working_dir {
        cmd.current_dir(dir);
    }

    let status = cmd.status()?;

    if !status.success() {
        bail!("Command failed with exit code: {}", status.code().unwrap_or(-1));
    }

    Ok(())
}
```

---

## Changes Required

### 1. Extend Whitelist
```rust
// Add to ALLOWED_COMMANDS
"vim", "nano", "vi", "emacs", "code"
```

### 2. Update TUI
```rust
fn run_interactive_command(terminal, editor, file_path) -> Result<()> {
    restore_terminal(terminal)?;

    CommandBuilder::new(editor)
        .arg(file_path)
        .execute_interactive()?;

    *terminal = setup_terminal()?;
    Ok(())
}
```

### 3. Add Tests
- Whitelist validation for editors
- Working directory support
- Environment variable support

---

## Checklist

- [ ] Add `execute_interactive()` method
- [ ] Extend whitelist with editor commands
- [ ] Refactor `run_interactive_command()` in TUI
- [ ] Add unit tests
- [ ] Manual test: `o` key in Details view
- [ ] Update AUDIT_STATUS.md

---

## Design Notes

- Returns `Result<()>` - no output capture for interactive commands
- Timeout not useful for editors - ignored
- Terminal management stays in TUI layer (separation of concerns)
- Keep `code` (VS Code) in whitelist for GUI editor users
