use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Build status states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Success,
    Failed,
    Running,
}

/// Build state tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildState {
    pub timestamp: i64, // Unix timestamp
    pub status: BuildStatus,
    pub task: String, // Build task name (e.g., "release", "debug")
}

/// Load build state from .byte/state/build.json
pub fn load_build_state(project_path: &str) -> Option<BuildState> {
    let state_file = PathBuf::from(project_path).join(".byte/state/build.json");

    if !state_file.exists() {
        return None;
    }

    // Read and parse JSON file
    let content = fs::read_to_string(&state_file).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save build state to .byte/state/build.json
pub fn save_build_state(project_path: &str, state: BuildState) -> Result<()> {
    // Ensure .byte/state directory exists
    let state_dir = PathBuf::from(project_path).join(".byte/state");
    fs::create_dir_all(&state_dir)?;

    // Write JSON file
    let state_file = state_dir.join("build.json");
    let json = serde_json::to_string_pretty(&state)?;
    fs::write(&state_file, json)?;

    Ok(())
}
