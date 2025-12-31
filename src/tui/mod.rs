use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;
use std::time::Duration;

// Theme colors optimized for OLED black backgrounds
mod theme {
    use ratatui::style::Color;

    // Primary colors
    pub const ACCENT: Color = Color::Cyan; // Brand color, primary actions
    pub const SUCCESS: Color = Color::Green; // Success states, active items
    pub const ERROR: Color = Color::Red; // Error states, warnings

    // Text hierarchy (optimized for OLED black)
    pub const TEXT_PRIMARY: Color = Color::White; // Primary content
    pub const TEXT_SECONDARY: Color = Color::Rgb(160, 160, 160); // Secondary content (descriptions)
    pub const TEXT_TERTIARY: Color = Color::Rgb(140, 140, 140); // Tertiary content (labels, hashtags)
    pub const TEXT_DIM: Color = Color::Rgb(120, 120, 120); // Subtle text (footer hints)

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

pub enum InputMode {
    Normal,
    AddingDirectory,
}

pub struct App {
    pub should_quit: bool,
    pub current_view: View,
    pub projects: Vec<Project>,
    pub commands: Vec<Command>,
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
    // Inline fuzzy matching
    pub fuzzy_matches: Vec<String>,
    pub fuzzy_selected: usize,
    pub fuzzy_browsing: bool, // true when navigating matches, false when typing
}

pub enum View {
    ProjectBrowser,
    CommandPalette,
    Detail,
    WorkspaceManager,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            should_quit: false,
            current_view: View::ProjectBrowser,
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
            fuzzy_matches: vec![],
            fuzzy_selected: 0,
            fuzzy_browsing: false,
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
                    let expanded_path = shellexpand::tilde(&workspace.path).to_string();
                    workspace.project_count = app
                        .projects
                        .iter()
                        .filter(|p| p.path.starts_with(&expanded_path))
                        .count();
                }

                if !app.projects.is_empty() {
                    app.project_list_state.select(Some(0));
                    app.status_message = format!("Discovered {} projects", app.projects.len());
                } else {
                    app.status_message = "No projects found. Use 'byte init' to create one.".to_string();
                }
            }
        }

        app
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn add_workspace(&mut self, path: &str) -> Result<(), String> {
        // Load config
        let mut config = crate::config::Config::load().map_err(|e| e.to_string())?;

        // Add the path
        config.add_workspace_path(path).map_err(|e| e.to_string())?;

        // Reload workspace directories and projects
        self.reload_workspaces();

        Ok(())
    }

    pub fn remove_workspace(&mut self, path: &str) -> Result<(), String> {
        // Load config
        let mut config = crate::config::Config::load().map_err(|e| e.to_string())?;

        // Remove the path
        config.remove_workspace_path(path).map_err(|e| e.to_string())?;

        // Reload workspace directories and projects
        self.reload_workspaces();

        Ok(())
    }

    fn reload_workspaces(&mut self) {
        // Reload config and projects (same logic as App::new())
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

            if !self.workspace_directories.is_empty() {
                let new_selected = self.selected_workspace.min(self.workspace_directories.len() - 1);
                self.selected_workspace = new_selected;
                self.workspace_list_state.select(Some(new_selected));
            }

            // Rediscover projects
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

                // Update project counts
                for workspace in &mut self.workspace_directories {
                    let expanded_path = shellexpand::tilde(&workspace.path).to_string();
                    workspace.project_count = self
                        .projects
                        .iter()
                        .filter(|p| p.path.starts_with(&expanded_path))
                        .count();
                }
            }
        }
    }


    /// Update fuzzy matches based on current input
    fn update_fuzzy_matches(&mut self) {
        if self.input_buffer.is_empty() {
            // Show common directories when empty
            self.fuzzy_matches.clear();
            if let Some(home) = dirs::home_dir() {
                self.fuzzy_matches.push("~".to_string());
                for subdir in &["Desktop", "Documents", "Downloads", "Music", "Pictures", "Videos", "projects"] {
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
            let expanded = shellexpand::tilde(&self.input_buffer).to_string();
            let path = std::path::Path::new(&expanded);

            // Determine what we're completing
            let (search_dir, prefix) = if expanded.ends_with('/') {
                (path, "")
            } else {
                let parent = path.parent().unwrap_or(std::path::Path::new("."));
                let filename = path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                (parent, filename)
            };

            // Read directory and collect matches
            if let Ok(entries) = std::fs::read_dir(search_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false) {
                        if let Ok(name) = entry.file_name().into_string() {
                            // Fuzzy match: check if all chars of prefix appear in order
                            if prefix.is_empty() || fuzzy_match(&name.to_lowercase(), &prefix.to_lowercase()) {
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
        let expanded = shellexpand::tilde(partial).to_string();
        let path = Path::new(&expanded);

        // Determine search directory and prefix
        let (search_dir, prefix) = if expanded.ends_with('/') {
            (path, "")
        } else {
            let parent = path.parent().unwrap_or(Path::new("."));
            let filename = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
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
            .filter(|e| {
                e.file_type()
                    .ok()
                    .map(|ft| ft.is_dir())
                    .unwrap_or(false)
            })
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

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.quit(),
            KeyCode::Char('1') => {
                self.current_view = View::ProjectBrowser;
                self.status_message = "Viewing projects".to_string();
            }
            KeyCode::Char('2') => {
                self.current_view = View::CommandPalette;
                self.status_message = "Viewing commands".to_string();
            }
            KeyCode::Char('3') => {
                self.current_view = View::Detail;
                self.status_message = format!(
                    "Viewing details for: {}",
                    self.projects
                        .get(self.selected_project)
                        .map(|p| p.name.as_str())
                        .unwrap_or("unknown")
                );
            }
            KeyCode::Char('4') => {
                self.current_view = View::WorkspaceManager;
                self.status_message = "Managing workspace directories".to_string();
            }
            // Workspace Manager specific keys
            KeyCode::Char('a') if matches!(self.current_view, View::WorkspaceManager) => {
                if matches!(self.input_mode, InputMode::Normal) {
                    self.input_mode = InputMode::AddingDirectory;
                    self.input_buffer.clear();
                    self.fuzzy_browsing = false;
                    self.update_fuzzy_matches();
                    self.status_message = "Type path, use ↑↓ to browse matches, Tab/Enter to select".to_string();
                }
            }
            KeyCode::Char('d') if matches!(self.current_view, View::WorkspaceManager) => {
                if matches!(self.input_mode, InputMode::Normal) {
                    if let Some(workspace) = self.workspace_directories.get(self.selected_workspace) {
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
            KeyCode::Esc if matches!(self.input_mode, InputMode::AddingDirectory) => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.status_message = "Cancelled".to_string();
            }
            KeyCode::Backspace if matches!(self.input_mode, InputMode::AddingDirectory) => {
                self.input_buffer.pop();
                self.fuzzy_browsing = false;
                self.update_fuzzy_matches();
            }
            KeyCode::Tab if matches!(self.input_mode, InputMode::AddingDirectory) => {
                // If browsing matches, select current match
                if self.fuzzy_browsing && !self.fuzzy_matches.is_empty() {
                    self.input_buffer = self.fuzzy_matches[self.fuzzy_selected].clone();
                    self.fuzzy_browsing = false;
                    self.update_fuzzy_matches();
                } else if let Some(completed) = self.complete_path(&self.input_buffer.clone()) {
                    self.input_buffer = completed;
                    self.update_fuzzy_matches();
                }
            }
            KeyCode::Char(c) if matches!(self.input_mode, InputMode::AddingDirectory) => {
                self.input_buffer.push(c);
                self.fuzzy_browsing = false;
                self.update_fuzzy_matches();
            }
            KeyCode::Up => {
                // Handle fuzzy match navigation when in input mode
                if matches!(self.input_mode, InputMode::AddingDirectory) && !self.fuzzy_matches.is_empty() {
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
                                self.workspace_list_state.select(Some(self.selected_workspace));
                            }
                        }
                        View::Detail => {}
                    }
                }
            },
            KeyCode::Down => {
                // Handle fuzzy match navigation when in input mode
                if matches!(self.input_mode, InputMode::AddingDirectory) && !self.fuzzy_matches.is_empty() {
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
                                self.status_message =
                                    format!("Selected: {}", self.projects[self.selected_project].name);
                            }
                        }
                        View::CommandPalette => {
                            if self.selected_command < self.commands.len().saturating_sub(1) {
                                self.selected_command += 1;
                                self.command_list_state.select(Some(self.selected_command));
                                self.status_message =
                                    format!("Selected: {}", self.commands[self.selected_command].name);
                            }
                        }
                        View::WorkspaceManager => {
                            if self.selected_workspace < self.workspace_directories.len().saturating_sub(1) {
                                self.selected_workspace += 1;
                                self.workspace_list_state.select(Some(self.selected_workspace));
                            }
                        }
                        View::Detail => {}
                    }
                }
            },
            KeyCode::Enter => match self.current_view {
                View::ProjectBrowser => {
                    if let Some(project) = self.projects.get(self.selected_project) {
                        self.status_message = format!("Opening {}...", project.name);
                        self.current_view = View::Detail;
                    }
                }
                View::CommandPalette => {
                    if let Some(cmd) = self.commands.get(self.selected_command) {
                        self.status_message = format!("Executing: {}", cmd.command);
                    }
                }
                View::WorkspaceManager => {
                    if matches!(self.input_mode, InputMode::AddingDirectory) {
                        let path = self.input_buffer.trim().to_string();
                        if !path.is_empty() {
                            match self.add_workspace(&path) {
                                Ok(_) => {
                                    self.status_message = format!("✓ Added {}", path);
                                }
                                Err(e) => {
                                    self.status_message = format!("✗ Error: {}", e);
                                }
                            }
                            self.input_mode = InputMode::Normal;
                            self.input_buffer.clear();
                        }
                    }
                }
                View::Detail => {}
            },
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
}

pub fn run() -> anyhow::Result<()> {
    if !is_tty() {
        anyhow::bail!("TUI requires a terminal. Please run in an interactive terminal.");
    }

    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    let res = run_app(&mut terminal, &mut app);

    restore_terminal(&mut terminal)?;

    res
}

fn is_tty() -> bool {
    atty::is(atty::Stream::Stdout)
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
        for subdir in &["Desktop", "Documents", "Downloads", "Music", "Pictures", "Videos", "projects"] {
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
            let expanded = shellexpand::tilde(current_input).to_string();
            if let Ok(entries) = std::fs::read_dir(&expanded) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false) {
                        if let Ok(name) = entry.file_name().into_string() {
                            let full_path = format!("{}/{}", current_input.trim_end_matches('/'), name);
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

    output.selected_items.first().map(|item| {
        item.output().to_string()
    })
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                // Handle Ctrl+D specially for fuzzy picker
                if matches!(app.input_mode, InputMode::AddingDirectory)
                    && key.code == KeyCode::Char('d')
                    && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
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
    }

    // Footer
    render_footer(f, chunks[3], app);
}

fn render_tab_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let current_view = match app.current_view {
        View::ProjectBrowser => 0,
        View::CommandPalette => 1,
        View::Detail => 2,
        View::WorkspaceManager => 3,
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
                Style::default().fg(theme::TEXT_TERTIARY),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                *label,
                Style::default().fg(theme::TEXT_TERTIARY),
            ));
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
            Span::styled(
                "?",
                Style::default()
                    .fg(theme::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                " help",
                Style::default()
                    .fg(theme::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ),
            Span::raw("  "),
            Span::styled(
                "q",
                Style::default()
                    .fg(theme::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                " quit",
                Style::default()
                    .fg(theme::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ),
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

    // Title
    let title_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner_area);

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
                Style::default().fg(theme::TEXT_TERTIARY),
            ),
        ]),
        Line::from(""),
    ]);
    f.render_widget(title, title_area[0]);

    // Project list
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

            let line1 = vec![
                Span::raw("  "),
                Span::styled(
                    project.name.clone(),
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
            ];

            let line2 = vec![
                Span::raw("  "),
                Span::styled(
                    project.description.clone(),
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
                Span::raw("  "),
                Span::styled(
                    drivers_display,
                    Style::default()
                        .fg(theme::TEXT_TERTIARY)
                        .add_modifier(Modifier::DIM),
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

    let mut state = app.project_list_state.clone();
    f.render_stateful_widget(list, title_area[1], &mut state);
}

fn render_command_palette(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Split into command list and preview
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(8),    // Command list
            Constraint::Length(1), // Separator
            Constraint::Length(6), // Preview
        ])
        .split(inner_area);

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
                Style::default().fg(theme::TEXT_TERTIARY),
            ),
        ]),
        Line::from(""),
    ]);
    f.render_widget(title, chunks[0]);

    // Command list
    let items: Vec<ListItem> = app
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == app.selected_command;

            let content = vec![
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        cmd.name.clone(),
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
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        cmd.description.clone(),
                        Style::default().fg(theme::TEXT_SECONDARY),
                    ),
                ]),
                Line::from(""),
            ];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default())
        .highlight_symbol("▸ ");

    let mut state = app.command_list_state.clone();
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Separator
    let separator = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(inner_area.width as usize),
        Style::default()
            .fg(theme::SEPARATOR)
            .add_modifier(Modifier::DIM),
    )]));
    f.render_widget(separator, chunks[2]);

    // Preview
    if let Some(cmd) = app.get_selected_command() {
        let preview = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Preview",
                Style::default()
                    .fg(theme::TEXT_TERTIARY)
                    .add_modifier(Modifier::DIM),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("$ ", Style::default().fg(theme::TEXT_TERTIARY)),
                Span::styled(cmd.command.clone(), Style::default().fg(theme::ACCENT)),
            ]),
        ]);
        f.render_widget(preview, chunks[3]);
    }
}

fn render_detail(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    if let Some(project) = app.get_selected_project() {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                project.name.clone(),
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                project.description.clone(),
                Style::default().fg(theme::TEXT_SECONDARY),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "PATH",
                Style::default()
                    .fg(theme::TEXT_TERTIARY)
                    .add_modifier(Modifier::DIM),
            )]),
            Line::from(vec![Span::styled(
                project.path.clone(),
                Style::default().fg(theme::TEXT_PRIMARY),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "DRIVERS",
                Style::default()
                    .fg(theme::TEXT_TERTIARY)
                    .add_modifier(Modifier::DIM),
            )]),
            Line::from(""),
        ];

        // Add driver details with nice formatting
        for driver in &project.drivers {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("●", Style::default().fg(theme::SUCCESS)),
                Span::raw("  "),
                Span::styled(driver.clone(), Style::default().fg(theme::TEXT_PRIMARY)),
                Span::raw("  "),
                Span::styled(
                    "active",
                    Style::default()
                        .fg(theme::TEXT_TERTIARY)
                        .add_modifier(Modifier::DIM),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "─".repeat(40),
            Style::default()
                .fg(theme::SEPARATOR)
                .add_modifier(Modifier::DIM),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "Press ",
                Style::default()
                    .fg(theme::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled("1", Style::default().fg(theme::ACCENT)),
            Span::styled(
                " to return to projects",
                Style::default()
                    .fg(theme::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ),
        ]));

        let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, inner_area);
    } else {
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "No project selected",
                Style::default().fg(theme::TEXT_TERTIARY),
            )]),
        ])
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);
        f.render_widget(paragraph, inner_area);
    }
}

fn render_workspace_manager(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let inner_area = area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    // Split into title and content
    // Give more space for help when showing fuzzy matches
    let help_height = if matches!(app.input_mode, InputMode::AddingDirectory) && !app.fuzzy_matches.is_empty() {
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
                Style::default().fg(theme::TEXT_TERTIARY),
            ),
        ]),
        Line::from(""),
    ]);
    f.render_widget(title, chunks[0]);

    // Workspace list
    let items: Vec<ListItem> = app
        .workspace_directories
        .iter()
        .enumerate()
        .map(|(i, workspace)| {
            let is_selected = i == app.selected_workspace;

            let mut line1 = vec![Span::raw("  ")];

            // Path
            line1.push(Span::styled(
                workspace.path.clone(),
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
                    Style::default()
                        .fg(theme::TEXT_TERTIARY)
                        .add_modifier(Modifier::DIM),
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
        Style::default()
            .fg(theme::SEPARATOR)
            .add_modifier(Modifier::DIM),
    )]));
    f.render_widget(separator, chunks[2]);

    // Help text or input prompt
    let help = if matches!(app.input_mode, InputMode::AddingDirectory) {
        // Build lines for input field and fuzzy matches
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Enter directory path: ",
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
                Span::styled(
                    &app.input_buffer,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("_", Style::default().fg(theme::ACCENT)),
            ]),
        ];

        // Show fuzzy matches if available - make them PROMINENT like zsh
        if !app.fuzzy_matches.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "───────────────────────────────────────────────",
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::DIM),
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
                (app.fuzzy_selected - visible_count + 1).min(total_matches.saturating_sub(visible_count))
            };
            let end_idx = (start_idx + visible_count).min(total_matches);

            // Show matches in visible window
            for (window_i, path) in app.fuzzy_matches[start_idx..end_idx].iter().enumerate() {
                let actual_idx = start_idx + window_i;
                let is_selected = actual_idx == app.fuzzy_selected && app.fuzzy_browsing;

                if is_selected {
                    // Selected item: bright cyan background, bold
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            format!("▸ {}", path),
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
                            path,
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
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::DIM),
                ),
            ]));
        }

        // Add keyboard help
        lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "[Tab]",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " complete  ",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    "[Ctrl+D]",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " fuzzy find  ",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    "[Enter]",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " add  ",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    "[Esc]",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " cancel",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
            ]));

        Paragraph::new(lines)
    } else {
        // Show normal help
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "a",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " add directory  ",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    "d",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " remove  ",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    "1",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    " back to projects",
                    Style::default()
                        .fg(theme::TEXT_DIM)
                        .add_modifier(Modifier::DIM),
                ),
            ]),
        ])
    };
    f.render_widget(help, chunks[3]);
}
