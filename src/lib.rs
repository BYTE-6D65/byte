pub mod cli;
pub mod config;
pub mod logger;
pub mod projects;
pub mod state;
pub mod tui;

// Re-export commonly used types
pub use config::{Config, GlobalConfig, ProjectConfig};
pub use projects::DiscoveredProject;
