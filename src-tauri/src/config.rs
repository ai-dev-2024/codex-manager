use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tracing::info;

use crate::models::{AppConfig, ProxyServerConfig, RoutingConfig, RoutingStrategy};

/// Get the application data directory
pub fn get_data_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let path = app_handle.path().app_data_dir()
        .context("Failed to get app data directory")?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Get the configuration file path
pub fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let data_dir = get_data_dir(app_handle)?;
    Ok(data_dir.join("config.json"))
}

/// Get the database file path
pub fn get_db_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    let data_dir = get_data_dir(app_handle)?;
    Ok(data_dir.join("accounts.db"))
}

/// Load configuration from file or create default
pub fn load_config(app_handle: &tauri::AppHandle) -> Result<AppConfig> {
    let config_path = get_config_path(app_handle)?;

    if config_path.exists() {
        info!("Loading configuration from {:?}", config_path);
        let content = fs::read_to_string(&config_path)?;
        let config: AppConfig = serde_json::from_str(&content)
            .context("Failed to parse configuration file")?;
        Ok(config)
    } else {
        info!("Configuration not found, creating default");
        let config = AppConfig::default();
        save_config(app_handle, &config)?;
        Ok(config)
    }
}

/// Save configuration to file
pub fn save_config(app_handle: &tauri::AppHandle, config: &AppConfig) -> Result<()> {
    let config_path = get_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize configuration")?;
    fs::write(config_path, content)?;
    Ok(())
}

/// Update proxy configuration
pub fn update_proxy_config(
    app_handle: &tauri::AppHandle,
    proxy_config: ProxyServerConfig,
) -> Result<()> {
    let mut config = load_config(app_handle)?;
    config.proxy = proxy_config;
    save_config(app_handle, &config)
}

/// Update routing configuration
pub fn update_routing_config(
    app_handle: &tauri::AppHandle,
    routing_config: RoutingConfig,
) -> Result<()> {
    let mut config = load_config(app_handle)?;
    config.routing = routing_config;
    save_config(app_handle, &config)
}

/// Get the master key from secure storage or environment
pub fn get_master_key(app_handle: &tauri::AppHandle) -> Result<String> {
    // Try to get from secure storage first
    if let Ok(store) = app_handle.store("codex-manager.secrets") {
        if let Some(key) = store.get("master_key") {
            if let Some(key_str) = key.as_str() {
                return Ok(key_str.to_string());
            }
        }
    }

    // Fall back to environment variable
    if let Ok(key) = std::env::var("CODEX_MANAGER_MASTER_KEY") {
        return Ok(key);
    }

    // Generate a default key (for development only)
    Ok("codex-manager-default-key".to_string())
}

/// Save master key to secure storage
pub fn save_master_key(app_handle: &tauri::AppHandle, key: &str) -> Result<()> {
    let store = app_handle.store("codex-manager.secrets")
        .context("Failed to open secure store")?;
    store.set("master_key", key);
    store.save()?;
    Ok(())
}

/// Tauri command: Load configuration
#[tauri::command]
pub async fn load_app_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    load_config(&app_handle).map_err(|e| e.to_string())
}

/// Tauri command: Save configuration
#[tauri::command]
pub async fn save_app_config(
    app_handle: tauri::AppHandle,
    config: AppConfig,
) -> Result<(), String> {
    save_config(&app_handle, &config).map_err(|e| e.to_string())
}

/// Tauri command: Update proxy configuration
#[tauri::command]
pub async fn update_proxy_configuration(
    app_handle: tauri::AppHandle,
    proxy: ProxyServerConfig,
) -> Result<(), String> {
    update_proxy_config(&app_handle, proxy).map_err(|e| e.to_string())
}

/// Tauri command: Update routing configuration
#[tauri::command]
pub async fn update_routing_configuration(
    app_handle: tauri::AppHandle,
    routing: RoutingConfig,
) -> Result<(), String> {
    update_routing_config(&app_handle, routing).map_err(|e| e.to_string())
}

/// Tauri command: Set master key
#[tauri::command]
pub async fn set_master_key(
    app_handle: tauri::AppHandle,
    key: String,
) -> Result<(), String> {
    save_master_key(&app_handle, &key).map_err(|e| e.to_string())
}

/// Tauri command: Get data directory path
#[tauri::command]
pub async fn get_data_directory(app_handle: tauri::AppHandle) -> Result<String, String> {
    let path = get_data_dir(&app_handle).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
