//! Local audit log — immutable append-only record of all policy decisions.
//!
//! Implemented on top of sled (embedded key-value store). Each entry is
//! stored under a composite key: `<type>:<timestamp_ms>:<nonce>`.
//! This ensures keys are unique and naturally sorted by time.
//!
//! Thread-safety: `AuditLog` itself is NOT Sync (contains Db), so it must
//! be wrapped in `Arc<RwLock<AuditLog>>` at the proxy level.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Single audit log entry — one of these per policy decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// "allowed" or "blocked"
    pub entry_type: String,
    /// Agent wallet address (checksum or lowercase, stored as-is).
    pub from: String,
    /// Destination contract or EOA.
    pub to: String,
    /// Transaction value in wei.
    pub value_wei: String,
    /// Human-readable reason (only for blocked).
    pub reason: Option<String>,
    /// eth_estimateGas revert reason if simulation failed.
    pub revert_reason: Option<String>,
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// TX hash if allowed (filled in after forwarding).
    pub tx_hash: Option<String>,
}

impl AuditEntry {
    fn allowed(from: &str, to: &str, value_wei: &str) -> Self {
        Self {
            entry_type: "allowed".to_string(),
            from: from.to_string(),
            to: to.to_string(),
            value_wei: value_wei.to_string(),
            reason: None,
            revert_reason: None,
            timestamp_ms: now_ms(),
            tx_hash: None,
        }
    }

    fn blocked(from: &str, to: &str, value_wei: &str, reason: &str) -> Self {
        Self {
            entry_type: "blocked".to_string(),
            from: from.to_string(),
            to: to.to_string(),
            value_wei: value_wei.to_string(),
            reason: Some(reason.to_string()),
            revert_reason: None,
            timestamp_ms: now_ms(),
            tx_hash: None,
        }
    }
}

/// AuditLog wraps a sled database. It is not Sync, so callers must
/// wrap it in `Arc<RwLock<AuditLog>>` (done in proxy.rs).
pub struct AuditLog {
    db: Db,
    /// Monotonic nonce to avoid key collisions within the same millisecond.
    nonce: Arc<std::sync::Mutex<u64>>,
}

impl AuditLog {
    /// Open (or create) a sled database at `path`.
    pub fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            db,
            nonce: Arc::new(std::sync::Mutex::new(0)),
        })
    }

    /// Record an allowed transaction.
    pub fn log_allowed(&mut self, from: &str, to: &str, value_wei: &str) -> Result<()> {
        let entry = AuditEntry::allowed(from, to, value_wei);
        let key = self.make_key("allowed", entry.timestamp_ms);
        self.db.insert(key, serde_json::to_vec(&entry)?)?;
        self.db.flush()?;
        Ok(())
    }

    /// Record a blocked transaction.
    pub fn log_blocked(&mut self, from: &str, to: &str, value_wei: &str, reason: &str) -> Result<()> {
        let entry = AuditEntry::blocked(from, to, value_wei, reason);
        let key = self.make_key("blocked", entry.timestamp_ms);
        self.db.insert(key, serde_json::to_vec(&entry)?)?;
        self.db.flush()?;
        Ok(())
    }

    /// Set the tx hash on an allowed entry (called after successful forwarding).
    /// We do this by finding the most recent allowed entry for `from` and patching it.
    /// In production, use the tx hash as part of the key directly.
    pub fn set_tx_hash(&mut self, from: &str, tx_hash: &str) -> Result<()> {
        let from_lower = from.to_lowercase();
        // Iterate in reverse (newest first), find most recent allowed entry for this from.
        for item in self.db.iter().rev() {
            if let Ok((_, v)) = item {
                if let Ok(mut entry) = serde_json::from_slice::<AuditEntry>(&v) {
                    if entry.entry_type == "allowed" && entry.from.to_lowercase() == from_lower {
                        if entry.tx_hash.is_none() {
                            entry.tx_hash = Some(tx_hash.to_string());
                            let key = self.make_key("allowed", entry.timestamp_ms);
                            self.db.insert(key, serde_json::to_vec(&entry)?)?;
                            self.db.flush()?;
                        }
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    /// Retrieve the `limit` most recent audit entries.
    pub fn get_recent(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        let mut entries = Vec::with_capacity(limit);
        for item in self.db.iter().rev().take(limit) {
            if let Ok((_, v)) = item {
                if let Ok(entry) = serde_json::from_slice(&v) {
                    entries.push(entry);
                }
            }
        }
        Ok(entries)
    }

    /// Count total entries.
    pub fn count(&self) -> usize {
        self.db.len()
    }

    /// Build a unique key: `<type>:<timestamp_ms>:<nonce>`.
    fn make_key(&self, entry_type: &str, ts_ms: u64) -> String {
        let mut nonce = self.nonce.lock().unwrap();
        let n = *nonce;
        *nonce = n.wrapping_add(1);
        format!("{}:{}:{}", entry_type, ts_ms, n)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}