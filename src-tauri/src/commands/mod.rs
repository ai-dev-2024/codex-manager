use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

pub mod account;
pub mod config;

pub use account::*;
pub use config::*;

// Shared application state
pub struct AppState {
    pub db_path: String,
}

impl AppState {
    pub fn new() -> Self {
        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("codex-manager");
        
        std::fs::create_dir_all(&app_dir).ok();
        
        Self {
            db_path: app_dir.join("codex.db").to_string_lossy().to_string(),
        }
    }
}

pub struct AppStateWrapper(pub Mutex<AppState>);
