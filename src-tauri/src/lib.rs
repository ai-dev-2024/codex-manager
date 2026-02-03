// Codex Manager - Tauri v2 Backend
// Version 0.3.0

pub mod config;
pub mod models;
pub mod proxy;
pub mod routing;
pub mod storage;
pub mod usage;

use std::sync::Arc;
use std::collections::HashMap;
use tauri::Manager;
use tracing::{info, error};
use uuid::Uuid;

use crate::models::{
    Account, AccountExport, AccountId, AccountStatus, AppConfig, CreateAccountRequest,
    ProxyServerConfig, ProxyStatus, RoutingConfig, RoutingStats, RoutingStrategy, UpdateAccountRequest,
    UsageSnapshot, ValidationResult,
};
use crate::routing::RoutingEngine;
use crate::storage::EncryptedStore;
use crate::usage::{OpenAIClient, UsagePoller};

/// Application state shared across Tauri commands
pub struct AppState {
    pub store: Arc<EncryptedStore>,
    pub routing_engine: Arc<RoutingEngine>,
    pub usage_poller: Arc<UsagePoller>,
}

// Re-export commands from modules
pub use config::{
    load_app_config, save_app_config, update_proxy_configuration, 
    update_routing_configuration, set_master_key, get_data_directory,
};
pub use proxy::{
    start_proxy_server, stop_proxy_server, get_proxy_status,
};
pub use usage::{
    fetch_account_usage, validate_api_key as validate_api_key_usage,
};

// ============================================================================
// Account Management Commands
// ============================================================================

/// Add a new OpenAI account
#[tauri::command]
pub async fn add_account(
    state: tauri::State<'_, AppState>,
    request: CreateAccountRequest,
) -> Result<Account, String> {
    let mut account = Account::new(request.label, request.api_key);
    
    if let Some(org_id) = request.org_id {
        account = account.with_org_id(org_id);
    }
    if let Some(models) = request.model_scope {
        account = account.with_model_scope(models);
    }
    if let Some(daily) = request.daily_limit {
        account = account.with_limits(Some(daily), request.monthly_limit);
    }
    if let Some(priority) = request.priority {
        account = account.with_priority(priority);
    }

    state.store.save_account(&account)
        .map_err(|e| format!("Failed to save account: {}", e))?;

    // Refresh routing engine
    refresh_routing_engine(&state).await?;

    info!("Added account: {} ({})", account.label, account.id);
    Ok(account)
}

/// Update an existing account
#[tauri::command]
pub async fn update_account(
    state: tauri::State<'_, AppState>,
    request: UpdateAccountRequest,
) -> Result<Account, String> {
    let mut account = state.store.load_account(request.id)
        .map_err(|e| format!("Failed to load account: {}", e))?
        .ok_or_else(|| "Account not found".to_string())?;

    if let Some(label) = request.label {
        account.label = label;
    }
    if let Some(org_id) = request.org_id {
        account.org_id = Some(org_id);
    }
    if let Some(models) = request.model_scope {
        account.model_scope = models;
    }
    if let Some(daily) = request.daily_limit {
        account.daily_limit = Some(daily);
    }
    if let Some(monthly) = request.monthly_limit {
        account.monthly_limit = Some(monthly);
    }
    if let Some(priority) = request.priority {
        account.priority = priority;
    }
    if let Some(enabled) = request.enabled {
        account.enabled = enabled;
    }

    account.updated_at = chrono::Utc::now();

    state.store.save_account(&account)
        .map_err(|e| format!("Failed to save account: {}", e))?;

    refresh_routing_engine(&state).await?;

    Ok(account)
}

/// Remove an account by ID
#[tauri::command]
pub async fn remove_account(
    state: tauri::State<'_, AppState>,
    id: AccountId,
) -> Result<bool, String> {
    let deleted = state.store.delete_account(id)
        .map_err(|e| format!("Failed to delete account: {}", e))?;

    if deleted {
        refresh_routing_engine(&state).await?;
        info!("Removed account: {}", id);
    }

    Ok(deleted)
}

/// Get a single account by ID
#[tauri::command]
pub async fn get_account(
    state: tauri::State<'_, AppState>,
    id: AccountId,
) -> Result<Option<Account>, String> {
    state.store.load_account(id)
        .map_err(|e| e.to_string())
}

/// List all accounts
#[tauri::command]
pub async fn list_accounts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Account>, String> {
    state.store.load_accounts()
        .map_err(|e| e.to_string())
}

/// List all accounts with their current status
#[tauri::command]
pub async fn list_account_statuses(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AccountStatus>, String> {
    state.routing_engine.get_account_statuses().await
}

/// Toggle account enabled state
#[tauri::command]
pub async fn toggle_account_enabled(
    state: tauri::State<'_, AppState>,
    id: AccountId,
) -> Result<Account, String> {
    let mut account = state.store.load_account(id)
        .map_err(|e| format!("Failed to load account: {}", e))?
        .ok_or_else(|| "Account not found".to_string())?;

    account.enabled = !account.enabled;
    account.updated_at = chrono::Utc::now();

    state.store.save_account(&account)
        .map_err(|e| format!("Failed to save account: {}", e))?;

    refresh_routing_engine(&state).await?;

    Ok(account)
}

// ============================================================================
// Usage Data Commands
// ============================================================================

/// Get usage snapshot for a specific account
#[tauri::command]
pub async fn get_account_usage(
    state: tauri::State<'_, AppState>,
    id: AccountId,
) -> Result<Option<UsageSnapshot>, String> {
    state.store.load_latest_usage(id)
        .map_err(|e| e.to_string())
}

/// Refresh usage data for all accounts
#[tauri::command]
pub async fn refresh_all_usage(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(AccountId, Result<UsageSnapshot, String>)>, String> {
    let accounts = state.store.load_accounts()
        .map_err(|e| e.to_string())?;

    let client = OpenAIClient::new();
    let mut results = Vec::new();

    for account in accounts {
        let result = match client.fetch_usage(&account).await {
            Ok(usage) => {
                if let Err(e) = state.store.save_usage_snapshot(&usage) {
                    Err(format!("Failed to save usage: {}", e))
                } else {
                    Ok(usage)
                }
            }
            Err(e) => Err(e.to_string()),
        };
        results.push((account.id, result));
    }

    // Refresh routing engine with new usage data
    refresh_routing_engine(&state).await?;

    Ok(results)
}

/// Refresh usage for a single account
#[tauri::command]
pub async fn refresh_account_usage(
    state: tauri::State<'_, AppState>,
    id: AccountId,
) -> Result<UsageSnapshot, String> {
    let account = state.store.load_account(id)
        .map_err(|e| format!("Failed to load account: {}", e))?
        .ok_or_else(|| "Account not found".to_string())?;

    let client = OpenAIClient::new();
    let usage = client.fetch_usage(&account).await
        .map_err(|e| e.to_string())?;

    state.store.save_usage_snapshot(&usage)
        .map_err(|e| format!("Failed to save usage: {}", e))?;

    refresh_routing_engine(&state).await?;

    Ok(usage)
}

// ============================================================================
// Routing Commands
// ============================================================================

/// Get current routing statistics
#[tauri::command]
pub async fn get_routing_stats(
    state: tauri::State<'_, AppState>,
) -> Result<RoutingStats, String> {
    state.routing_engine.get_stats().await
}

/// Set the routing strategy
#[tauri::command]
pub async fn set_routing_strategy(
    _state: tauri::State<'_, AppState>,
    strategy: RoutingStrategy,
) -> Result<(), String> {
    // Note: This requires mutable access, which we don't have with Arc
    // In a real implementation, we'd use interior mutability
    // For now, this is a placeholder
    info!("Setting routing strategy to {:?}", strategy);
    Ok(())
}

/// Clear all session mappings
#[tauri::command]
pub async fn clear_routing_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.routing_engine.clear_sessions();
    Ok(())
}

// ============================================================================
// Import/Export Commands
// ============================================================================

/// Export all accounts to a JSON structure
#[tauri::command]
pub async fn export_accounts(
    state: tauri::State<'_, AppState>,
) -> Result<AccountExport, String> {
    let accounts = state.store.load_accounts()
        .map_err(|e| e.to_string())?;

    Ok(AccountExport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now(),
        accounts,
    })
}

/// Import accounts from a JSON structure
#[tauri::command]
pub async fn import_accounts(
    state: tauri::State<'_, AppState>,
    export: AccountExport,
) -> Result<Vec<Account>, String> {
    let mut imported = Vec::new();

    for mut account in export.accounts {
        // Generate new ID to avoid conflicts
        account.id = Uuid::new_v4();
        account.created_at = chrono::Utc::now();
        account.updated_at = chrono::Utc::now();
        account.last_used = None;

        state.store.save_account(&account)
            .map_err(|e| format!("Failed to import account {}: {}", account.label, e))?;
        
        imported.push(account);
    }

    refresh_routing_engine(&state).await?;

    info!("Imported {} accounts", imported.len());
    Ok(imported)
}

/// Validate an API key without saving
#[tauri::command]
pub async fn validate_api_key(
    api_key: String,
    org_id: Option<String>,
) -> Result<ValidationResult, String> {
    let client = OpenAIClient::new();
    client.validate_key(&api_key, org_id.as_deref()).await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Refresh the routing engine with current accounts and usage
async fn refresh_routing_engine(state: &AppState) -> Result<(), String> {
    let accounts = state.store.load_accounts()
        .map_err(|e| e.to_string())?;

    let mut usage_map = HashMap::new();
    for account in &accounts {
        if let Ok(Some(usage)) = state.store.load_latest_usage(account.id) {
            usage_map.insert(account.id, usage);
        }
    }

    state.routing_engine.update_accounts(accounts, usage_map).await;
    Ok(())
}

// ============================================================================
// Tauri Plugin Setup
// ============================================================================

/// Initialize the Tauri application
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main")
                .map(|window| {
                    let _ = window.show();
                    let _ = window.set_focus();
                    #[cfg(target_os = "macos")]
                    app.set_activation_policy(tauri::ActivationPolicy::Regular).unwrap_or(());
                });
        }))
        .setup(|app| {
            // Initialize tracing
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .init();

            // Get paths
            let db_path = config::get_db_path(app.handle())?;
            let master_key = config::get_master_key(app.handle())?;

            // Initialize encrypted store
            let store = Arc::new(EncryptedStore::open(&db_path, &master_key)
                .map_err(|e| format!("Failed to open database: {}", e))?);

            // Load accounts and usage
            let accounts = store.load_accounts()
                .map_err(|e| format!("Failed to load accounts: {}", e))?;

            let mut usage_map = HashMap::new();
            for account in &accounts {
                if let Ok(Some(usage)) = store.load_latest_usage(account.id) {
                    usage_map.insert(account.id, usage);
                }
            }

            // Initialize routing engine
            let config = config::load_config(app.handle())?;
            let routing_engine = Arc::new(RoutingEngine::new(config.routing.strategy));
            
            // Use tokio runtime to update accounts
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                routing_engine.update_accounts(accounts, usage_map).await;
            });

            // Create usage poller
            let usage_poller = Arc::new(UsagePoller::new());

            // Create app state
            let app_state = AppState {
                store,
                routing_engine,
                usage_poller,
            };

            app.manage(app_state);

            info!("Codex Manager v{} initialized", env!("CARGO_PKG_VERSION"));

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                #[cfg(target_os = "macos")]
                {
                    use tauri::Manager;
                    window.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory).unwrap_or(());
                }
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Account management
            add_account,
            update_account,
            remove_account,
            get_account,
            list_accounts,
            list_account_statuses,
            toggle_account_enabled,
            
            // Usage data
            get_account_usage,
            refresh_all_usage,
            refresh_account_usage,
            
            // Routing
            get_routing_stats,
            set_routing_strategy,
            clear_routing_sessions,
            
            // Proxy server
            start_proxy_server,
            stop_proxy_server,
            get_proxy_status,
            
            // Import/Export
            export_accounts,
            import_accounts,
            validate_api_key,
            
            // Configuration
            load_app_config,
            save_app_config,
            update_proxy_configuration,
            update_routing_configuration,
            set_master_key,
            get_data_directory,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Handle macOS dock icon click to reopen window
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    app_handle.set_activation_policy(tauri::ActivationPolicy::Regular).unwrap_or(());
                }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app_handle, event);
        });
}
