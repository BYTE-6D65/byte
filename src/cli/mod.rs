use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new project
    Init {
        /// Ecosystem (e.g., go, bun, rust)
        ecosystem: String,

        /// Project type (e.g., cli, desktop, web)
        project_type: String,

        /// Project name
        name: String,
    },

    /// Discover and list all projects
    Discover,

    /// Launch TUI
    Tui,
}

pub fn run() -> Result<()> {
    let config = crate::config::Config::load()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init {
            ecosystem,
            project_type,
            name,
        }) => {
            let workspace_path = &config.global.workspace.path;

            println!(
                "Creating {} {} project '{}'...",
                ecosystem, project_type, name
            );

            match crate::projects::init_project(workspace_path, &ecosystem, &project_type, &name) {
                Ok(path) => {
                    println!("✓ Project created at: {}", path.display());
                    println!("\nNext steps:");
                    println!("  cd {}", path.display());
                    println!("  byte tui  # to view in the project browser");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ Failed to create project: {}", e);
                    Err(e)
                }
            }
        }
        Some(Commands::Discover) => {
            println!("Discovering projects...\n");

            match crate::projects::discover_projects(&config.global) {
                Ok(projects) => {
                    if projects.is_empty() {
                        println!("No projects found.");
                        println!("\nSearched in:");
                        println!("  - Primary workspace: {}", config.global.workspace.path);
                        for path in &config.global.workspace.registered {
                            println!("  - Registered: {}", path);
                        }
                    } else {
                        println!("Found {} project{}:\n",
                            projects.len(),
                            if projects.len() == 1 { "" } else { "s" }
                        );

                        for project in &projects {
                            let ecosystem = &project.config.project.ecosystem;
                            let project_type = &project.config.project.project_type;
                            println!("  • {} ({}/{}) at {}",
                                project.config.project.name,
                                ecosystem,
                                project_type,
                                project.path.display()
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ Discovery failed: {}", e);
                    Err(e)
                }
            }
        }
        Some(Commands::Tui) => {
            println!("Launching TUI...");
            crate::tui::run()?;
            Ok(())
        }
        None => {
            // Default to TUI
            println!("Launching TUI...");
            crate::tui::run()?;
            Ok(())
        }
    }
}
