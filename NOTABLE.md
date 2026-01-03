# Notable Behaviors, Edge Cases, and Design Decisions

This document tracks important behaviors, edge cases, and design decisions in the Byte project manager.

## Command Execution

### Shell Operators Support
**Decision**: Allow full shell operators (`&&`, `||`, `|`, `;`, `>`, `<`, etc.) in byte.toml commands.

**Rationale**:
- byte.toml is a trusted developer config file, not runtime user input
- Enables monorepo workflows: `cd subdir && command`
- Allows command chaining: `build && test`
- Security maintained by command whitelist for direct binary execution

**Location**: `src/exec/mod.rs:182` - `validate_shell_command()`

### Command Categorization (Menu 2 Filters)

**Implementation**: Keyword-based categorization (language-agnostic)

**How it works**:
1. Strips `cd dir &&` prefix (any depth: `cd a/b/c && cmd` → `cmd`)
2. Matches on action keywords:
   - **Build**: build, compile, bundle, dev, run, start, watch, serve
   - **Test**: test, spec, coverage, bench
   - **Lint**: lint, fmt, format, clippy, check, prettier, eslint
   - **Git**: must start with "git " (after prefix stripping)

**Location**: `src/tui/mod.rs:1372` - `categorize_command()`

**Edge Cases**:

1. **Nested directories**: ✓ Handles any depth
   ```toml
   cmd = "cd a/b/c/d/e && npm run build"  # Works correctly
   ```

2. **Chained commands**: ⚠️ Partial support
   ```toml
   # Works - strips cd, finds "build" keyword
   cmd = "cd frontend && npm install && npm run build"  → Build

   # Edge case - git check happens after strip, so this won't match Git filter
   cmd = "cd dir && git pull && npm run build"  → Build (not Git)
   ```

   **Note**: Currently strips only the first `cd dir &&` prefix. Subsequent commands in a chain are analyzed together. This is acceptable for now but may need refinement if users frequently chain git commands.

3. **Keyword conflicts**: Test keywords checked before Build
   ```toml
   # "test" is more specific than "run", so Test wins
   cmd = "npm run test"  → Test ✓
   ```

## Project Discovery

### Scan Depth
**Setting**: `max_depth(3)` in directory walker

**Location**: `src/projects.rs:73`

**Implications**:
- Projects nested deeper than 3 levels won't be discovered
- Example paths that work:
  - `~/projects/my-project/byte.toml` (depth 2) ✓
  - `~/projects/category/my-project/byte.toml` (depth 3) ✓
  - `~/projects/category/subcategory/my-project/byte.toml` (depth 4) ✗

**Workaround for deeper nesting**:
- Use `cd subdir &&` pattern in commands
- Place byte.toml at shallower depth
- Register deeper paths in workspace.registered config

### Monorepo Support
**Pattern**: byte.toml at root, use `cd subdir &&` for subproject commands

**Example**:
```toml
# At: ~/projects/monorepo/byte.toml
[build]
frontend = "cd frontend && npm run build"
backend = "cd backend && cargo build"
```

This allows single byte.toml to manage multiple subprojects while staying within scan depth limits.

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

## Error Handling

### Command Execution Failures
**Behavior**: Failed commands now log stderr and stdout for debugging

**Format**:
```
[timestamp] ERROR [EXEC] Failed: command
[timestamp] ERROR [EXEC]   stderr: error message
[timestamp] ERROR [EXEC]   stdout: output if any
```

**Location**: `src/tui/mod.rs:449-455`

## Path Management and Security

### SafePath Abstraction
**Decision**: Centralize all path handling through `SafePath` struct

**Location**: `src/path/mod.rs`

**Features**:
- Tilde expansion (`~/path` → `/Users/name/path`)
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

**Migration**: Replaced 16 scattered `shellexpand::tilde()` calls across codebase

### Project Name Validation
**Decision**: Strict validation to prevent path traversal and filesystem conflicts

**Location**: `src/projects.rs:23-87`

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

**Example valid names**:
- `my-project`, `MyProject`, `my_project`, `project123`
- `app.v2`, `my-app.2024` (dots in middle are OK)

**Example invalid names**:
- `../../etc/evil` (path traversal)
- `.hidden` (starts with dot)
- `target` (reserved name)
- `my project` (space not allowed)
- `project@v2` (@ not allowed)

**Integration**: Used in `init_project()` and TUI command parsing

### Remote Path Planning
**Decision**: Detect and reject remote paths **now**, with helpful error messages

**Supported in future**: SSH (`user@host:/path`), UNC (`\\server\share`), URLs (`ssh://`, `sftp://`, `smb://`)

**Current behavior**: SafePath rejects these with:
```
Remote URL paths not yet supported: ssh://user@host/path
Remote execution is planned for a future release.
```

**Why helpful rejection over silent failure**:
- Clear user feedback ("not yet supported" vs "invalid path")
- Documents the feature for future implementation
- Easy jump-back-in point (just change `bail!` to implementation)

## Future Considerations

### Potential Improvements
1. **Smarter chained command categorization**: Parse each command in chain separately
2. **Configurable scan depth**: Allow users to set max_depth in config
3. **Command source tracking**: Tag commands with their TOML section ([build] vs [commands]) for better categorization
4. **Regex-based categorization**: More flexible than keyword matching
5. **User-defined filters**: Allow custom categorization rules in config

---

**Last Updated**: 2026-01-02
