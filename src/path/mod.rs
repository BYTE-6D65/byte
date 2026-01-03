/// Safe path handling with validation and expansion
///
/// Provides centralized path management with:
/// - Tilde expansion (~/ → /Users/name/)
/// - Path normalization (trailing slashes, etc.)
/// - Canonicalization (symlink resolution, absolute paths)
/// - Permission validation
/// - Remote path detection (for future features)

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Safe path with validation and expansion
///
/// Stores multiple representations of the same path:
/// - `original`: User input (for display and config storage)
/// - `expanded`: Tilde-expanded path
/// - `canonical`: Resolved symlinks, absolute path (if exists)
#[derive(Debug, Clone)]
pub struct SafePath {
    /// Original user input (e.g., "~/projects")
    #[allow(dead_code)]
    original: String,

    /// Expanded path with tilde replaced (e.g., "/Users/name/projects")
    expanded: PathBuf,

    /// Canonical path - symlinks resolved, absolute (None if doesn't exist yet)
    canonical: Option<PathBuf>,
}

impl SafePath {
    /// Create SafePath from user input
    ///
    /// Handles:
    /// - Tilde expansion (~/ → home directory)
    /// - Path normalization
    /// - Canonicalization (if path exists)
    /// - Remote path detection (rejects for now)
    ///
    /// # Examples
    /// ```
    /// use byte::path::SafePath;
    ///
    /// let path = SafePath::from_user_input("~/projects").unwrap();
    /// assert!(!path.expanded().to_str().unwrap().contains('~'));
    /// ```
    pub fn from_user_input(input: &str) -> Result<Self> {
        // 1. Empty check
        let trimmed = input.trim();
        if trimmed.is_empty() {
            anyhow::bail!("Path cannot be empty");
        }

        // 2. Detect and reject remote paths (for future implementation)
        Self::reject_remote_paths(input)?;

        // 3. Expand tilde
        let expanded_str = shellexpand::tilde(input).to_string();
        let expanded = PathBuf::from(&expanded_str);

        // 4. Try to canonicalize (resolves symlinks, makes absolute)
        // Don't fail if path doesn't exist - some operations create it
        let canonical = expanded.canonicalize().ok();

        Ok(Self {
            original: input.to_string(),
            expanded,
            canonical,
        })
    }

    /// Detect remote/network paths and reject with helpful message
    fn reject_remote_paths(input: &str) -> Result<()> {
        // Windows UNC paths
        if input.starts_with("\\\\") {
            anyhow::bail!(
                "Network UNC paths not yet supported: {}\n\
                 Remote execution is planned for a future release.",
                input
            );
        }

        // SSH-style paths: user@host:/path
        if input.contains('@') && input.contains(':') {
            // Check it's not just a Windows path like C:\...
            #[cfg(not(windows))]
            {
                anyhow::bail!(
                    "SSH/remote paths not yet supported: {}\n\
                     Remote execution is planned for a future release.",
                    input
                );
            }
        }

        // URL-style remote paths
        if input.starts_with("ssh://")
            || input.starts_with("sftp://")
            || input.starts_with("smb://")
            || input.starts_with("nfs://")
        {
            anyhow::bail!(
                "Remote URL paths not yet supported: {}\n\
                 Remote execution is planned for a future release.",
                input
            );
        }

        Ok(())
    }

    /// Get the expanded path (tilde replaced)
    pub fn expanded(&self) -> &Path {
        &self.expanded
    }

    /// Get the canonical path if available (symlinks resolved, absolute)
    pub fn canonical(&self) -> Option<&Path> {
        self.canonical.as_deref()
    }

    /// Get the original user input
    #[allow(dead_code)]
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Get the best available path representation
    ///
    /// Returns canonical if available, otherwise expanded
    #[allow(dead_code)]
    pub fn as_path(&self) -> &Path {
        self.canonical().unwrap_or(self.expanded())
    }

    /// Check if path exists
    pub fn exists(&self) -> bool {
        self.expanded.exists()
    }

    /// Validate that path exists and is a directory
    pub fn validate_directory(&self) -> Result<()> {
        if !self.exists() {
            anyhow::bail!("Path does not exist: {}", self.expanded.display());
        }

        if !self.expanded.is_dir() {
            anyhow::bail!(
                "Path is not a directory: {}",
                self.expanded.display()
            );
        }

        Ok(())
    }

    /// Validate that path is writable
    ///
    /// Checks:
    /// - Path exists and is a directory
    /// - Read, write, and execute permissions (Unix)
    /// - Not read-only (cross-platform)
    /// - Can actually create files (test write)
    #[allow(dead_code)]
    pub fn validate_writable(&self) -> Result<()> {
        // Must be a directory first
        self.validate_directory()?;

        let metadata = fs::metadata(&self.expanded)?;
        let permissions = metadata.permissions();

        // Check Unix permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = permissions.mode();

            // Need read + write + execute (rwx for owner)
            if mode & 0o700 != 0o700 {
                anyhow::bail!(
                    "Insufficient permissions on: {} (need owner rwx)\n\
                     Current mode: {:o}",
                    self.expanded.display(),
                    mode
                );
            }
        }

        // Check read-only flag (cross-platform)
        if permissions.readonly() {
            anyhow::bail!(
                "Directory is read-only: {}",
                self.expanded.display()
            );
        }

        // Try actually creating a test file
        let test_file = self.expanded.join(".byte_write_test");
        match fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_file); // Clean up
                Ok(())
            }
            Err(e) => {
                anyhow::bail!(
                    "Cannot write to directory: {} ({})",
                    self.expanded.display(),
                    e
                );
            }
        }
    }

    /// Compare two paths by their canonical form (or expanded if not canonical)
    ///
    /// This handles symlinks and relative paths correctly
    pub fn equals(&self, other: &SafePath) -> bool {
        match (&self.canonical, &other.canonical) {
            (Some(a), Some(b)) => a == b,
            _ => self.expanded == other.expanded,
        }
    }

    /// Join with a relative path component
    ///
    /// Validates that component doesn't contain path separators (prevents traversal)
    ///
    /// # Examples
    /// ```
    /// use byte::path::SafePath;
    ///
    /// let base = SafePath::from_user_input("/tmp").unwrap();
    /// let subdir = base.join("mydir").unwrap();
    /// assert!(subdir.expanded().ends_with("mydir"));
    /// ```
    #[allow(dead_code)]
    pub fn join(&self, component: &str) -> Result<SafePath> {
        // Validate component doesn't contain path separators
        if component.contains('/') || component.contains('\\') {
            anyhow::bail!(
                "Path component cannot contain separators: '{}'\n\
                 Use multiple join() calls or from_user_input() for complex paths.",
                component
            );
        }

        // Also check for traversal attempts
        if component == ".." || component == "." {
            anyhow::bail!(
                "Path component cannot be '.' or '..': '{}'\n\
                 Use from_user_input() for relative paths.",
                component
            );
        }

        let new_path = self.expanded.join(component);

        Ok(SafePath {
            original: format!("{}/{}", self.original, component),
            expanded: new_path.clone(),
            canonical: new_path.canonicalize().ok(),
        })
    }

    /// Create SafePath for a workspace (must be writable directory)
    ///
    /// Convenience wrapper that validates writability
    #[allow(dead_code)]
    pub fn workspace(path: &str) -> Result<Self> {
        let safe = Self::from_user_input(path)?;
        safe.validate_writable()?;
        Ok(safe)
    }

    /// Create SafePath for a project root (must be existing directory)
    ///
    /// Convenience wrapper that validates directory exists
    #[allow(dead_code)]
    pub fn project_root(path: &str) -> Result<Self> {
        let safe = Self::from_user_input(path)?;
        safe.validate_directory()?;
        Ok(safe)
    }
}

impl std::fmt::Display for SafePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expanded.display())
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        self.expanded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_tilde_expansion() {
        let path = SafePath::from_user_input("~/test").unwrap();
        assert!(!path.expanded().to_str().unwrap().contains('~'));
        assert!(path.original().contains('~'));
    }

    #[test]
    fn test_empty_path() {
        assert!(SafePath::from_user_input("").is_err());
        assert!(SafePath::from_user_input("   ").is_err());
    }

    #[test]
    fn test_remote_path_detection() {
        // UNC paths
        assert!(SafePath::from_user_input("\\\\server\\share").is_err());

        // SSH paths
        #[cfg(not(windows))]
        {
            assert!(SafePath::from_user_input("user@host:/path").is_err());
        }

        // URL paths
        assert!(SafePath::from_user_input("ssh://user@host/path").is_err());
        assert!(SafePath::from_user_input("sftp://server/path").is_err());
        assert!(SafePath::from_user_input("smb://server/share").is_err());
    }

    #[test]
    fn test_canonical_path() {
        // Create a temp directory
        let temp_dir = std::env::temp_dir().join("byte_test_canonical");
        let _ = fs::create_dir_all(&temp_dir);

        let path = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        // Should have canonical path since it exists
        assert!(path.canonical().is_some());
        assert!(path.canonical().unwrap().is_absolute());

        // Cleanup
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_join_validation() {
        let temp_dir = std::env::temp_dir();
        let base = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        // Valid join
        assert!(base.join("subdir").is_ok());
        assert!(base.join("my-dir").is_ok());
        assert!(base.join("dir_name").is_ok());

        // Invalid joins
        assert!(base.join("../etc").is_err()); // Contains separator
        assert!(base.join("sub/dir").is_err()); // Contains separator
        assert!(base.join("..").is_err()); // Parent reference
        assert!(base.join(".").is_err()); // Current dir
    }

    #[test]
    fn test_path_equality() {
        let temp_dir = std::env::temp_dir();
        let path1 = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();
        let path2 = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        assert!(path1.equals(&path2));
    }

    #[test]
    fn test_validate_directory() {
        let temp_dir = std::env::temp_dir();
        let path = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        // Should pass - temp dir exists and is a directory
        assert!(path.validate_directory().is_ok());

        // Non-existent path should fail
        let fake_path = SafePath::from_user_input("/this/does/not/exist/byte_test").unwrap();
        assert!(fake_path.validate_directory().is_err());
    }

    #[test]
    fn test_validate_writable() {
        let temp_dir = std::env::temp_dir();
        let path = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        // Should pass - temp dir is writable
        assert!(path.validate_writable().is_ok());
    }

    #[test]
    fn test_display() {
        let temp_dir = std::env::temp_dir();
        let path = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        let display_str = format!("{}", path);
        assert!(!display_str.is_empty());
    }

    #[test]
    fn test_as_ref() {
        let temp_dir = std::env::temp_dir();
        let path = SafePath::from_user_input(temp_dir.to_str().unwrap()).unwrap();

        let path_ref: &Path = path.as_ref();
        assert_eq!(path_ref, path.expanded());
    }

    #[test]
    fn test_workspace_convenience() {
        let temp_dir = std::env::temp_dir();
        let workspace = SafePath::workspace(temp_dir.to_str().unwrap());

        // Should succeed - temp is writable
        assert!(workspace.is_ok());

        // Non-existent should fail
        let fake_workspace = SafePath::workspace("/this/does/not/exist/byte_test");
        assert!(fake_workspace.is_err());
    }

    #[test]
    fn test_project_root_convenience() {
        let temp_dir = std::env::temp_dir();
        let project = SafePath::project_root(temp_dir.to_str().unwrap());

        // Should succeed - temp exists and is dir
        assert!(project.is_ok());
    }
}
