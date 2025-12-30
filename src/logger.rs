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
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        eprintln!("[LOGGER DEBUG] Current dir: {:?}", cwd);

        let log_dir = if cwd.ends_with("target/debug") {
            cwd.join("../../target/logs")
        } else if cwd.ends_with("byte") {
            cwd.join("target/logs")
        } else {
            cwd.join("byte/target/logs")
        };

        eprintln!("[LOGGER DEBUG] Log dir: {:?}", log_dir);
        let logger = Logger::new(log_dir.to_str().unwrap_or("target/logs"));
        logger.info("Logger initialized");
        logger
    };
}

pub fn info(message: &str) {
    LOGGER.info(message);
}

pub fn error(message: &str) {
    LOGGER.error(message);
}
