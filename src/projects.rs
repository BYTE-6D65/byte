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
        if let Ok(workspace_projects) = scan_directory(&workspace_path) {
            projects.extend(workspace_projects);
        }
    }

    // Add manually registered projects
    for registered_path in &global_config.workspace.registered {
        let expanded_path = shellexpand::tilde(registered_path).to_string();
        if let Ok(project) = load_project(&expanded_path) {
            projects.push(project);
        }
    }

    Ok(projects)
}

/// Scan a directory for byte.toml files
fn scan_directory(path: &str) -> Result<Vec<DiscoveredProject>> {
    let mut projects = Vec::new();
    let path = Path::new(path);

    if !path.exists() {
        return Ok(projects);
    }

    // Walk directory looking for byte.toml files
    for entry in WalkDir::new(path)
        .max_depth(3) // Don't go too deep
        .follow_links(false)
    {
        let entry = entry?;
        if entry.file_name() == "byte.toml" {
            if let Some(project_dir) = entry.path().parent() {
                if let Ok(project) = load_project(project_dir.to_str().unwrap_or("")) {
                    projects.push(project);
                }
            }
        }
    }

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

/// Initialize a new project
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
        anyhow::bail!("Project directory already exists: {}", project_path.display());
    }

    fs::create_dir_all(&project_path)?;

    // Create .byte directory structure
    let byte_dir = project_path.join(".byte");
    fs::create_dir_all(byte_dir.join("logs"))?;
    fs::create_dir_all(byte_dir.join("state"))?;

    // Create target directory for build artifacts (standard across all ecosystems)
    fs::create_dir_all(project_path.join("target"))?;

    // Create byte.toml
    let config = ProjectConfig {
        project: crate::config::types::ProjectMeta {
            name: name.to_string(),
            project_type: project_type.to_string(),
            ecosystem: ecosystem.to_string(),
            description: None,
        },
    };

    let config_toml = toml::to_string_pretty(&config)?;
    fs::write(project_path.join("byte.toml"), config_toml)?;

    // Run ecosystem-specific setup
    match ecosystem {
        "go" => init_go_project(&project_path, name)?,
        "bun" => init_bun_project(&project_path, name)?,
        "rust" => init_rust_project(&project_path, name)?,
        _ => {
            eprintln!("Warning: No driver found for ecosystem '{}'", ecosystem);
        }
    }

    // Add .byte/ to .gitignore
    add_to_gitignore(&project_path)?;

    Ok(project_path)
}

/// Add .byte/ to .gitignore (or create .gitignore if it doesn't exist)
fn add_to_gitignore(project_path: &Path) -> Result<()> {
    let gitignore_path = project_path.join(".gitignore");

    let byte_entry = "\n# Byte runtime data\n.byte/\n";

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.contains(".byte/") {
            fs::write(&gitignore_path, format!("{}{}", content, byte_entry))?;
        }
    } else {
        fs::write(&gitignore_path, byte_entry)?;
    }

    Ok(())
}

/// Initialize Go project
fn init_go_project(project_path: &Path, name: &str) -> Result<()> {
    use std::process::Command;

    // Create basic structure
    fs::create_dir_all(project_path.join("cmd").join(name))?;
    fs::create_dir_all(project_path.join("internal"))?;
    fs::create_dir_all(project_path.join("pkg"))?;

    // Initialize go module
    Command::new("go")
        .args(&["mod", "init", name])
        .current_dir(project_path)
        .status()?;

    // Create main.go
    let main_go = format!(
        r#"package main

import "fmt"

func main() {{
    fmt.Println("Hello from {}!")
}}
"#,
        name
    );
    fs::write(project_path.join("cmd").join(name).join("main.go"), main_go)?;

    Ok(())
}

/// Initialize Bun project
fn init_bun_project(project_path: &Path, name: &str) -> Result<()> {
    use std::process::Command;

    // Create basic structure
    fs::create_dir_all(project_path.join("src"))?;

    // Initialize package.json
    Command::new("bun")
        .args(&["init", "-y"])
        .current_dir(project_path)
        .status()?;

    // Create index.ts
    let index_ts = format!(
        r#"console.log("Hello from {}!");
"#,
        name
    );
    fs::write(project_path.join("src").join("index.ts"), index_ts)?;

    Ok(())
}

/// Initialize Rust project
fn init_rust_project(project_path: &Path, name: &str) -> Result<()> {
    use std::process::Command;

    // Use cargo init
    Command::new("cargo")
        .args(&["init", "--name", name])
        .current_dir(project_path)
        .status()?;

    Ok(())
}
