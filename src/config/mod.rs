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

    /// Save global config to ~/.config/byte/config.toml
    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".config/byte");

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(&self.global)?;
        fs::write(&config_path, toml_string)?;

        Ok(())
    }

    /// Add a workspace path to the registered list
    pub fn add_workspace_path(&mut self, path: &str) -> Result<()> {
        // Expand tilde
        let expanded = shellexpand::tilde(path).to_string();

        // Validate path exists
        let path_buf = PathBuf::from(&expanded);
        if !path_buf.exists() {
            anyhow::bail!("Path does not exist: {}", expanded);
        }

        if !path_buf.is_dir() {
            anyhow::bail!("Path is not a directory: {}", expanded);
        }

        // Check if already registered (compare expanded paths)
        let primary_expanded = shellexpand::tilde(&self.global.workspace.path).to_string();
        if expanded == primary_expanded {
            anyhow::bail!("Path is already the primary workspace");
        }

        for registered in &self.global.workspace.registered {
            let registered_expanded = shellexpand::tilde(registered).to_string();
            if expanded == registered_expanded {
                anyhow::bail!("Path is already registered");
            }
        }

        // Add to registered list (store with tilde for portability)
        self.global.workspace.registered.push(path.to_string());

        // Save to file
        self.save()?;

        Ok(())
    }

    /// Remove a workspace path from the registered list
    pub fn remove_workspace_path(&mut self, path: &str) -> Result<()> {
        let expanded = shellexpand::tilde(path).to_string();

        // Find and remove the path (comparing expanded versions)
        let original_len = self.global.workspace.registered.len();
        self.global.workspace.registered.retain(|p| {
            let p_expanded = shellexpand::tilde(p).to_string();
            p_expanded != expanded
        });

        if self.global.workspace.registered.len() == original_len {
            anyhow::bail!("Path not found in registered workspaces: {}", path);
        }

        // Save to file
        self.save()?;

        Ok(())
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
