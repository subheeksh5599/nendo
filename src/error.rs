//! Nendo error types — proper error enum with thiserror
//!
//! Replaces all `String` errors and `.unwrap()` calls with typed errors.
//! Enables the proxy to be tested with mock PolicyProviders.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NendoError {
    #[error("RPC error: {0}")]
    Rpc(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Sled database error: {0}")]
    Sled(#[from] sled::Error),

    #[error("Policy violation: {rule} — {reason}")]
    PolicyViolation { rule: String, reason: String },

    #[error("Invalid params: {0}")]
    InvalidParams(String),

    #[error("Upstream RPC unreachable: {0}")]
    RpcUnreachable(String),

    #[error("Simulation failed: {0}")]
    SimulationFailed(String),

    #[error("Signature recovery failed: {0}")]
    SignatureRecoveryFailed(String),

    #[error("Transaction blocked: {0}")]
    Blocked(String),

    #[error("Backfill error: {0}")]
    Backfill(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Result alias used throughout the codebase.
pub type NendoResult<T> = Result<T, NendoError>;
