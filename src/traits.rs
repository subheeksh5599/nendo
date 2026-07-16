//! Trait abstractions for testability.
//!
//! Instead of the proxy directly instantiating an RPC client inside the
//! handler, it depends on these traits. In tests, you inject mock
//! implementations. In production, you inject the real ones.

use crate::error::NendoResult;
use async_trait::async_trait;

/// A parsed transaction ready for policy evaluation.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// The cryptographically verified sender (from signature recovery).
    pub sender: String,
    /// The destination address (contract or EOA).
    pub to: String,
    /// Value in wei.
    pub value_wei: u128,
    /// Raw calldata (hex, without 0x prefix).
    pub data: String,
    /// Original hex value string for audit logging.
    pub value_hex: String,
}

/// Result of a simulation.
#[derive(Debug, Clone)]
pub struct SimOutput {
    pub allowed: bool,
    pub revert_reason: Option<String>,
    pub gas_used: u64,
    pub balance_after: u128,
    pub token_drain_detected: Option<String>,
}

/// Policy decision.
#[derive(Debug)]
pub enum PolicyDecision {
    Allowed { sim: SimOutput },
    Blocked { rule: String, reason: String },
    Escalate { reason: String },
}

/// PolicyProvider — checks transactions against rules.
/// Mock this in tests without needing a real Avalanche node.
#[async_trait]
pub trait PolicyProvider: Send + Sync + Clone + 'static {
    /// Evaluate a transaction and record the spend atomically.
    /// Returns the policy decision after full evaluation (simulation + rules).
    async fn evaluate_and_record(&self, tx: &Transaction) -> PolicyDecision;

    /// Set whether the firewall is paused.
    async fn set_paused(&self, paused: bool);

    /// Add a contract to the allowlist.
    async fn add_allowed_contract(&self, contract: &str);

    /// Block a recipient address.
    async fn block_recipient(&self, recipient: &str);
}

/// AuditStore — persists audit entries.
/// Mock this in tests. Production uses sled + on-chain backfill.
#[async_trait]
pub trait AuditStore: Send + Sync + 'static {
    /// Log an allowed transaction.
    async fn log_allowed(&self, sender: &str, to: &str, value_hex: &str);

    /// Log a blocked transaction.
    async fn log_blocked(&self, sender: &str, to: &str, value_hex: &str, reason: &str);

    /// Set the tx hash on the most recent allowed entry for a sender.
    async fn set_tx_hash(&self, sender: &str, tx_hash: &str);

    /// Get the total number of entries.
    fn entry_count(&self) -> usize;

    /// Flush to disk.
    fn flush(&self) -> NendoResult<()>;
}

/// RpcForwarder — sends transactions to the upstream Avalanche RPC.
/// Mock this in tests.
#[async_trait]
pub trait RpcForwarder: Send + Sync + Clone + 'static {
    /// Forward a signed raw transaction.
    async fn send_raw_transaction(&self, raw_hex: &str) -> NendoResult<String>;

    /// Forward an unsigned transaction object.
    async fn send_transaction(&self, params: &serde_json::Value) -> NendoResult<String>;

    /// Forward any JSON-RPC request and return the raw response body.
    async fn forward_raw(&self, body: &[u8]) -> NendoResult<Vec<u8>>;
}
