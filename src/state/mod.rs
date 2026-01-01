pub mod build;
pub mod git;

pub use build::{BuildState, BuildStatus};
pub use git::GitStatus;

/// Complete project state including git and build information
#[derive(Debug, Clone)]
pub struct ProjectState {
    pub git: GitStatus,
    pub build: Option<BuildState>,
}

/// Get the complete state for a project
pub fn get_project_state(project_path: &str) -> ProjectState {
    let git = git::get_git_status(project_path);
    let build = build::load_build_state(project_path);

    ProjectState { git, build }
}
