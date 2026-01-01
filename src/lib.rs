pub mod cli;
pub mod config;
pub mod exec;
pub mod forms;
pub mod fs;
pub mod projects;
pub mod state;
pub mod tui;

// Re-export commonly used types
pub use config::{Config, GlobalConfig, ProjectConfig};
pub use projects::DiscoveredProject;
