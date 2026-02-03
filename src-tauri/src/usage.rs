use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::{Client, Method};
use serde::Deserialize;
use tracing::{debug, instrument, warn};

use crate::models::{Account, UsageSnapshot, ValidationResult};

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
    fn build_request(
        &self,
        account: &Account,
        method: Method,
        path: &str,
    ) -> reqwest::RequestBuilder {
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
    pub async fn fetch_usage(&self, account: &Account) -> Result<UsageSnapshot> {
        let mut snapshot = UsageSnapshot::new(account.id);

        match self.fetch_billing_usage(account).await {
            Ok(usage) => {
                snapshot.monthly_usage = usage.total_usage / 100.0;
                debug!(
                    "Fetched billing usage for {}: ${:.2}",
                    account.label, snapshot.monthly_usage
                );
            }
            Err(e) => {
                warn!("Failed to fetch billing usage for {}: {}", account.label, e);
            }
        }

        match self.fetch_subscription(account).await {
            Ok(sub) => {
                snapshot.hard_limit = sub.hard_limit_usd;
                snapshot.soft_limit = sub.soft_limit_usd;

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

    /// Fetch billing usage
    async fn fetch_billing_usage(&self, account: &Account) -> Result<BillingUsageResponse> {
        let now = Utc::now();
        let start_date = now.format("%Y-%m-01").to_string();
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

    /// Fetch subscription info
    async fn fetch_subscription(&self, account: &Account) -> Result<SubscriptionResponse> {
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

    /// Fetch token usage
    async fn fetch_token_usage(&self, account: &Account) -> Result<TokenUsageSummary> {
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

        let total_tokens: u64 = usage
            .data
            .iter()
            .map(|d| d.n_generated_tokens + d.n_context_tokens)
            .sum();

        let total_cost: f64 = usage
            .data
            .iter()
            .map(|d| {
                let input_cost = d.n_context_tokens as f64 * 0.000_001_5;
                let output_cost = d.n_generated_tokens as f64 * 0.000_006;
                input_cost + output_cost
            })
            .sum();

        Ok(TokenUsageSummary {
            total_tokens,
            total_cost,
        })
    }

    /// Validate that an API key is working
    pub async fn validate_key(&self, api_key: &str, org_id: Option<&str>) -> Result<ValidationResult> {
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
            let text = resp.text().await.unwrap_or_default();
            return Ok(ValidationResult {
                valid: false,
                org_id: None,
                error: Some(format!("API error {}: {}", status, text)),
            });
        }

        let org_header = resp.headers().get("openai-organization");
        let org_id = org_header.and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        Ok(ValidationResult {
            valid: true,
            org_id,
            error: None,
        })
    }
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct BillingUsageResponse {
    #[serde(rename = "total_usage")]
    pub total_usage: f64,
}

#[derive(Debug, Deserialize)]
struct SubscriptionResponse {
    #[serde(rename = "soft_limit_usd")]
    pub soft_limit_usd: Option<f64>,
    #[serde(rename = "hard_limit_usd")]
    pub hard_limit_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TokenUsageResponse {
    pub data: Vec<TokenUsageData>,
}

#[derive(Debug, Deserialize)]
struct TokenUsageData {
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

/// Usage poller for periodic updates
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

    pub async fn poll_account(
        &self,
        account: &Account,
        _last_error: Option<&std::time::Instant>,
    ) -> Result<UsageSnapshot> {
        self.client.fetch_usage(account).await
    }

    pub fn next_interval(&self, consecutive_errors: u32) -> std::time::Duration {
        let backoff = std::time::Duration::from_secs(2_u64.pow(consecutive_errors.min(5)));
        std::cmp::min(self.min_interval + backoff, self.max_interval)
    }
}

impl Default for UsagePoller {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command: Fetch usage for an account
#[tauri::command]
pub async fn fetch_account_usage(
    api_key: String,
    org_id: Option<String>,
) -> Result<UsageSnapshot, String> {
    let account = Account::new("temp".to_string(), api_key).with_org_id(org_id.unwrap_or_default());
    let client = OpenAIClient::new();
    
    client
        .fetch_usage(&account)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri command: Validate API key
#[tauri::command]
pub async fn validate_api_key(
    api_key: String,
    org_id: Option<String>,
) -> Result<ValidationResult, String> {
    let client = OpenAIClient::new();
    
    client
        .validate_key(&api_key, org_id.as_deref())
        .await
        .map_err(|e| e.to_string())
}
