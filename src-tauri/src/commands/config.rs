use serde::{Deserialize, Serialize};
use tauri::State;
use std::fs;
use std::path::PathBuf;

use super::AppStateWrapper;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme: String,
    pub language: String,
    pub auto_start: bool,
    pub proxy: ProxyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub port: u16,
    pub auto_start: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "en".to_string(),
            auto_start: false,
            proxy: ProxyConfig {
                enabled: false,
                port: 8080,
                auto_start: false,
            },
        }
    }
}

fn get_config_path() -> Result<PathBuf, String> {
    let app_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("codex-manager");
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    Ok(app_dir.join("config.toml"))
}

#[tauri::command]
pub fn load_config(_state: State<AppStateWrapper>) -> Result<AppConfig, String> {
    let config_path = get_config_path()?;
    
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    
    let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let config: AppConfig = toml::from_str(&content).map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub fn save_config(
    config: AppConfig,
    _state: State<AppStateWrapper>,
) -> Result<(), String> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    
    fs::write(&config_path, content).map_err(|e| e.to_string())?;
    
    Ok(())
}
