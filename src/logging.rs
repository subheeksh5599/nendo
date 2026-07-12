//! Local audit log using sled (embedded key-value store)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEntry {
    Allowed {
        from: String,
        to: String,
        value: String,
        timestamp: u64,
    },
    Blocked {
        from: String,
        to: String,
        value: String,
        reason: String,
        timestamp: u64,
    },
}

pub struct AuditLog {
    db: Db,
}

impl AuditLog {
    pub fn open(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    pub fn log_allowed(&mut self, from: &str, to: &str, value: &str) -> Result<()> {
        let entry = AuditEntry::Allowed {
            from: from.to_string(),
            to: to.to_string(),
            value: value.to_string(),
            timestamp: now_ts(),
        };
        let key = format!("allowed:{}:{}", entry.timestamp, uuid_simple());
        self.db.insert(key, serde_json::to_vec(&entry)?)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn log_blocked(&mut self, from: &str, to: &str, value: &str, reason: &str) -> Result<()> {
        let entry = AuditEntry::Blocked {
            from: from.to_string(),
            to: to.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            timestamp: now_ts(),
        };
        let key = format!("blocked:{}:{}", entry.timestamp, uuid_simple());
        self.db.insert(key, serde_json::to_vec(&entry)?)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        let mut entries = vec![];
        for item in self.db.iter().rev().take(limit) {
            if let Ok((_, v)) = item {
                if let Ok(entry) = serde_json::from_slice(&v) {
                    entries.push(entry);
                }
            }
        }
        Ok(entries)
    }
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    format!("{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())
}