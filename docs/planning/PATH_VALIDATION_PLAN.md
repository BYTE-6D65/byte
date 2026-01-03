# Path Management & Name Validation - Current State & Plan

## Current State Analysis

### 1. Project Name Validation
**Status**: ❌ **NONE**

**Where names come from**:
- CLI: `byte init <ecosystem> <project_type> <name>` → `src/cli/mod.rs:22`
- TUI: Forms input (via "byte init" command string parsing) → `src/tui/mod.rs:495-506`

**Current flow** (`src/projects.rs:122-156`):
```rust
pub fn init_project(
    workspace_path: &str,
    ecosystem: &str,
    project_type: &str,
    name: &str,  // ❌ NO VALIDATION
) -> Result<PathBuf> {
    let workspace = shellexpand::tilde(workspace_path).to_string();
    let workspace_path = Path::new(&workspace);

    // Create project directory
    let project_path = workspace_path.join(name);  // ⚠️ VULNERABLE TO PATH TRAVERSAL
    if project_path.exists() {
        anyhow::bail!("Project directory already exists: {}", project_path.display());
    }

    fs::create_dir_all(&project_path)?;  // ⚠️ WILL CREATE ANYWHERE
    // ...
}
```

**Vulnerabilities**:
- ✗ No check for path separators: `../../etc/passwd`
- ✗ No check for reserved names: `.git`, `.byte`, `target`
- ✗ No length limits (filesystem max = 255, but no enforcement)
- ✗ No character restrictions (could use special chars, unicode, etc.)
- ✗ No check for empty/whitespace-only names

**Attack vectors**:
```bash
# Path traversal
byte init go cli "../../etc/evil"  # Creates directory outside workspace

# Reserved name conflicts
byte init go cli ".git"  # Could corrupt git repos

# Filesystem abuse
byte init go cli "$(cat /etc/passwd)"  # Shell expansion if name used in commands

# Unicode confusion
byte init go cli "prоject"  # Uses Cyrillic 'о' instead of Latin 'o'
```

---

### 2. Path Management
**Status**: ⚠️ **SCATTERED, NO ABSTRACTION**

**Current pattern** (repeated **18 times** across codebase):
```rust
let expanded = shellexpand::tilde(path).to_string();
let normalized = expanded.trim_end_matches('/').to_string();
let path_buf = PathBuf::from(&normalized);
// Sometimes: canonicalize, sometimes not
// Sometimes: check exists, sometimes not
// Sometimes: validate writable, usually not
```

**Locations using this pattern**:
1. `src/projects.rs:21` - Workspace scanning
2. `src/projects.rs:37` - Registered paths
3. `src/projects.rs:129` - Project init
4. `src/tui/mod.rs:299` - Project counting
5. `src/tui/mod.rs:466` - Command execution
6. `src/tui/mod.rs:605` - Path completion
7. `src/tui/mod.rs:677` - Directory picker
8. `src/tui/mod.rs:1277` - Workspace matching
9. `src/tui/mod.rs:1327` - Hotload counting
10. `src/tui/mod.rs:1589` - File watcher
11. `src/tui/mod.rs:1702` - Fuzzy matches
12. `src/config/mod.rs:61` - Add workspace
13. `src/config/mod.rs:80` - Primary workspace check
14. `src/config/mod.rs:89` - Registered check
15. `src/config/mod.rs:112` - Remove workspace
16. `src/config/mod.rs:117` - Retention filter

**Inconsistencies**:
- ✓ Always uses `shellexpand::tilde()`
- ⚠️ Sometimes trims trailing `/`, sometimes not
- ⚠️ Sometimes calls `canonicalize()`, sometimes not
- ⚠️ Sometimes checks `exists()`, sometimes not
- ✗ Never checks write permissions
- ✗ Never validates directory (vs file)
- ✗ No safety checks against symlink attacks

**Example of inconsistency**:
```rust
// config/mod.rs - DOES canonicalize and compare
let canonical = path_buf.canonicalize()?;
if canonical == primary_canonical { ... }

// projects.rs - DOESN'T canonicalize
let workspace = shellexpand::tilde(workspace_path).to_string();
let workspace_path = Path::new(&workspace);  // Just uses it directly
```

---

## Proposed Solution: Two-Phase Approach

### Phase 1: Name Validation (Quick Fix)
**Goal**: Stop path traversal attacks immediately
**Time**: 30 minutes
**Location**: New function in `src/projects.rs` or new `src/validation.rs`

```rust
/// Validate project name for safety and filesystem compatibility
pub fn validate_project_name(name: &str) -> anyhow::Result<()> {
    // 1. Empty/whitespace check
    if name.trim().is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }

    // 2. Path separator check (prevents traversal)
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        anyhow::bail!("Project name cannot contain path separators (/, \\, or null)");
    }

    // 3. Reserved name check
    const RESERVED: &[&str] = &[
        ".", "..", ".git", ".byte",
        "target", "node_modules", "dist", "build",
        ".vscode", ".idea", ".vs"
    ];
    if RESERVED.contains(&name) || RESERVED.contains(&name.to_lowercase().as_str()) {
        anyhow::bail!("Project name '{}' is reserved", name);
    }

    // 4. Length check (filesystem limit)
    if name.len() > 255 {
        anyhow::bail!("Project name too long (max 255 characters, got {})", name.len());
    }

    // 5. Character restrictions (conservative)
    let valid_chars = name.chars().all(|c| {
        c.is_alphanumeric()
        || c == '-'
        || c == '_'
        || c == '.'  // Allow dots for extensions/versions
    });

    if !valid_chars {
        anyhow::bail!(
            "Project name can only contain letters, numbers, hyphens, underscores, and dots. Got: '{}'",
            name
        );
    }

    // 6. Starts/ends with safe characters (not dot or hyphen)
    if name.starts_with('.') || name.starts_with('-') {
        anyhow::bail!("Project name cannot start with '.' or '-'");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("MyProject").is_ok());
        assert!(validate_project_name("my_project_v2").is_ok());
        assert!(validate_project_name("app-2024").is_ok());
    }

    #[test]
    fn test_path_traversal() {
        assert!(validate_project_name("../../etc/passwd").is_err());
        assert!(validate_project_name("../parent").is_err());
        assert!(validate_project_name("sub/dir").is_err());
    }

    #[test]
    fn test_reserved_names() {
        assert!(validate_project_name(".git").is_err());
        assert!(validate_project_name(".byte").is_err());
        assert!(validate_project_name("target").is_err());
    }

    #[test]
    fn test_invalid_characters() {
        assert!(validate_project_name("my project").is_err());  // space
        assert!(validate_project_name("my@project").is_err());  // @
        assert!(validate_project_name("my$project").is_err());  // $
    }

    #[test]
    fn test_edge_cases() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("   ").is_err());
        assert!(validate_project_name(".hidden").is_err());  // starts with dot
        assert!(validate_project_name("-dash").is_err());    // starts with dash
    }
}
```

**Integration points**:
1. `src/projects.rs:127` - Call at top of `init_project()`
2. `src/tui/mod.rs:506` - Call when parsing "byte init" command

---

### Phase 2: Path Abstraction (Proper Fix)
**Goal**: Centralize all path handling, prevent future issues
**Time**: 2-3 hours
**Location**: New module `src/path/mod.rs`

```rust
/// Safe path handling with validation
#[derive(Debug, Clone)]
pub struct SafePath {
    /// Original user input (for display and config storage)
    original: String,

    /// Expanded path (tilde replaced)
    expanded: PathBuf,

    /// Canonical path (symlinks resolved, absolute)
    canonical: Option<PathBuf>,
}

impl SafePath {
    /// Create from user input (e.g., "~/projects")
    pub fn from_user_input(input: &str) -> anyhow::Result<Self> {
        // Validate non-empty
        if input.trim().is_empty() {
            anyhow::bail!("Path cannot be empty");
        }

        // Expand tilde
        let expanded_str = shellexpand::tilde(input).to_string();
        let expanded = PathBuf::from(&expanded_str);

        // Try to canonicalize (resolves symlinks, makes absolute)
        // Don't fail if path doesn't exist yet - some operations create it
        let canonical = expanded.canonicalize().ok();

        Ok(Self {
            original: input.to_string(),
            expanded,
            canonical,
        })
    }

    /// Get the expanded path (tilde replaced)
    pub fn expanded(&self) -> &Path {
        &self.expanded
    }

    /// Get the canonical path if available
    pub fn canonical(&self) -> Option<&Path> {
        self.canonical.as_deref()
    }

    /// Get original user input
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Check if path exists
    pub fn exists(&self) -> bool {
        self.expanded.exists()
    }

    /// Validate path exists and is a directory
    pub fn validate_directory(&self) -> anyhow::Result<()> {
        if !self.exists() {
            anyhow::bail!("Path does not exist: {}", self.expanded.display());
        }

        if !self.expanded.is_dir() {
            anyhow::bail!("Path is not a directory: {}", self.expanded.display());
        }

        Ok(())
    }

    /// Validate path is writable
    pub fn validate_writable(&self) -> anyhow::Result<()> {
        self.validate_directory()?;

        let metadata = fs::metadata(&self.expanded)?;
        let permissions = metadata.permissions();

        // Check Unix permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = permissions.mode();

            // Need read + write + execute
            if mode & 0o700 != 0o700 {
                anyhow::bail!(
                    "Insufficient permissions on: {} (need rwx)",
                    self.expanded.display()
                );
            }
        }

        // Check read-only flag
        if permissions.readonly() {
            anyhow::bail!("Directory is read-only: {}", self.expanded.display());
        }

        // Try creating a test file
        let test_file = self.expanded.join(".byte_write_test");
        match fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_file);  // Clean up
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Cannot write to directory: {} ({})", self.expanded.display(), e);
            }
        }
    }

    /// Compare two paths by canonical form
    pub fn equals(&self, other: &SafePath) -> bool {
        match (&self.canonical, &other.canonical) {
            (Some(a), Some(b)) => a == b,
            _ => self.expanded == other.expanded,
        }
    }

    /// Join with a relative path component (validates component)
    pub fn join(&self, component: &str) -> anyhow::Result<SafePath> {
        // Validate component doesn't contain path separators
        if component.contains('/') || component.contains('\\') {
            anyhow::bail!("Path component cannot contain separators: {}", component);
        }

        let new_path = self.expanded.join(component);

        Ok(SafePath {
            original: format!("{}/{}", self.original, component),
            expanded: new_path.clone(),
            canonical: new_path.canonicalize().ok(),
        })
    }
}

impl std::fmt::Display for SafePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expanded.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilde_expansion() {
        let path = SafePath::from_user_input("~/test").unwrap();
        assert!(!path.expanded().to_str().unwrap().contains('~'));
    }

    #[test]
    fn test_empty_path() {
        assert!(SafePath::from_user_input("").is_err());
        assert!(SafePath::from_user_input("   ").is_err());
    }

    #[test]
    fn test_join_validation() {
        let base = SafePath::from_user_input("/tmp").unwrap();

        // Valid join
        assert!(base.join("subdir").is_ok());

        // Invalid joins
        assert!(base.join("../etc").is_err());
        assert!(base.join("sub/dir").is_err());
    }
}
```

**Migration strategy**:
1. Create `src/path/mod.rs` with `SafePath`
2. Add helper functions for common patterns:
   ```rust
   impl SafePath {
       pub fn workspace(path: &str) -> anyhow::Result<Self> {
           let safe = Self::from_user_input(path)?;
           safe.validate_writable()?;
           Ok(safe)
       }

       pub fn project_root(path: &str) -> anyhow::Result<Self> {
           let safe = Self::from_user_input(path)?;
           safe.validate_directory()?;
           Ok(safe)
       }
   }
   ```
3. Replace usage incrementally:
   - Start with `src/projects.rs` (3 usages)
   - Move to `src/config/mod.rs` (6 usages)
   - Finally `src/tui/mod.rs` (9 usages)
4. Add deprecation warnings to direct `shellexpand::tilde()` usage

---

## Implementation Plan

### Step 1: Quick Security Fix (30 min)
- [ ] Create `validate_project_name()` function
- [ ] Add tests
- [ ] Call in `init_project()` at line 127
- [ ] Call in TUI command parsing at line 506
- [ ] Test with malicious inputs

### Step 2: Path Abstraction Core (1 hour)
- [ ] Create `src/path/mod.rs`
- [ ] Implement `SafePath` struct
- [ ] Implement `from_user_input()`
- [ ] Implement validation methods
- [ ] Add comprehensive tests

### Step 3: Migration - projects.rs (30 min)
- [ ] Replace line 21: `discover_projects()`
- [ ] Replace line 37: registered paths loop
- [ ] Replace line 129: `init_project()` workspace
- [ ] Test project discovery and init

### Step 4: Migration - config/mod.rs (45 min)
- [ ] Replace `add_workspace_path()` - lines 61-93
- [ ] Replace `remove_workspace_path()` - lines 112-119
- [ ] Update workspace comparison logic
- [ ] Test config operations

### Step 5: Migration - tui/mod.rs (1 hour)
- [ ] Replace command execution paths (2 usages)
- [ ] Replace path completion logic (2 usages)
- [ ] Replace workspace operations (5 usages)
- [ ] Test TUI workflows

### Step 6: Cleanup (15 min)
- [ ] Remove direct `shellexpand::tilde()` imports where replaced
- [ ] Update AUDIT_STATUS.md - mark API #2 complete
- [ ] Update NOTABLE.md - document SafePath pattern
- [ ] Run full test suite

---

## Decision Points

### 1. Character Restrictions - How Strict?

**Conservative** (recommended):
```rust
c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
```
✅ Safe, predictable
✅ Works across all filesystems
❌ Rejects some valid names (spaces, unicode)

**Permissive**:
```rust
!c.is_control() && c != '/' && c != '\\'
```
✅ Allows spaces, unicode, special chars
❌ Might break on Windows FAT32
❌ Shell escaping issues

**Recommendation**: Start conservative, relax later if users complain

---

### 2. Reserved Names - How Many?

**Minimal**:
```rust
&[".", "..", ".git", ".byte"]
```

**Comprehensive** (recommended):
```rust
&[
    ".", "..", ".git", ".byte",
    "target", "node_modules", "dist", "build",  // Build artifacts
    ".vscode", ".idea", ".vs",  // IDE configs
    "CON", "PRN", "AUX", "NUL",  // Windows reserved
]
```

**Recommendation**: Comprehensive - prevents common foot-guns

---

### 3. SafePath - Fail on Non-Existent Paths?

**Option A**: Require existence
```rust
pub fn from_user_input(input: &str) -> Result<Self> {
    // ...
    if !expanded.exists() {
        bail!("Path does not exist");
    }
}
```
✅ Catches typos early
❌ Can't use for paths we're about to create

**Option B**: Allow non-existent (recommended)
```rust
pub fn from_user_input(input: &str) -> Result<Self> {
    // ...
    let canonical = expanded.canonicalize().ok();  // Optional
}

pub fn validate_exists(&self) -> Result<()> {  // Separate validation
    // ...
}
```
✅ Flexible for init operations
✅ Explicit validation when needed

**Recommendation**: Option B with separate validation methods

---

## Files to Create/Modify

**Create**:
- `src/path/mod.rs` - SafePath abstraction (~200 lines + tests)
- `src/validation.rs` - Name validation (~100 lines + tests) OR add to projects.rs

**Modify**:
- `src/lib.rs` - Add `pub mod path;`
- `src/projects.rs` - Add validation call, replace 3 path patterns
- `src/config/mod.rs` - Replace 6 path patterns
- `src/tui/mod.rs` - Replace 9 path patterns

**Update**:
- `AUDIT_STATUS.md` - Mark Security #2 and API #2 complete
- `NOTABLE.md` - Document SafePath pattern
- `Cargo.toml` - No new deps needed!

---

## Risk Assessment

**Risks of doing this**:
- Breaking existing configs with strict validation
- Users with valid but unusual project names get blocked
- Migration introduces bugs in path handling

**Risks of NOT doing this**:
- Path traversal attacks (user creates `../../etc/evil`)
- Filesystem corruption (reserved names)
- Inconsistent behavior (sometimes canonical, sometimes not)
- **HIGH**: Users can currently break their workspace

**Mitigation**:
- Comprehensive tests before deployment
- Clear error messages explaining why names are rejected
- Validate existing project names on scan (warn, don't fail)
- Document naming restrictions in docs

---

**Estimated Total Time**: 4-5 hours for complete implementation
**Priority**: HIGH - security issue
**Dependencies**: None
**Next Steps**: Decide on strictness levels, then start with Step 1
