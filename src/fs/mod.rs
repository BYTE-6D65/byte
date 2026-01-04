/// File system abstraction for all Byte file operations
/// Addresses audit finding: File System Operations Module (API #5)
///
/// Responsibilities:
/// - Project structure creation (.byte/, ecosystem-specific dirs)
/// - Command log management (write, read, cleanup)
/// - State persistence (build.json, etc.)
/// - Atomic file operations
/// - .gitignore generation

use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File system manager for a Byte project
pub struct ProjectFileSystem {
    project_root: PathBuf,
}

/// Metadata about a log file
#[derive(Debug, Clone)]
pub struct LogFile {
    pub path: PathBuf,
    pub category: String,
    pub timestamp: SystemTime,
    pub filename: String,
}

impl ProjectFileSystem {
    /// Create a new file system manager for a project
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let project_root = root.into();

        if !project_root.exists() {
            fs::create_dir_all(&project_root)
                .with_context(|| format!("Failed to create project root: {}", project_root.display()))?;
        }

        Ok(Self { project_root })
    }

    /// Get the .byte directory path
    pub fn byte_dir(&self) -> PathBuf {
        self.project_root.join(".byte")
    }

    // ========================================================================
    // Project Initialization
    // ========================================================================

    /// Initialize complete project structure (all directories and base files)
    pub fn init_project(&self, ecosystem: &str, project_type: &str, project_name: &str) -> Result<()> {
        // 1. Create .byte/ runtime structure
        self.init_byte_structure()?;

        // 2. Create ecosystem-specific structure
        self.init_ecosystem_structure(ecosystem, project_type, project_name)?;

        // 3. Create .gitignore
        self.create_gitignore()?;

        Ok(())
    }

    /// Initialize .byte/ runtime directory structure
    pub fn init_byte_structure(&self) -> Result<()> {
        let byte_dir = self.byte_dir();

        // Create .byte/logs/commands/{category}/ directories
        for category in crate::tui::CommandFilter::log_categories() {
            let log_dir = byte_dir.join("logs").join("commands").join(category);
            fs::create_dir_all(&log_dir)
                .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;
        }

        // Create .byte/state/ directory
        let state_dir = byte_dir.join("state");
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create state directory: {}", state_dir.display()))?;

        Ok(())
    }

    /// Create ecosystem-specific project structure
    fn init_ecosystem_structure(&self, ecosystem: &str, project_type: &str, project_name: &str) -> Result<()> {
        match ecosystem {
            "rust" => self.init_rust_structure(project_type, project_name)?,
            "go" => self.init_go_structure(project_type, project_name)?,
            "bun" => self.init_bun_structure(project_type, project_name)?,
            _ => anyhow::bail!("Unsupported ecosystem: {}", ecosystem),
        }
        Ok(())
    }

    /// Initialize Rust project structure
    fn init_rust_structure(&self, project_type: &str, _project_name: &str) -> Result<()> {
        // Create src/ directory
        let src_dir = self.project_root.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create starter file based on type
        match project_type {
            "cli" | "bin" => {
                let main_rs = src_dir.join("main.rs");
                fs::write(&main_rs, "fn main() {\n    println!(\"Hello, world!\");\n}\n")?;
            }
            "lib" => {
                let lib_rs = src_dir.join("lib.rs");
                fs::write(&lib_rs, "pub fn add(left: u64, right: u64) -> u64 {\n    left + right\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn it_works() {\n        let result = add(2, 2);\n        assert_eq!(result, 4);\n    }\n}\n")?;
            }
            _ => anyhow::bail!("Unsupported Rust project type: {}", project_type),
        };

        // Note: Cargo.toml is created by `cargo init` via exec API
        // We just create the directory structure here

        Ok(())
    }

    /// Initialize Go project structure
    fn init_go_structure(&self, project_type: &str, project_name: &str) -> Result<()> {
        match project_type {
            "cli" | "bin" => {
                // Create cmd/{project_name}/ structure
                let cmd_dir = self.project_root.join("cmd").join(project_name);
                fs::create_dir_all(&cmd_dir)?;

                let main_go = cmd_dir.join("main.go");
                fs::write(&main_go, "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello, world!\")\n}\n")?;
            }
            "api" | "web" => {
                // Create cmd/server/ structure
                let cmd_dir = self.project_root.join("cmd").join("server");
                fs::create_dir_all(&cmd_dir)?;

                let main_go = cmd_dir.join("main.go");
                fs::write(&main_go, "package main\n\nimport (\n\t\"fmt\"\n\t\"net/http\"\n)\n\nfunc main() {\n\thttp.HandleFunc(\"/\", func(w http.ResponseWriter, r *http.Request) {\n\t\tfmt.Fprintf(w, \"Hello, world!\")\n\t})\n\thttp.ListenAndServe(\":8080\", nil)\n}\n")?;
            }
            _ => anyhow::bail!("Unsupported Go project type: {}", project_type),
        }

        // Create pkg/ and internal/ directories
        fs::create_dir_all(self.project_root.join("pkg"))?;
        fs::create_dir_all(self.project_root.join("internal"))?;

        // Note: go.mod is created by `go mod init` via exec API

        Ok(())
    }

    /// Initialize Bun/TypeScript project structure
    fn init_bun_structure(&self, _project_type: &str, _project_name: &str) -> Result<()> {
        // Create src/ directory
        let src_dir = self.project_root.join("src");
        fs::create_dir_all(&src_dir)?;

        // Create index.ts
        let index_ts = src_dir.join("index.ts");
        fs::write(&index_ts, "console.log(\"Hello, world!\");\n")?;

        // Note: package.json and tsconfig.json are created by `bun init` via exec API

        Ok(())
    }

    /// Create .gitignore with .byte/ excluded
    pub fn create_gitignore(&self) -> Result<()> {
        let gitignore_path = self.project_root.join(".gitignore");

        let content = "# Byte runtime data\n.byte/\n\n# Build artifacts\ntarget/\nnode_modules/\ndist/\nbuild/\n\n# IDE\n.vscode/\n.idea/\n*.swp\n*.swo\n\n# OS\n.DS_Store\nThumbs.db\n";

        fs::write(&gitignore_path, content)
            .with_context(|| format!("Failed to create .gitignore: {}", gitignore_path.display()))?;

        Ok(())
    }


    // ========================================================================
    // Command Log Management
    // ========================================================================

    /// Write command execution log to .byte/logs/commands/{category}/
    pub fn write_command_log(
        &self,
        category: &str,
        command: &str,
        stdout: &str,
        stderr: &str,
        exit_code: i32,
    ) -> Result<PathBuf> {
        let log_dir = self.byte_dir().join("logs").join("commands").join(category);
        fs::create_dir_all(&log_dir)?;

        // Generate timestamped filename
        let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
        let command_name = Self::extract_command_name(command);
        let filename = format!("{}-{}.log", timestamp, command_name);
        let log_file = log_dir.join(&filename);

        // Write log with metadata
        let mut content = String::new();
        content.push_str(&format!("Command: {}\n", command));
        content.push_str(&format!("Timestamp: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        content.push_str(&format!("Exit Code: {}\n", exit_code));
        content.push_str(&format!("Working Directory: {}\n", self.project_root.display()));
        content.push_str("\n--- STDOUT ---\n");
        content.push_str(stdout);
        content.push_str("\n--- STDERR ---\n");
        content.push_str(stderr);

        self.write_file_atomic(&log_file, content.as_bytes())?;

        // Cleanup old logs (keep last 20)
        self.cleanup_old_logs(category, 20)?;

        Ok(log_file)
    }

    /// Extract command name for log filename
    fn extract_command_name(command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[1].chars().filter(|c| c.is_alphanumeric() || *c == '-').collect()
        } else if !parts.is_empty() {
            parts[0].chars().filter(|c| c.is_alphanumeric() || *c == '-').collect()
        } else {
            "cmd".to_string()
        }
    }

    /// Get recent command logs for a category
    pub fn recent_logs(&self, category: &str, limit: usize) -> Result<Vec<LogFile>> {
        let log_dir = self.byte_dir().join("logs").join("commands").join(category);

        if !log_dir.exists() {
            return Ok(Vec::new());
        }

        let mut logs = Vec::new();

        for entry in fs::read_dir(&log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("log") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        logs.push(LogFile {
                            path: path.clone(),
                            category: category.to_string(),
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

        // Sort by timestamp (newest first)
        logs.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        logs.truncate(limit);

        Ok(logs)
    }

    /// Get recent logs across ALL categories
    pub fn recent_logs_all(&self, limit: usize) -> Result<Vec<LogFile>> {
        let commands_dir = self.byte_dir().join("logs").join("commands");

        if !commands_dir.exists() {
            return Ok(Vec::new());
        }

        let mut all_logs = Vec::new();

        // Scan all category directories
        for category_entry in fs::read_dir(&commands_dir)? {
            let category_entry = category_entry?;
            if !category_entry.path().is_dir() {
                continue;
            }

            let category = category_entry.file_name().to_string_lossy().to_string();

            // Get logs from this category
            if let Ok(category_logs) = self.recent_logs(&category, usize::MAX) {
                all_logs.extend(category_logs);
            }
        }

        // Sort by timestamp (newest first) across all categories
        all_logs.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        all_logs.truncate(limit);

        Ok(all_logs)
    }

    /// Remove old logs, keeping only the most recent N logs
    pub fn cleanup_old_logs(&self, category: &str, keep_count: usize) -> Result<usize> {
        let log_dir = self.byte_dir().join("logs").join("commands").join(category);

        if !log_dir.exists() {
            return Ok(0);
        }

        let mut entries: Vec<(PathBuf, SystemTime)> = Vec::new();

        for entry in fs::read_dir(&log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("log") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        entries.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        entries.sort_by_key(|e| std::cmp::Reverse(e.1));

        // Remove old files beyond keep_count
        let mut removed = 0;
        for (path, _) in entries.iter().skip(keep_count) {
            if fs::remove_file(path).is_ok() {
                removed += 1;
            }
        }

        Ok(removed)
    }

    // ========================================================================
    // Atomic Operations
    // ========================================================================

    /// Write file atomically (write to temp, then rename)
    fn write_file_atomic(&self, path: &Path, content: &[u8]) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temporary file
        let temp_path = path.with_extension("tmp");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;

        file.write_all(content)
            .with_context(|| format!("Failed to write to temp file: {}", temp_path.display()))?;

        file.sync_all()?;
        drop(file);

        // Atomic rename
        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_byte_structure() {
        let temp = TempDir::new().unwrap();
        let fs = ProjectFileSystem::new(temp.path()).unwrap();

        fs.init_byte_structure().unwrap();

        assert!(fs.byte_dir().join("logs/commands/build").exists());
        assert!(fs.byte_dir().join("logs/commands/lint").exists());
        assert!(fs.byte_dir().join("logs/commands/git").exists());
        assert!(fs.byte_dir().join("state").exists());
    }


    #[test]
    fn test_create_gitignore() {
        let temp = TempDir::new().unwrap();
        let fs = ProjectFileSystem::new(temp.path()).unwrap();

        fs.create_gitignore().unwrap();

        let content = std::fs::read_to_string(temp.path().join(".gitignore")).unwrap();
        assert!(content.contains(".byte/"));
        assert!(content.contains("target/"));
    }
}
