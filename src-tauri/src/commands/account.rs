use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use super::AppStateWrapper;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AddAccountRequest {
    pub name: String,
    pub api_key: String,
}

#[tauri::command]
pub fn list_accounts(state: State<AppStateWrapper>) -> Result<Vec<Account>, String> {
    let app_state = state.0.lock().map_err(|e| e.to_string())?;
    
    // TODO: Implement database operations
    // For now, return empty list
    Ok(vec![])
}

#[tauri::command]
pub fn add_account(
    request: AddAccountRequest,
    state: State<AppStateWrapper>,
) -> Result<Account, String> {
    let _app_state = state.0.lock().map_err(|e| e.to_string())?;
    
    let account = Account {
        id: Uuid::new_v4().to_string(),
        name: request.name,
        api_key: request.api_key,
        is_active: false,
        created_at: Utc::now().to_rfc3339(),
    };
    
    // TODO: Save to database
    
    Ok(account)
}

#[tauri::command]
pub fn delete_account(id: String, state: State<AppStateWrapper>) -> Result<(), String> {
    let _app_state = state.0.lock().map_err(|e| e.to_string())?;
    
    // TODO: Delete from database
    
    Ok(())
}

#[tauri::command]
pub fn switch_account(id: String, state: State<AppStateWrapper>) -> Result<(), String> {
    let _app_state = state.0.lock().map_err(|e| e.to_string())?;
    
    // TODO: Update active account in database
    
    Ok(())
}

#[tauri::command]
pub fn get_current_account(state: State<AppStateWrapper>) -> Result<Option<Account>, String> {
    let _app_state = state.0.lock().map_err(|e| e.to_string())?;
    
    // TODO: Fetch current active account from database
    
    Ok(None)
}
