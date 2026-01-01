pub mod types;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub use types::{GlobalConfig, ProjectConfig};

pub struct Config {
    pub global: GlobalConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let global = Self::load_global_config()?;

        Ok(Self { global })
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
        // Expand tilde and normalize (remove trailing slashes)
        let expanded = shellexpand::tilde(path).to_string();
        let normalized = expanded.trim_end_matches('/').to_string();

        // Validate path exists
        let path_buf = PathBuf::from(&normalized);
        if !path_buf.exists() {
            anyhow::bail!("Path does not exist: {}", normalized);
        }

        if !path_buf.is_dir() {
            anyhow::bail!("Path is not a directory: {}", normalized);
        }

        // Use canonical path for comparison (handles case-insensitivity and symlinks)
        let canonical = path_buf
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("Failed to canonicalize path: {}", e))?;

        // Check if already registered (compare canonical paths)
        let primary_expanded = shellexpand::tilde(&self.global.workspace.path).to_string();
        let primary_path = PathBuf::from(primary_expanded.trim_end_matches('/'));
        if let Ok(primary_canonical) = primary_path.canonicalize() {
            if canonical == primary_canonical {
                anyhow::bail!("Path is already the primary workspace");
            }
        }

        for registered in &self.global.workspace.registered {
            let registered_expanded = shellexpand::tilde(registered).to_string();
            let registered_path = PathBuf::from(registered_expanded.trim_end_matches('/'));
            if let Ok(registered_canonical) = registered_path.canonicalize() {
                if canonical == registered_canonical {
                    anyhow::bail!("Path is already registered");
                }
            }
        }

        // Normalize the original path (remove trailing slash but keep tilde)
        let normalized_input = path.trim_end_matches('/').to_string();

        // Add to registered list (store normalized with tilde for portability)
        self.global.workspace.registered.push(normalized_input);

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
        }
    }
}
