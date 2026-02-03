use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

use crate::models::{Account, UsageSnapshot};

/// OpenAI API client for fetching usage and billing information
pub struct OpenAIClient {
    http: Client,
    base_url: String,
}

impl OpenAIClient {
    const DEFAULT_BASE_URL: &str = "https://api.openai.com";

    pub fn new() -> Self {
        Self {
            http: Client::new(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Build authenticated request for an account
    fn build_request(&self, account: &Account, method: Method, path: &str,
    ) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url);

        req = req.header("Authorization", format!("Bearer {}", account.api_key));

        if let Some(org_id) = &account.org_id {
            req = req.header("OpenAI-Organization", org_id);
        }

        req
    }

    /// Fetch current usage snapshot for an account
    #[instrument(skip(self, account), fields(account_id = %account.id, account_label = %account.label))]
    pub async fn fetch_usage(&self,
        account: &Account,
    ) -> Result<UsageSnapshot> {
        let mut snapshot = UsageSnapshot::new(account.id);

        // Try to fetch usage from various endpoints
        match self.fetch_billing_usage(account).await {
            Ok(usage) => {
                snapshot.monthly_usage = usage.total_usage / 100.0; // Convert cents to dollars
                debug!(
                    "Fetched billing usage for {}: ${:.2}",
                    account.label, snapshot.monthly_usage
                );
            }
            Err(e) => {
                warn!("Failed to fetch billing usage for {}: {}", account.label, e);
            }
        }

        // Fetch billing limits/subscription info
        match self.fetch_subscription(account).await {
            Ok(sub) => {
                snapshot.hard_limit = sub.hard_limit_usd;
                snapshot.soft_limit = sub.soft_limit_usd;

                // Calculate remaining budget
                if let Some(hard) = snapshot.hard_limit {
                    snapshot.remaining_budget = Some(hard - snapshot.monthly_usage);
                }

                debug!(
                    "Fetched subscription for {}: limit=${:?}, usage=${:.2}",
                    account.label, snapshot.hard_limit, snapshot.monthly_usage
                );
            }
            Err(e) => {
                warn!("Failed to fetch subscription for {}: {}", account.label, e);
            }
        }

        // Try to fetch token usage from the newer usage endpoint
        match self.fetch_token_usage(account).await {
            Ok(token_usage) => {
                snapshot.tokens_used = token_usage.total_tokens;
                snapshot.cost_estimate = token_usage.total_cost;
                debug!(
                    "Fetched token usage for {}: {} tokens, ${:.4}",
                    account.label, snapshot.tokens_used, snapshot.cost_estimate
                );
            }
            Err(e) => {
                debug!("Token usage endpoint not available for {}: {}", account.label, e);
            }
        }

        snapshot.timestamp = Utc::now();
        Ok(snapshot)
    }

    /// Fetch billing usage (v1/dashboard/billing/usage)
    async fn fetch_billing_usage(&self,
        account: &Account,
    ) -> Result<BillingUsageResponse> {
        // Get current billing period
        let now = Utc::now();
        let start_date = now.with_day(1).unwrap_or(now).format("%Y-%m-%d").to_string();
        let end_date = now.format("%Y-%m-%d").to_string();

        let resp = self
            .build_request(account, Method::GET, "/v1/dashboard/billing/usage")
            .query(&[("start_date", start_date), ("end_date", end_date)])
            .send()
            .await
            .context("Failed to send billing usage request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Billing usage API error: {} - {}", status, text);
        }

        let usage: BillingUsageResponse = resp
            .json()
            .await
            .context("Failed to parse billing usage response")?;

        Ok(usage)
    }

    /// Fetch subscription info (v1/dashboard/billing/subscription)
    async fn fetch_subscription(&self,
        account: &Account,
    ) -> Result<SubscriptionResponse> {
        let resp = self
            .build_request(account, Method::GET, "/v1/dashboard/billing/subscription")
            .send()
            .await
            .context("Failed to send subscription request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Subscription API error: {} - {}", status, text);
        }

        let sub: SubscriptionResponse = resp
            .json()
            .await
            .context("Failed to parse subscription response")?;

        Ok(sub)
    }

    /// Fetch token usage from the usage endpoint (if available)
    async fn fetch_token_usage(&self,
        account: &Account,
    ) -> Result<TokenUsageSummary> {
        // This endpoint is newer and may not be available for all accounts
        let resp = self
            .build_request(account, Method::GET, "/v1/usage")
            .send()
            .await
            .context("Failed to send token usage request")?;

        if resp.status() == 404 {
            anyhow::bail!("Token usage endpoint not available (404)");
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Token usage API error: {} - {}", status, text);
        }

        let usage: TokenUsageResponse = resp
            .json()
            .await
            .context("Failed to parse token usage response")?;

        // Aggregate token usage
        let total_tokens: u64 = usage
            .data
            .iter()
            .map(|d| d.n_generated_tokens + d.n_context_tokens)
            .sum();

        // Estimate cost (rough approximation)
        let total_cost: f64 = usage
            .data
            .iter()
            .map(|d| {
                let input_cost = d.n_context_tokens as f64 * 0.000_001_5; // $1.50 per 1M tokens
                let output_cost = d.n_generated_tokens as f64 * 0.000_006; // $6.00 per 1M tokens
                input_cost + output_cost
            })
            .sum();

        Ok(TokenUsageSummary {
            total_tokens,
            total_cost,
        })
    }

    /// Validate that an API key is working
    pub async fn validate_key(&self,
        api_key: &str,
        org_id: Option<&str>,
    ) -> Result<AccountInfo> {
        let mut req = self
            .http
            .request(Method::GET, format!("{}/v1/models", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key));

        if let Some(org) = org_id {
            req = req.header("OpenAI-Organization", org);
        }

        let resp = req.send().await.context("Failed to validate API key")?;

        if !resp.status().is_success() {
            let status = resp.status();
            anyhow::bail!("Invalid API key or organization: {}", status);
        }

        // Extract org info from headers if available
        let org_header = resp.headers().get("openai-organization");
        let org_id = org_header.and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        Ok(AccountInfo {
            org_id,
            is_valid: true,
        })
    }
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from billing usage endpoint
#[derive(Debug, Deserialize)]
struct BillingUsageResponse {
    pub object: String,
    #[serde(rename = "daily_costs")]
    pub daily_costs: Vec<DailyCost>,
    #[serde(rename = "total_usage")]
    pub total_usage: f64, // In cents
}

#[derive(Debug, Deserialize)]
struct DailyCost {
    pub timestamp: i64,
    #[serde(rename = "line_items")]
    pub line_items: Vec<LineItem>,
}

#[derive(Debug, Deserialize)]
struct LineItem {
    pub name: String,
    pub cost: f64,
}

/// Response from subscription endpoint
#[derive(Debug, Deserialize)]
struct SubscriptionResponse {
    #[serde(rename = "object")]
    pub object: String,
    #[serde(rename = "has_payment_method")]
    pub has_payment_method: bool,
    #[serde(rename = "canceled")]
    pub canceled: bool,
    #[serde(rename = "canceled_at")]
    pub canceled_at: Option<i64>,
    #[serde(rename = "delinquent")]
    pub delinquent: Option<bool>,
    #[serde(rename = "access_until")]
    pub access_until: i64,
    #[serde(rename = "soft_limit_usd")]
    pub soft_limit_usd: Option<f64>,
    #[serde(rename = "hard_limit_usd")]
    pub hard_limit_usd: Option<f64>,
    #[serde(rename = "system_hard_limit_usd")]
    pub system_hard_limit_usd: Option<f64>,
    #[serde(rename = "soft_limit")]
    pub soft_limit: Option<f64>,
    #[serde(rename = "hard_limit")]
    pub hard_limit: Option<f64>,
    #[serde(rename = "plan")]
    pub plan: Option<PlanInfo>,
}

#[derive(Debug, Deserialize)]
struct PlanInfo {
    pub title: String,
    pub id: String,
}

/// Response from token usage endpoint
#[derive(Debug, Deserialize)]
struct TokenUsageResponse {
    pub object: String,
    pub data: Vec<TokenUsageData>,
}

#[derive(Debug, Deserialize)]
struct TokenUsageData {
    #[serde(rename = "organization_id")]
    pub organization_id: String,
    #[serde(rename = "organization_name")]
    pub organization_name: String,
    #[serde(rename = "aggregation_timestamp")]
    pub aggregation_timestamp: i64,
    #[serde(rename = "n_requests")]
    pub n_requests: i64,
    #[serde(rename = "operation")]
    pub operation: String,
    #[serde(rename = "snapshot_id")]
    pub snapshot_id: String,
    #[serde(rename = "model")]
    pub model: Option<String>,
    #[serde(rename = "n_generated_tokens")]
    pub n_generated_tokens: u64,
    #[serde(rename = "n_context_tokens")]
    pub n_context_tokens: u64,
}

#[derive(Debug)]
struct TokenUsageSummary {
    pub total_tokens: u64,
    pub total_cost: f64,
}

/// Account validation info
#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub org_id: Option<String>,
    pub is_valid: bool,
}

/// Usage poller that periodically updates usage data for all accounts
pub struct UsagePoller {
    client: OpenAIClient,
    min_interval: std::time::Duration,
    max_interval: std::time::Duration,
}

impl UsagePoller {
    pub fn new() -> Self {
        Self {
            client: OpenAIClient::new(),
            min_interval: std::time::Duration::from_secs(60),
            max_interval: std::time::Duration::from_secs(3600),
        }
    }

    /// Poll usage for a single account with exponential backoff
    pub async fn poll_account(
        &self,
        account: &Account,
        last_error: Option<&std::time::Instant>,
    ) -> Result<UsageSnapshot> {
        // Implement backoff if there was a recent error
        if let Some(last_err) = last_error {
            let elapsed = last_err.elapsed();
            if elapsed < self.min_interval {
                tokio::time::sleep(self.min_interval - elapsed).await;
            }
        }

        self.client.fetch_usage(account).await
    }

    /// Calculate next poll interval based on consecutive errors
    pub fn next_interval(&self,
        consecutive_errors: u32,
    ) -> std::time::Duration {
        let backoff = std::time::Duration::from_secs(2_u64.pow(consecutive_errors.min(5)));
        std::cmp::min(self.min_interval + backoff, self.max_interval)
    }
}

impl Default for UsagePoller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_poller_interval() {
        let poller = UsagePoller::new();

        // No errors - minimum interval
        assert_eq!(poller.next_interval(0).as_secs(), 60);

        // Some errors - exponential backoff
        assert_eq!(poller.next_interval(1).as_secs(), 62);
        assert_eq!(poller.next_interval(2).as_secs(), 64);
        assert_eq!(poller.next_interval(5).as_secs(), 92);

        // Max errors - capped at max_interval
        assert_eq!(poller.next_interval(10).as_secs(), 3600);
    }
}
