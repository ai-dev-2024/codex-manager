use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

mod config;
mod models;
mod proxy;
mod routing;
mod storage;
mod ui;
mod usage;

use config::Config;
use routing::{RoutingEngine, RoutingStrategy};
use storage::EncryptedStore;

/// Codex Account Manager - Multi-account OpenAI API management tool
#[derive(Parser)]
#[command(name = "codex-account-manager")]
#[command(about = "Manage multiple OpenAI API accounts with intelligent routing")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Master password for database encryption
    #[arg(short, long, env = "CAM_MASTER_KEY")]
    master_key: Option<String>,

    /// Run in proxy-only mode (no TUI)
    #[arg(short, long)]
    proxy_only: bool,

    /// Proxy bind address
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,

    /// API key for proxy authentication
    #[arg(long, default_value = "sk-codex-account-manager")]
    api_key: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the proxy server
    Proxy {
        /// Bind address
        #[arg(short, long)]
        bind: Option<SocketAddr>,
    },
    /// Add a new account
    Add {
        /// Account label
        label: String,
        /// API key
        api_key: String,
        /// Organization ID (optional)
        #[arg(short, long)]
        org_id: Option<String>,
    },
    /// List all accounts
    List,
    /// Remove an account
    Remove {
        /// Account ID or label
        identifier: String,
    },
    /// Show account details
    Show {
        /// Account ID or label
        identifier: String,
    },
    /// Refresh usage data for all accounts
    Refresh,
    /// Configure settings
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// Run interactive TUI
    Tui,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set {
        /// Key (e.g., proxy.bind_addr)
        key: String,
        /// Value
        value: String,
    },
    /// Reset to defaults
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Load or create configuration
    let (config, config_path) = Config::load()?;

    // Get master key
    let master_key = cli.master_key.unwrap_or_else(|| {
        // Try to get from environment or prompt
        std::env::var("CAM_MASTER_KEY").unwrap_or_else(|_| {
            // In a real implementation, we'd use dialoguer here
            // For now, use a default (NOT FOR PRODUCTION)
            warn!("Using default master key - set CAM_MASTER_KEY environment variable!");
            "codex-account-manager-default-key".to_string()
        })
    });

    // Initialize encrypted store
    let db_path = Config::db_path()?;
    std::fs::create_dir_all(db_path.parent().unwrap())?;

    let store = EncryptedStore::open(&db_path, &master_key)
        .context("Failed to open encrypted database. Check your master key.")?;

    // Execute command or start TUI
    match cli.command {
        Some(Commands::Proxy { bind }) => {
            let bind_addr = bind.unwrap_or(cli.bind);
            run_proxy(bind_addr, cli.api_key, store).await?;
        }
        Some(Commands::Add {
            label,
            api_key,
            org_id,
        }) => {
            add_account(store, label, api_key, org_id).await?;
        }
        Some(Commands::List) => {
            list_accounts(store).await?;
        }
        Some(Commands::Remove { identifier }) => {
            remove_account(store, identifier).await?;
        }
        Some(Commands::Show { identifier }) => {
            show_account(store, identifier).await?;
        }
        Some(Commands::Refresh) => {
            refresh_usage(store).await?;
        }
        Some(Commands::Config { action }) => {
            match action {
                ConfigCommands::Show => {
                    println!("Configuration file: {:?}", config_path);
                    println!("{}", toml::to_string_pretty(&config)?);
                }
                ConfigCommands::Set { key, value } => {
                    println!("Setting {} = {}", key, value);
                    // Implementation would update config and save
                }
                ConfigCommands::Reset => {
                    let default_config = Config::default();
                    default_config.save(&config_path)?;
                    println!("Configuration reset to defaults");
                }
            }
        }
        Some(Commands::Tui) | None => {
            // Start TUI
            let strategy = parse_routing_strategy(&config.routing.strategy);
            let routing_engine = Arc::new(RoutingEngine::new(strategy));

            if cli.proxy_only {
                run_proxy(cli.bind, cli.api_key, store).await?;
            } else {
                run_tui(store, routing_engine, config).await?;
            }
        }
    }

    Ok(())
}

/// Parse routing strategy from string
fn parse_routing_strategy(s: &str) -> RoutingStrategy {
    match s.to_lowercase().as_str() {
        "round_robin" | "round-robin" => RoutingStrategy::RoundRobin,
        "priority" => RoutingStrategy::Priority,
        "sticky" => RoutingStrategy::Sticky,
        _ => RoutingStrategy::LeastUtilized,
    }
}

/// Run the proxy server
async fn run_proxy(
    bind_addr: SocketAddr,
    api_key: String,
    store: EncryptedStore,
) -> Result<()> {
    info!("Starting proxy server on http://{}", bind_addr);

    // Load accounts
    let accounts = store.load_accounts()?;
    info!("Loaded {} accounts", accounts.len());

    // Create routing engine
    let strategy = RoutingStrategy::LeastUtilized;
    let routing_engine = Arc::new(RoutingEngine::new(strategy));

    // Update with current accounts
    let usage_map = std::collections::HashMap::new();
    routing_engine.update_accounts(accounts, usage_map).await;

    // Start proxy
    let proxy_config = proxy::ProxyConfig {
        bind_addr,
        api_key,
        openai_base_url: "https://api.openai.com".to_string(),
    };

    let server = proxy::ProxyServer::new(routing_engine, proxy_config);
    server.start().await?;

    info!("Proxy server running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

/// Run the TUI application
async fn run_tui(
    store: EncryptedStore,
    routing_engine: Arc<RoutingEngine>,
    config: Config,
) -> Result<()> {
    let mut app = ui::CliApp::new(store, routing_engine, config);
    app.run().await?;
    Ok(())
}

/// Add a new account
async fn add_account(
    store: EncryptedStore,
    label: String,
    api_key: String,
    org_id: Option<String>,
) -> Result<()> {
    let mut account = models::Account::new(label.clone(), api_key);

    if let Some(org) = org_id {
        account = account.with_org_id(org);
    }

    store.save_account(&account)?;
    println!("✓ Added account: {} ({})", label, account.id);

    Ok(())
}

/// List all accounts
async fn list_accounts(store: EncryptedStore) -> Result<()> {
    let accounts = store.load_accounts()?;

    if accounts.is_empty() {
        println!("No accounts configured. Use 'cam add' to add one.");
        return Ok(());
    }

    println!("\n{:<36} {:<20} {:<10} {:<10}", "ID", "Label", "Priority", "Enabled");
    println!("{}", "-".repeat(80));

    for account in accounts {
        println!(
            "{:<36} {:<20} {:<10} {:<10}",
            account.id,
            account.label,
            account.priority,
            if account.enabled { "✓" } else { "✗" }
        );
    }

    println!();
    Ok(())
}

/// Remove an account
async fn remove_account(store: EncryptedStore, identifier: String) -> Result<()> {
    // Try to parse as UUID first
    let id = if let Ok(uuid) = identifier.parse::<uuid::Uuid>() {
        uuid
    } else {
        // Try to find by label
        let accounts = store.load_accounts()?;
        let found = accounts.iter().find(|a| a.label == identifier);

        match found {
            Some(account) => account.id,
            None => {
                anyhow::bail!("Account not found: {}", identifier);
            }
        }
    };

    if store.delete_account(id)? {
        println!("✓ Removed account: {}", identifier);
    } else {
        println!("✗ Account not found: {}", identifier);
    }

    Ok(())
}

/// Show account details
async fn show_account(store: EncryptedStore, identifier: String) -> Result<()> {
    // Try to parse as UUID first
    let account = if let Ok(uuid) = identifier.parse::<uuid::Uuid>() {
        store.load_account(uuid)?
    } else {
        // Try to find by label
        let accounts = store.load_accounts()?;
        accounts.into_iter().find(|a| a.label == identifier)
    };

    match account {
        Some(acc) => {
            println!("\nAccount Details");
            println!("{}", "=".repeat(40));
            println!("ID:        {}", acc.id);
            println!("Label:     {}", acc.label);
            println!("Priority:  {}", acc.priority);
            println!("Enabled:   {}", acc.enabled);

            if let Some(org) = acc.org_id {
                println!("Org ID:    {}", org);
            }

            if let Some(daily) = acc.daily_limit {
                println!("Daily:     ${:.2}", daily);
            }

            if let Some(monthly) = acc.monthly_limit {
                println!("Monthly:   ${:.2}", monthly);
            }

            // Show usage if available
            if let Ok(Some(usage)) = store.load_latest_usage(acc.id) {
                println!("\nUsage Snapshot");
                println!("{}", "-".repeat(40));
                println!("Tokens:      {}", usage.tokens_used);
                println!("Cost:        ${:.4}", usage.cost_estimate);
                println!("Monthly:     ${:.2}", usage.monthly_usage);
                if let Some(remaining) = usage.remaining_budget {
                    println!("Remaining:   ${:.2}", remaining);
                }
                println!(
                    "Utilization: {:.1}%",
                    usage.utilization_ratio() * 100.0
                );
            }

            println!();
        }
        None => {
            anyhow::bail!("Account not found: {}", identifier);
        }
    }

    Ok(())
}

/// Refresh usage data for all accounts
async fn refresh_usage(store: EncryptedStore) -> Result<()> {
    use crate::usage::{OpenAIClient, UsagePoller};

    let accounts = store.load_accounts()?;
    let client = OpenAIClient::new();
    let poller = UsagePoller::new();

    println!("Refreshing usage for {} accounts...", accounts.len());

    for account in accounts {
        print!("  {} ... ", account.label);

        match poller.poll_account(&account, None).await {
            Ok(usage) => {
                store.save_usage_snapshot(&usage)?;
                println!(
                    "✓ (${:.2} used, {:.1}%)",
                    usage.monthly_usage,
                    usage.utilization_ratio() * 100.0
                );
            }
            Err(e) => {
                println!("✗ ({})", e);
            }
        }
    }

    println!("\nDone!");
    Ok(())
}
