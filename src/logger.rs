use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    pub fn new(log_dir: &str) -> Self {
        let absolute_dir = if PathBuf::from(log_dir).is_absolute() {
            PathBuf::from(log_dir)
        } else {
            env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(log_dir)
        };

        if let Err(e) = fs::create_dir_all(&absolute_dir) {
            eprintln!("Failed to create log dir {:?}: {}", absolute_dir, e);
        }

        Self {
            log_dir: absolute_dir,
        }
    }

    pub fn info(&self, message: &str) {
        self.log(&format!("INFO: {}\n", message));
    }

    pub fn error(&self, message: &str) {
        self.log(&format!("ERROR: {}\n", message));
    }

    fn log(&self, message: &str) {
        let log_file = self.log_dir.join("byte.log");

        let mut file = match OpenOptions::new().append(true).create(true).open(&log_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open log file {:?}: {}", log_file, e);
                return;
            }
        };

        if let Err(e) = file.write_all(message.as_bytes()) {
            eprintln!("Failed to write to log file: {}", e);
        }
    }
}

lazy_static::lazy_static! {
    static ref LOGGER: Logger = {
        // Byte standard: all logs go to .byte/logs/ relative to project root
        // This makes byte itself "byte compatible"
        let log_dir = ".byte/logs";

        let logger = Logger::new(log_dir);
        logger.info("Logger initialized - using .byte/logs/");
        logger
    };
}

pub fn info(message: &str) {
    LOGGER.info(message);
}

pub fn error(message: &str) {
    LOGGER.error(message);
}

/// Log command output to dedicated category directories
pub fn log_command_output(
    project_path: &str,
    command: &str,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> Result<PathBuf, std::io::Error> {
    use chrono::Local;

    // Auto-detect category from command
    let category = categorize_command(command);

    // Create directory structure: .byte/logs/commands/{category}/
    let log_base = PathBuf::from(project_path).join(".byte/logs/commands").join(category);
    fs::create_dir_all(&log_base)?;

    // Generate timestamped filename
    let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
    let command_name = extract_command_name(command);
    let filename = format!("{}-{}.log", timestamp, command_name);
    let log_file = log_base.join(&filename);

    // Write log with metadata
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_file)?;

    writeln!(file, "Command: {}", command)?;
    writeln!(file, "Timestamp: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(file, "Exit Code: {}", exit_code)?;
    writeln!(file, "Working Directory: {}", project_path)?;
    writeln!(file, "\n--- STDOUT ---")?;
    file.write_all(stdout.as_bytes())?;
    writeln!(file, "\n--- STDERR ---")?;
    file.write_all(stderr.as_bytes())?;

    // Cleanup old logs (keep last 20 per category)
    cleanup_old_logs(&log_base, 20)?;

    Ok(log_file)
}

/// Categorize command into build/git/lint/other
fn categorize_command(command: &str) -> &str {
    let cmd_lower = command.to_lowercase();

    if cmd_lower.contains("cargo build") || cmd_lower.contains("cargo run")
        || cmd_lower.contains("cargo test") || cmd_lower.contains("cargo check")
        || cmd_lower.contains("go build") || cmd_lower.contains("bun build") {
        "build"
    } else if cmd_lower.contains("cargo clippy") || cmd_lower.contains("cargo fmt")
        || cmd_lower.contains("eslint") || cmd_lower.contains("prettier") {
        "lint"
    } else if cmd_lower.starts_with("git ") {
        "git"
    } else {
        "other"
    }
}

/// Extract a short name from command for filename
fn extract_command_name(command: &str) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.len() >= 2 {
        // For "cargo build", return "build"
        // For "git status", return "status"
        parts[1].chars().filter(|c| c.is_alphanumeric() || *c == '-').collect()
    } else if !parts.is_empty() {
        parts[0].chars().filter(|c| c.is_alphanumeric() || *c == '-').collect()
    } else {
        "cmd".to_string()
    }
}

/// Keep only the last N log files in a directory
fn cleanup_old_logs(dir: &PathBuf, keep_count: usize) -> Result<(), std::io::Error> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "log").unwrap_or(false))
        .collect();

    // Sort by modification time (newest first)
    entries.sort_by_key(|e| std::cmp::Reverse(
        e.metadata().ok().and_then(|m| m.modified().ok())
    ));

    // Remove old files beyond keep_count
    for entry in entries.iter().skip(keep_count) {
        let _ = fs::remove_file(entry.path());
    }

    Ok(())
}

/// Get recent command logs for a project
pub fn get_recent_logs(project_path: &str, limit: usize) -> Vec<LogEntry> {
    let commands_dir = PathBuf::from(project_path).join(".byte/logs/commands");

    if !commands_dir.exists() {
        return Vec::new();
    }

    let mut logs = Vec::new();

    // Scan all category directories
    if let Ok(categories) = fs::read_dir(&commands_dir) {
        for category_entry in categories.filter_map(|e| e.ok()) {
            if !category_entry.path().is_dir() {
                continue;
            }

            let category = category_entry.file_name().to_string_lossy().to_string();

            // Scan log files in this category
            if let Ok(log_files) = fs::read_dir(category_entry.path()) {
                for log_file in log_files.filter_map(|e| e.ok()) {
                    let path = log_file.path();
                    if path.extension().map(|e| e == "log").unwrap_or(false) {
                        if let Ok(metadata) = log_file.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                logs.push(LogEntry {
                                    path: path.clone(),
                                    category: category.clone(),
                                    timestamp: modified,
                                    filename: path.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by timestamp (newest first)
    logs.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
    logs.truncate(limit);
    logs
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub path: PathBuf,
    pub category: String,
    pub timestamp: std::time::SystemTime,
    pub filename: String,
}

/// Get user's default editor from environment variables
pub fn get_default_editor() -> String {
    use std::process::Command;

    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Try to find a common terminal editor
            let editors = ["vim", "nano", "vi", "emacs"];

            for editor in &editors {
                if Command::new("which")
                    .arg(editor)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    return editor.to_string();
                }
            }

            // Last resort: use vi (should be on all Unix systems)
            "vi".to_string()
        })
}
