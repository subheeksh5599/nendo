//! Configuration loader for Nendo
//!
//! Config file: config.toml in the current working directory.
//! All values also overridable via environment variables with prefix NENDO_.

use std::path::Path;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Avalanche C-Chain RPC URL (public or private).
    pub rpc_url: String,
    /// Chain ID (43113 = Fuji testnet, 43114 = mainnet).
    pub chain_id: u64,
    /// On-chain NendoPolicy contract address (optional).
    #[serde(default = "zero_address")]
    pub policy_contract: String,
    /// On-chain NendoAudit contract address (optional).
    #[serde(default = "zero_address")]
    pub audit_contract: String,
    /// Owner private key (for on-chain governance). Hex with 0x prefix.
    #[serde(default)]
    pub owner_private_key: String,
    /// Interface to bind the proxy HTTP server.
    pub server_host: String,
    /// Port for the proxy HTTP server.
    pub server_port: u16,
    /// Path for the local sled audit database.
    pub audit_path: String,
    /// Default policy rules for agents without a personal override.
    #[serde(default)]
    pub policy: PolicyDefaults,
}

fn zero_address() -> String {
    "0x0000000000000000000000000000000000000000".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyDefaults {
    /// Max AVAX per transaction (wei hex, e.g. "0xde0b6b3a7640000" = 10 AVAX).
    #[serde(default = "default_max_per_tx")]
    pub max_avax_per_tx: String,
    /// Max AVAX per 24h rolling window (wei hex).
    #[serde(default = "default_max_daily")]
    pub max_avax_daily: String,
    /// Minimum milliseconds between transactions per agent.
    #[serde(default = "default_min_interval")]
    pub min_interval_seconds: u64,
    /// If true, reject all transactions unless paused = false.
    #[serde(default)]
    pub paused: bool,
    /// Contracts the agent is allowed to interact with. Empty = allow all.
    #[serde(default)]
    pub allowed_contracts: Vec<String>,
    /// Contracts/EOAs the agent is blocked from sending to.
    #[serde(default)]
    pub blocked_recipients: Vec<String>,
}

fn default_max_per_tx() -> String {
    "0x56bc75e2d63100000".to_string() // 10 AVAX = 0x56bc75e2d63100000
}

fn default_max_daily() -> String {
    "0x3782dace9d900000".to_string()  // 100 AVAX = 0x3782dace9d900000
}

fn default_min_interval() -> u64 {
    5
}

impl Default for PolicyDefaults {
    fn default() -> Self {
        Self {
            max_avax_per_tx: default_max_per_tx(),
            max_avax_daily: default_max_daily(),
            min_interval_seconds: default_min_interval(),
            paused: false,
            allowed_contracts: vec![],
            blocked_recipients: vec![],
        }
    }
}

impl Config {
    /// Load from config.toml, falling back to demo defaults if not found.
    pub fn load() -> Result<Self> {
        let path: Option<String> = if Path::new("config.toml").exists() {
            Some("config.toml".to_string())
        } else if let Ok(p) = std::env::var("NENDO_CONFIG") {
            Some(p)
        } else {
            None
        };

        match path {
            Some(p) => {
                let contents = std::fs::read_to_string(&p)?;
                let config: Config = toml::from_str(&contents)?;
                Ok(config)
            }
            None => {
                tracing::info!("no config.toml found, using demo defaults (Fuji testnet)");
                Ok(Self::default_for_demo())
            }
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    /// Demo configuration pointing at Avalanche Fuji testnet.
    pub fn default_for_demo() -> Self {
        Self {
            rpc_url: "https://api.avax-test.network/ext/bc/C/rpc".to_string(),
            chain_id: 43113,
            policy_contract: zero_address(),
            audit_contract: zero_address(),
            owner_private_key: String::new(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8545,
            audit_path: "./nendo_audit.db".to_string(),
            policy: PolicyDefaults::default(),
        }
    }
}