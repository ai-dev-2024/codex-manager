use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for accounts
pub type AccountId = Uuid;

/// Account model representing a single OpenAI API tenant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub id: AccountId,
    pub label: String,
    pub api_key: String,
    pub org_id: Option<String>,
    pub model_scope: Vec<String>,
    pub daily_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl Account {
    pub fn new(label: String, api_key: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            label,
            api_key,
            org_id: None,
            model_scope: vec![],
            daily_limit: None,
            monthly_limit: None,
            priority: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
            last_used: None,
        }
    }

    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn with_model_scope(mut self, models: Vec<String>) -> Self {
        self.model_scope = models;
        self
    }

    pub fn with_limits(mut self, daily: Option<f64>, monthly: Option<f64>) -> Self {
        self.daily_limit = daily;
        self.monthly_limit = monthly;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Account status combining account config with usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatus {
    pub account: Account,
    pub usage: UsageSnapshot,
    pub is_available: bool,
    pub disable_reason: Option<String>,
}

/// Usage snapshot for an account at a point in time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub account_id: AccountId,
    pub tokens_used: u64,
    pub cost_estimate: f64,
    pub hard_limit: Option<f64>,
    pub soft_limit: Option<f64>,
    pub remaining_budget: Option<f64>,
    pub daily_usage: f64,
    pub monthly_usage: f64,
    pub timestamp: DateTime<Utc>,
}

impl UsageSnapshot {
    pub fn new(account_id: AccountId) -> Self {
        Self {
            account_id,
            tokens_used: 0,
            cost_estimate: 0.0,
            hard_limit: None,
            soft_limit: None,
            remaining_budget: None,
            daily_usage: 0.0,
            monthly_usage: 0.0,
            timestamp: Utc::now(),
        }
    }

    /// Calculate utilization ratio (0.0 - 1.0)
    pub fn utilization_ratio(&self) -> f64 {
        if let Some(hard) = self.hard_limit {
            if hard > 0.0 {
                return (self.monthly_usage / hard).clamp(0.0, 1.0);
            }
        }
        0.0
    }

    /// Check if account is over its limits
    pub fn is_over_limit(&self, account: &Account) -> bool {
        if let Some(daily) = account.daily_limit {
            if self.daily_usage >= daily {
                return true;
            }
        }
        if let Some(monthly) = account.monthly_limit {
            if self.monthly_usage >= monthly {
                return true;
            }
        }
        if let Some(remaining) = self.remaining_budget {
            if remaining <= 0.0 {
                return true;
            }
        }
        false
    }
}

/// Request context passed to routing engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub model: String,
    pub estimated_tokens: Option<u64>,
    pub session_id: Option<String>,
    pub priority: Option<i32>,
}

impl RequestContext {
    pub fn new(model: String) -> Self {
        Self {
            model,
            estimated_tokens: None,
            session_id: None,
            priority: None,
        }
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Account filtering criteria for routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountFilter {
    pub enabled_only: bool,
    pub under_limit_only: bool,
    pub supports_model: Option<String>,
    pub min_priority: Option<i32>,
}

/// Account creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub label: String,
    pub api_key: String,
    pub org_id: Option<String>,
    pub model_scope: Option<Vec<String>>,
    pub daily_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub priority: Option<i32>,
}

/// Account update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: AccountId,
    pub label: Option<String>,
    pub org_id: Option<String>,
    pub model_scope: Option<Vec<String>>,
    pub daily_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

/// Routing strategy types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    LeastUtilized,
    RoundRobin,
    Priority,
    Sticky,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::LeastUtilized
    }
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub strategy: RoutingStrategy,
    pub min_request_interval_ms: u64,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::LeastUtilized,
            min_request_interval_ms: 100,
        }
    }
}

/// Proxy server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyServerConfig {
    pub bind_addr: String,
    pub api_key: String,
    pub openai_base_url: String,
}

impl Default for ProxyServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".to_string(),
            api_key: "sk-codex-manager".to_string(),
            openai_base_url: "https://api.openai.com".to_string(),
        }
    }
}

/// App configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub proxy: ProxyServerConfig,
    pub routing: RoutingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyServerConfig::default(),
            routing: RoutingConfig::default(),
        }
    }
}

/// Routing decision response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub account_id: AccountId,
    pub account_label: String,
    pub reason: String,
    pub utilization_ratio: f64,
    pub remaining_budget: Option<f64>,
}

/// Routing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total_accounts: usize,
    pub available_accounts: usize,
    pub strategy: RoutingStrategy,
    pub open_circuits: usize,
    pub active_sessions: usize,
}

/// Proxy server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub bind_addr: String,
    pub request_count: u64,
    pub uptime_seconds: u64,
}

/// Import/Export data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountExport {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub accounts: Vec<Account>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub org_id: Option<String>,
    pub error: Option<String>,
}
