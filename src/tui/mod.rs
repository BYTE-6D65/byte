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
}

pub enum View {
    ProjectBrowser,
    CommandPalette,
    Detail,
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
            KeyCode::Up => match self.current_view {
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
                View::Detail => {}
            },
            KeyCode::Down => match self.current_view {
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
                View::Detail => {}
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                app.handle_key(key.code);
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
    }

    // Footer
    render_footer(f, chunks[3], app);
}

fn render_tab_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let current_view = match app.current_view {
        View::ProjectBrowser => 0,
        View::CommandPalette => 1,
        View::Detail => 2,
    };

    let tabs = vec![
        ("1", "Projects", 0),
        ("2", "Commands", 1),
        ("3", "Details", 2),
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
