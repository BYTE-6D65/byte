/// Simple file-based logger for Byte runtime diagnostics
/// Writes to ~/.byte/logs/byte.log (global) or .byte/logs/byte.log (project-local)
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Log levels
#[derive(Debug, Clone, Copy)]
pub enum Level {
    Info,
    Warn,
    Error,
    Debug,
}

impl Level {
    fn as_str(&self) -> &str {
        match self {
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Debug => "DEBUG",
        }
    }
}

/// Global logger instance
static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// Logger that writes to byte.log
pub struct Logger {
    log_path: PathBuf,
}

impl Logger {
    /// Initialize the global logger
    /// Tries project-local .byte/logs/byte.log first, falls back to ~/.byte/logs/byte.log
    pub fn init() {
        let log_path = Self::determine_log_path();

        // Create log directory
        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let logger = Logger { log_path };

        // Store in global
        if let Ok(mut global_logger) = LOGGER.lock() {
            *global_logger = Some(logger);
        }
    }

    /// Determine where to write logs
    fn determine_log_path() -> PathBuf {
        // Try current directory's .byte/logs/
        let local_log = PathBuf::from(".byte/logs/byte.log");
        if local_log.parent().map(|p| p.exists()).unwrap_or(false) {
            return local_log;
        }

        // Fall back to ~/.byte/logs/
        if let Some(home) = dirs::home_dir() {
            let global_log = home.join(".byte/logs/byte.log");
            if let Some(parent) = global_log.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return global_log;
        }

        // Last resort: /tmp
        PathBuf::from("/tmp/byte.log")
    }

    /// Write a log entry
    fn write(&self, level: Level, category: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!(
            "[{}] {} [{}] {}\n",
            timestamp,
            level.as_str(),
            category,
            message
        );

        // Append to log file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}

/// Log an info message
pub fn info(category: &str, message: &str) {
    if let Ok(logger_guard) = LOGGER.lock() {
        if let Some(logger) = logger_guard.as_ref() {
            logger.write(Level::Info, category, message);
        }
    }
}

/// Log a warning message
#[allow(dead_code)]
pub fn warn(category: &str, message: &str) {
    if let Ok(logger_guard) = LOGGER.lock() {
        if let Some(logger) = logger_guard.as_ref() {
            logger.write(Level::Warn, category, message);
        }
    }
}

/// Log an error message
pub fn error(category: &str, message: &str) {
    if let Ok(logger_guard) = LOGGER.lock() {
        if let Some(logger) = logger_guard.as_ref() {
            logger.write(Level::Error, category, message);
        }
    }
}

/// Log a debug message
#[allow(dead_code)]
pub fn debug(category: &str, message: &str) {
    if let Ok(logger_guard) = LOGGER.lock() {
        if let Some(logger) = logger_guard.as_ref() {
            logger.write(Level::Debug, category, message);
        }
    }
}

/// Convenience macro for logging
#[macro_export]
macro_rules! log_info {
    ($category:expr, $($arg:tt)*) => {
        $crate::log::info($category, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($category:expr, $($arg:tt)*) => {
        $crate::log::error($category, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($category:expr, $($arg:tt)*) => {
        $crate::log::warn($category, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($category:expr, $($arg:tt)*) => {
        $crate::log::debug($category, &format!($($arg)*))
    };
}
