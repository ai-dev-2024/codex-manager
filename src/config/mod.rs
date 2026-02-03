use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tracing::info;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Proxy server configuration
    pub proxy: ProxyConfig,
    /// Routing configuration
    pub routing: RoutingConfig,
    /// Polling configuration
    pub polling: PollingConfig,
    /// UI configuration
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub bind_addr: SocketAddr,
    pub api_key: String,
    pub openai_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub strategy: String,
    pub min_request_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub backoff_multiplier: f64,
    pub max_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub refresh_rate_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                bind_addr: "127.0.0.1:8080".parse().unwrap(),
                api_key: "sk-codex-account-manager".to_string(),
                openai_base_url: "https://api.openai.com".to_string(),
            },
            routing: RoutingConfig {
                strategy: "least_utilized".to_string(),
                min_request_interval_ms: 100,
            },
            polling: PollingConfig {
                enabled: true,
                interval_seconds: 300,
                backoff_multiplier: 2.0,
                max_interval_seconds: 3600,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                refresh_rate_ms: 1000,
            },
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load() -> Result<(Self, PathBuf)> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            info!("Loading configuration from {:?}", config_path);
            let content = fs::read_to_string(&config_path)?;
            let config: Config =
                toml::from_str(&content).context("Failed to parse configuration file")?;
            Ok((config, config_path))
        } else {
            info!(
                "Configuration not found, creating default at {:?}",
                config_path
            );
            let config = Config::default();
            config.save(&config_path)?;
            Ok((config, config_path))
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "codex", "account-manager")
            .context("Failed to determine config directory")?;

        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Get database path
    pub fn db_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "codex", "account-manager")
            .context("Failed to determine data directory")?;

        Ok(proj_dirs.data_dir().join("accounts.db"))
    }

    /// Get log directory
    pub fn log_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "codex", "account-manager")
            .context("Failed to determine log directory")?;

        Ok(proj_dirs.data_dir().join("logs"))
    }
}

/// Get data directory for the application
pub fn data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "codex", "account-manager")
        .context("Failed to determine data directory")?;

    fs::create_dir_all(proj_dirs.data_dir())?;
    Ok(proj_dirs.data_dir().to_path_buf())
}
