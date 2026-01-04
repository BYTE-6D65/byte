use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::state::{self, BuildState, BuildStatus, GitStatus, ProjectState, get_project_state};

// Theme colors optimized for OLED black backgrounds
mod theme {
    use ratatui::style::Color;

    // Primary colors
    pub const ACCENT: Color = Color::Cyan; // Brand color, primary actions
    pub const SUCCESS: Color = Color::Green; // Success states, active items
    pub const ERROR: Color = Color::Red; // Error states, warnings

    // Text hierarchy (simplified for OLED black - high contrast)
    pub const TEXT_PRIMARY: Color = Color::White; // Primary content, main text
    pub const TEXT_SECONDARY: Color = Color::Rgb(180, 180, 180); // Secondary content, metadata, paths

    // UI elements
    pub const SEPARATOR: Color = Color::Rgb(60, 60, 60); // Lines, dividers
    // pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 40); // Selected item background (reserved for future use)
    pub const BADGE_BG: Color = Color::Cyan; // Badge backgrounds
    pub const BADGE_TEXT: Color = Color::Black; // Badge text
}

#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub drivers: Vec<String>,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub command: String,
}

#[derive(Clone, Debug)]
pub struct WorkspaceDir {
    pub path: String,
    pub is_primary: bool,
    pub project_count: usize,
}

// ============================================================================
// Overlay API - Unified overlay system for full-screen UI elements
// ============================================================================

/// Position and sizing for overlay content
#[derive(Clone, Debug)]
pub enum OverlayPosition {
    /// Full screen overlay (log viewer, help screen)
    FullScreen,
    /// Centered modal with specified dimensions (forms)
    Centered { width: u16, height: u16 },
    /// Custom positioned overlay
    #[allow(dead_code)]
    Custom { x: u16, y: u16, width: u16, height: u16 },
}

/// Content types that can be displayed in an overlay
#[derive(Clone, Debug)]
pub enum OverlayContent {
    /// Log file viewer with scroll support
    LogViewer {
        log_path: PathBuf,
        scroll_offset: usize,
        project_path: String,
    },
    /// Form input (project creation, git tag, etc.)
    Form {
        form: crate::forms::Form,
        position: OverlayPosition,
    },
    // Future overlay types:
    // SubProjectBrowser { sub_projects: Vec<Project>, selected_index: usize },
    // HelpScreen,
    // CommandHistory { entries: Vec<CommandHistoryEntry>, selected: usize },
}

impl OverlayContent {
    /// Get the position/sizing for this overlay
    #[allow(dead_code)]
    pub fn position(&self) -> OverlayPosition {
        match self {
            OverlayContent::LogViewer { .. } => OverlayPosition::FullScreen,
            OverlayContent::Form { position, .. } => position.clone(),
        }
    }
}

// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommandFilter {
    All,
    Build,
    Lint,
    Git,
    Test,
    Other,
}

impl CommandFilter {
    pub fn as_str(&self) -> &str {
        match self {
            CommandFilter::All => "All",
            CommandFilter::Build => "Build",
            CommandFilter::Lint => "Lint",
            CommandFilter::Git => "Git",
            CommandFilter::Test => "Test",
            CommandFilter::Other => "Other",
        }
    }

    pub fn all_filters() -> Vec<CommandFilter> {
        vec![
            CommandFilter::All,
            CommandFilter::Build,
            CommandFilter::Lint,
            CommandFilter::Git,
            CommandFilter::Test,
            CommandFilter::Other,
        ]
    }

    /// Get all log categories (excluding "All" which is UI-only)
    pub fn log_categories() -> Vec<&'static str> {
        vec!["build", "lint", "git", "test", "other"]
    }

    pub fn next(&self) -> CommandFilter {
        match self {
            CommandFilter::All => CommandFilter::Build,
            CommandFilter::Build => CommandFilter::Lint,
            CommandFilter::Lint => CommandFilter::Git,
            CommandFilter::Git => CommandFilter::Test,
            CommandFilter::Test => CommandFilter::Other,
            CommandFilter::Other => CommandFilter::All,
        }
    }

    pub fn prev(&self) -> CommandFilter {
        match self {
            CommandFilter::All => CommandFilter::Other,
            CommandFilter::Build => CommandFilter::All,
            CommandFilter::Lint => CommandFilter::Build,
            CommandFilter::Git => CommandFilter::Lint,
            CommandFilter::Test => CommandFilter::Git,
            CommandFilter::Other => CommandFilter::Test,
        }
    }
}

pub enum InputMode {
    Normal,
    AddingDirectory,
    EditingCommand,
}

pub struct App {
    pub should_quit: bool,
    pub current_view: View,
    pub previous_view: Option<View>,       // Where to return when overlay closes
    pub active_overlay: Option<OverlayContent>, // Currently active overlay content
    pub projects: Vec<Project>,
    pub commands: Vec<Command>,
    pub command_filter: CommandFilter, // Active filter for commands view
    pub selected_project: usize,
    pub selected_command: usize,
    pub project_list_state: ListState,
    pub command_list_state: ListState,
    pub status_message: String,
    // Workspace manager
    pub workspace_directories: Vec<WorkspaceDir>,
    pub selected_workspace: usize,
    pub workspace_list_state: ListState,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub launch_fuzzy_picker: bool,
    pub editing_workspace_index: Option<usize>, // Track which workspace is being edited
    pub selected_target_workspace: usize,       // Which workspace to use as command target
    // Inline fuzzy matching
    pub fuzzy_matches: Vec<String>,
    pub fuzzy_selected: usize,
    pub fuzzy_browsing: bool, // true when navigating matches, false when typing
    // Project state caching
    pub project_states: HashMap<String, ProjectState>,
    pub last_state_refresh: Instant,
    pub last_hotload: Instant,
    // Command execution animation
    pub executing_command: Option<String>,
    pub build_animation_frame: usize,
    pub build_animation_start: Option<Instant>,
    pub command_tx: Option<std::sync::mpsc::Sender<CommandResult>>,
    pub pending_result: Option<CommandResult>,
    pub command_result_display: Option<(bool, Instant)>, // (success, timestamp) for showing result
    // Interactive editor request
    pub pending_editor: Option<(String, String)>, // (editor, file_path)
    // Log navigation in Details view
    pub selected_log: usize,
    // Flag to trigger terminal clear on next draw
    pub needs_clear: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    ProjectBrowser,
    CommandPalette,
    Detail,
    WorkspaceManager,
    Overlay, // Unified overlay view (log viewer, forms, help, etc.)
}

#[derive(Clone)]
pub struct CommandResult {
    pub success: bool,
    pub command: String,
    pub working_dir: String,
    pub is_build_cmd: bool,
    pub task_name: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            should_quit: false,
            current_view: View::ProjectBrowser,
            previous_view: None,
            active_overlay: None,
            projects: vec![],
            commands: vec![
                Command {
                    name: "init go cli <name>".to_string(),
                    description: "Initialize Go CLI project".to_string(),
                    command: "byte init go cli my-project".to_string(),
                },
                Command {
                    name: "init bun web <name>".to_string(),
                    description: "Initialize Bun web application".to_string(),
                    command: "byte init bun web my-app".to_string(),
                },
                Command {
                    name: "init rust cli <name>".to_string(),
                    description: "Initialize Rust CLI project".to_string(),
                    command: "byte init rust cli my-tool".to_string(),
                },
            ],
            command_filter: CommandFilter::All,
            selected_project: 0,
            selected_command: 0,
            project_list_state: ListState::default(),
            command_list_state: ListState::default(),
            status_message: "Welcome to Byte!".to_string(),
            // Workspace manager
            workspace_directories: vec![],
            selected_workspace: 0,
            workspace_list_state: ListState::default(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            launch_fuzzy_picker: false,
            editing_workspace_index: None,
            selected_target_workspace: 0, // Default to primary workspace
            fuzzy_matches: vec![],
            fuzzy_selected: 0,
            fuzzy_browsing: false,
            project_states: HashMap::new(),
            last_state_refresh: Instant::now(),
            last_hotload: Instant::now(),
            executing_command: None,
            build_animation_frame: 0,
            build_animation_start: None,
            command_tx: None,
            pending_result: None,
            command_result_display: None,
            pending_editor: None,
            selected_log: 0,
            needs_clear: false,
        };

        if !app.projects.is_empty() {
            app.project_list_state.select(Some(0));
        }
        app.command_list_state.select(Some(0));
        app
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self::default();

        // Load config and discover projects
        if let Ok(config) = crate::config::Config::load() {
            // Load workspace directories
            let workspace_path = &config.global.workspace.path;

            app.workspace_directories.push(WorkspaceDir {
                path: workspace_path.clone(),
                is_primary: true,
                project_count: 0, // Will be counted after discovery
            });

            for registered in &config.global.workspace.registered {
                app.workspace_directories.push(WorkspaceDir {
                    path: registered.clone(),
                    is_primary: false,
                    project_count: 0,
                });
            }

            if !app.workspace_directories.is_empty() {
                app.workspace_list_state.select(Some(0));
            }

            // Discover projects
            if let Ok(discovered) = crate::projects::discover_projects(&config.global) {
                app.projects = discovered
                    .into_iter()
                    .map(|p| Project {
                        name: p.config.project.name.clone(),
                        description: p.config.project.description.clone().unwrap_or_else(|| {
                            format!("{} project", p.config.project.project_type)
                        }),
                        drivers: vec![p.config.project.ecosystem],
                        path: p.path.to_string_lossy().to_string(),
                    })
                    .collect();

                // Update project counts for workspaces
                for workspace in &mut app.workspace_directories {
                    let safe_path = match crate::path::SafePath::from_user_input(&workspace.path) {
                        Ok(path) => path,
                        Err(_) => continue, // Skip invalid paths
                    };
                    let expanded_path = safe_path.to_string();
                    let normalized_workspace = expanded_path.trim_end_matches('/').to_lowercase();

                    crate::log::info("COUNT", &format!("Counting projects for workspace: {}", expanded_path));

                    workspace.project_count = app
                        .projects
                        .iter()
                        .filter(|p| {
                            let normalized_proj = p.path.trim_end_matches('/').to_lowercase();
                            normalized_proj.starts_with(&normalized_workspace)
                        })
                        .count();

                    crate::log::info("COUNT", &format!("Final count for {}: {}", expanded_path, workspace.project_count));
                }

                if !app.projects.is_empty() {
                    app.project_list_state.select(Some(0));
                    app.status_message = format!("Discovered {} projects", app.projects.len());
                } else {
                    app.status_message =
                        "No projects found. Use 'byte init' to create one.".to_string();
                }
            }

            // Load project states (git status, build state)
            app.refresh_project_states();
        }

        app
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    // ========================================================================
    // Overlay API Methods
    // ========================================================================

    /// Open an overlay, suspending the current view
    pub fn open_overlay(&mut self, content: OverlayContent) {
        // Store current view to return to later
        self.previous_view = Some(self.current_view);
        self.active_overlay = Some(content);
        self.current_view = View::Overlay;
        self.needs_clear = true; // Ensure clean render
    }

    /// Close the active overlay and return to previous view
    pub fn close_overlay(&mut self) {
        self.active_overlay = None;
        if let Some(prev) = self.previous_view.take() {
            self.current_view = prev;
        } else {
            // Fallback to project browser if no previous view
            self.current_view = View::ProjectBrowser;
        }
        self.needs_clear = true; // Ensure clean render after overlay closes
    }

    /// Check if an overlay is currently active
    #[allow(dead_code)]
    pub fn has_overlay(&self) -> bool {
        self.active_overlay.is_some()
    }

    /// Get mutable reference to active overlay content
    #[allow(dead_code)]
    pub fn overlay_mut(&mut self) -> Option<&mut OverlayContent> {
        self.active_overlay.as_mut()
    }

    // ========================================================================

    pub fn add_workspace(&mut self, path: &str) -> Result<(), String> {
        // Load config
        let mut config = crate::config::Config::load().map_err(|e| e.to_string())?;

        // Add the path
        config.add_workspace_path(path).map_err(|e| e.to_string())?;

        // Reload workspace directories and projects
        self.hotload();

        Ok(())
    }

    pub fn edit_workspace(&mut self, old_path: &str, new_path: &str) -> Result<(), String> {
        // Load config
        let mut config = crate::config::Config::load().map_err(|e| e.to_string())?;

        // Remove old path and add new path
        config
            .remove_workspace_path(old_path)
            .map_err(|e| e.to_string())?;
        config
            .add_workspace_path(new_path)
            .map_err(|e| e.to_string())?;

        // Reload workspace directories and projects
        self.hotload();

        Ok(())
    }

    pub fn remove_workspace(&mut self, path: &str) -> Result<(), String> {
        // Load config
        let mut config = crate::config::Config::load().map_err(|e| e.to_string())?;

        // Remove the path
        config
            .remove_workspace_path(path)
            .map_err(|e| e.to_string())?;

        // Reload workspace directories and projects
        self.hotload();

        Ok(())
    }

    /// Refresh project states (called on hotload or manual refresh)
    pub fn refresh_project_states(&mut self) {
        self.project_states.clear();

        for project in &self.projects {
            let state = get_project_state(&project.path);
            self.project_states.insert(project.path.clone(), state);
        }

        self.last_state_refresh = Instant::now();
    }

    /// Get cached state for current selected project
    pub fn get_current_project_state(&self) -> Option<&ProjectState> {
        let project = self.get_selected_project()?;
        self.project_states.get(&project.path)
    }

    /// Check if a command is a build command
    fn is_build_command(&self, command_str: &str) -> bool {
        // Check if command matches a build task from current project
        for cmd in &self.commands {
            if cmd.name.starts_with("build: ") && cmd.command == command_str {
                return true;
            }
        }
        false
    }

    /// Extract task name from a build command
    fn extract_task_name(&self, command_str: &str) -> Option<String> {
        for cmd in &self.commands {
            if cmd.command == command_str && cmd.name.starts_with("build: ") {
                return Some(cmd.name.strip_prefix("build: ")?.to_string());
            }
        }
        None
    }

    fn handle_command_result(&mut self, result: CommandResult) {
        // Update build state after execution (for build commands)
        if result.is_build_cmd {
            if let Some(task) = result.task_name {
                let build_status = if result.success {
                    BuildStatus::Success
                } else {
                    BuildStatus::Failed
                };

                let state = BuildState {
                    timestamp: chrono::Utc::now().timestamp(),
                    status: build_status,
                    task,
                };
                let _ = state::build::save_build_state(&result.working_dir, state);
            }
        }

        // Show result in progress bar for 3 seconds
        self.command_result_display = Some((result.success, Instant::now()));

        if result.success {
            self.status_message = format!("✓ {}", result.command);
            crate::log::info("EXEC", &format!("Success: {}", result.command));
            self.hotload();
        } else {
            self.status_message = format!("✗ Command failed");
            crate::log::error("EXEC", &format!("Failed: {}", result.command));
            if !result.stderr.is_empty() {
                crate::log::error("EXEC", &format!("  stderr: {}", result.stderr.trim()));
            }
            if !result.stdout.is_empty() {
                crate::log::error("EXEC", &format!("  stdout: {}", result.stdout.trim()));
            }
        }
    }

    fn execute_command(&mut self, command_str: &str) {
        // Determine working directory based on context
        let working_dir = if let Some(project) = self.get_selected_project() {
            // Project selected: run commands in project directory
            project.path.clone()
        } else {
            // No project selected: use target workspace (for init commands)
            let target_workspace = self.get_target_workspace();
            match crate::path::SafePath::from_user_input(&target_workspace) {
                Ok(safe_path) => safe_path.to_string(),
                Err(_) => target_workspace, // Fallback to original if expansion fails
            }
        };

        crate::log::info("EXEC", &format!("Executing: {} in {}", command_str, working_dir));

        // Check if this is a build command
        let is_build_cmd = self.is_build_command(command_str);
        let task_name = if is_build_cmd {
            self.extract_task_name(command_str)
        } else {
            None
        };

        // Save "Running" state before execution (for build commands)
        if let (true, Some(task)) = (is_build_cmd, &task_name) {
            let state = BuildState {
                timestamp: chrono::Utc::now().timestamp(),
                status: BuildStatus::Running,
                task: task.clone(),
            };
            let _ = state::build::save_build_state(&working_dir, state);
        }

        // Start animation for ALL commands
        self.executing_command = Some(command_str.to_string());
        self.build_animation_frame = 0;
        self.build_animation_start = Some(Instant::now());

        // Spawn background thread to execute command
        if let Some(tx) = self.command_tx.clone() {
            let command = command_str.to_string();
            let working_dir_clone = working_dir.clone();

            std::thread::spawn(move || {
                use crate::exec::CommandBuilder;

                // Parse command to check if it's a byte init command
                let parts: Vec<&str> = command.split_whitespace().collect();
                let is_byte_init = parts.len() >= 4 && parts[0] == "byte" && parts[1] == "init";

                let (success, _stdout, _stderr) = if is_byte_init {
                    // Handle byte init commands specially
                    let ecosystem = parts[2];
                    let project_type = parts[3];
                    let name = if parts.len() > 4 {
                        parts[4]
                    } else {
                        "my-project"
                    };

                    // Validate project name before attempting to create
                    if let Err(e) = crate::projects::validate_project_name(name) {
                        let msg = format!("Invalid project name: {}", e);
                        (false, String::new(), msg)
                    } else {
                        match crate::projects::init_project(
                        &working_dir_clone,
                        ecosystem,
                        project_type,
                        name,
                    ) {
                        Ok(project_path) => {
                            let msg = format!("Created project at {}", project_path.display());
                            (true, msg, String::new())
                        }
                        Err(e) => {
                            let msg = format!("Failed to create project: {}", e);
                            (false, String::new(), msg)
                        }
                    }
                    }
                } else {
                    // Execute regular shell command using exec API (with validation)
                    let result = CommandBuilder::shell(&command)
                        .working_dir(&working_dir_clone)
                        .execute();

                    match result {
                        Ok(cmd_result) => {
                            let success = cmd_result.success;
                            let stdout = cmd_result.stdout.clone();
                            let stderr = cmd_result.stderr.clone();
                            let exit_code = cmd_result.exit_code;

                            // Log command output using FS API
                            if let Ok(fs_api) = crate::fs::ProjectFileSystem::new(&working_dir_clone) {
                                // Categorize command to determine log directory
                                let category = Self::categorize_command(&command).as_str().to_lowercase();
                                let _ = fs_api.write_command_log(
                                    &category,
                                    &command,
                                    &stdout,
                                    &stderr,
                                    exit_code,
                                );
                            }

                            (success, stdout, stderr)
                        }
                        Err(e) => {
                            let msg = format!("Command execution failed: {}", e);
                            (false, String::new(), msg)
                        }
                    }
                };

                // Send result back
                let _ = tx.send(CommandResult {
                    success,
                    command,
                    working_dir: working_dir_clone,
                    is_build_cmd,
                    task_name,
                    stdout: _stdout,
                    stderr: _stderr,
                });
            });
        }
    }

    /// Update fuzzy matches based on current input
    fn update_fuzzy_matches(&mut self) {
        if self.input_buffer.is_empty() {
            // Show common directories when empty
            self.fuzzy_matches.clear();
            if let Some(home) = dirs::home_dir() {
                self.fuzzy_matches.push("~".to_string());
                for subdir in &[
                    "Desktop",
                    "Documents",
                    "Downloads",
                    "Music",
                    "Pictures",
                    "Videos",
                    "projects",
                ] {
                    let path = home.join(subdir);
                    if path.exists() {
                        self.fuzzy_matches.push(format!("~/{}", subdir));
                    }
                }
            }
        } else {
            // Get candidates and fuzzy match
            let mut candidates = Vec::new();

            // Expand and get directory to search
            let expanded = match crate::path::SafePath::from_user_input(&self.input_buffer) {
                Ok(safe_path) => safe_path.to_string(),
                Err(_) => self.input_buffer.clone(), // Use original if expansion fails
            };
            let path = std::path::Path::new(&expanded);

            // Determine what we're completing
            let (search_dir, prefix) = if expanded.ends_with('/') {
                (path, "")
            } else {
                let parent = path.parent().unwrap_or(std::path::Path::new("."));
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                (parent, filename)
            };

            // Read directory and collect matches
            if let Ok(entries) = std::fs::read_dir(search_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry
                        .file_type()
                        .ok()
                        .map(|ft| ft.is_dir())
                        .unwrap_or(false)
                    {
                        if let Ok(name) = entry.file_name().into_string() {
                            // Fuzzy match: check if all chars of prefix appear in order
                            if prefix.is_empty()
                                || fuzzy_match(&name.to_lowercase(), &prefix.to_lowercase())
                            {
                                let full_path = if search_dir.to_str() == Some(".") {
                                    name.clone()
                                } else {
                                    let mut full = search_dir.to_path_buf();
                                    full.push(&name);

                                    // Convert back to tilde if in home
                                    if let Some(home) = dirs::home_dir() {
                                        let home_str = home.to_string_lossy();
                                        let full_str = full.to_string_lossy();
                                        if full_str.starts_with(home_str.as_ref()) {
                                            full_str.replacen(home_str.as_ref(), "~", 1)
                                        } else {
                                            full_str.to_string()
                                        }
                                    } else {
                                        full.to_string_lossy().to_string()
                                    }
                                };
                                candidates.push(full_path);
                            }
                        }
                    }
                }
            }

            candidates.sort();
            self.fuzzy_matches = candidates;
        }

        // Reset selection if out of bounds
        if self.fuzzy_selected >= self.fuzzy_matches.len() {
            self.fuzzy_selected = 0;
        }
    }

    /// Tab completion for directory paths
    fn complete_path(&mut self, partial: &str) -> Option<String> {
        use std::fs;
        use std::path::Path;

        if partial.is_empty() {
            return None;
        }

        // Expand tilde
        let expanded = match crate::path::SafePath::from_user_input(partial) {
            Ok(safe_path) => safe_path.to_string(),
            Err(_) => partial.to_string(), // Use original if expansion fails
        };
        let path = Path::new(&expanded);

        // Determine search directory and prefix
        let (search_dir, prefix) = if expanded.ends_with('/') {
            (path, "")
        } else {
            let parent = path.parent().unwrap_or(Path::new("."));
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            (parent, filename)
        };

        // Read directory and find matching subdirectories
        let entries = match fs::read_dir(search_dir) {
            Ok(e) => e,
            Err(_) => {
                self.status_message = format!("Cannot read directory: {}", search_dir.display());
                return None;
            }
        };

        let mut matches: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name.starts_with(prefix))
            .collect();

        matches.sort();

        match matches.len() {
            0 => {
                self.status_message = "No matching directories".to_string();
                None
            }
            1 => {
                // Single match - complete it
                let completed = search_dir.join(&matches[0]);
                let completed_str = completed.to_string_lossy().to_string();

                // Convert back to tilde format if it's in home dir
                if let Some(home) = dirs::home_dir() {
                    let home_str = home.to_string_lossy().to_string();
                    if completed_str.starts_with(&home_str) {
                        let relative = completed_str.replacen(&home_str, "~", 1);
                        self.status_message = format!("Completed: {}", relative);
                        return Some(relative + "/");
                    }
                }

                self.status_message = format!("Completed: {}", completed_str);
                Some(completed_str + "/")
            }
            _ => {
                // Multiple matches - show them
                let display = if matches.len() <= 5 {
                    matches.join(", ")
                } else {
                    format!("{} and {} more", matches[..3].join(", "), matches.len() - 3)
                };
                self.status_message = format!("{} matches: {}", matches.len(), display);
                None
            }
        }
    }

    /// Single dispatch point for all form submissions
    /// Compiler enforces exhaustive handling of all FormType variants
    fn handle_form_submission(&mut self, form_type: crate::forms::FormType, values: std::collections::HashMap<String, crate::forms::FormValue>) {
        use crate::forms::FormType;

        match form_type {
            FormType::NewProject => self.submit_new_project(values),
            FormType::GitTag => self.submit_git_tag(values),
            FormType::EditCommand => {
                // Future: handle command editing
                self.status_message = "Command editing not yet implemented".to_string();
            }
            FormType::CommandHistory => {
                // Future: handle command history
                self.status_message = "Command history not yet implemented".to_string();
            }
        }
    }

    /// Handle new project creation form submission
    fn submit_new_project(&mut self, values: std::collections::HashMap<String, crate::forms::FormValue>) {
        use crate::forms::FormValue;

        // Extract form values
        let workspace = if let Some(FormValue::Text(w)) = values.get("workspace") {
            w.clone()
        } else {
            self.status_message = "Error: Missing workspace selection".to_string();
            return;
        };

        let ecosystem = if let Some(FormValue::Text(e)) = values.get("ecosystem") {
            e.clone()
        } else {
            self.status_message = "Error: Missing ecosystem selection".to_string();
            return;
        };

        let project_type = if let Some(FormValue::Text(pt)) = values.get("project_type") {
            pt.clone()
        } else {
            self.status_message = "Error: Missing project type selection".to_string();
            return;
        };

        let name = if let Some(FormValue::Text(n)) = values.get("name") {
            n.clone()
        } else {
            self.status_message = "Error: Missing project name".to_string();
            return;
        };

        // Validate project name
        if let Err(e) = crate::projects::validate_project_name(&name) {
            self.status_message = format!("Invalid project name: {}", e);
            return;
        }

        // Create the project
        match crate::projects::init_project(&workspace, &ecosystem, &project_type, &name) {
            Ok(project_path) => {
                self.status_message = format!("✓ Created project: {} at {}", name, project_path.display());

                // Reload projects to show the new one
                self.hotload();

                // Try to select the newly created project
                if let Some(pos) = self.projects.iter().position(|p| p.name == name) {
                    self.selected_project = pos;
                    self.project_list_state.select(Some(pos));

                    // Switch to detail view to show the new project
                    self.current_view = View::Detail;
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to create project: {}", e);
            }
        }
    }

    /// Handle git tag creation form submission
    fn submit_git_tag(&mut self, _values: std::collections::HashMap<String, crate::forms::FormValue>) {
        // Future: implement git tag creation
        self.status_message = "Git tag creation not yet implemented".to_string();
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.quit(),
            KeyCode::Char('r') | KeyCode::Char('R')
                if !matches!(
                    self.input_mode,
                    InputMode::AddingDirectory | InputMode::EditingCommand
                ) =>
            {
                self.hotload();
                self.status_message = "✓ Reloaded all state from disk".to_string();
            }
            KeyCode::Char('n')
                if matches!(self.current_view, View::ProjectBrowser)
                    && matches!(self.input_mode, InputMode::Normal) =>
            {
                // New project creation form (from ProjectBrowser view)
                // Load config to get workspace options
                let workspace_options: Vec<String> = if let Ok(config) = crate::config::Config::load() {
                    std::iter::once(config.global.workspace.path.clone())
                        .chain(config.global.workspace.registered.clone())
                        .collect()
                } else {
                    vec!["~/projects".to_string()] // Fallback
                };

                let form = crate::forms::Form::new("Create New Project", crate::forms::FormType::NewProject)
                    .description("Initialize a new project with byte scaffolding")
                    .select("workspace", "Target Workspace", workspace_options)
                    .select("ecosystem", "Ecosystem", vec![
                        "rust".to_string(),
                        "go".to_string(),
                        "bun".to_string(),
                    ])
                    .select("project_type", "Project Type", vec![
                        "cli".to_string(),
                        "web".to_string(),
                        "lib".to_string(),
                    ])
                    .text_input("name", "Project Name", "my-project")
                    .text_area("description", "Description (optional)", "A brief description...", 3);

                // Use new overlay API for forms
                self.open_overlay(OverlayContent::Form {
                    form,
                    position: OverlayPosition::Centered { width: 80, height: 40 },
                });
                self.status_message = "Creating new project - press Enter to submit, Esc to cancel".to_string();
            }
            KeyCode::Char('t')
                if matches!(self.current_view, View::Detail)
                    && matches!(self.input_mode, InputMode::Normal) =>
            {
                // Git tag creation form (from Details view with git status)
                let form = crate::forms::Form::new("Create Git Tag", crate::forms::FormType::GitTag)
                    .description("Create a new Git tag for this project")
                    .text_input("tag_name", "Tag Name", "v1.0.0")
                    .text_area("message", "Tag Message", "Release notes...", 4)
                    .checkbox("annotated", "Create annotated tag")
                    .checkbox("push", "Push tag to remote");

                // Use new overlay API for forms
                self.open_overlay(OverlayContent::Form {
                    form,
                    position: OverlayPosition::Centered { width: 80, height: 35 },
                });
                self.status_message = "Creating git tag - press Enter to submit, Esc to cancel".to_string();
            }
            // View switching keys - only when NOT in input mode or overlay view
            KeyCode::Char('1')
                if !matches!(self.input_mode, InputMode::AddingDirectory)
                && !matches!(self.current_view, View::Overlay) =>
            {
                self.current_view = View::ProjectBrowser;
                self.command_result_display = None; // Clear command result on view switch
                self.status_message = "Viewing projects".to_string();
            }
            KeyCode::Char('2')
                if !matches!(self.input_mode, InputMode::AddingDirectory)
                && !matches!(self.current_view, View::Overlay) =>
            {
                self.current_view = View::CommandPalette;
                self.update_commands();
                self.command_result_display = None; // Clear command result on view switch
                self.status_message = "Viewing commands".to_string();
            }
            KeyCode::Char('3')
                if !matches!(self.input_mode, InputMode::AddingDirectory)
                && !matches!(self.current_view, View::Overlay) =>
            {
                self.current_view = View::Detail;
                self.selected_log = 0; // Reset log selection
                self.command_result_display = None; // Clear command result on view switch
                self.status_message = format!(
                    "Viewing details for: {}",
                    self.projects
                        .get(self.selected_project)
                        .map(|p| p.name.as_str())
                        .unwrap_or("unknown")
                );
            }
            KeyCode::Char('4')
                if !matches!(self.input_mode, InputMode::AddingDirectory)
                && !matches!(self.current_view, View::Overlay) =>
            {
                self.current_view = View::WorkspaceManager;
                self.command_result_display = None; // Clear command result on view switch
                self.status_message = "Managing workspace directories".to_string();
            }
            // Workspace Manager specific keys
            KeyCode::Char('a') if matches!(self.current_view, View::WorkspaceManager) => {
                if matches!(self.input_mode, InputMode::Normal) {
                    self.input_mode = InputMode::AddingDirectory;
                    self.input_buffer.clear();
                    self.fuzzy_browsing = false;
                    self.editing_workspace_index = None;
                    self.update_fuzzy_matches();
                    self.status_message =
                        "Type path, use ↑↓ to browse matches, Tab/Enter to select".to_string();
                }
            }
            KeyCode::Char('e') if matches!(self.current_view, View::WorkspaceManager) => {
                if matches!(self.input_mode, InputMode::Normal) {
                    if let Some(workspace) = self.workspace_directories.get(self.selected_workspace)
                    {
                        if workspace.is_primary {
                            self.status_message =
                                "✗ Cannot edit primary workspace (use config file)".to_string();
                        } else {
                            // Enter edit mode with current path pre-filled
                            self.input_mode = InputMode::AddingDirectory;
                            self.input_buffer = workspace.path.clone();
                            self.fuzzy_browsing = false;
                            self.editing_workspace_index = Some(self.selected_workspace);
                            self.update_fuzzy_matches();
                            self.status_message =
                                "Editing path - use ↑↓ to browse matches, Tab/Enter to save"
                                    .to_string();
                        }
                    }
                }
            }
            KeyCode::Char('d') if matches!(self.current_view, View::WorkspaceManager) => {
                if matches!(self.input_mode, InputMode::Normal) {
                    if let Some(workspace) = self.workspace_directories.get(self.selected_workspace)
                    {
                        if workspace.is_primary {
                            self.status_message = "✗ Cannot remove primary workspace".to_string();
                        } else {
                            let path = workspace.path.clone();
                            match self.remove_workspace(&path) {
                                Ok(_) => {
                                    self.status_message = format!("✓ Removed {}", path);
                                }
                                Err(e) => {
                                    self.status_message = format!("✗ Error: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Char('t')
                if matches!(self.current_view, View::CommandPalette)
                    && matches!(self.input_mode, InputMode::Normal) =>
            {
                self.cycle_target_workspace();
            }
            KeyCode::Char('l')
                if matches!(self.current_view, View::Detail)
                    && matches!(self.input_mode, InputMode::Normal) =>
            {
                // Open log viewer overlay
                if let Some(project) = self.get_selected_project() {
                    let logs = crate::fs::ProjectFileSystem::new(&project.path).ok().and_then(|fs| fs.recent_logs_all(5).ok()).unwrap_or_default();
                    if let Some(log) = logs.get(self.selected_log) {
                        // Use new overlay API
                        self.open_overlay(OverlayContent::LogViewer {
                            log_path: log.path.clone(),
                            scroll_offset: 0,
                            project_path: project.path.clone(),
                        });
                        self.status_message = format!("Viewing: {}", log.filename);
                    } else {
                        self.status_message = "✗ No logs available".to_string();
                    }
                } else {
                    self.status_message = "✗ No project selected".to_string();
                }
            }
            KeyCode::Char('o')
                if matches!(self.current_view, View::Detail)
                    && matches!(self.input_mode, InputMode::Normal) =>
            {
                // Open selected log file in external editor
                if let Some(project) = self.get_selected_project() {
                    let logs = crate::fs::ProjectFileSystem::new(&project.path).ok().and_then(|fs| fs.recent_logs_all(5).ok()).unwrap_or_default();
                    if let Some(log) = logs.get(self.selected_log) {
                        let editor = crate::exec::get_default_editor();
                        let log_path = log.path.to_string_lossy().to_string();

                        // Set pending editor request (will be handled in main loop)
                        self.pending_editor = Some((editor, log_path));
                    } else {
                        self.status_message = "✗ No logs available".to_string();
                    }
                } else {
                    self.status_message = "✗ No project selected".to_string();
                }
            }
            KeyCode::Esc if matches!(self.current_view, View::Overlay) => {
                // Close any overlay and return to previous view
                let overlay_type = self.active_overlay.as_ref().map(|o| match o {
                    OverlayContent::LogViewer { .. } => "log viewer",
                    OverlayContent::Form { .. } => "form",
                });
                self.close_overlay();
                if let Some(name) = overlay_type {
                    self.status_message = format!("Closed {}", name);
                }
            }
            KeyCode::Esc
                if matches!(
                    self.input_mode,
                    InputMode::AddingDirectory | InputMode::EditingCommand
                ) =>
            {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.editing_workspace_index = None;
                self.status_message = "Cancelled".to_string();
            }
            KeyCode::Backspace
                if matches!(
                    self.input_mode,
                    InputMode::AddingDirectory | InputMode::EditingCommand
                ) =>
            {
                self.input_buffer.pop();
                if matches!(self.input_mode, InputMode::AddingDirectory) {
                    self.fuzzy_browsing = false;
                    self.update_fuzzy_matches();
                }
            }
            KeyCode::Backspace if matches!(self.current_view, View::Overlay) => {
                // Handle backspace in overlay form fields
                if let Some(OverlayContent::Form { form, .. }) = &mut self.active_overlay {
                    if let Some(field) = form.current_field_mut() {
                        field.handle_backspace();
                    }
                }
            }
            KeyCode::Tab if matches!(self.input_mode, InputMode::AddingDirectory) => {
                // If browsing matches, select current match
                if self.fuzzy_browsing && !self.fuzzy_matches.is_empty() {
                    let selected = self.fuzzy_matches[self.fuzzy_selected].clone();
                    // Add trailing slash to match tab completion behavior
                    self.input_buffer = if selected.ends_with('/') {
                        selected
                    } else {
                        selected + "/"
                    };
                    self.fuzzy_browsing = false;
                    self.status_message = format!("Selected: {}", self.input_buffer);
                    self.update_fuzzy_matches();
                } else if let Some(completed) = self.complete_path(&self.input_buffer.clone()) {
                    self.input_buffer = completed;
                    self.update_fuzzy_matches();
                }
            }
            KeyCode::Char(c)
                if matches!(
                    self.input_mode,
                    InputMode::AddingDirectory | InputMode::EditingCommand
                ) =>
            {
                self.input_buffer.push(c);
                if matches!(self.input_mode, InputMode::AddingDirectory) {
                    self.fuzzy_browsing = false;
                    self.update_fuzzy_matches();
                }
            }
            KeyCode::Char(c) if matches!(self.current_view, View::Overlay) && c != ' ' => {
                // Handle character input in overlay form fields (space is handled separately)
                if let Some(OverlayContent::Form { form, .. }) = &mut self.active_overlay {
                    if let Some(field) = form.current_field_mut() {
                        field.handle_char(c);
                    }
                }
            }
            KeyCode::Up => {
                // Handle fuzzy match navigation when in input mode
                if matches!(self.input_mode, InputMode::AddingDirectory)
                    && !self.fuzzy_matches.is_empty()
                {
                    self.fuzzy_browsing = true;
                    if self.fuzzy_selected > 0 {
                        self.fuzzy_selected -= 1;
                    }
                } else {
                    match self.current_view {
                        View::ProjectBrowser => {
                            if self.selected_project > 0 {
                                self.selected_project -= 1;
                                self.project_list_state.select(Some(self.selected_project));
                                self.status_message = "Selected previous project".to_string();
                            }
                        }
                        View::CommandPalette => {
                            if self.selected_command > 0 {
                                self.selected_command -= 1;
                                self.command_list_state.select(Some(self.selected_command));
                                self.status_message = "Selected previous command".to_string();
                            }
                        }
                        View::WorkspaceManager => {
                            if self.selected_workspace > 0 {
                                self.selected_workspace -= 1;
                                self.workspace_list_state
                                    .select(Some(self.selected_workspace));
                            }
                        }
                        View::Detail => {
                            // Navigate log list in detail view
                            if self.selected_log > 0 {
                                self.selected_log -= 1;
                            }
                        }
                        View::Overlay => {
                            // Handle overlay-specific up navigation
                            if let Some(overlay) = &mut self.active_overlay {
                                match overlay {
                                    OverlayContent::LogViewer { scroll_offset, .. } => {
                                        *scroll_offset = scroll_offset.saturating_sub(1);
                                    }
                                    OverlayContent::Form { form, .. } => {
                                        if let Some(field) = form.current_field_mut() {
                                            field.handle_up();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Down => {
                // Handle fuzzy match navigation when in input mode
                if matches!(self.input_mode, InputMode::AddingDirectory)
                    && !self.fuzzy_matches.is_empty()
                {
                    self.fuzzy_browsing = true;
                    if self.fuzzy_selected < self.fuzzy_matches.len().saturating_sub(1) {
                        self.fuzzy_selected += 1;
                    }
                } else {
                    match self.current_view {
                        View::ProjectBrowser => {
                            if self.selected_project < self.projects.len().saturating_sub(1) {
                                self.selected_project += 1;
                                self.project_list_state.select(Some(self.selected_project));
                                self.status_message = format!(
                                    "Selected: {}",
                                    self.projects[self.selected_project].name
                                );
                            }
                        }
                        View::CommandPalette => {
                            if self.selected_command < self.commands.len().saturating_sub(1) {
                                self.selected_command += 1;
                                self.command_list_state.select(Some(self.selected_command));
                                self.status_message = format!(
                                    "Selected: {}",
                                    self.commands[self.selected_command].name
                                );
                            }
                        }
                        View::WorkspaceManager => {
                            if self.selected_workspace
                                < self.workspace_directories.len().saturating_sub(1)
                            {
                                self.selected_workspace += 1;
                                self.workspace_list_state
                                    .select(Some(self.selected_workspace));
                            }
                        }
                        View::Detail => {
                            // Navigate log list in detail view
                            if let Some(project) = self.get_selected_project() {
                                let log_count = crate::fs::ProjectFileSystem::new(&project.path).ok().and_then(|fs| fs.recent_logs_all(5).ok()).unwrap_or_default().len();
                                if self.selected_log < log_count.saturating_sub(1) {
                                    self.selected_log += 1;
                                }
                            }
                        }
                        View::Overlay => {
                            // Handle overlay-specific down navigation
                            if let Some(overlay) = &mut self.active_overlay {
                                match overlay {
                                    OverlayContent::LogViewer { scroll_offset, .. } => {
                                        *scroll_offset = scroll_offset.saturating_add(1);
                                    }
                                    OverlayContent::Form { form, .. } => {
                                        if let Some(field) = form.current_field_mut() {
                                            field.handle_down();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::PageUp => {
                // Scroll overlay log viewer up (page)
                if matches!(self.current_view, View::Overlay) {
                    if let Some(OverlayContent::LogViewer { scroll_offset, .. }) = &mut self.active_overlay {
                        *scroll_offset = scroll_offset.saturating_sub(10);
                    }
                }
            }
            KeyCode::PageDown => {
                // Scroll overlay log viewer down (page)
                if matches!(self.current_view, View::Overlay) {
                    if let Some(OverlayContent::LogViewer { scroll_offset, .. }) = &mut self.active_overlay {
                        *scroll_offset = scroll_offset.saturating_add(10);
                    }
                }
            }
            KeyCode::Left => {
                // Navigate command filter tabs (only in CommandPalette view)
                if matches!(self.current_view, View::CommandPalette)
                    && matches!(self.input_mode, InputMode::Normal)
                {
                    self.command_filter = self.command_filter.prev();
                    self.update_commands();
                    self.status_message = format!("Filter: {}", self.command_filter.as_str());
                }
            }
            KeyCode::Right => {
                // Navigate command filter tabs (only in CommandPalette view)
                if matches!(self.current_view, View::CommandPalette)
                    && matches!(self.input_mode, InputMode::Normal)
                {
                    self.command_filter = self.command_filter.next();
                    self.update_commands();
                    self.status_message = format!("Filter: {}", self.command_filter.as_str());
                }
            }
            KeyCode::Enter => match self.current_view {
                View::ProjectBrowser => {
                    if let Some(project) = self.projects.get(self.selected_project) {
                        self.status_message = format!("Opening {}...", project.name);
                        self.current_view = View::Detail;
                    }
                }
                View::CommandPalette => {
                    if matches!(self.input_mode, InputMode::EditingCommand) {
                        // Execute the edited command
                        let command_str = self.input_buffer.trim().to_string();
                        if !command_str.is_empty() {
                            self.execute_command(&command_str);
                            self.input_mode = InputMode::Normal;
                            self.input_buffer.clear();
                        }
                    } else if let Some(cmd) = self.commands.get(self.selected_command) {
                        // Enter edit mode with command pre-filled
                        self.input_mode = InputMode::EditingCommand;
                        self.input_buffer = cmd.command.clone();
                        self.status_message = format!(
                            "Edit command (working dir: {}) then press Enter to execute",
                            self.get_target_workspace()
                        );
                    }
                }
                View::WorkspaceManager => {
                    if matches!(self.input_mode, InputMode::AddingDirectory) {
                        let path = self.input_buffer.trim().trim_end_matches('/').to_string();
                        if !path.is_empty() {
                            // Check if we're editing or adding
                            if let Some(index) = self.editing_workspace_index {
                                // Editing existing workspace
                                if let Some(workspace) = self.workspace_directories.get(index) {
                                    let old_path = workspace.path.clone();
                                    match self.edit_workspace(&old_path, &path) {
                                        Ok(_) => {
                                            self.status_message = format!("✓ Updated to {}", path);
                                        }
                                        Err(e) => {
                                            self.status_message = format!("✗ Error: {}", e);
                                        }
                                    }
                                }
                            } else {
                                // Adding new workspace
                                match self.add_workspace(&path) {
                                    Ok(_) => {
                                        self.status_message = format!("✓ Added {}", path);
                                    }
                                    Err(e) => {
                                        self.status_message = format!("✗ Error: {}", e);
                                    }
                                }
                            }
                            self.input_mode = InputMode::Normal;
                            self.input_buffer.clear();
                            self.editing_workspace_index = None;
                        }
                    }
                }
                View::Detail => {}
                View::Overlay => {
                    // Submit form on Enter (if in form overlay)
                    if let Some(OverlayContent::Form { form, .. }) = &mut self.active_overlay {
                        let form_type = form.form_type;
                        match form.submit() {
                            Ok(values) => {
                                // Close overlay and dispatch to handler
                                self.close_overlay();
                                self.handle_form_submission(form_type, values);
                            }
                            Err(err) => {
                                self.status_message = format!("Validation error: {}", err);
                            }
                        }
                    }
                }
            },
            KeyCode::Tab => {
                // Tab to next field in overlay forms
                if matches!(self.current_view, View::Overlay) {
                    if let Some(OverlayContent::Form { form, .. }) = &mut self.active_overlay {
                        form.next_field();
                    }
                }
            }
            KeyCode::BackTab => {
                // Shift+Tab to previous field in overlay forms
                if matches!(self.current_view, View::Overlay) {
                    if let Some(OverlayContent::Form { form, .. }) = &mut self.active_overlay {
                        form.prev_field();
                    }
                }
            }
            KeyCode::Char(' ') if matches!(self.current_view, View::Overlay) => {
                // Space to toggle checkboxes/multi-select in overlay forms
                if let Some(OverlayContent::Form { form, .. }) = &mut self.active_overlay {
                    if let Some(field) = form.current_field_mut() {
                        field.handle_space();
                    }
                }
            }
            KeyCode::Char('?') => {
                self.status_message =
                    "Press 1-3 for views, ↑↓ to navigate, Enter to select, q to quit".to_string();
            }
            _ => {}
        }
    }

    pub fn get_selected_project(&self) -> Option<&Project> {
        self.projects.get(self.selected_project)
    }

    pub fn get_selected_command(&self) -> Option<&Command> {
        self.commands.get(self.selected_command)
    }

    pub fn get_target_workspace(&self) -> String {
        self.workspace_directories
            .get(self.selected_target_workspace)
            .map(|w| w.path.clone())
            .unwrap_or_else(|| "~/projects".to_string())
    }

    pub fn cycle_target_workspace(&mut self) {
        if !self.workspace_directories.is_empty() {
            self.selected_target_workspace =
                (self.selected_target_workspace + 1) % self.workspace_directories.len();
            self.status_message = format!("Target: {}", self.get_target_workspace());
        }
    }

    pub fn get_workspace_for_project(&self, project_path: &str) -> String {
        // Find which workspace this project belongs to
        for workspace in &self.workspace_directories {
            if let Ok(safe_workspace) = crate::path::SafePath::from_user_input(&workspace.path) {
                let expanded_workspace = safe_workspace.to_string();
                if project_path.starts_with(&expanded_workspace) {
                    return workspace.path.clone();
                }
            }
        }
        // Fallback to showing the path
        project_path.to_string()
    }

    /// Hot reload all state from disk
    /// Called on file changes (via watcher) or manual refresh (r key)
    pub fn hotload(&mut self) {
        crate::log::info("HOTLOAD", "Reloading all state from disk");

        // Reload config and rediscover projects
        if let Ok(config) = crate::config::Config::load() {
            // Clear and reload workspace directories
            self.workspace_directories.clear();

            let workspace_path = &config.global.workspace.path;
            self.workspace_directories.push(WorkspaceDir {
                path: workspace_path.clone(),
                is_primary: true,
                project_count: 0,
            });

            for registered in &config.global.workspace.registered {
                self.workspace_directories.push(WorkspaceDir {
                    path: registered.clone(),
                    is_primary: false,
                    project_count: 0,
                });
            }

            // Rediscover all projects
            if let Ok(discovered) = crate::projects::discover_projects(&config.global) {
                self.projects = discovered
                    .into_iter()
                    .map(|p| Project {
                        name: p.config.project.name.clone(),
                        description: p.config.project.description.clone().unwrap_or_else(|| {
                            format!("{} project", p.config.project.project_type)
                        }),
                        drivers: vec![p.config.project.ecosystem],
                        path: p.path.to_string_lossy().to_string(),
                    })
                    .collect();

                // Update project counts for workspaces
                for workspace in &mut self.workspace_directories {
                    let safe_path = match crate::path::SafePath::from_user_input(&workspace.path) {
                        Ok(path) => path,
                        Err(_) => continue, // Skip invalid paths
                    };
                    let expanded_path = safe_path.to_string();
                    let normalized_workspace = expanded_path.trim_end_matches('/').to_lowercase();
                    workspace.project_count = self
                        .projects
                        .iter()
                        .filter(|p| {
                            let normalized_proj = p.path.trim_end_matches('/').to_lowercase();
                            normalized_proj.starts_with(&normalized_workspace)
                        })
                        .count();
                }

                // Update UI state
                if !self.projects.is_empty() {
                    // Keep selection valid
                    if self.selected_project >= self.projects.len() {
                        self.selected_project = self.projects.len() - 1;
                    }
                    self.project_list_state.select(Some(self.selected_project));
                } else {
                    self.selected_project = 0;
                    self.project_list_state.select(None);
                }
            }

            // Update workspace list state
            if !self.workspace_directories.is_empty() {
                if self.selected_workspace >= self.workspace_directories.len() {
                    self.selected_workspace = self.workspace_directories.len() - 1;
                }
                self.workspace_list_state
                    .select(Some(self.selected_workspace));
            }
        }

        // Reload commands for current context
        self.update_commands();

        // Refresh project states (git status, build state)
        self.refresh_project_states();

        crate::log::info("HOTLOAD", "Reload complete");
    }

    /// Categorize a command intelligently using keyword matching
    fn categorize_command(command: &str) -> CommandFilter {
        let mut cmd_lower = command.to_lowercase();

        // Strip common prefixes to get to the actual command
        // Handle: "cd dir &&", "cd dir/nested &&"
        if let Some(idx) = cmd_lower.find(" && ") {
            if cmd_lower.trim_start().starts_with("cd ") {
                cmd_lower = cmd_lower[idx + 4..].to_string();
            }
        }

        // Git commands are special - must start with "git "
        if cmd_lower.trim_start().starts_with("git ") {
            return CommandFilter::Git;
        }

        // Keyword-based categorization (language-agnostic)
        // Build: compilation, bundling, development servers
        const BUILD_KEYWORDS: &[&str] = &["build", "compile", "bundle", "dev", "run", "start", "watch", "serve"];

        // Test: testing, coverage, specs
        const TEST_KEYWORDS: &[&str] = &["test", "spec", "coverage", "bench"];

        // Lint: formatting, linting, type checking
        const LINT_KEYWORDS: &[&str] = &["lint", "fmt", "format", "clippy", "check", "prettier", "eslint"];

        // Check for test first (more specific than build)
        if TEST_KEYWORDS.iter().any(|kw| cmd_lower.contains(kw)) {
            CommandFilter::Test
        } else if LINT_KEYWORDS.iter().any(|kw| cmd_lower.contains(kw)) {
            CommandFilter::Lint
        } else if BUILD_KEYWORDS.iter().any(|kw| cmd_lower.contains(kw)) {
            CommandFilter::Build
        } else {
            CommandFilter::Other
        }
    }

    pub fn update_commands(&mut self) {
        // Save current selection
        let previous_selection = self.selected_command;

        self.commands.clear();

        // Get project path if a project is selected (clone to avoid borrow issues)
        let project_path = self.get_selected_project().map(|p| p.path.clone());

        if let Some(path) = project_path {
            // Project selected: load project-specific commands
            self.load_project_commands(&path);
        } else {
            // No project selected: show init commands
            self.load_init_commands();
        }

        // Filter commands based on active filter
        if self.command_filter != CommandFilter::All {
            self.commands.retain(|cmd| {
                Self::categorize_command(&cmd.command) == self.command_filter
            });
        }

        // Preserve selection if still valid, otherwise reset to 0
        if !self.commands.is_empty() {
            let new_selection = if previous_selection < self.commands.len() {
                previous_selection
            } else {
                0
            };
            self.command_list_state.select(Some(new_selection));
            self.selected_command = new_selection;
        }
    }

    fn load_init_commands(&mut self) {
        self.commands = vec![
            Command {
                name: "init go cli <name>".to_string(),
                description: "Initialize Go CLI project".to_string(),
                command: "byte init go cli my-project".to_string(),
            },
            Command {
                name: "init bun web <name>".to_string(),
                description: "Initialize Bun web application".to_string(),
                command: "byte init bun web my-app".to_string(),
            },
            Command {
                name: "init rust cli <name>".to_string(),
                description: "Initialize Rust CLI project".to_string(),
                command: "byte init rust cli my-tool".to_string(),
            },
        ];
    }

    fn load_project_commands(&mut self, project_path: &str) {
        use std::path::PathBuf;

        let config_path = PathBuf::from(project_path).join("byte.toml");

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<crate::config::ProjectConfig>(&content) {
                // Add build commands
                if let Some(build_cmds) = config.build {
                    for (name, cmd) in build_cmds.iter() {
                        self.commands.push(Command {
                            name: format!("build: {}", name),
                            description: format!("Run build task: {}", name),
                            command: cmd.clone(),
                        });
                    }
                }

                // Add custom commands
                if let Some(custom_cmds) = config.commands {
                    for (name, cmd) in custom_cmds.iter() {
                        self.commands.push(Command {
                            name: name.clone(),
                            description: format!("Run: {}", name),
                            command: cmd.clone(),
                        });
                    }
                }
            }
        }

        // Add common git commands
        self.commands.push(Command {
            name: "git status".to_string(),
            description: "Show git status".to_string(),
            command: "git status".to_string(),
        });
        self.commands.push(Command {
            name: "git diff".to_string(),
            description: "Show uncommitted changes".to_string(),
            command: "git diff".to_string(),
        });
    }
}

pub fn run() -> anyhow::Result<()> {
    if !is_tty() {
        anyhow::bail!("TUI requires a terminal. Please run in an interactive terminal.");
    }

    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    // Set up file watcher
    let (file_tx, file_rx) = std::sync::mpsc::channel();
    let watcher = setup_file_watcher(file_tx, &app)?;

    // Set up command execution channel
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    app.command_tx = Some(cmd_tx);

    let res = run_app(&mut terminal, &mut app, file_rx, cmd_rx);

    // Clean up watcher
    drop(watcher);

    restore_terminal(&mut terminal)?;

    res
}

fn is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
}

fn setup_file_watcher(
    tx: std::sync::mpsc::Sender<()>,
    app: &App,
) -> anyhow::Result<
    notify_debouncer_full::Debouncer<notify::RecommendedWatcher, notify_debouncer_full::FileIdMap>,
> {
    use notify::{RecursiveMode, Watcher};
    use notify_debouncer_full::new_debouncer;
    use std::time::Duration;

    // Create debounced watcher (500ms debounce)
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |result: Result<Vec<notify_debouncer_full::DebouncedEvent>, Vec<notify::Error>>| {
            match result {
                Ok(events) => {
                    // Check if any event is for a byte.toml, config.toml, or git file
                    for event in events {
                        let path = event.paths.first();
                        if let Some(path) = path {
                            let should_reload =
                                // byte.toml or config.toml changes
                                path.file_name().and_then(|n| n.to_str()) == Some("byte.toml")
                                || path.file_name().and_then(|n| n.to_str()) == Some("config.toml")
                                // Git changes: HEAD (branch switch), refs (commits), index (staging)
                                || path.to_string_lossy().contains(".git/HEAD")
                                || path.to_string_lossy().contains(".git/refs/heads/")
                                || path.to_string_lossy().contains(".git/index");

                            if should_reload {
                                crate::log::info("WATCHER", &format!("Detected change: {:?}", path));
                                // Notify main loop to reload
                                let _ = tx.send(());
                                break;
                            }
                        }
                    }
                }
                Err(_errors) => {
                    // Silently ignore watcher errors
                }
            }
        },
    )?;

    // Watch all workspace directories
    for workspace in &app.workspace_directories {
        let safe_path = match crate::path::SafePath::from_user_input(&workspace.path) {
            Ok(path) => path,
            Err(e) => {
                crate::log::error("WATCHER", &format!("Invalid workspace path '{}': {}", workspace.path, e));
                continue;
            }
        };
        let expanded = safe_path.to_string();
        match debouncer
            .watcher()
            .watch(safe_path.expanded(), RecursiveMode::Recursive)
        {
            Ok(_) => crate::log::info("WATCHER", &format!("Watching: {}", expanded)),
            Err(e) => crate::log::error("WATCHER", &format!("Failed to watch {}: {}", expanded, e)),
        }
    }

    // Watch global config directory
    if let Some(config_dir) = dirs::config_dir() {
        let byte_config = config_dir.join("byte");
        if byte_config.exists() {
            match debouncer
                .watcher()
                .watch(&byte_config, RecursiveMode::NonRecursive)
            {
                Ok(_) => crate::log::info("WATCHER", &format!("Watching config: {:?}", byte_config)),
                Err(e) => crate::log::error("WATCHER", &format!("Failed to watch config: {}", e)),
            }
        }
    }

    Ok(debouncer)
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// Suspend TUI, run interactive command with terminal access, then resume TUI
fn run_interactive_command(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    editor: &str,
    file_path: &str,
) -> anyhow::Result<()> {
    use crate::exec::CommandBuilder;

    // Suspend TUI
    restore_terminal(terminal)?;

    // Run editor with inherited stdin/stdout/stderr via exec API
    let result = CommandBuilder::new(editor)
        .arg(file_path)
        .execute_interactive();

    // Resume TUI
    *terminal = setup_terminal()?;

    result
}

/// Simple fuzzy matching: all chars from needle must appear in haystack in order
fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut hay_chars = haystack.chars();
    for n in needle.chars() {
        if !hay_chars.any(|h| h == n) {
            return false;
        }
    }
    true
}

/// Truncate a path string to fit within max_width, preserving start and end
fn truncate_path(path: &str, max_width: usize) -> String {
    let char_count = path.chars().count();

    if char_count <= max_width {
        return path.to_string();
    }

    if max_width <= 3 {
        return "...".to_string();
    }

    // Reserve 3 chars for "..."
    let available = max_width.saturating_sub(3);
    let start_len = (available + 1) / 2; // Slightly favor start
    let end_len = available / 2;

    // Use character-based iteration to avoid breaking UTF-8
    let start: String = path.chars().take(start_len).collect();
    let end: String = path.chars().rev().take(end_len).collect::<Vec<_>>()
        .into_iter().rev().collect();

    format!("{}...{}", start, end)
}

/// Strip ANSI escape codes from a string (aggressive full coverage)
fn strip_ansi_codes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // ESC sequence detected
            match chars.peek() {
                Some(&'[') => {
                    // CSI sequence: ESC [ ... letter
                    chars.next(); // consume '['
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        // CSI terminators are ASCII letters (A-Z, a-z) or @ through ~
                        if next_ch.is_ascii_alphabetic() || ('@'..='~').contains(&next_ch) {
                            break;
                        }
                    }
                }
                Some(&']') => {
                    // OSC sequence: ESC ] ... BEL or ESC \
                    chars.next(); // consume ']'
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch == '\x07' {
                            // BEL terminator
                            chars.next();
                            break;
                        }
                        if next_ch == '\x1b' {
                            chars.next(); // consume ESC
                            if chars.peek() == Some(&'\\') {
                                // ESC \ terminator
                                chars.next();
                                break;
                            }
                        } else {
                            chars.next();
                        }
                    }
                }
                Some(&'(') | Some(&')') => {
                    // Character set selection: ESC ( X or ESC ) X
                    chars.next(); // consume '(' or ')'
                    chars.next(); // skip the next character
                }
                _ => {
                    // Other ESC sequences, skip one more character
                    chars.next();
                }
            }
        } else if ch >= ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            // Printable character or whitespace
            result.push(ch);
        }
        // Skip control characters
    }

    result
}

/// Launch fuzzy finder for directory selection
fn run_fuzzy_picker(current_input: &str) -> Option<String> {
    use skim::prelude::*;
    use std::io::Cursor;

    // Build list of candidate directories
    let mut candidates = Vec::new();

    // Add common directories
    if let Some(home) = dirs::home_dir() {
        candidates.push("~".to_string());

        // Common subdirectories
        for subdir in &[
            "Desktop",
            "Documents",
            "Downloads",
            "Music",
            "Pictures",
            "Videos",
            "projects",
        ] {
            let path = home.join(subdir);
            if path.exists() {
                candidates.push(format!("~/{}", subdir));
            }
        }

        // Add registered workspaces from config
        if let Ok(config) = crate::config::Config::load() {
            candidates.push(config.global.workspace.path.clone());
            for registered in &config.global.workspace.registered {
                candidates.push(registered.clone());
            }
        }

        // If there's a partial path typed, add subdirectories
        if !current_input.is_empty() {
            let expanded = match crate::path::SafePath::from_user_input(current_input) {
                Ok(safe_path) => safe_path.to_string(),
                Err(_) => current_input.to_string(), // Use original if expansion fails
            };
            if let Ok(entries) = std::fs::read_dir(&expanded) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry
                        .file_type()
                        .ok()
                        .map(|ft| ft.is_dir())
                        .unwrap_or(false)
                    {
                        if let Ok(name) = entry.file_name().into_string() {
                            let full_path =
                                format!("{}/{}", current_input.trim_end_matches('/'), name);
                            candidates.push(full_path);
                        }
                    }
                }
            }
        }
    }

    // Remove duplicates
    candidates.sort();
    candidates.dedup();

    if candidates.is_empty() {
        return None;
    }

    // Prepare skim input
    let input = candidates.join("\n");

    // Configure skim options
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .reverse(true)
        .prompt("Select directory> ".to_string())
        .build()
        .ok()?;

    // Run skim
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let output = Skim::run_with(&options, Some(items))?;

    // Handle selection
    if output.is_abort {
        return None;
    }

    output
        .selected_items
        .first()
        .map(|item| item.output().to_string())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    file_rx: std::sync::mpsc::Receiver<()>,
    cmd_rx: std::sync::mpsc::Receiver<CommandResult>,
) -> anyhow::Result<()> {
    // Clear any initialization logs before first draw
    terminal.clear()?;

    while !app.should_quit {
        // Clear terminal if needed (e.g., after closing log preview)
        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }

        terminal.draw(|f| ui(f, app))?;

        // Update animation frame if command is executing
        if app.executing_command.is_some() {
            app.build_animation_frame += 1;
        }

        // Auto-dismiss command result display after 3 seconds
        if let Some((_, timestamp)) = app.command_result_display {
            if timestamp.elapsed() >= Duration::from_secs(3) {
                app.command_result_display = None;
            }
        }

        // Check for command completion (non-blocking)
        if let Ok(result) = cmd_rx.try_recv() {
            // Store the result but keep animating for minimum duration
            app.pending_result = Some(result);
        }

        // Process pending result after minimum animation duration (500ms)
        if let Some(_) = &app.pending_result {
            if let Some(start_time) = app.build_animation_start {
                let elapsed = start_time.elapsed();
                if elapsed >= Duration::from_millis(500) {
                    // Minimum animation time has passed, process result
                    let result = app.pending_result.take().unwrap();
                    app.executing_command = None;
                    app.handle_command_result(result);
                }
            } else {
                // No start time? Process immediately
                let result = app.pending_result.take().unwrap();
                app.executing_command = None;
                app.handle_command_result(result);
            }
        }

        // Check for file system events (non-blocking)
        if let Ok(_) = file_rx.try_recv() {
            // Throttle: only hotload if it's been at least 1 second since last hotload
            if app.last_hotload.elapsed() >= Duration::from_secs(1) {
                app.hotload();
                app.last_hotload = Instant::now();
                // Don't override status message - visual updates are enough feedback
            }
        }

        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                // Handle Ctrl+D specially for fuzzy picker
                if matches!(app.input_mode, InputMode::AddingDirectory)
                    && key.code == KeyCode::Char('d')
                    && key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    app.launch_fuzzy_picker = true;
                } else {
                    app.handle_key(key.code);
                }
            }
        }

        // Handle fuzzy picker launch (suspend TUI)
        if app.launch_fuzzy_picker {
            app.launch_fuzzy_picker = false;

            // Restore terminal
            restore_terminal(terminal)?;

            // Run fuzzy picker
            if let Some(selected) = run_fuzzy_picker(&app.input_buffer) {
                app.input_buffer = selected;
                app.status_message = format!("Selected: {}", app.input_buffer);
            } else {
                app.status_message = "Cancelled".to_string();
            }

            // Re-setup terminal
            *terminal = setup_terminal()?;
        }

        // Handle pending editor request (suspend TUI and run editor)
        if let Some((editor, file_path)) = app.pending_editor.take() {
            match run_interactive_command(terminal, &editor, &file_path) {
                Ok(_) => {
                    app.status_message = format!("✓ Closed {}", editor);
                }
                Err(e) => {
                    app.status_message = format!("✗ Editor error: {}", e);
                }
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Length(1), // Tab bar
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer/status
            ]
            .as_ref(),
        )
        .split(f.area());

    // Header - clean, centered branding
    let header = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("●", Style::default().fg(theme::ACCENT)),
        Span::raw("  "),
        Span::styled(
            "B Y T E",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("│", Style::default().fg(theme::SEPARATOR)),
        Span::raw("  "),
        Span::styled(
            "Project Orchestration",
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default()),
    );
    f.render_widget(header, chunks[0]);

    // Tab bar
    render_tab_bar(f, chunks[1], app);

    // Main content
    match app.current_view {
        View::ProjectBrowser => render_project_browser(f, chunks[2], app),
        View::CommandPalette => render_command_palette(f, chunks[2], app),
        View::Detail => render_detail(f, chunks[2], app),
        View::WorkspaceManager => render_workspace_manager(f, chunks[2], app),
        View::Overlay => render_overlay(f, chunks[2], app),
    }

    // Footer
    render_footer(f, chunks[3], app);

    // Horizontal progress bar on right side (if command is executing or showing result)
    if app.executing_command.is_some() || app.command_result_display.is_some() {
        render_progress_bar(f, f.area(), app);
    }
}

fn render_tab_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let current_view = match app.current_view {
        View::ProjectBrowser => 0,
        View::CommandPalette => 1,
        View::Detail => 2,
        View::WorkspaceManager => 3,
        View::Overlay => 99, // Overlay is modal, not a tab
    };

    let tabs = vec![
        ("1", "Projects", 0),
        ("2", "Commands", 1),
        ("3", "Details", 2),
        ("4", "Workspace", 3),
    ];

    let mut spans = vec![Span::raw("  ")];

    for (i, (key, label, index)) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }

        let is_active = current_view == *index;

        if is_active {
            spans.push(Span::styled(
                *key,
                Style::default()
                    .fg(theme::BADGE_TEXT)
                    .bg(theme::BADGE_BG)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                *label,
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                *key,
                Style::default().fg(theme::TEXT_SECONDARY),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                *label,
                Style::default().fg(theme::TEXT_SECONDARY),
            ));
        }
    }

    // Add selected project on the right side
    if let Some(project) = app.get_selected_project() {
        // Calculate padding to right-align the project name
        let tabs_text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        let tabs_len = tabs_text.len();
        let project_text = format!("Project: {}", project.name);
        let available_width = area.width as usize;

        if tabs_len + project_text.len() + 4 < available_width {
            let padding_len = available_width - tabs_len - project_text.len() - 2;
            spans.push(Span::raw(" ".repeat(padding_len)));
            spans.push(Span::styled(
                project_text,
                Style::default().fg(theme::TEXT_SECONDARY),
            ));
            spans.push(Span::raw("  "));
        }
    }

    let tabs_line = Line::from(spans);
    let paragraph = Paragraph::new(tabs_line).alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let status_style = if app.status_message.contains("Error") {
        Style::default().fg(theme::ERROR)
    } else {
        Style::default().fg(theme::TEXT_SECONDARY)
    };

    let footer = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(app.status_message.clone(), status_style),
            Span::raw("  "),
            Span::styled("│", Style::default().fg(theme::SEPARATOR)),
            Span::raw("  "),
            Span::styled("?", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" help", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw("  "),
            Span::styled("r", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" reload", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw("  "),
            Span::styled("q", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" quit", Style::default().fg(theme::TEXT_SECONDARY)),
        ]),
    ])
    .alignment(Alignment::Left);
    f.render_widget(footer, area);
}

fn render_project_browser(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Split into title and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(0),    // Project list
        ])
        .split(inner_area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Projects",
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}", app.projects.len()),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]),
        Line::from(""),
    ]);
    f.render_widget(title, chunks[0]);

    // Project list with table layout: Project (left) | Location (right)
    let items: Vec<ListItem> = app
        .projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let is_selected = i == app.selected_project;

            let drivers_display = project
                .drivers
                .iter()
                .map(|d| format!("#{}", d))
                .collect::<Vec<_>>()
                .join(" ");

            // Calculate column widths (60% project, 40% location)
            let total_width = inner_area.width.saturating_sub(4) as usize; // Account for padding
            let project_width = (total_width * 60) / 100;
            let location_width = total_width.saturating_sub(project_width);

            // Show workspace instead of full path
            let workspace_path = app.get_workspace_for_project(&project.path);
            let display_path = if workspace_path.chars().count() > location_width {
                let skip = workspace_path.chars().count() - location_width + 1;
                let truncated: String = workspace_path.chars().skip(skip).collect();
                format!("…{}", truncated)
            } else {
                workspace_path
            };

            // Line 1: Name (left) | Path (right)
            let name_text = format!("{:width$}", project.name, width = project_width);
            let line1 = vec![
                Span::raw("  "),
                Span::styled(
                    name_text,
                    Style::default()
                        .fg(if is_selected {
                            theme::ACCENT
                        } else {
                            theme::TEXT_PRIMARY
                        })
                        .add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(display_path, Style::default().fg(theme::TEXT_SECONDARY)),
            ];

            // Line 2: Description (left) | empty (right)
            let max_desc_width = total_width.saturating_sub(2); // Account for "  " indent
            let truncated_desc = truncate_path(&project.description, max_desc_width);
            let line2 = vec![
                Span::raw("  "),
                Span::styled(
                    truncated_desc,
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ];

            // Line 3: Ecosystem tags (left) | empty (right)
            let line3 = vec![
                Span::raw("  "),
                Span::styled(drivers_display, Style::default().fg(theme::TEXT_SECONDARY)),
            ];

            let content = vec![
                Line::from(line1),
                Line::from(line2),
                Line::from(line3),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default())
        .highlight_symbol("▸ ");

    let mut state = app.project_list_state.clone();
    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_command_palette(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Split into tab bar, command list, and preview
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter tabs
            Constraint::Length(2), // Title
            Constraint::Min(8),    // Command list
            Constraint::Length(1), // Separator
            Constraint::Length(6), // Preview
        ])
        .split(inner_area);

    // Filter tabs (horizontal bar)
    let tab_spans: Vec<Span> = CommandFilter::all_filters()
        .into_iter()
        .flat_map(|filter| {
            let is_active = filter == app.command_filter;
            vec![
                Span::styled(
                    "[",
                    Style::default().fg(if is_active {
                        theme::ACCENT
                    } else {
                        theme::TEXT_SECONDARY
                    }),
                ),
                Span::styled(
                    filter.as_str().to_string(),
                    Style::default()
                        .fg(if is_active {
                            theme::ACCENT
                        } else {
                            theme::TEXT_SECONDARY
                        })
                        .add_modifier(if is_active {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(
                    "]",
                    Style::default().fg(if is_active {
                        theme::ACCENT
                    } else {
                        theme::TEXT_SECONDARY
                    }),
                ),
                Span::raw(" "),
            ]
        })
        .collect();

    let tabs = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tabs, chunks[0]);

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Commands",
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}", app.commands.len()),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]),
        Line::from(""),
    ]);
    f.render_widget(title, chunks[1]);

    // Command list with table layout: Command (left) | Target (right)
    let items: Vec<ListItem> = app
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == app.selected_command;

            // Calculate column widths (65% command, 35% target)
            let total_width = inner_area.width.saturating_sub(4) as usize;
            let command_width = (total_width * 65) / 100;
            let target_width = total_width.saturating_sub(command_width);

            // Truncate target path to fit
            // Show project's path when project selected, otherwise target workspace
            let target_path = if let Some(project) = app.get_selected_project() {
                project.path.clone()
            } else {
                app.get_target_workspace()
            };

            let display_target = if target_path.chars().count() > target_width {
                let skip = target_path.chars().count() - target_width + 1;
                let truncated: String = target_path.chars().skip(skip).collect();
                format!("…{}", truncated)
            } else {
                target_path
            };

            // Line 1: Command name (left) | Target directory (right)
            let name_text = format!("{:width$}", cmd.name, width = command_width);
            let line1 = vec![
                Span::raw("  "),
                Span::styled(
                    name_text,
                    Style::default()
                        .fg(if is_selected {
                            theme::ACCENT
                        } else {
                            theme::TEXT_PRIMARY
                        })
                        .add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(display_target, Style::default().fg(theme::TEXT_SECONDARY)),
            ];

            // Line 2: Description (left) | empty (right)
            let max_desc_width = total_width.saturating_sub(2); // Account for "  " indent
            let truncated_desc = truncate_path(&cmd.description, max_desc_width);
            let line2 = vec![
                Span::raw("  "),
                Span::styled(
                    truncated_desc,
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ];

            let content = vec![Line::from(line1), Line::from(line2), Line::from("")];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default())
        .highlight_symbol("▸ ");

    let mut state = app.command_list_state.clone();
    f.render_stateful_widget(list, chunks[2], &mut state);

    // Separator
    let separator = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(inner_area.width as usize),
        Style::default().fg(theme::SEPARATOR),
    )]));
    f.render_widget(separator, chunks[3]);

    // Preview or Edit Mode
    if matches!(app.input_mode, InputMode::EditingCommand) {
        // Edit mode: show working directory and editable command
        let edit_ui = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("Working dir: {}", app.get_target_workspace()),
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("$ ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(
                    &app.input_buffer,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("_", Style::default().fg(theme::ACCENT)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[Enter]", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(" execute  ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled("[Esc]", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(" cancel", Style::default().fg(theme::TEXT_SECONDARY)),
            ]),
        ]);
        f.render_widget(edit_ui, chunks[4]);
    } else if let Some(cmd) = app.get_selected_command() {
        // Preview mode
        let preview = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Preview",
                Style::default().fg(theme::TEXT_SECONDARY),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("$ ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(cmd.command.clone(), Style::default().fg(theme::ACCENT)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[t]", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(
                    " change target workspace",
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]),
        ]);
        f.render_widget(preview, chunks[4]);
    }
}

fn render_detail(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    // Note: Log viewing now uses the overlay API (View::Overlay with OverlayContent::LogViewer)
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    if let Some(project) = app.get_selected_project() {
        let mut lines = vec![];

        // Full 2-column layout
        let total_width = inner_area.width.saturating_sub(4) as usize;
        let left_width = (total_width * 70) / 100;
        let right_width = total_width.saturating_sub(left_width);

        // Truncate path to fit in right column (accounting for "PATH: " prefix)
        let path_prefix = "PATH: ";
        let max_path_width = right_width.saturating_sub(path_prefix.len());
        let truncated_path = truncate_path(&project.path, max_path_width);

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:width$}", project.name, width = left_width),
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}{}", path_prefix, truncated_path),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            project.description.clone(),
            Style::default().fg(theme::TEXT_SECONDARY),
        )]));
        lines.push(Line::from(""));

        // Git Status and Build State
        if let Some(state) = app.get_current_project_state() {
            lines.extend(render_git_status(&state.git));
            lines.push(Line::from(""));

            if let Some(build) = &state.build {
                lines.extend(render_build_state(build));
                lines.push(Line::from(""));
            }
        }

        // Recent Logs
        lines.extend(render_recent_logs(&project.path, app.selected_log));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled(
            "─".repeat(40),
            Style::default().fg(theme::SEPARATOR),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("1", Style::default().fg(theme::ACCENT)),
            Span::styled(
                " to return to projects",
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]));

        let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, inner_area);
    } else {
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "No project selected",
                Style::default().fg(theme::TEXT_SECONDARY),
            )]),
        ])
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
        f.render_widget(paragraph, inner_area);
    }
}

/// Render git status information
fn render_git_status(git: &GitStatus) -> Vec<Line<'_>> {
    let mut lines = vec![];

    if !git.is_repo {
        lines.push(Line::from(vec![Span::styled(
            "Not a git repository",
            Style::default().fg(theme::TEXT_SECONDARY),
        )]));
        return lines;
    }

    // Branch and status line
    let branch_text = git
        .branch
        .as_ref()
        .map(|b| format!("Branch: {}", b))
        .unwrap_or_else(|| "Branch: (detached HEAD)".to_string());

    let status_color = if git.is_clean {
        theme::SUCCESS
    } else {
        theme::ERROR
    };

    let status_text = if git.is_clean {
        " ✓ Clean"
    } else {
        " ● Modified"
    };

    lines.push(Line::from(vec![
        Span::styled(branch_text, Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled(
            status_text,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // File counts (if not clean)
    if !git.is_clean {
        let mut parts = vec![];

        if git.staged > 0 {
            parts.push(format!("{} staged", git.staged));
        }
        if git.modified > 0 {
            parts.push(format!("{} modified", git.modified));
        }
        if git.untracked > 0 {
            parts.push(format!("{} untracked", git.untracked));
        }

        if !parts.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", parts.join(", ")),
                Style::default().fg(theme::TEXT_SECONDARY),
            )]));
        }
    }

    // Tracking info (ahead/behind)
    if git.ahead > 0 || git.behind > 0 {
        let mut tracking = vec![];
        if git.ahead > 0 {
            tracking.push(format!("↑{}", git.ahead));
        }
        if git.behind > 0 {
            tracking.push(format!("↓{}", git.behind));
        }

        lines.push(Line::from(vec![Span::styled(
            format!("  {}", tracking.join(" ")),
            Style::default().fg(theme::ACCENT),
        )]));
    }

    lines
}

/// Render build state information
fn render_build_state(build: &BuildState) -> Vec<Line<'_>> {
    use chrono::{DateTime, Utc};

    let mut lines = vec![];

    // Status line
    let (status_text, status_color) = match build.status {
        BuildStatus::Success => ("✓ Success", theme::SUCCESS),
        BuildStatus::Failed => ("✗ Failed", theme::ERROR),
        BuildStatus::Running => ("⟳ Running", theme::ACCENT),
    };

    lines.push(Line::from(vec![
        Span::styled(
            format!("Build: {}", build.task),
            Style::default().fg(theme::TEXT_PRIMARY),
        ),
        Span::raw("  "),
        Span::styled(
            status_text,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Timestamp
    if let Some(dt) = DateTime::<Utc>::from_timestamp(build.timestamp, 0) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt);

        let time_ago = if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else {
            format!("{} days ago", duration.num_days())
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  Last build: {}", time_ago),
            Style::default().fg(theme::TEXT_SECONDARY),
        )]));
    }

    lines
}

/// Render recent command logs
fn render_recent_logs(project_path: &str, selected_log: usize) -> Vec<Line<'_>> {
    let mut lines = vec![];

    lines.push(Line::from(vec![Span::styled(
        "Recent Logs",
        Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD),
    )]));

    let logs = crate::fs::ProjectFileSystem::new(project_path).ok().and_then(|fs| fs.recent_logs_all(5).ok()).unwrap_or_default();

    if logs.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No logs available",
            Style::default().fg(theme::TEXT_SECONDARY),
        )]));
    } else {
        use chrono::{DateTime, Utc};

        for (i, log) in logs.iter().enumerate() {
            let is_selected = i == selected_log;
            // Format timestamp as relative time
            let time_str = if let Ok(system_time) = log.timestamp.duration_since(std::time::UNIX_EPOCH) {
                if let Some(dt) = DateTime::<Utc>::from_timestamp(system_time.as_secs() as i64, 0) {
                    let now = Utc::now();
                    let duration = now.signed_duration_since(dt);

                    if duration.num_seconds() < 60 {
                        "just now".to_string()
                    } else if duration.num_minutes() < 60 {
                        format!("{}m ago", duration.num_minutes())
                    } else if duration.num_hours() < 24 {
                        format!("{}h ago", duration.num_hours())
                    } else {
                        format!("{}d ago", duration.num_days())
                    }
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            // Category badge color
            let category_color = match log.category.as_str() {
                "build" => theme::ACCENT,
                "git" => ratatui::style::Color::Rgb(255, 165, 0), // Orange
                "lint" => ratatui::style::Color::Rgb(147, 112, 219), // Purple
                _ => theme::TEXT_SECONDARY,
            };

            // Selection indicator and styling
            let (indicator, text_style) = if is_selected {
                (">", Style::default().add_modifier(Modifier::BOLD))
            } else {
                (" ", Style::default())
            };

            lines.push(Line::from(vec![
                Span::styled(indicator, Style::default().fg(theme::ACCENT)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", log.category),
                    Style::default().fg(category_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    log.filename.clone(),
                    text_style.fg(theme::TEXT_PRIMARY),
                ),
                Span::raw("  "),
                Span::styled(
                    time_str,
                    text_style.fg(theme::TEXT_SECONDARY),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Use ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("↑↓", Style::default().fg(theme::ACCENT)),
            Span::styled(" to navigate, ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("l", Style::default().fg(theme::ACCENT)),
            Span::styled(" to preview, ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("o", Style::default().fg(theme::ACCENT)),
            Span::styled(" to open in editor", Style::default().fg(theme::TEXT_SECONDARY)),
        ]));
    }

    lines
}


fn render_workspace_manager(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Split into title and content
    // Give more space for help when showing fuzzy matches
    let help_height =
        if matches!(app.input_mode, InputMode::AddingDirectory) && !app.fuzzy_matches.is_empty() {
            Constraint::Min(12) // Enough room for input + matches + help
        } else {
            Constraint::Length(3) // Normal help text
        };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(8),    // Workspace list
            Constraint::Length(1), // Separator
            help_height,           // Help text (dynamic)
        ])
        .split(inner_area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Workspace Manager",
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}", app.workspace_directories.len()),
                Style::default().fg(theme::TEXT_SECONDARY),
            ),
        ]),
        Line::from(""),
    ]);
    f.render_widget(title, chunks[0]);

    // Workspace list
    // Calculate max path width (reserve space for count and indicators)
    let max_path_width = inner_area.width.saturating_sub(30) as usize;

    let items: Vec<ListItem> = app
        .workspace_directories
        .iter()
        .enumerate()
        .map(|(i, workspace)| {
            let is_selected = i == app.selected_workspace;

            let mut line1 = vec![Span::raw("  ")];

            // Truncate path if too long - show end of path (most relevant part)
            let display_path = if workspace.path.chars().count() > max_path_width {
                let skip = workspace.path.chars().count() - max_path_width + 1; // +1 for ellipsis
                let truncated: String = workspace.path.chars().skip(skip).collect();
                format!("…{}", truncated)
            } else {
                workspace.path.clone()
            };

            // Path
            line1.push(Span::styled(
                display_path,
                Style::default()
                    .fg(if is_selected {
                        theme::ACCENT
                    } else {
                        theme::TEXT_PRIMARY
                    })
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ));

            // Spacing
            line1.push(Span::raw("  "));

            // Project count
            let count_text = if workspace.project_count == 1 {
                "1 project".to_string()
            } else {
                format!("{} projects", workspace.project_count)
            };
            line1.push(Span::styled(
                count_text,
                Style::default().fg(theme::TEXT_SECONDARY),
            ));

            // Primary indicator
            if workspace.is_primary {
                line1.push(Span::raw("  "));
                line1.push(Span::styled(
                    "[primary]",
                    Style::default().fg(theme::TEXT_SECONDARY),
                ));
            }

            let content = vec![Line::from(line1), Line::from("")];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default())
        .highlight_symbol("▸ ");

    let mut state = app.workspace_list_state.clone();
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Separator
    let separator = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(inner_area.width as usize),
        Style::default().fg(theme::SEPARATOR),
    )]));
    f.render_widget(separator, chunks[2]);

    // Help text or input prompt
    let help = if matches!(app.input_mode, InputMode::AddingDirectory) {
        // Build lines for input field and fuzzy matches
        let label = "Enter directory path: ";
        let available_width = inner_area
            .width
            .saturating_sub((2 + label.len() + 1) as u16) as usize;

        // Create input line with truncated buffer if needed
        let input_line = if app.input_buffer.chars().count() > available_width {
            let skip = app.input_buffer.chars().count() - available_width;
            let truncated: String = app.input_buffer.chars().skip(skip).collect();
            Line::from(vec![
                Span::raw("  "),
                Span::styled(label, Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(
                    format!("…{}", truncated),
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("_", Style::default().fg(theme::ACCENT)),
            ])
        } else {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(label, Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(
                    &app.input_buffer,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("_", Style::default().fg(theme::ACCENT)),
            ])
        };

        let mut lines = vec![Line::from(""), input_line];

        // Show fuzzy matches if available - make them PROMINENT like zsh
        if !app.fuzzy_matches.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "───────────────────────────────────────────────",
                    Style::default().fg(theme::ACCENT),
                ),
            ]));

            // Scrolling window: show 8 matches at a time, scroll to keep selection visible
            let visible_count = 8;
            let total_matches = app.fuzzy_matches.len();

            // Calculate visible window based on selection
            let start_idx = if app.fuzzy_selected < visible_count {
                0
            } else {
                // Keep selection near bottom of window
                (app.fuzzy_selected - visible_count + 1)
                    .min(total_matches.saturating_sub(visible_count))
            };
            let end_idx = (start_idx + visible_count).min(total_matches);

            // Show matches in visible window
            // Calculate max path width to prevent overflow (accounting for padding and indicators)
            let max_path_width = inner_area.width.saturating_sub(8) as usize;

            for (window_i, path) in app.fuzzy_matches[start_idx..end_idx].iter().enumerate() {
                let actual_idx = start_idx + window_i;
                let is_selected = actual_idx == app.fuzzy_selected && app.fuzzy_browsing;

                // Truncate path if too long - show end of path (most relevant part)
                let display_path = if path.chars().count() > max_path_width {
                    let skip = path.chars().count() - max_path_width + 1; // +1 for ellipsis
                    let truncated: String = path.chars().skip(skip).collect();
                    format!("…{}", truncated)
                } else {
                    path.clone()
                };

                if is_selected {
                    // Selected item: bright cyan background, bold
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("▸ {}", display_path),
                            Style::default()
                                .fg(theme::BADGE_TEXT)
                                .bg(theme::ACCENT)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                } else {
                    // Unselected: bright white text, no dim
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            display_path,
                            Style::default()
                                .fg(theme::TEXT_PRIMARY)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
            }

            // Show scroll indicator
            if total_matches > visible_count {
                let hidden_above = start_idx;
                let hidden_below = total_matches - end_idx;

                let mut indicator_parts = vec![];
                if hidden_above > 0 {
                    indicator_parts.push(format!("↑ {} more above", hidden_above));
                }
                if hidden_below > 0 {
                    indicator_parts.push(format!("↓ {} more below", hidden_below));
                }

                if !indicator_parts.is_empty() {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("  {}", indicator_parts.join("  •  ")),
                            Style::default().fg(theme::TEXT_SECONDARY),
                        ),
                    ]));
                }
            }

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "───────────────────────────────────────────────",
                    Style::default().fg(theme::ACCENT),
                ),
            ]));
        }

        // Add keyboard help
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("[Tab]", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" complete  ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("[Ctrl+D]", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" fuzzy find  ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("[Enter]", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" add  ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("[Esc]", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(" cancel", Style::default().fg(theme::TEXT_SECONDARY)),
        ]));

        Paragraph::new(lines)
    } else {
        // Show normal help
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("a", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(" add  ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled("e", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(" edit  ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled("d", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(" remove  ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled("1", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(
                    " back to projects",
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]),
        ])
    };
    f.render_widget(help, chunks[3]);
}

/// Create a centered modal overlay with cleared background
/// Returns the cleared modal area ready for content rendering
fn create_centered_modal(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    width: u16,
    height: u16,
) -> ratatui::layout::Rect {
    let modal_width = area.width.min(width);
    let modal_height = area.height.min(height);

    let modal_area = ratatui::layout::Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width,
        height: modal_height,
    };

    // Clear the modal area to prevent background bleed
    f.render_widget(Clear, modal_area);

    modal_area
}

fn render_overlay(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let Some(overlay) = &app.active_overlay else {
        // No overlay active - this shouldn't happen if View::Overlay is set
        return;
    };

    match overlay {
        OverlayContent::LogViewer { log_path, scroll_offset, project_path } => {
            render_overlay_log_viewer(f, area, log_path, *scroll_offset, project_path);
        }
        OverlayContent::Form { form, position } => {
            render_overlay_form(f, area, form, position);
        }
    }
}

/// Render log viewer in overlay mode (full screen)
fn render_overlay_log_viewer(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    log_path: &PathBuf,
    scroll_offset: usize,
    project_path: &str,
) {
    use std::fs;
    use std::io::{BufRead, BufReader};

    // Use minimal margin for maximum content visibility
    let inner_area = area.inner(Margin {
        horizontal: 1,
        vertical: 0,
    });

    // Get filename for display
    let filename = log_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Read log file
    let content = match fs::File::open(log_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines()
                .filter_map(|line| line.ok())
                .collect();
            lines
        }
        Err(e) => {
            vec![format!("Error reading log file: {}", e)]
        }
    };

    // Calculate visible range - maximize visible content
    let total_lines = content.len();
    let visible_height = inner_area.height.saturating_sub(4) as usize; // borders + 2-line header
    let start_line = scroll_offset.min(total_lines.saturating_sub(1));
    let end_line = (start_line + visible_height).min(total_lines);

    // Create lines to display
    let mut display_lines = vec![];

    // Calculate available width for path truncation (account for borders: 2 chars)
    let available_width = inner_area.width.saturating_sub(2) as usize;
    let path_prefix = "PATH: ";
    let max_path_width = available_width.saturating_sub(path_prefix.len());
    let truncated_project_path = truncate_path(project_path, max_path_width);

    // Ultra-compact 2-line header
    display_lines.push(Line::from(vec![
        Span::styled(filename, Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            format!("({}/{})", start_line + 1, total_lines),
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
        Span::raw("  "),
        Span::styled("[Esc] close  [↑↓] scroll  [PgUp/PgDn] page", Style::default().fg(theme::TEXT_SECONDARY)),
    ]));
    display_lines.push(Line::from(vec![
        Span::styled(
            format!("{}{}", path_prefix, truncated_project_path),
            Style::default().fg(theme::TEXT_SECONDARY),
        ),
    ]));

    // Add visible lines with character-based truncation (prevents text bleed)
    let content_width = available_width;
    for line in content.iter().skip(start_line).take(end_line - start_line) {
        // Strip ANSI codes and truncate to fit width (character-based, not byte-based)
        let stripped = strip_ansi_codes(line);
        let char_count = stripped.chars().count();
        let truncated = if char_count > content_width {
            let truncated_chars: String = stripped.chars().take(content_width.saturating_sub(1)).collect();
            format!("{}…", truncated_chars)
        } else {
            stripped
        };
        display_lines.push(Line::from(truncated));
    }

    let paragraph = Paragraph::new(display_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(theme::ACCENT)));

    f.render_widget(paragraph, inner_area);
}

/// Render form in overlay mode (centered modal)
fn render_overlay_form(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    form: &crate::forms::Form,
    position: &OverlayPosition,
) {
    // Determine modal dimensions based on position
    let modal_area = match position {
        OverlayPosition::FullScreen => area,
        OverlayPosition::Centered { width, height } => {
            create_centered_modal(f, area, *width, *height)
        }
        OverlayPosition::Custom { x, y, width, height } => {
            ratatui::layout::Rect {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            }
        }
    };

    // Inner area with padding
    let inner_area = modal_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Split into sections: title, description, fields, help
    let mut constraints = vec![
        Constraint::Length(2), // Title
    ];
    if form.description.is_some() {
        constraints.push(Constraint::Length(2)); // Description
    }
    constraints.push(Constraint::Min(0)); // Fields
    constraints.push(Constraint::Length(3)); // Help text

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let mut chunk_idx = 0;

    // Render title with background fill
    let title = Paragraph::new(vec![Line::from(vec![Span::styled(
        &form.title,
        Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::BOLD),
    )])])
    .style(Style::default().bg(Color::Black))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black)),
    );
    f.render_widget(title, modal_area);
    chunk_idx += 1;

    // Render description if present
    if let Some(desc) = &form.description {
        let description = Paragraph::new(vec![Line::from(vec![Span::styled(
            desc,
            Style::default().fg(theme::TEXT_SECONDARY),
        )])])
        .style(Style::default().bg(Color::Black));
        f.render_widget(description, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    // Render fields
    let fields_area = chunks[chunk_idx];
    let mut field_lines = Vec::new();

    for (i, field) in form.fields.iter().enumerate() {
        let is_current = i == form.current_field;
        let label = field.label();

        // Add spacing between fields
        if i > 0 {
            field_lines.push(Line::from(""));
        }

        // Render field label
        let label_style = if is_current {
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_PRIMARY)
        };

        field_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(if is_current { "▶ " } else { "  " }, label_style),
            Span::styled(label, label_style),
        ]));

        // Render field value based on type
        render_form_field(&mut field_lines, field, is_current);
    }

    let fields_widget = Paragraph::new(field_lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .style(Style::default().bg(Color::Black));
    f.render_widget(fields_widget, fields_area);

    // Render help text
    chunk_idx += 1;
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Tab/Shift+Tab", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw(" navigate  "),
            Span::styled("↑↓", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw(" select  "),
            Span::styled("Space", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw(" toggle  "),
            Span::styled("Enter", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw(" submit  "),
            Span::styled("Esc", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::raw(" cancel"),
        ]),
    ];
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Black));
    f.render_widget(help, chunks[chunk_idx]);
}

/// Helper function to render form field values
fn render_form_field<'a>(field_lines: &mut Vec<Line<'a>>, field: &'a crate::forms::FormField, is_current: bool) {
    match field {
        crate::forms::FormField::TextInput { value, placeholder, .. }
        | crate::forms::FormField::Email { value, placeholder, .. } => {
            let display = if value.is_empty() {
                placeholder.as_str()
            } else {
                value.as_str()
            };
            let value_style = if value.is_empty() {
                Style::default().fg(theme::TEXT_SECONDARY)
            } else if is_current {
                Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };
            field_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(display, value_style),
                if is_current {
                    Span::styled("█", Style::default().fg(theme::ACCENT))
                } else {
                    Span::raw("")
                },
            ]));
        }
        crate::forms::FormField::TextArea { value, placeholder, height, .. } => {
            let display = if value.is_empty() {
                placeholder.as_str()
            } else {
                value.as_str()
            };
            let value_style = if value.is_empty() {
                Style::default().fg(theme::TEXT_SECONDARY)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };
            for line in display.lines().take(*height) {
                field_lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(line, value_style),
                ]));
            }
            if is_current {
                field_lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled("█", Style::default().fg(theme::ACCENT)),
                ]));
            }
        }
        crate::forms::FormField::Number { value, min, max, .. } => {
            let mut display_parts = vec![value.to_string()];
            if let Some(min_val) = min {
                if let Some(max_val) = max {
                    display_parts.push(format!(" (range: {}-{})", min_val, max_val));
                } else {
                    display_parts.push(format!(" (min: {})", min_val));
                }
            } else if let Some(max_val) = max {
                display_parts.push(format!(" (max: {})", max_val));
            }
            let display = display_parts.join("");
            let value_style = if is_current {
                Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };
            field_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(display, value_style),
                if is_current {
                    Span::styled("█", Style::default().fg(theme::ACCENT))
                } else {
                    Span::raw("")
                },
            ]));
        }
        crate::forms::FormField::Select { options, selected, .. } => {
            for (idx, option) in options.iter().enumerate() {
                let is_selected = idx == *selected;
                let marker = if is_selected { "●" } else { "○" };
                let style = if is_current && is_selected {
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(theme::SUCCESS)
                } else {
                    Style::default().fg(theme::TEXT_SECONDARY)
                };
                field_lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(marker, style),
                    Span::raw(" "),
                    Span::styled(option, style),
                ]));
            }
        }
        crate::forms::FormField::MultiSelect { options, selected, .. } => {
            for (idx, option) in options.iter().enumerate() {
                let is_selected = selected.contains(&idx);
                let marker = if is_selected { "☑" } else { "☐" };
                let style = if is_current {
                    if is_selected {
                        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::ACCENT)
                    }
                } else if is_selected {
                    Style::default().fg(theme::SUCCESS)
                } else {
                    Style::default().fg(theme::TEXT_SECONDARY)
                };
                field_lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(marker, style),
                    Span::raw(" "),
                    Span::styled(option, style),
                ]));
            }
        }
        crate::forms::FormField::Path { value, kind, .. } => {
            let display = if value.is_empty() {
                match kind {
                    crate::forms::PathKind::File => "Select a file...",
                    crate::forms::PathKind::Directory => "Select a directory...",
                    crate::forms::PathKind::Any => "Select a path...",
                }
            } else {
                value.as_str()
            };
            let value_style = if value.is_empty() {
                Style::default().fg(theme::TEXT_SECONDARY)
            } else if is_current {
                Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(theme::TEXT_PRIMARY)
            };
            field_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(display, value_style),
                if is_current {
                    Span::styled("█", Style::default().fg(theme::ACCENT))
                } else {
                    Span::raw("")
                },
            ]));
        }
        crate::forms::FormField::Checkbox { checked, .. } => {
            let marker = if *checked { "☑" } else { "☐" };
            let style = if is_current {
                Style::default().fg(theme::ACCENT)
            } else if *checked {
                Style::default().fg(theme::SUCCESS)
            } else {
                Style::default().fg(theme::TEXT_SECONDARY)
            };
            field_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(marker, style),
            ]));
        }
    }
}

// ============================================================================

/// Render horizontal progress bar on the right side
fn render_progress_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    // Check if showing result or running animation
    if let Some((success, _)) = app.command_result_display {
        render_command_result(f, area, success);
        return;
    }

    // Progress bar configuration
    let bar_width = 32; // Width of the progress bar
    let bar_height = 4; // Height: border + text line + bar line + border

    // Position aligned with preview section in command palette
    // Header (3) + Tab bar (1) + Title (2) + Command list (min 8) + Separator (1) = ~15 lines from top
    let x_pos = area.width.saturating_sub(bar_width + 2);
    let y_pos = area.height.saturating_sub(12); // Position ~12 lines from bottom (aligns with preview)

    let bar_area = ratatui::layout::Rect {
        x: x_pos,
        y: y_pos,
        width: bar_width,
        height: bar_height,
    };

    // Create scanner/spotlight effect - bright segment moving through darker bar
    let bar_content_width = (bar_width.saturating_sub(4)) as usize; // Account for borders + padding
    let bright_segment_width = 8; // Fixed bright segment width

    // Calculate position of bright segment (0 to bar_content_width)
    let total_positions = bar_content_width;
    let position = (app.build_animation_frame / 2) % total_positions; // Slow down animation

    // Build the bar: dark before position, bright at position, dark after
    let left_count = position.min(bar_content_width);
    let bright_start = left_count;
    let bright_end = (left_count + bright_segment_width).min(bar_content_width);
    let right_start = bright_end;

    let left_str = "█".repeat(left_count);
    let bright_str = "█".repeat(bright_end.saturating_sub(bright_start));
    let right_str = "█".repeat(bar_content_width.saturating_sub(right_start));

    // Get elapsed time
    let elapsed_text = if let Some(start) = app.build_animation_start {
        let elapsed = start.elapsed().as_millis();
        if elapsed >= 1000 {
            format!("{}s", elapsed / 1000)
        } else {
            format!("{}ms", elapsed)
        }
    } else {
        "0ms".to_string()
    };

    // Create the widget with three-tone bar effect
    let bar_line = vec![
        Span::raw(" "),
        Span::styled(
            left_str,
            Style::default().fg(ratatui::style::Color::Rgb(100, 100, 100)),
        ),
        Span::styled(
            bright_str,
            Style::default()
                .fg(ratatui::style::Color::Rgb(255, 255, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            right_str,
            Style::default().fg(ratatui::style::Color::Rgb(100, 100, 100)),
        ),
        Span::raw(" "),
    ];

    let progress_widget = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" Running ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                elapsed_text,
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(bar_line),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT))
            .border_type(ratatui::widgets::BorderType::Rounded),
    );

    f.render_widget(progress_widget, bar_area);
}

/// Render command result (success or failure) in attention-grabbing style
fn render_command_result(f: &mut Frame, area: ratatui::layout::Rect, success: bool) {
    let bar_width = 32;
    let bar_height = 4;

    let x_pos = area.width.saturating_sub(bar_width + 2);
    let y_pos = area.height.saturating_sub(12);

    let bar_area = ratatui::layout::Rect {
        x: x_pos,
        y: y_pos,
        width: bar_width,
        height: bar_height,
    };

    // Choose color and message based on success/failure
    let (color, icon, message) = if success {
        (ratatui::style::Color::Green, "✓", "SUCCESS")
    } else {
        (ratatui::style::Color::Red, "✗", "FAILED")
    };

    // Create a solid bar in the result color
    let bar_content_width = bar_width.saturating_sub(4) as usize;
    let solid_bar = "█".repeat(bar_content_width);

    let result_widget = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!(" {} {}", icon, message),
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!(" {}", solid_bar),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .border_type(ratatui::widgets::BorderType::Rounded),
    );

    f.render_widget(result_widget, bar_area);
}
