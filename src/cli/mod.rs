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
