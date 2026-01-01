# Implementation Plan: Add `execute_interactive()` to Exec API

**Status:** Planned
**Priority:** High
**Created:** 2026-01-01
**Related Issues:** AUDIT_REPORT.md - API #1 (Command Execution Abstraction)

---

## Executive Summary

Add `execute_interactive()` method to `CommandBuilder` to enable interactive command execution through the exec API, eliminating the current workaround that bypasses the API for editor operations.


**Benefits:**
- API completeness (covers all three execution modes)
- Consistency (all command execution through single API)
- Security enhancement (centralized editor validation)
- Feature inheritance (working_dir, env_vars, validation)
- Testability (mockable for unit tests)
- Extensibility (foundation for remote execution, progress tracking)
- Better API design (clear intent)
- Simplified code (remove bypass patterns)
- Audit trail (log interactive commands)
- Reduces technical debt

---

## Current State Analysis

### Current API (`src/exec/mod.rs`)
```rust
pub fn execute(&self) -> Result<CommandResult>  // Uses .output()
pub fn execute_status(&self) -> Result<bool>    // Uses .status()
```

### Current Workaround (`src/tui/mod.rs:1642-1670`)
```rust
// ❌ Direct std::process::Command usage - bypasses exec API
fn run_interactive_command(terminal, editor, file_path) {
    restore_terminal(terminal)?;
    let status = Command::new(editor).arg(file_path).status()?;
    *terminal = setup_terminal()?;
}
```

### Current Interactive Use Cases
1. **Editor** (`tui/mod.rs:899`): Open log files in external editor
2. **Fuzzy picker** (`tui/mod.rs:1684`): Directory selection (uses skim library, not exec API)

---

## Design Decisions

### 1. Method Signature
```rust
pub fn execute_interactive(&self) -> Result<()>
```

**Rationale:**
- Returns `Result<()>` since output cannot be captured (interactive)
- Consistent with `execute_status()` pattern
- Simple success/failure is sufficient for interactive commands

### 2. Security & Validation
- **Extend whitelist** to include editors: `vim`, `nano`, `vi`, `emacs`, `code`
- **Reuse existing `validate()` method** for consistency
- **Keep file path validation** in TUI layer (system-controlled, not user input)

### 3. Terminal Management
- **Keep terminal restore/resume in TUI layer** (`restore_terminal()`, `setup_terminal()`)
- Exec API only handles command execution, not terminal state
- Separation of concerns: TUI owns terminal, exec API owns commands

### 4. Feature Support
- ✅ Working directory
- ✅ Environment variables
- ❌ Timeout (not useful for interactive editors)
- ❌ Logging (cannot capture output)
- ✅ Validation (whitelist)

---

## Implementation Plan

### Phase 1: Add `execute_interactive()` Method
**File:** `src/exec/mod.rs`

**Tasks:**
1. Add method after `execute_status()` (line ~254)
2. Implement using `.status()` instead of `.output()`
3. Call `validate()` before execution
4. Support `working_dir` and `env_vars`
5. Return `Result<()>` (no captured output)

**Implementation:**
```rust
/// Execute the command interactively (inherits stdin/stdout/stderr)
///
/// Use for editors, prompts, or commands requiring user interaction.
/// Cannot capture stdout/stderr - user sees output directly.
pub fn execute_interactive(&self) -> Result<()> {
    self.validate()?;

    let mut cmd = Command::new(&self.command);
    cmd.args(&self.args);

    if let Some(dir) = &self.working_dir {
        cmd.current_dir(dir);
    }

    for (key, value) in &self.env_vars {
        cmd.env(key, value);
    }

    let status = cmd.status()
        .with_context(|| format!("Failed to execute interactive command: {}", self.command))?;

    if !status.success() {
        bail!("Interactive command '{}' failed with exit code: {}",
              self.command,
              status.code().unwrap_or(-1));
    }

    Ok(())
}
```

---

### Phase 2: Extend Command Whitelist
**File:** `src/exec/mod.rs:146-155`

**Tasks:**
1. Add editors to `ALLOWED_COMMANDS` array
2. Consider adding `code` (VS Code) as non-terminal editor

**Changes:**
```rust
const ALLOWED_COMMANDS: &[&str] = &[
    "cargo", "rustc", "rustfmt", "clippy-driver",
    "go", "gofmt",
    "bun", "npm", "node", "npx",
    "git",
    "make", "cmake",
    "python", "python3",
    "sh", "bash", // Allowed for shell mode
    "which",     // For checking command existence
    "vim", "nano", "vi", "emacs", "code", // Interactive editors
];
```

**Note:** Consider whether `code` (VS Code) should be allowed. It's a GUI editor, not a terminal editor, but still valid.

---

### Phase 3: Update TUI to Use New API
**File:** `src/tui/mod.rs`

**Tasks:**
1. Simplify `run_interactive_command` function
2. Replace `std::process::Command` with `CommandBuilder`
3. Remove inline comments about bypassing exec API

**Refactored Implementation:**
```rust
/// Suspend TUI, run interactive command with terminal access, then resume TUI
fn run_interactive_command(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    editor: &str,
    file_path: &str,
) -> anyhow::Result<()> {
    use crate::exec::CommandBuilder;

    // Suspend TUI
    restore_terminal(terminal)?;

    // Run editor with inherited stdin/stdout/stderr via exec API
    CommandBuilder::new(editor)
        .arg(file_path)
        .execute_interactive()?;

    // Resume TUI
    *terminal = setup_terminal()?;

    Ok(())
}
```

**Benefits:**
- Cleaner code
- Consistent with rest of codebase
- Future-proof for new features (env vars, validation, etc.)

---

### Phase 4: Add Unit Tests
**File:** `src/exec/mod.rs` (tests module)

**Tasks:**
1. Test successful interactive execution
2. Test validation blocks non-whitelisted editors
3. Test working directory support
4. Test environment variable support

**Test Cases:**
```rust
#[test]
fn test_interactive_execution_allows_whitelisted_editor() {
    let cmd = CommandBuilder::new("vim").arg("/tmp/test.txt");
    assert!(cmd.validate().is_ok());
}

#[test]
fn test_interactive_execution_blocks_non_whitelisted_editor() {
    let cmd = CommandBuilder::new("unknown-editor").arg("/tmp/test.txt");
    assert!(cmd.validate().is_err());
}

#[test]
fn test_interactive_with_working_dir() {
    let cmd = CommandBuilder::new("vim")
        .arg("file.txt")
        .working_dir("/tmp");
    assert!(cmd.validate().is_ok());
}

#[test]
fn test_interactive_with_env_vars() {
    let cmd = CommandBuilder::new("vim")
        .arg("file.txt")
        .env("TEST_VAR", "value");
    assert!(cmd.validate().is_ok());
}
```

**Note:** Integration tests for actual interactive execution are difficult (require terminal). Unit tests focus on validation and configuration.

---

### Phase 5: Update Documentation
**Files to Update:**

1. **AUDIT_REPORT.md** - Mark API #1 as "fully implemented"
2. **ROADMAP.md** - Add milestone for "Interactive command API"
3. **ARCHITECTURE.md** - Document exec API capabilities
4. **README.md** - No changes needed (internal API)

**Documentation Updates:**

**AUDIT_REPORT.md:**
```markdown
### 1. **Shell Command Execution Abstraction** (Priority: CRITICAL)
Status: ✅ COMPLETED

- ✅ Command execution API with validation
- ✅ Output capture via `execute()`
- ✅ Status check via `execute_status()`
- ✅ Interactive execution via `execute_interactive()` (NEW)
- ✅ Editor whitelist for interactive commands (NEW)
```

**ARCHITECTURE.md:**
```markdown
### Exec API (`src/exec/`)

**Purpose:** Safe command execution with validation and logging

**Methods:**
- `execute()` - Capture stdout/stderr for non-interactive commands
- `execute_status()` - Check status without capturing output
- `execute_interactive()` - Run with inherited terminal for editors/prompts

**Security:**
- Command whitelist validation
- Shell command injection protection
- Editor whitelist for interactive commands
```

---

### Phase 6: Verify No Regressions
**Tasks:**
1. Run existing test suite
2. Test editor functionality manually (`o` key in Details view)
3. Verify terminal restoration works correctly
4. Check for any compilation errors

**Manual Testing Checklist:**
- [ ] Launch TUI: `cargo run -- tui`
- [ ] Select a project with command logs
- [ ] Press `o` to open log in editor
- [ ] Verify editor opens correctly
- [ ] Verify TUI restores after closing editor
- [ ] Check status message shows success/error

---

## Migration Checklist

### Code Changes
- [ ] Add `execute_interactive()` method to `CommandBuilder`
- [ ] Extend `ALLOWED_COMMANDS` whitelist with editors
- [ ] Refactor `run_interactive_command()` in `tui/mod.rs`
- [ ] Remove bypass comments explaining why exec API wasn't used

### Testing
- [ ] Add unit tests for validation
- [ ] Run existing test suite (`cargo test`)
- [ ] Manual test of editor functionality

### Documentation
- [ ] Update AUDIT_REPORT.md
- [ ] Update ARCHITECTURE.md
- [ ] Update ROADMAP.md

### Verification
- [ ] No compilation errors
- [ ] No test failures
- [ ] No functional regressions

---

## Risk Assessment

### Low Risk
- **Scope:** Single method addition + one refactoring
- **Isolation:** Only affects editor functionality
- **Rollback:** Easy to revert (git restore)

### Medium Risk
- **Terminal state:** Must ensure restore/resume works correctly
- **Editor whitelist:** Must include all common editors
- **Testing:** Interactive commands harder to test automatically

### Mitigation
- Keep existing terminal management code
- Start with conservative whitelist (vim, nano, vi, emacs)
- Manual testing of editor workflow

---

## Future Considerations

### Potential Enhancements
1. **Timeout support:** Could add optional timeout with cancellation
2. **Post-execution hooks:** Trigger actions after editor closes
3. **Output capture:** Hybrid mode (interactive + capture final output)
4. **Remote execution:** Extend to SSH targets (planned feature)

### Related Features
1. **Batch interactive execution:** Run editor across multiple projects
2. **Custom editor config:** User-defined editor preferences
3. **Editor detection:** Auto-detect available editors

---

## Success Criteria

✅ **Complete when:**
1. `execute_interactive()` method implemented and tested
2. Editor whitelist extended and documented
3. TUI uses exec API instead of direct `std::process::Command`
4. All tests pass
5. Editor functionality works correctly (manual test)
6. Documentation updated

---

## Questions for Review

1. **Editor whitelist:** Should `code` (VS Code) be allowed? It's a GUI editor, not terminal-based.
2. **Logging:** Should we log metadata (timestamp, command, exit code) even without output capture?
3. **Timeout:** Should `execute_interactive()` respect the `timeout` field, or ignore it?
4. **Return type:** Should it return `Result<()>` or `Result<ExitCode>` for more granular error handling?

---

## Related Documentation

- **AUDIT_REPORT.md** - Command execution abstraction (API #1)
- **ARCHITECTURE.md** - Exec API section
- **ROADMAP.md** - Future command execution features
- **src/exec/mod.rs** - Current implementation
- **src/tui/mod.rs** - Current workaround usage

---

## Appendix: Benefits Detail

### 1. **API Completeness**
   - Covers all three standard execution modes:
     - Capture output (build, test, lint)
     - Check status only (which, check command exists)
     - **Interactive terminal** (editors, prompts) ← **Missing**

### 2. **Consistency**
   - All command execution through single API
   - Unified error handling and logging
   - Eliminates "special bypass" patterns

### 3. **Security Enhancement**
   - **Editor whitelist validation** centralized
   - Currently validation logic scattered across `tui/mod.rs` and `logger.rs`
   - Prevents command injection (low risk but better consistency)

### 4. **Feature Inheritance**
   Inherits all `CommandBuilder` capabilities:
   - `working_dir` - open editor in project directory
   - `env_vars` - set editor environment variables
   - `log_category` - log interactive commands
   - `timeout` - timeout protection (prevent editor hang)

### 5. **Testability**
   - Can mock `execute_interactive()` for testing
   - Current workaround not testable
   - Enables unit tests for TUI editor workflow

### 6. **Extensibility**
   Extension points for future features:
   - Remote execution (SSH)
   - Cancellation tokens (`cancel_token`)
   - Progress callbacks
   - Pre/post editor hooks

### 7. **Better API Design**
   - **Clear intent:**
     ```rust
     cmd.execute()              // explicit: capture output
     cmd.execute_status()        // explicit: check status only
     cmd.execute_interactive()?  // explicit: interactive execution
     ```
   - No need to remember when to bypass the API

### 8. **Simplifies Code**
   - Remove `run_interactive_command` function from `tui/mod.rs`
   - Replace with:
     ```rust
     CommandBuilder::new(editor)
         .arg(file_path)
         .execute_interactive()?;
     ```

### 9. **Audit Trail**
   - Log all interactive command executions
   - Currently only non-interactive commands are logged
   - Meets security audit requirements

### 10. **Reduces Technical Debt**
   - Addresses AUDIT_REPORT.md's "command execution logic repeated 10+ times"
   - Unifies editor startup logic

---

## Trade-offs

**Potential downsides:**
- Adds one public method
- Requires adjusting existing editor startup code
- Short-term testing overhead

**The tradeoff:**
- Benefits far outweigh costs
- Aligns with audit recommendations and architecture goals
- Foundation for future features
