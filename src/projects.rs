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

/// Scan workspace and registered paths for projects
pub fn discover_projects(global_config: &GlobalConfig) -> Result<Vec<DiscoveredProject>> {
    let mut projects = Vec::new();

    // Scan workspace if auto_scan is enabled
    if global_config.workspace.auto_scan {
        let workspace_path = shellexpand::tilde(&global_config.workspace.path).to_string();
        eprintln!("[INFO] {}", format!(
            "[DISCOVERY] Scanning primary workspace: {}",
            workspace_path
        ));
        if let Ok(workspace_projects) = scan_directory(&workspace_path) {
            eprintln!("[INFO] {}", format!(
                "[DISCOVERY] Found {} projects in primary workspace",
                workspace_projects.len()
            ));
            projects.extend(workspace_projects);
        } else {
            eprintln!("[INFO] [DISCOVERY] Failed to scan primary workspace");
        }
    }

    // Scan manually registered directories for projects
    for registered_path in &global_config.workspace.registered {
        let expanded_path = shellexpand::tilde(registered_path).to_string();
        eprintln!("[INFO] {}", format!(
            "[DISCOVERY] Scanning registered path: {}",
            expanded_path
        ));
        match scan_directory(&expanded_path) {
            Ok(registered_projects) => {
                eprintln!("[INFO] {}", format!(
                    "[DISCOVERY] Found {} projects in {}",
                    registered_projects.len(),
                    expanded_path
                ));
                for proj in &registered_projects {
                    eprintln!("[INFO] {}", format!(
                        "[DISCOVERY]   - {} at {}",
                        proj.config.project.name,
                        proj.path.display()
                    ));
                }
                projects.extend(registered_projects);
            }
            Err(e) => {
                eprintln!("[INFO] {}", format!(
                    "[DISCOVERY] Failed to scan {}: {}",
                    expanded_path, e
                ));
            }
        }
    }

    eprintln!("[INFO] {}", format!(
        "[DISCOVERY] Total projects discovered: {}",
        projects.len()
    ));
    Ok(projects)
}

/// Scan a directory for byte.toml files
fn scan_directory(path: &str) -> Result<Vec<DiscoveredProject>> {
    let mut projects = Vec::new();
    let path = Path::new(path);

    eprintln!("[INFO] {}", format!("[SCAN] Scanning directory: {}", path.display()));

    if !path.exists() {
        eprintln!("[INFO] {}", format!("[SCAN] Path does not exist: {}", path.display()));
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
                    eprintln!("[INFO] {}", format!(
                        "[SCAN] Found byte.toml at: {}",
                        entry.path().display()
                    ));
                    if let Some(project_dir) = entry.path().parent() {
                        match load_project(project_dir.to_str().unwrap_or("")) {
                            Ok(project) => {
                                eprintln!("[INFO] {}", format!(
                                    "[SCAN] Successfully loaded project: {}",
                                    project.config.project.name
                                ));
                                projects.push(project);
                            }
                            Err(e) => {
                                eprintln!("[INFO] {}", format!(
                                    "[SCAN] Failed to load project from {}: {}",
                                    project_dir.display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[INFO] {}", format!("[SCAN] Error reading entry: {}", e));
            }
        }
    }

    eprintln!("[INFO] {}", format!(
        "[SCAN] Scanned {} entries, found {} projects",
        entry_count,
        projects.len()
    ));

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
    // Expand workspace path
    let workspace = shellexpand::tilde(workspace_path).to_string();
    let workspace_path = Path::new(&workspace);

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
