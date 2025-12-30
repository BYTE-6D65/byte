pub mod types;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub use types::{GlobalConfig, ProjectConfig};

pub struct Config {
    pub global: GlobalConfig,
    pub project: Option<ProjectConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let global = Self::load_global_config()?;
        let project = Self::load_project_config().ok();

        Ok(Self { global, project })
    }

    fn load_global_config() -> Result<GlobalConfig> {
        let config_paths: Vec<Option<PathBuf>> = vec![
            dirs::home_dir().map(|p| p.join(".config/byte/config.toml")),
            Some(PathBuf::from("byte.toml")),
        ];

        for path_opt in config_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    let content = fs::read_to_string(&path)?;
                    let config: GlobalConfig = toml::from_str(&content).map_err(|e| {
                        anyhow::anyhow!("Failed to parse global config TOML: {}", e)
                    })?;
                    return Ok(config);
                }
            }
        }

        Ok(GlobalConfig::default())
    }

    fn load_project_config() -> Result<ProjectConfig> {
        let path = PathBuf::from("byte.toml.example");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: ProjectConfig = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse project config TOML: {}", e))?;
            Ok(config)
        } else {
            Err(anyhow::anyhow!("Project config not found"))
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            project: None,
        }
    }
}
