//! TypeScript SDK for Nendo (to be published as @nendo/sdk)

pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PolicyConfig {
        pub max_per_tx: String,       // in wei (e.g. "10000000000000000000" = 10 AVAX)
        pub max_daily: String,        // in wei
        pub min_interval_seconds: u64,
        pub allowed_contracts: Vec<String>,
        pub blocked_recipients: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SimulationResult {
        pub allowed: bool,
        pub reason: Option<String>,
        pub rule: Option<String>,
        pub net_balance_change: String,
        pub gas_used: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuditEntry {
        pub entry_type: String,
        pub from: String,
        pub to: String,
        pub value: String,
        pub reason: Option<String>,
        pub timestamp: u64,
    }
}

pub struct NendoClient {
    rpc_url: String,
    policy_address: String,
    audit_address: String,
}

impl NendoClient {
    pub fn new(rpc_url: &str, policy_address: &str, audit_address: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            policy_address: policy_address.to_string(),
            audit_address: audit_address.to_string(),
        }
    }

    pub fn utils() -> NendoUtils {
        NendoUtils
    }
}

pub struct NendoUtils;

impl NendoUtils {
    pub fn avax_to_wei(&self, avax: &str) -> String {
        // Simple conversion: avax * 10^18
        let avax_f: f64 = avax.parse().unwrap_or(0.0);
        let wei = (avax_f * 1e18) as u64;
        format!("0x{:x}", wei)
    }
}

// Re-export
pub use types::{AuditEntry, PolicyConfig, SimulationResult};