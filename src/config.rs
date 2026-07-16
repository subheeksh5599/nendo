//! Configuration loader for Nendo

use std::path::Path;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub chain_id: u64,
    #[serde(default = "zero_address")]
    pub policy_contract: String,
    #[serde(default = "zero_address")]
    pub audit_contract: String,
    #[serde(default)]
    pub owner_private_key: String,
    pub server_host: String,
    pub server_port: u16,
    pub audit_path: String,
    pub max_body_size: usize,
    pub rpc_timeout_secs: u64,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub policy: PolicyDefaults,
}

fn zero_address() -> String {
    "0x0000000000000000000000000000000000000000".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyDefaults {
    #[serde(default = "default_max_per_tx")]
    pub max_avax_per_tx: String,
    #[serde(default = "default_max_daily")]
    pub max_avax_daily: String,
    #[serde(default = "default_min_interval")]
    pub min_interval_seconds: u64,
    #[serde(default)]
    pub paused: bool,
    #[serde(default = "default_true")]
    pub allowlist_mode: bool,
    #[serde(default)]
    pub allowed_contracts: Vec<String>,
    #[serde(default)]
    pub blocked_recipients: Vec<String>,
}

fn default_true() -> bool { true }
fn default_max_per_tx() -> String { "0x8ac7230489e80000".to_string() }
fn default_max_daily() -> String { "0x56bc75e2d63100000".to_string() }
fn default_min_interval() -> u64 { 5 }

impl Default for PolicyDefaults {
    fn default() -> Self {
        Self {
            max_avax_per_tx: default_max_per_tx(),
            max_avax_daily: default_max_daily(),
            min_interval_seconds: 5,
            paused: false,
            allowlist_mode: true,
            allowed_contracts: vec![],
            blocked_recipients: vec![],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = if Path::new("config.toml").exists() {
            Some("config.toml".to_string())
        } else if let Ok(p) = std::env::var("NENDO_CONFIG") {
            Some(p)
        } else {
            None
        };
        match path {
            Some(p) => {
                let contents = std::fs::read_to_string(&p)?;
                Ok(toml::from_str(&contents)?)
            }
            None => {
                tracing::info!("no config.toml, using demo defaults");
                Ok(Self::default_for_demo())
            }
        }
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

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
            max_body_size: 1_048_576,
            rpc_timeout_secs: 5,
            api_key: String::new(),
            policy: PolicyDefaults::default(),
        }
    }
}
