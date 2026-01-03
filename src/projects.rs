use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{GlobalConfig, ProjectConfig};

/// Discovered project with path and config
#[derive(Debug, Clone)]
pub struct DiscoveredProject {
    pub path: PathBuf,
    pub config: ProjectConfig,
}

/// Validate project name for safety and filesystem compatibility
///
/// Prevents:
/// - Path traversal attacks (../, sub/dir)
/// - Reserved names (.git, target, etc.)
/// - Invalid characters
/// - Length violations
/// - Shell injection via special characters
pub fn validate_project_name(name: &str) -> Result<()> {
    // 1. Empty/whitespace check
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Project name cannot be empty");
    }

    // 2. Path separator check (prevents traversal)
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        anyhow::bail!(
            "Project name cannot contain path separators (/, \\, or null bytes)"
        );
    }

    // 3. Reserved name check (case-insensitive for cross-platform safety)
    const RESERVED: &[&str] = &[
        ".", "..", ".git", ".byte",
        "target", "node_modules", "dist", "build", "out",
        ".vscode", ".idea", ".vs", ".fleet",
        // Windows reserved names (uppercase for comparison)
        "con", "prn", "aux", "nul",
        "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
        "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];

    let name_lower = name.to_lowercase();
    if RESERVED.contains(&name) || RESERVED.contains(&name_lower.as_str()) {
        anyhow::bail!("Project name '{}' is reserved and cannot be used", name);
    }

    // 4. Length check (filesystem limit is typically 255 bytes)
    if name.len() > 255 {
        anyhow::bail!(
            "Project name too long ({} characters). Maximum is 255.",
            name.len()
        );
    }

    // 5. Character restrictions (conservative: alphanumeric + dash + underscore + dot)
    let valid_chars = name.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
    });

    if !valid_chars {
        anyhow::bail!(
            "Project name can only contain ASCII letters, numbers, hyphens (-), underscores (_), and dots (.). Got: '{}'",
            name
        );
    }

    // 6. Cannot start with dot or hyphen (hidden files, command flags)
    if name.starts_with('.') {
        anyhow::bail!("Project name cannot start with '.' (would create hidden directory)");
    }
    if name.starts_with('-') {
        anyhow::bail!("Project name cannot start with '-' (conflicts with command flags)");
    }

    // 7. Cannot end with dot (Windows filesystem issue)
    if name.ends_with('.') {
        anyhow::bail!("Project name cannot end with '.'");
    }

    Ok(())
}

/// Scan workspace and registered paths for projects
pub fn discover_projects(global_config: &GlobalConfig) -> Result<Vec<DiscoveredProject>> {
    let mut projects = Vec::new();

    // Scan workspace if auto_scan is enabled
    if global_config.workspace.auto_scan {
        match crate::path::SafePath::from_user_input(&global_config.workspace.path) {
            Ok(workspace_path) => {
                crate::log::debug("DISCOVERY", &format!("Scanning primary workspace: {}", workspace_path));

                match scan_directory(workspace_path.expanded()) {
                    Ok(workspace_projects) => {
                        crate::log::debug("DISCOVERY", &format!("Found {} projects in primary workspace", workspace_projects.len()));
                        projects.extend(workspace_projects);
                    }
                    Err(e) => {
                        crate::log::error("DISCOVERY", &format!("Failed to scan primary workspace: {}", e));
                    }
                }
            }
            Err(e) => {
                crate::log::error("DISCOVERY", &format!("Invalid workspace path '{}': {}", global_config.workspace.path, e));
            }
        }
    }

    // Scan manually registered directories for projects
    for registered_path in &global_config.workspace.registered {
        match crate::path::SafePath::from_user_input(registered_path) {
            Ok(safe_path) => {
                crate::log::debug("DISCOVERY", &format!("Scanning registered path: {}", safe_path));

                match scan_directory(safe_path.expanded()) {
                    Ok(registered_projects) => {
                        crate::log::debug("DISCOVERY", &format!("Found {} projects in {}", registered_projects.len(), safe_path));
                        for proj in &registered_projects {
                            crate::log::debug("DISCOVERY", &format!("  - {} at {}", proj.config.project.name, proj.path.display()));
                        }
                        projects.extend(registered_projects);
                    }
                    Err(e) => {
                        crate::log::error("DISCOVERY", &format!("Failed to scan {}: {}", safe_path, e));
                    }
                }
            }
            Err(e) => {
                crate::log::error("DISCOVERY", &format!("Invalid registered path '{}': {}", registered_path, e));
            }
        }
    }

                crate::log::debug("DISCOVERY", &format!("Total projects discovered: {}", projects.len()));
    Ok(projects)
}

/// Scan a directory for byte.toml files
fn scan_directory(path: &Path) -> Result<Vec<DiscoveredProject>> {
    let mut projects = Vec::new();

    crate::log::debug("SCAN", &format!("Scanning directory: {}", path.display()));

    if !path.exists() {
        crate::log::debug("SCAN", &format!("Path does not exist: {}", path.display()));
        return Ok(projects);
    }

    // Walk directory looking for byte.toml files
    let mut entry_count = 0;
    for entry in WalkDir::new(path)
        .max_depth(3) // Don't go too deep
        .follow_links(false)
    {
        match entry {
            Ok(entry) => {
                entry_count += 1;
                let file_name = entry.file_name().to_string_lossy();
                if file_name == "byte.toml" {
                    crate::log::debug("SCAN", &format!("Found byte.toml at: {}", entry.path().display()));

                    if let Some(project_dir) = entry.path().parent() {
                        match load_project(project_dir.to_str().unwrap_or("")) {
                            Ok(project) => {
                                crate::log::debug("SCAN", &format!("Successfully loaded project: {}", project.config.project.name));
                                projects.push(project);
                            }
                            Err(e) => {
                                crate::log::error("SCAN", &format!("Failed to load project from {}: {}", project_dir.display(), e));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                crate::log::error("SCAN", &format!("Error reading entry: {}", e));
            }
        }
    }

    crate::log::debug("SCAN", &format!("Scanned {} entries, found {} projects", entry_count, projects.len()));

    Ok(projects)
}

/// Load a project from a directory containing byte.toml
fn load_project(path: &str) -> Result<DiscoveredProject> {
    let project_path = PathBuf::from(path);
    let config_path = project_path.join("byte.toml");

    let config_content = fs::read_to_string(&config_path)?;
    let config: ProjectConfig = toml::from_str(&config_content)?;

    Ok(DiscoveredProject {
        path: project_path,
        config,
    })
}

/// Initialize a new project using the FS and Exec APIs
pub fn init_project(
    workspace_path: &str,
    ecosystem: &str,
    project_type: &str,
    name: &str,
) -> Result<PathBuf> {
    // Validate project name for security and filesystem compatibility
    validate_project_name(name)?;

    // Expand and validate workspace path
    let safe_workspace = crate::path::SafePath::from_user_input(workspace_path)?;
    let workspace_path = safe_workspace.expanded();

    // Create workspace if it doesn't exist
    if !workspace_path.exists() {
        fs::create_dir_all(workspace_path)?;
    }

    // Create project directory
    let project_path = workspace_path.join(name);
    if project_path.exists() {
        anyhow::bail!(
            "Project directory already exists: {}",
            project_path.display()
        );
    }

    fs::create_dir_all(&project_path)?;

    // Use FS API to initialize project structure
    let fs_api = crate::fs::ProjectFileSystem::new(&project_path)?;
    fs_api.init_project(ecosystem, project_type, name)?;

    // Initialize git repository
    init_git_repo(&project_path, name)?;

    Ok(project_path)
}

/// Initialize git repository with initial commit using Exec API
fn init_git_repo(project_path: &Path, name: &str) -> Result<()> {
    use crate::exec::CommandBuilder;

    // Initialize git repository
    CommandBuilder::git("init")
        .working_dir(project_path)
        .execute()?;

    // Stage all files
    CommandBuilder::git("add")
        .arg(".")
        .working_dir(project_path)
        .execute()?;

    // Create initial commit
    let commit_message = format!("Initial commit: {} project", name);
    CommandBuilder::git("commit")
        .arg("-m")
        .arg(&commit_message)
        .working_dir(project_path)
        .execute()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_names() {
        // Standard names
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("MyProject").is_ok());
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("project123").is_ok());

        // With dots (versions, extensions)
        assert!(validate_project_name("app.v2").is_ok());
        assert!(validate_project_name("my-app.2024").is_ok());

        // Mixed
        assert!(validate_project_name("My_Cool-Project.v1").is_ok());
    }

    #[test]
    fn test_path_traversal_rejected() {
        assert!(validate_project_name("../../etc/passwd").is_err());
        assert!(validate_project_name("../parent").is_err());
        assert!(validate_project_name("sub/dir").is_err());
        assert!(validate_project_name("..").is_err());
        assert!(validate_project_name(".").is_err());
        assert!(validate_project_name("path\\to\\project").is_err());
    }

    #[test]
    fn test_reserved_names_rejected() {
        // Build artifacts
        assert!(validate_project_name("target").is_err());
        assert!(validate_project_name("node_modules").is_err());
        assert!(validate_project_name("dist").is_err());
        assert!(validate_project_name("build").is_err());

        // IDE configs
        assert!(validate_project_name(".vscode").is_err());
        assert!(validate_project_name(".idea").is_err());

        // Git
        assert!(validate_project_name(".git").is_err());
        assert!(validate_project_name(".byte").is_err());

        // Windows reserved (case insensitive)
        assert!(validate_project_name("CON").is_err());
        assert!(validate_project_name("con").is_err());
        assert!(validate_project_name("PRN").is_err());
        assert!(validate_project_name("AUX").is_err());
        assert!(validate_project_name("NUL").is_err());
        assert!(validate_project_name("COM1").is_err());
        assert!(validate_project_name("LPT1").is_err());
    }

    #[test]
    fn test_invalid_characters_rejected() {
        assert!(validate_project_name("my project").is_err());  // space
        assert!(validate_project_name("my@project").is_err());  // @
        assert!(validate_project_name("my$project").is_err());  // $
        assert!(validate_project_name("my#project").is_err());  // #
        assert!(validate_project_name("my!project").is_err());  // !
        assert!(validate_project_name("my*project").is_err());  // *
        assert!(validate_project_name("my&project").is_err());  // &
        assert!(validate_project_name("project<script>").is_err());  // < >
    }

    #[test]
    fn test_starting_characters_rejected() {
        assert!(validate_project_name(".hidden").is_err());     // starts with dot
        assert!(validate_project_name("-dash").is_err());       // starts with dash
        assert!(validate_project_name("..relative").is_err());  // starts with ..
    }

    #[test]
    fn test_ending_characters_rejected() {
        assert!(validate_project_name("project.").is_err());  // ends with dot
    }

    #[test]
    fn test_empty_and_whitespace_rejected() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("   ").is_err());
        assert!(validate_project_name("\t").is_err());
        assert!(validate_project_name("\n").is_err());
    }

    #[test]
    fn test_length_limits() {
        // Valid length
        let name_254 = "a".repeat(254);
        assert!(validate_project_name(&name_254).is_ok());

        let name_255 = "a".repeat(255);
        assert!(validate_project_name(&name_255).is_ok());

        // Too long
        let name_256 = "a".repeat(256);
        assert!(validate_project_name(&name_256).is_err());
    }

    #[test]
    fn test_null_bytes_rejected() {
        assert!(validate_project_name("project\0name").is_err());
    }
}
