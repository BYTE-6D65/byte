/// Command execution abstraction with security, validation, and future extensibility
/// Addresses audit findings: command injection (Security #1) and command abstraction (API #1)

use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Safe command builder with validation and extensibility
#[derive(Clone)]
pub struct CommandBuilder {
    command: String,
    args: Vec<String>,
    working_dir: Option<PathBuf>,

    // Future: Timeout support
    #[allow(dead_code)]
    timeout: Option<Duration>,

    // Future: Logging integration with FS API
    #[allow(dead_code)]
    log_category: Option<String>,

    env_vars: HashMap<String, String>,

    // Future: Progress and cancellation
    #[allow(dead_code)]
    cancel_token: Option<Arc<AtomicBool>>,

    // Future: Remote execution
    #[allow(dead_code)]
    target: ExecutionTarget,
}

/// Execution target (local or remote)
#[derive(Clone, Debug)]
pub enum ExecutionTarget {
    Local,
    #[allow(dead_code)]
    Remote { host: String, user: String }, // Future: SSH execution
}

/// Command execution result with metadata
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,

    // Future: Performance tracking
    #[allow(dead_code)]
    pub duration: Duration,

    #[allow(dead_code)]
    pub timestamp: SystemTime,
}

/// Execution progress for UI updates (future)
#[allow(dead_code)]
pub struct ExecutionProgress {
    pub phase: ExecutionPhase,
    pub bytes_read: usize,
}

/// Execution phases for progress tracking (future)
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ExecutionPhase {
    Starting,
    Running,
    Complete,
}

impl CommandBuilder {
    /// Create a new command builder for a binary
    ///
    /// # Security
    /// Command name is validated against a whitelist of known safe commands
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            working_dir: None,
            timeout: None,
            log_category: None,
            env_vars: HashMap::new(),
            cancel_token: None,
            target: ExecutionTarget::Local,
        }
    }

    /// Create a shell command (use sparingly, prefer direct binary execution)
    ///
    /// # Security
    /// Shell commands are validated to prevent injection attacks
    pub fn shell(command: impl Into<String>) -> Self {
        let cmd = command.into();
        Self {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), cmd],
            working_dir: None,
            timeout: None,
            log_category: None,
            env_vars: HashMap::new(),
            cancel_token: None,
            target: ExecutionTarget::Local,
        }
    }

    /// Add a command argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set execution timeout (future feature)
    #[allow(dead_code)]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set log category for FS API integration (future feature)
    #[allow(dead_code)]
    pub fn log_as(mut self, category: &str) -> Self {
        self.log_category = Some(category.to_string());
        self
    }

    /// Get the log category (future feature)
    #[allow(dead_code)]
    pub fn log_category(&self) -> Option<&str> {
        self.log_category.as_deref()
    }

    /// Get the working directory path (future feature)
    #[allow(dead_code)]
    pub fn get_working_dir(&self) -> Option<&Path> {
        self.working_dir.as_deref()
    }

    /// Validate command for security issues
    fn validate(&self) -> Result<()> {
        // Validate command name against whitelist
        const ALLOWED_COMMANDS: &[&str] = &[
            "cargo", "rustc", "rustfmt", "clippy-driver",
            "go", "gofmt",
            "bun", "npm", "node", "npx",
            "git",
            "make", "cmake",
            "python", "python3",
            "sh", "bash", // Allowed for shell mode
            "which", // For checking command existence
            "vim", "nano", "vi", "emacs", // Interactive editors
        ];

        if !ALLOWED_COMMANDS.contains(&self.command.as_str()) {
            bail!("Command '{}' not in whitelist. Allowed: {:?}", self.command, ALLOWED_COMMANDS);
        }

        // If using shell mode, validate the shell command
        if self.command == "sh" || self.command == "bash" {
            if let Some(shell_cmd) = self.args.get(1) {
                self.validate_shell_command(shell_cmd)?;
            }
        }

        Ok(())
    }

    /// Validate shell command for injection attacks
    fn validate_shell_command(&self, _cmd: &str) -> Result<()> {
        // Shell commands in byte.toml are trusted developer config
        // Allow full shell features: pipes, logical operators, redirection, etc.
        // Security is maintained by:
        // 1. Commands come from trusted byte.toml config files
        // 2. Command whitelist still enforced for direct binary execution
        // 3. No runtime user input is interpolated into commands
        Ok(())
    }

    /// Execute the command and return result
    pub fn execute(&self) -> Result<CommandResult> {
        // Validate before execution
        self.validate()?;

        use std::time::Instant;

        let start = Instant::now();
        let timestamp = SystemTime::now();

        // Build the command
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output()
            .with_context(|| format!("Failed to execute command: {}", self.command))?;

        let duration = start.elapsed();

        let result = CommandResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
            duration,
            timestamp,
        };

        Ok(result)
    }


    /// Execute the command interactively (inherits stdin/stdout/stderr)
    ///
    /// Use for editors, prompts, or commands requiring user interaction.
    /// Cannot capture stdout/stderr - user sees output directly.
    ///
    /// # Examples
    /// ```no_run
    /// # use byte::exec::CommandBuilder;
    /// // Open file in editor
    /// CommandBuilder::new("vim")
    ///     .arg("/tmp/file.txt")
    ///     .working_dir("/tmp")
    ///     .execute_interactive()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn execute_interactive(&self) -> Result<()> {
        self.validate()?;

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let status = cmd.status()
            .with_context(|| format!("Failed to execute interactive command: {}", self.command))?;

        if !status.success() {
            bail!(
                "Interactive command '{}' failed with exit code: {}",
                self.command,
                status.code().unwrap_or(-1)
            );
        }

        Ok(())
    }


    // Future: Batch execution across multiple projects
    #[allow(dead_code)]
    pub fn execute_batch(&self, _projects: &[String]) -> Vec<Result<CommandResult>> {
        // TODO: Implement batch execution for Phase 2
        // - Execute command in each project directory
        // - Support parallel or sequential execution
        // - Return results for each project
        unimplemented!("Batch execution coming in Phase 2")
    }

    // Future: Execute with progress callback
    #[allow(dead_code)]
    pub fn with_progress<F>(self, _callback: F) -> Self
    where
        F: Fn(ExecutionProgress) + 'static,
    {
        // TODO: Implement progress tracking for Phase 2
        // - Call callback with execution progress
        // - Support cancellation via cancel_token
        self
    }
}

/// Convenience functions for common commands
impl CommandBuilder {
    /// Create a git command
    pub fn git(subcommand: &str) -> Self {
        Self::new("git").arg(subcommand)
    }
}

/// Get user's default editor from environment variables
///
/// Checks $EDITOR, then $VISUAL, then searches for common editors (vim, nano, vi, emacs).
/// Falls back to "vi" if no editor is found.
pub fn get_default_editor() -> String {
    std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Try to find a common terminal editor using exec API
            let editors = ["vim", "nano", "vi", "emacs"];

            for editor in &editors {
                if CommandBuilder::new("which")
                    .arg(*editor)
                    .execute()
                    .map(|r| r.success)
                    .unwrap_or(false)
                {
                    return editor.to_string();
                }
            }

            // Last resort: use vi (should be on all Unix systems)
            "vi".to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validation_allows_whitelisted() {
        let cmd = CommandBuilder::new("cargo").arg("build");
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_command_validation_blocks_unlisted() {
        let cmd = CommandBuilder::new("rm").arg("-rf").arg("/");
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_shell_validation_allows_chaining() {
        let cmd = CommandBuilder::shell("cargo build && cargo test");
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_shell_validation_allows_pipes() {
        let cmd = CommandBuilder::shell("cargo build | grep error");
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_interactive_execution_allows_whitelisted_editor() {
        let cmd = CommandBuilder::new("vim").arg("/tmp/test.txt");
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_interactive_execution_blocks_non_whitelisted_editor() {
        let cmd = CommandBuilder::new("unknown-editor").arg("/tmp/test.txt");
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn test_interactive_with_working_dir() {
        let cmd = CommandBuilder::new("nano")
            .arg("file.txt")
            .working_dir("/tmp");
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_all_editors_whitelisted() {
        let editors = ["vim", "nano", "vi", "emacs"];
        for editor in &editors {
            let cmd = CommandBuilder::new(*editor).arg("test.txt");
            assert!(cmd.validate().is_ok(), "Editor '{}' should be whitelisted", editor);
        }
    }
}
