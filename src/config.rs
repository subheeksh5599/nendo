//! Configuration loader for Nendo

use std::path::Path;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub chain_id: u64,
    pub policy_contract: String,
    pub audit_contract: String,
    pub owner_private_key: String,
    pub server_host: String,
    pub server_port: u16,
    pub audit_path: String,
    #[serde(default)]
    pub policy: PolicyDefaults,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyDefaults {
    pub max_avax_per_tx: String,
    pub max_avax_daily: String,
    pub min_interval_seconds: u64,
    #[serde(default]
    pub paused: bool,
    #[serde(default]
    pub allowed_contracts: Vec<String>,
    #[serde(default]
    pub blocked_recipients: Vec<String>,
}

impl Default for PolicyDefaults {
    fn default() -> Self {
        Self {
            max_avax_per_tx: "10".to_string(),
            max_avax_daily: "100".to_string(),
            min_interval_seconds: 5,
            paused: false,
            allowed_contracts: vec![],
            blocked_recipients: vec![],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try loading from config.toml in current directory
        let path = if Path::new("config.toml").exists() {
            "config.toml"
        } else if Path::new("NENDO_CONFIG").exists() {
            std::env::var("NENDO_CONFIG")?
        } else {
            // Return defaults for demo
            return Ok(Self::default());
        };

        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    pub fn default_for_demo() -> Self {
        Self {
            rpc_url: "https://api.avax-test.network/ext/bc/C/rpc".to_string(),
            chain_id: 43113,
            policy_contract: "0x0000000000000000000000000000000000000000".to_string(),
            audit_contract: "0x0000000000000000000000000000000000000000".to_string(),
            owner_private_key: "0x".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8545,
            audit_path: "./nendo_audit.db".to_string(),
            policy: PolicyDefaults::default(),
        }
    }
}