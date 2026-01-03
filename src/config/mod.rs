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
        // Create SafePath and validate it's a directory
        let safe_path = crate::path::SafePath::from_user_input(path)?;
        safe_path.validate_directory()?;

        // Use canonical path for comparison (handles case-insensitivity and symlinks)
        let canonical = safe_path.canonical().ok_or_else(|| {
            anyhow::anyhow!("Failed to canonicalize path: {}", safe_path)
        })?;

        // Check if already registered (compare canonical paths)
        let primary_safe = crate::path::SafePath::from_user_input(&self.global.workspace.path)?;
        if let Some(primary_canonical) = primary_safe.canonical() {
            if canonical == primary_canonical {
                anyhow::bail!("Path is already the primary workspace");
            }
        }

        for registered in &self.global.workspace.registered {
            if let Ok(registered_safe) = crate::path::SafePath::from_user_input(registered) {
                if let Some(registered_canonical) = registered_safe.canonical() {
                    if canonical == registered_canonical {
                        anyhow::bail!("Path is already registered");
                    }
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
        let safe_path = crate::path::SafePath::from_user_input(path)?;

        // Find and remove the path (comparing SafePath equality)
        let original_len = self.global.workspace.registered.len();
        self.global.workspace.registered.retain(|p| {
            if let Ok(p_safe) = crate::path::SafePath::from_user_input(p) {
                !safe_path.equals(&p_safe)
            } else {
                true // Keep paths that fail to parse
            }
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
