use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, instrument, trace, warn};

use crate::models::{Account, AccountFilter, AccountStatus, RequestContext, UsageSnapshot};

/// Routing strategy for selecting accounts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// Prefer accounts with lowest utilization ratio
    LeastUtilized,
    /// Round-robin through available accounts
    RoundRobin,
    /// Use priority order (highest priority first)
    Priority,
    /// Sticky sessions - route same content to same account
    Sticky,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::LeastUtilized
    }
}

/// Routing decision with metadata
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub account_id: uuid::Uuid,
    pub account_label: String,
    pub api_key: String,
    pub org_id: Option<String>,
    pub reason: RoutingReason,
    pub utilization_ratio: f64,
    pub remaining_budget: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum RoutingReason {
    LeastUtilized,
    RoundRobin { index: usize },
    Priority { priority: i32 },
    Sticky { session_id: String },
    Fallback,
    ErrorRecovery,
}

/// Circuit breaker state for tracking account health
#[derive(Debug, Clone)]
enum CircuitState {
    Closed, // Normal operation
    Open { since: Instant }, // Failing, don't use
    HalfOpen, // Testing if recovered
}

impl CircuitState {
    fn is_available(&self) -> bool {
        matches!(self, CircuitState::Closed | CircuitState::HalfOpen)
    }

    fn can_attempt(&self) -> bool {
        match self {
            CircuitState::Closed => true,
            CircuitState::Open { since } => since.elapsed() > Duration::from_secs(60),
            CircuitState::HalfOpen => true,
        }
    }
}

/// Account routing state
struct AccountRouteState {
    circuit: CircuitState,
    consecutive_errors: u32,
    last_used: Option<Instant>,
}

/// The routing engine - determines which account to use for requests
pub struct RoutingEngine {
    strategy: RoutingStrategy,
    accounts: Arc<RwLock<Vec<AccountStatus>>>,
    session_map: DashMap<String, uuid::Uuid>, // session_id -> account_id
    circuit_states: DashMap<uuid::Uuid, AccountRouteState>,
    round_robin_index: RwLock<usize>,
    min_request_interval: Duration,
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            accounts: Arc::new(RwLock::new(Vec::new())),
            session_map: DashMap::new(),
            circuit_states: DashMap::new(),
            round_robin_index: RwLock::new(0),
            min_request_interval: Duration::from_millis(100),
        }
    }

    /// Update the accounts and usage data
    pub async fn update_accounts(&self,
        accounts: Vec<Account>,
        usage_map: std::collections::HashMap<uuid::Uuid, UsageSnapshot>,
    ) {
        let mut statuses = Vec::new();

        for account in accounts {
            let usage = usage_map
                .get(&account.id)
                .cloned()
                .unwrap_or_else(|| UsageSnapshot::new(account.id));

            let is_available = account.enabled
                && !usage.is_over_limit(&account)
                && self.is_circuit_available(account.id).await;

            let disable_reason = if !account.enabled {
                Some("Account disabled".to_string())
            } else if usage.is_over_limit(&account) {
                Some("Over usage limit".to_string())
            } else if !self.is_circuit_available(account.id).await {
                Some("Circuit breaker open".to_string())
            } else {
                None
            };

            statuses.push(AccountStatus {
                account,
                usage,
                is_available,
                disable_reason,
            });
        }

        let mut guard = self.accounts.write().await;
        *guard = statuses;
        debug!("Updated {} accounts in routing engine", guard.len());
    }

    /// Check if account circuit is available
    async fn is_circuit_available(&self,
        account_id: uuid::Uuid,
    ) -> bool {
        self.circuit_states
            .get(&account_id)
            .map(|s| s.is_available())
            .unwrap_or(true)
    }

    /// Resolve which account to use for a request
    #[instrument(skip(self, ctx), fields(model = %ctx.model))]
    pub async fn resolve_account(&self,
        ctx: &RequestContext,
    ) -> Result<RoutingDecision> {
        let accounts = self.accounts.read().await;

        // Filter to available accounts that support the model
        let candidates: Vec<&AccountStatus> = accounts
            .iter()
            .filter(|s| {
                s.is_available
                    && self.supports_model(&s.account, &ctx.model)
                    && self.circuit_states
                        .get(&s.account.id)
                        .map(|state| state.can_attempt())
                        .unwrap_or(true)
            })
            .collect();

        if candidates.is_empty() {
            anyhow::bail!("No available accounts for model {}", ctx.model);
        }

        // Apply routing strategy
        let selected = match self.strategy {
            RoutingStrategy::LeastUtilized => {
                self.select_least_utilized(&candidates).await
            }
            RoutingStrategy::RoundRobin => {
                self.select_round_robin(&candidates).await
            }
            RoutingStrategy::Priority => {
                self.select_by_priority(&candidates).await
            }
            RoutingStrategy::Sticky = {
                self.select_sticky(&candidates, ctx.session_id.as_deref()).await
            }
        };

        // Update last used time
        if let Some(state) = self.circuit_states.get_mut(&selected.account.id) {
            state.last_used = Some(Instant::now());
        }

        trace!(
            "Selected account {} ({}) for model {}",
            selected.account.label,
            selected.account.id,
            ctx.model
        );

        Ok(RoutingDecision {
            account_id: selected.account.id,
            account_label: selected.account.label.clone(),
            api_key: selected.account.api_key.clone(),
            org_id: selected.account.org_id.clone(),
            reason: self.build_reason(ctx, &selected),
            utilization_ratio: selected.usage.utilization_ratio(),
            remaining_budget: selected.usage.remaining_budget,
        })
    }

    /// Check if account supports the requested model
    fn supports_model(&self,
        account: &Account,
        model: &str,
    ) -> bool {
        if account.model_scope.is_empty() {
            return true; // Empty scope = all models
        }
        account.model_scope.iter().any(|m| {
            // Support wildcards like "gpt-4*" or exact matches
            if m.ends_with('*') {
                model.starts_with(&m[..m.len()-1])
            } else {
                m == model
            }
        })
    }

    /// Select account with lowest utilization ratio
    async fn select_least_utilized(&self,
        candidates: &[&AccountStatus],
    ) -> &AccountStatus {
        candidates
            .iter()
            .min_by(|a, b| {
                let util_a = a.usage.utilization_ratio();
                let util_b = b.usage.utilization_ratio();
                util_a.partial_cmp(&util_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(candidates[0])
    }

    /// Select account via round-robin
    async fn select_round_robin(&self,
        candidates: &[&AccountStatus],
    ) -> &AccountStatus {
        let mut index = self.round_robin_index.write().await;
        let selected = candidates[*index % candidates.len()];
        *index = (*index + 1) % candidates.len();
        selected
    }

    /// Select account by priority (highest first)
    async fn select_by_priority(&self,
        candidates: &[&AccountStatus],
    ) -> &AccountStatus {
        candidates
            .iter()
            .max_by_key(|s| s.account.priority)
            .copied()
            .unwrap_or(candidates[0])
    }

    /// Select account with session stickiness
    async fn select_sticky(
        &self,
        candidates: &[&AccountStatus],
        session_id: Option<&str>,
    ) -> &AccountStatus {
        // If we have a session ID, try to stick to the same account
        if let Some(session) = session_id {
            if let Some(account_id) = self.session_map.get(session) {
                if let Some(status) = candidates.iter().find(|s| s.account.id == *account_id) {
                    return status;
                }
            }

            // No existing mapping or account unavailable - create new mapping
            let selected = self.select_least_utilized(candidates).await;
            self.session_map.insert(session.to_string(), selected.account.id);
            return selected;
        }

        // No session - fall back to least utilized
        self.select_least_utilized(candidates).await
    }

    /// Build routing reason for decision
    fn build_reason(
        &self,
        ctx: &RequestContext,
        status: &AccountStatus,
    ) -> RoutingReason {
        match self.strategy {
            RoutingStrategy::LeastUtilized => RoutingReason::LeastUtilized,
            RoutingStrategy::RoundRobin => {
                let index = *self.round_robin_index.blocking_read();
                RoutingReason::RoundRobin { index }
            }
            RoutingStrategy::Priority => {
                RoutingReason::Priority { priority: status.account.priority }
            }
            RoutingStrategy::Sticky => {
                if let Some(session) = &ctx.session_id {
                    RoutingReason::Sticky { session_id: session.clone() }
                } else {
                    RoutingReason::Fallback
                }
            }
        }
    }

    /// Report success for an account (resets circuit breaker)
    pub fn report_success(&self,
        account_id: uuid::Uuid,
    ) {
        let mut state = self.circuit_states
            .entry(account_id)
            .or_insert_with(|| AccountRouteState {
                circuit: CircuitState::Closed,
                consecutive_errors: 0,
                last_used: None,
            });

        state.consecutive_errors = 0;
        state.circuit = CircuitState::Closed;
    }

    /// Report error for an account (may open circuit breaker)
    pub fn report_error(
        &self,
        account_id: uuid::Uuid,
        is_fatal: bool,
    ) {
        let mut state = self.circuit_states
            .entry(account_id)
            .or_insert_with(|| AccountRouteState {
                circuit: CircuitState::Closed,
                consecutive_errors: 0,
                last_used: None,
            });

        if is_fatal {
            state.consecutive_errors += 1;

            // Open circuit after 3 consecutive fatal errors
            if state.consecutive_errors >= 3 {
                warn!(
                    "Opening circuit breaker for account {} after {} errors",
                    account_id, state.consecutive_errors
                );
                state.circuit = CircuitState::Open { since: Instant::now() };
            }
        }
    }

    /// Get current routing statistics
    pub async fn get_stats(&self,
    ) -> RoutingStats {
        let accounts = self.accounts.read().await;

        RoutingStats {
            total_accounts: accounts.len(),
            available_accounts: accounts.iter().filter(|s| s.is_available).count(),
            strategy: self.strategy,
            open_circuits: self.circuit_states
                .iter()
                .filter(|s| !s.is_available())
                .count(),
            active_sessions: self.session_map.len(),
        }
    }

    /// Clear session mappings (e.g., on config reload)
    pub fn clear_sessions(&self,
    ) {
        self.session_map.clear();
    }
}

/// Routing statistics
#[derive(Debug, Clone)]
pub struct RoutingStats {
    pub total_accounts: usize,
    pub available_accounts: usize,
    pub strategy: RoutingStrategy,
    pub open_circuits: usize,
    pub active_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_account(id: uuid::Uuid, priority: i32, enabled: bool) -> Account {
        Account {
            id,
            label: format!("Account {}", priority),
            api_key: "sk-test".to_string(),
            org_id: None,
            model_scope: vec![],
            daily_limit: None,
            monthly_limit: None,
            priority,
            enabled,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    #[tokio::test]
    async fn test_least_utilized_routing() {
        let engine = RoutingEngine::new(RoutingStrategy::LeastUtilized);

        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();

        let accounts = vec![
            create_test_account(id1, 1, true),
            create_test_account(id2, 2, true),
        ];

        let mut usage_map = std::collections::HashMap::new();
        usage_map.insert(id1, UsageSnapshot {
            account_id: id1,
            tokens_used: 1000,
            cost_estimate: 0.5,
            hard_limit: Some(100.0),
            soft_limit: None,
            remaining_budget: Some(50.0),
            daily_usage: 50.0,
            monthly_usage: 50.0,
            timestamp: chrono::Utc::now(),
        });
        usage_map.insert(id2, UsageSnapshot {
            account_id: id2,
            tokens_used: 1000,
            cost_estimate: 0.5,
            hard_limit: Some(100.0),
            soft_limit: None,
            remaining_budget: Some(90.0),
            daily_usage: 10.0,
            monthly_usage: 10.0,
            timestamp: chrono::Utc::now(),
        });

        engine.update_accounts(accounts, usage_map).await;

        let ctx = RequestContext::new("gpt-4".to_string());
        let decision = engine.resolve_account(&ctx).await.unwrap();

        // Should select account with lower utilization (id2 at 10% vs id1 at 50%)
        assert_eq!(decision.account_id, id2);
    }

    #[tokio::test]
    async fn test_priority_routing() {
        let engine = RoutingEngine::new(RoutingStrategy::Priority);

        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();

        let accounts = vec![
            create_test_account(id1, 1, true),
            create_test_account(id2, 5, true),
        ];

        let usage_map = std::collections::HashMap::new();
        engine.update_accounts(accounts, usage_map).await;

        let ctx = RequestContext::new("gpt-4".to_string());
        let decision = engine.resolve_account(&ctx).await.unwrap();

        // Should select higher priority account
        assert_eq!(decision.account_id, id2);
    }

    #[tokio::test]
    async fn test_disabled_account_filtering() {
        let engine = RoutingEngine::new(RoutingStrategy::LeastUtilized);

        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();

        let mut acc1 = create_test_account(id1, 1, true);
        acc1.enabled = false;
        let acc2 = create_test_account(id2, 2, true);

        let accounts = vec![acc1, acc2];
        let usage_map = std::collections::HashMap::new();
        engine.update_accounts(accounts, usage_map).await;

        let ctx = RequestContext::new("gpt-4".to_string());
        let decision = engine.resolve_account(&ctx).await.unwrap();

        // Should only select enabled account
        assert_eq!(decision.account_id, id2);
    }
}
