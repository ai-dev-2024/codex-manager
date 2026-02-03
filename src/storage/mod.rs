use aes_gcm::{
    aead::{Aead, NewAead},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2, PasswordHasher,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::rngs::OsRng;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use uuid::Uuid;

use crate::models::{Account, AccountId, UsageSnapshot};

/// Manages encrypted SQLite storage for accounts and usage data
pub struct EncryptedStore {
    conn: Connection,
    cipher: Aes256Gcm,
}

impl EncryptedStore {
    /// Initialize or open the encrypted database
    pub fn open(db_path: &Path, master_key: &str) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open database")?;

        // Derive encryption key from master password
        let cipher = Self::derive_cipher(master_key)?;

        let store = Self { conn, cipher };
        store.init_schema()?;

        Ok(store)
    }

    /// Create an in-memory database (for testing)
    pub fn open_in_memory(master_key: &str) -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;

        let cipher = Self::derive_cipher(master_key)?;
        let store = Self { conn, cipher };
        store.init_schema()?;

        Ok(store)
    }

    /// Derive AES-256-GCM cipher from master key using Argon2
    fn derive_cipher(master_key: &str) -> Result<Aes256Gcm> {
        // Generate a salt (in production, store this separately)
        let salt = SaltString::generate(&mut OsRng);

        // Use Argon2id to derive a 256-bit key
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(master_key.as_bytes(), &salt)
            .context("Failed to hash password")?;

        // Extract the hash output as key material
        let hash = password_hash
            .hash
            .ok_or_else(|| anyhow::anyhow!("No hash output from Argon2"))?;

        // Create key from hash bytes
        let key_bytes = hash.as_bytes();
        let key = aes_gcm::Key::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);

        Ok(cipher)
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        // Combine nonce + ciphertext and encode
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt(&self, ciphertext_b64: &str) -> Result<String> {
        let combined = BASE64
            .decode(ciphertext_b64)
            .context("Invalid base64 encoding")?;

        if combined.len() < 12 {
            anyhow::bail!("Ciphertext too short");
        }

        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted data")
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                api_key_encrypted TEXT NOT NULL,
                org_id TEXT,
                model_scope TEXT, -- JSON array
                daily_limit REAL,
                monthly_limit REAL,
                priority INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_used TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_accounts_enabled ON accounts(enabled);
            CREATE INDEX IF NOT EXISTS idx_accounts_priority ON accounts(priority);

            CREATE TABLE IF NOT EXISTS usage_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id TEXT NOT NULL,
                tokens_used INTEGER DEFAULT 0,
                cost_estimate REAL DEFAULT 0.0,
                hard_limit REAL,
                soft_limit REAL,
                remaining_budget REAL,
                daily_usage REAL DEFAULT 0.0,
                monthly_usage REAL DEFAULT 0.0,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_usage_account ON usage_snapshots(account_id);
            CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON usage_snapshots(timestamp);

            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "#,
            )
            .context("Failed to initialize database schema")?;

        Ok(())
    }

    /// Save or update an account
    pub fn save_account(&self, account: &Account) -> Result<()> {
        let encrypted_key = self.encrypt(&account.api_key)?;
        let model_scope_json = serde_json::to_string(&account.model_scope)?;

        self.conn
            .execute(
                r#"
            INSERT INTO accounts (
                id, label, api_key_encrypted, org_id, model_scope,
                daily_limit, monthly_limit, priority, enabled,
                created_at, updated_at, last_used
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                label = excluded.label,
                api_key_encrypted = excluded.api_key_encrypted,
                org_id = excluded.org_id,
                model_scope = excluded.model_scope,
                daily_limit = excluded.daily_limit,
                monthly_limit = excluded.monthly_limit,
                priority = excluded.priority,
                enabled = excluded.enabled,
                updated_at = excluded.updated_at,
                last_used = excluded.last_used
            "#,
                params![
                    account.id.to_string(),
                    account.label,
                    encrypted_key,
                    account.org_id,
                    model_scope_json,
                    account.daily_limit,
                    account.monthly_limit,
                    account.priority,
                    account.enabled as i32,
                    account.created_at.to_rfc3339(),
                    account.updated_at.to_rfc3339(),
                    account.last_used.map(|t| t.to_rfc3339()),
                ],
            )
            .context("Failed to save account")?;

        Ok(())
    }

    /// Load all accounts
    pub fn load_accounts(&self) -> Result<Vec<Account>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM accounts ORDER BY priority DESC, created_at ASC")?;

        let accounts = stmt.query_map([], |row| {
            let encrypted_key: String = row.get("api_key_encrypted")?;
            let api_key = self.decrypt(&encrypted_key).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let model_scope_json: String = row.get("model_scope")?;
            let model_scope: Vec<String> =
                serde_json::from_str(&model_scope_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(Account {
                id: row.get::<String, _>("id")?.parse().unwrap(),
                label: row.get("label")?,
                api_key,
                org_id: row.get("org_id")?,
                model_scope,
                daily_limit: row.get("daily_limit")?,
                monthly_limit: row.get("monthly_limit")?,
                priority: row.get("priority")?,
                enabled: row.get::<i32, _>("enabled")? != 0,
                created_at: row.get::<String, _>("created_at")?.parse().unwrap(),
                updated_at: row.get::<String, _>("updated_at")?.parse().unwrap(),
                last_used: row
                    .get::<Option<String>, _>("last_used")?
                    .map(|s| s.parse().unwrap()),
            })
        })?;

        accounts
            .collect::<Result<_, _>>()
            .context("Failed to load accounts")
    }

    /// Load a single account by ID
    pub fn load_account(&self, id: AccountId) -> Result<Option<Account>> {
        let mut stmt = self.conn.prepare("SELECT * FROM accounts WHERE id = ?1")?;

        let account = stmt
            .query_row([id.to_string()], |row| {
                let encrypted_key: String = row.get("api_key_encrypted")?;
                let api_key = self.decrypt(&encrypted_key).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let model_scope_json: String = row.get("model_scope")?;
                let model_scope: Vec<String> =
                    serde_json::from_str(&model_scope_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Account {
                    id: row.get::<String, _>("id")?.parse().unwrap(),
                    label: row.get("label")?,
                    api_key,
                    org_id: row.get("org_id")?,
                    model_scope,
                    daily_limit: row.get("daily_limit")?,
                    monthly_limit: row.get("monthly_limit")?,
                    priority: row.get("priority")?,
                    enabled: row.get::<i32, _>("enabled")? != 0,
                    created_at: row.get::<String, _>("created_at")?.parse().unwrap(),
                    updated_at: row.get::<String, _>("updated_at")?.parse().unwrap(),
                    last_used: row
                        .get::<Option<String>, _>("last_used")?
                        .map(|s| s.parse().unwrap()),
                })
            })
            .optional()?;

        Ok(account)
    }

    /// Delete an account
    pub fn delete_account(&self, id: AccountId) -> Result<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM accounts WHERE id = ?1", [id.to_string()])?;

        // Also delete usage snapshots
        self.conn.execute(
            "DELETE FROM usage_snapshots WHERE account_id = ?1",
            [id.to_string()],
        )?;

        Ok(rows > 0)
    }

    /// Save a usage snapshot
    pub fn save_usage_snapshot(&self, snapshot: &UsageSnapshot) -> Result<()> {
        self.conn
            .execute(
                r#"
            INSERT INTO usage_snapshots (
                account_id, tokens_used, cost_estimate, hard_limit,
                soft_limit, remaining_budget, daily_usage, monthly_usage, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
                params![
                    snapshot.account_id.to_string(),
                    snapshot.tokens_used as i64,
                    snapshot.cost_estimate,
                    snapshot.hard_limit,
                    snapshot.soft_limit,
                    snapshot.remaining_budget,
                    snapshot.daily_usage,
                    snapshot.monthly_usage,
                    snapshot.timestamp.to_rfc3339(),
                ],
            )
            .context("Failed to save usage snapshot")?;

        Ok(())
    }

    /// Load the latest usage snapshot for an account
    pub fn load_latest_usage(&self, account_id: AccountId) -> Result<Option<UsageSnapshot>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM usage_snapshots WHERE account_id = ?1 ORDER BY timestamp DESC LIMIT 1",
        )?;

        let snapshot = stmt
            .query_row([account_id.to_string()], |row| {
                Ok(UsageSnapshot {
                    account_id: row.get::<String, _>("account_id")?.parse().unwrap(),
                    tokens_used: row.get::<i64, _>("tokens_used")? as u64,
                    cost_estimate: row.get("cost_estimate")?,
                    hard_limit: row.get("hard_limit")?,
                    soft_limit: row.get("soft_limit")?,
                    remaining_budget: row.get("remaining_budget")?,
                    daily_usage: row.get("daily_usage")?,
                    monthly_usage: row.get("monthly_usage")?,
                    timestamp: row.get::<String, _>("timestamp")?.parse().unwrap(),
                })
            })
            .optional()?;

        Ok(snapshot)
    }

    /// Get database metadata
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let value = self
            .conn
            .query_row("SELECT value FROM metadata WHERE key = ?1", [key], |row| {
                row.get::<String, _>(0)
            })
            .optional()?;

        Ok(value)
    }

    /// Set database metadata
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_storage() {
        let store = EncryptedStore::open_in_memory("test_master_key").unwrap();

        // Create test account
        let account = Account::new(
            "Test Account".to_string(),
            "sk-test-secret-key-12345".to_string(),
        )
        .with_limits(Some(10.0), Some(100.0))
        .with_priority(5);

        // Save account
        store.save_account(&account).unwrap();

        // Load and verify
        let loaded = store.load_account(account.id).unwrap().unwrap();
        assert_eq!(loaded.label, account.label);
        assert_eq!(loaded.api_key, account.api_key);
        assert_eq!(loaded.daily_limit, Some(10.0));
        assert_eq!(loaded.priority, 5);

        // Save usage snapshot
        let mut snapshot = UsageSnapshot::new(account.id);
        snapshot.daily_usage = 5.0;
        snapshot.monthly_usage = 50.0;
        store.save_usage_snapshot(&snapshot).unwrap();

        // Load latest usage
        let loaded_snapshot = store.load_latest_usage(account.id).unwrap().unwrap();
        assert_eq!(loaded_snapshot.daily_usage, 5.0);
        assert_eq!(loaded_snapshot.monthly_usage, 50.0);
    }
}
