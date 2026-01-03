# Notable Behaviors, Edge Cases, and Design Decisions

This document tracks important behaviors, edge cases, and design decisions in the Byte project manager.

**Last Updated**: 2026-01-03

---

## Command Execution

### Shell Operators Support
**Decision**: Allow full shell operators (`&&`, `||`, `|`, `;`, `>`, `<`, etc.) in byte.toml commands.

**Rationale**:
- byte.toml is a trusted developer config file, not runtime user input
- Enables monorepo workflows: `cd subdir && command`
- Allows command chaining: `build && test`
- Security maintained by command whitelist for direct binary execution

**Location**: `src/exec/mod.rs` - `validate_shell_command()`

### Command Categorization (Menu 2 Filters)

**Implementation**: Keyword-based categorization (language-agnostic)

**How it works**:
1. Strips `cd dir &&` prefix (any depth: `cd a/b/c && cmd` -> `cmd`)
2. Matches on action keywords:
   - **Build**: build, compile, bundle, dev, run, start, watch, serve
   - **Test**: test, spec, coverage, bench
   - **Lint**: lint, fmt, format, clippy, check, prettier, eslint
   - **Git**: must start with "git " (after prefix stripping)

**Location**: `src/tui/mod.rs` - `categorize_command()`

**Edge Cases**:

1. **Nested directories**: Handles any depth
   ```toml
   cmd = "cd a/b/c/d/e && npm run build"  # Works correctly
   ```

2. **Chained commands**: Partial support
   ```toml
   # Works - strips cd, finds "build" keyword
   cmd = "cd frontend && npm install && npm run build"  # -> Build

   # Edge case - git check happens after strip, so this won't match Git filter
   cmd = "cd dir && git pull && npm run build"  # -> Build (not Git)
   ```

3. **Keyword conflicts**: Test keywords checked before Build
   ```toml
   cmd = "npm run test"  # -> Test (not Build)
   ```

---

## Project Discovery

### Scan Depth
**Setting**: `max_depth(3)` in directory walker

**Location**: `src/projects.rs`

**Implications**:
- Projects nested deeper than 3 levels won't be discovered
- Example paths that work:
  - `~/projects/my-project/byte.toml` (depth 2)
  - `~/projects/category/my-project/byte.toml` (depth 3)
  - `~/projects/category/subcategory/my-project/byte.toml` (depth 4) - NOT FOUND

**Workarounds for deeper nesting**:
- Use `cd subdir &&` pattern in commands
- Place byte.toml at shallower depth
- Register deeper paths in `workspace.registered` config

### Monorepo Support
**Pattern**: byte.toml at root, use `cd subdir &&` for subproject commands

**Example**:
```toml
# At: ~/projects/monorepo/byte.toml
[build]
frontend = "cd frontend && npm run build"
backend = "cd backend && cargo build"
```

---

## Path Management and Security

### SafePath Abstraction
**Decision**: Centralize all path handling through `SafePath` struct

**Location**: `src/path/mod.rs`

**Features**:
- Tilde expansion (`~/path` -> `/Users/name/path`)
- Canonicalization (symlink resolution, absolute paths)
- Path traversal protection in `join()` method
- Remote path detection (SSH, UNC, URLs)
- Permission validation
- Triple representation: original/expanded/canonical

**Why**:
- Single source of truth for path operations
- Consistent validation and error messages
- Easier to test and maintain
- Future-proofing for remote execution

### Project Name Validation
**Decision**: Strict validation to prevent path traversal and filesystem conflicts

**Location**: `src/projects.rs`

**Validation Rules**:
1. **No path separators**: Blocks `/`, `\`, `\0`
2. **Reserved names** (case-insensitive):
   - Build artifacts: `target`, `node_modules`, `dist`, `build`, `out`
   - IDE configs: `.vscode`, `.idea`, `.vs`, `.fleet`
   - Git/Byte: `.git`, `.byte`
   - Windows reserved: `CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`
3. **Length limit**: Max 255 bytes (filesystem limit)
4. **Character set**: ASCII alphanumeric + `-_` only
5. **Leading characters**: Cannot start with `.` (hidden) or `-` (flag)
6. **Trailing characters**: Cannot end with `.` (Windows issue)

### Remote Path Planning
**Decision**: Detect and reject remote paths now, with helpful error messages

**Supported in future**: SSH (`user@host:/path`), UNC (`\\server\share`), URLs (`ssh://`, `sftp://`, `smb://`)

**Current behavior**: SafePath rejects these with informative messages about future support.

---

## Logging

### File-based Logging
**Location**: `.byte/logs/byte.log` (project-local) or `~/.byte/logs/byte.log` (global)

**Categories**:
- `DISCOVERY`: Project scanning at workspace level
- `SCAN`: Directory walking and byte.toml detection
- `EXEC`: Command execution (with stdout/stderr on failure)
- `HOTLOAD`: State reloading
- `WATCHER`: File change detection

**Why no eprintln**: TUI uses alternate screen mode - eprintln writes directly to screen and corrupts the display

**Location**: `src/log.rs`

---

## Error Handling

### Command Execution Failures
**Behavior**: Failed commands log stderr and stdout for debugging

**Format**:
```
[timestamp] ERROR [EXEC] Failed: command
[timestamp] ERROR [EXEC]   stderr: error message
[timestamp] ERROR [EXEC]   stdout: output if any
```

---

## Future Considerations

### Potential Improvements
1. **Smarter chained command categorization**: Parse each command in chain separately
2. **Configurable scan depth**: Allow users to set max_depth in config
3. **Command source tracking**: Tag commands with their TOML section for better categorization
4. **Regex-based categorization**: More flexible than keyword matching
5. **User-defined filters**: Allow custom categorization rules in config
