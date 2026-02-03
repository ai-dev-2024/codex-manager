use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyStats {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub bytes_transferred: u64,
}
