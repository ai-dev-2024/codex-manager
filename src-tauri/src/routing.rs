use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, instrument, trace, warn};

use crate::models::{
    Account, AccountFilter, AccountStatus, RequestContext, RoutingDecision, RoutingStats,
    RoutingStrategy, UsageSnapshot,
};

/// Routing reason for decision tracking
#[derive(Debug, Clone)]
pub enum RoutingReason {
    LeastUtilized,
    RoundRobin { index: usize },
    Priority { priority: i32 },
    Sticky { session_id: String },
    Fallback,
    ErrorRecovery,
}

impl RoutingReason {
    pub fn to_string(&self) -> String {
        match self {
            RoutingReason::LeastUtilized => "least_utilized".to_string(),
            RoutingReason::RoundRobin { index } => format!("round_robin:{}", index),
            RoutingReason::Priority { priority } => format!("priority:{}", priority),
            RoutingReason::Sticky { session_id } => format!("sticky:{}", session_id),
            RoutingReason::Fallback => "fallback".to_string(),
            RoutingReason::ErrorRecovery => "error_recovery".to_string(),
        }
    }
}

/// Circuit breaker state for tracking account health
#[derive(Debug, Clone)]
enum CircuitState {
    Closed,
    Open { since: Instant },
    HalfOpen,
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
    session_map: DashMap<String, uuid::Uuid>,
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

    /// Update the routing strategy
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
    }

    /// Get current strategy
    pub fn get_strategy(&self) -> RoutingStrategy {
        self.strategy
    }

    /// Update the accounts and usage data
    pub async fn update_accounts(
        &self,
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
    async fn is_circuit_available(&self, account_id: uuid::Uuid) -> bool {
        self.circuit_states
            .get(&account_id)
            .map(|s| s.is_available())
            .unwrap_or(true)
    }

    /// Resolve which account to use for a request
    #[instrument(skip(self, ctx), fields(model = %ctx.model))]
    pub async fn resolve_account(&self, ctx: &RequestContext) -> Result<RoutingDecision> {
        let accounts = self.accounts.read().await;

        // Filter to available accounts that support the model
        let candidates: Vec<&AccountStatus> = accounts
            .iter()
            .filter(|s| {
                s.is_available
                    && self.supports_model(&s.account, &ctx.model)
                    && self
                        .circuit_states
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
            RoutingStrategy::LeastUtilized => self.select_least_utilized(&candidates).await,
            RoutingStrategy::RoundRobin => self.select_round_robin(&candidates).await,
            RoutingStrategy::Priority => self.select_by_priority(&candidates).await,
            RoutingStrategy::Sticky => {
                self.select_sticky(&candidates, ctx.session_id.as_deref()).await
            }
        };

        // Update last used time
        if let Some(mut state) = self.circuit_states.get_mut(&selected.account.id) {
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
            reason: self.build_reason(ctx, &selected).to_string(),
            utilization_ratio: selected.usage.utilization_ratio(),
            remaining_budget: selected.usage.remaining_budget,
        })
    }

    /// Check if account supports the requested model
    fn supports_model(&self, account: &Account, model: &str) -> bool {
        if account.model_scope.is_empty() {
            return true;
        }
        account.model_scope.iter().any(|m| {
            if m.ends_with('*') {
                model.starts_with(&m[..m.len() - 1])
            } else {
                m == model
            }
        })
    }

    /// Select account with lowest utilization ratio
    async fn select_least_utilized<'a>(
        &self,
        candidates: &[&'a AccountStatus],
    ) -> &'a AccountStatus {
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
    async fn select_round_robin<'a>(
        &self,
        candidates: &[&'a AccountStatus],
    ) -> &'a AccountStatus {
        let mut index = self.round_robin_index.write().await;
        let selected = candidates[*index % candidates.len()];
        *index = (*index + 1) % candidates.len();
        selected
    }

    /// Select account by priority (highest first)
    async fn select_by_priority<'a>(
        &self,
        candidates: &[&'a AccountStatus],
    ) -> &'a AccountStatus {
        candidates
            .iter()
            .max_by_key(|s| s.account.priority)
            .copied()
            .unwrap_or(candidates[0])
    }

    /// Select account with session stickiness
    async fn select_sticky<'a>(
        &self,
        candidates: &[&'a AccountStatus],
        session_id: Option<&str>,
    ) -> &'a AccountStatus {
        if let Some(session) = session_id {
            if let Some(account_id) = self.session_map.get(session) {
                if let Some(status) = candidates.iter().find(|s| s.account.id == *account_id) {
                    return status;
                }
            }

            let selected = self.select_least_utilized(candidates).await;
            self.session_map
                .insert(session.to_string(), selected.account.id);
            return selected;
        }

        self.select_least_utilized(candidates).await
    }

    /// Build routing reason for decision
    fn build_reason(&self, ctx: &RequestContext, status: &AccountStatus) -> RoutingReason {
        match self.strategy {
            RoutingStrategy::LeastUtilized => RoutingReason::LeastUtilized,
            RoutingStrategy::RoundRobin => {
                let index = *self.round_robin_index.blocking_read();
                RoutingReason::RoundRobin { index }
            }
            RoutingStrategy::Priority => RoutingReason::Priority {
                priority: status.account.priority,
            },
            RoutingStrategy::Sticky => {
                if let Some(session) = &ctx.session_id {
                    RoutingReason::Sticky {
                        session_id: session.clone(),
                    }
                } else {
                    RoutingReason::Fallback
                }
            }
        }
    }

    /// Report success for an account (resets circuit breaker)
    pub fn report_success(&self, account_id: uuid::Uuid) {
        let mut state = self
            .circuit_states
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
    pub fn report_error(&self, account_id: uuid::Uuid, is_fatal: bool) {
        let mut state = self
            .circuit_states
            .entry(account_id)
            .or_insert_with(|| AccountRouteState {
                circuit: CircuitState::Closed,
                consecutive_errors: 0,
                last_used: None,
            });

        if is_fatal {
            state.consecutive_errors += 1;

            if state.consecutive_errors >= 3 {
                warn!(
                    "Opening circuit breaker for account {} after {} errors",
                    account_id, state.consecutive_errors
                );
                state.circuit = CircuitState::Open {
                    since: Instant::now(),
                };
            }
        }
    }

    /// Get current routing statistics
    pub async fn get_stats(&self) -> RoutingStats {
        let accounts = self.accounts.read().await;

        RoutingStats {
            total_accounts: accounts.len(),
            available_accounts: accounts.iter().filter(|s| s.is_available).count(),
            strategy: self.strategy,
            open_circuits: self
                .circuit_states
                .iter()
                .filter(|s| !s.is_available())
                .count(),
            active_sessions: self.session_map.len(),
        }
    }

    /// Clear session mappings
    pub fn clear_sessions(&self) {
        self.session_map.clear();
    }

    /// Get all account statuses
    pub async fn get_account_statuses(&self) -> Vec<AccountStatus> {
        self.accounts.read().await.clone()
    }
}
