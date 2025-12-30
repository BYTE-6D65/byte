use serde::{Deserialize, Serialize};

/// Global Byte configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub workspace: WorkspaceConfig,
    pub drivers: DriversConfig,
    pub tui: TuiConfig,
    pub explain: ExplainConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub path: String,
    pub auto_scan: bool,
    pub registered: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriversConfig {
    pub search_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TuiConfig {
    pub refresh_rate_ms: u64,
    pub animations: bool,
    pub default_view: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExplainConfig {
    pub dry_run_by_default: bool,
    pub show_command_traces: bool,
    pub show_file_preview: bool,
}

/// Project configuration (byte.toml in project directory)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub ecosystem: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig {
                path: "~/projects".to_string(),
                auto_scan: true,
                registered: vec![],
            },
            drivers: DriversConfig {
                search_paths: vec![
                    "~/.config/byte/drivers".to_string(),
                    "~/.local/share/byte/drivers".to_string(),
                ],
            },
            tui: TuiConfig {
                refresh_rate_ms: 16,
                animations: true,
                default_view: "browser".to_string(),
            },
            explain: ExplainConfig {
                dry_run_by_default: false,
                show_command_traces: true,
                show_file_preview: true,
            },
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: ProjectMeta::default(),
        }
    }
}

impl Default for ProjectMeta {
    fn default() -> Self {
        Self {
            name: "my-project".to_string(),
            project_type: "cli".to_string(),
            ecosystem: "rust".to_string(),
            description: None,
        }
    }
}
