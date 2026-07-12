//! Policy engine — evaluates transactions against on-chain and local rules

use crate::config::Config;
use crate::simulation::{SimulationResult, Simulator};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub max_per_tx_wei: String,
    pub max_daily_wei: String,
    pub min_interval_seconds: u64,
    pub allowed_contracts: Vec<String>,
    pub blocked_recipients: Vec<String>,
    pub paused: bool,
}

impl Default for PolicyRule {
    fn default() -> Self {
        Self {
            max_per_tx_wei: "10000000000000000000".to_string(), // 10 AVAX
            max_daily_wei: "100000000000000000000".to_string(),  // 100 AVAX
            min_interval_seconds: 5,
            allowed_contracts: vec![],
            blocked_recipients: vec![],
            paused: false,
        }
    }
}

#[derive(Debug)]
pub enum PolicyResult {
    Allowed {
        simulation: SimulationResult,
    },
    Blocked {
        reason: String,
        rule: &'static str,
    },
    Escalate {
        reason: String,
        transaction: String,
    },
}

pub struct PolicyEngine {
    config: Config,
    rules: Arc<RwLock<PolicyRule>>,
    simulator: Simulator,
}

impl PolicyEngine {
    pub fn new(config: Config) -> Result<Self> {
        let rule = PolicyRule {
            max_per_tx_wei: config.policy.max_avax_per_tx.clone(),
            max_daily_wei: config.policy.max_avax_daily.clone(),
            min_interval_seconds: config.policy.min_interval_seconds,
            allowed_contracts: config.policy.allowed_contracts.clone(),
            blocked_recipients: config.policy.blocked_recipients.clone(),
            paused: config.policy.paused,
        };

        Ok(Self {
            config,
            rules: Arc::new(RwLock::new(rule)),
            simulator: Simulator::new(),
        })
    }

    pub async fn evaluate(&self, tx: &str, from: &str, to: &str, value: &str) -> PolicyResult {
        let rules = self.rules.read().await;

        // 1. Circuit breaker check
        if rules.paused {
            return PolicyResult::Blocked {
                reason: "Firewall is paused".to_string(),
                rule: "circuit_breaker",
            };
        }

        // 2. Recipient blocklist check
        if rules.blocked_recipients.contains(&to.to_lowercase()) {
            return PolicyResult::Blocked {
                reason: "Recipient is on blocklist".to_string(),
                rule: "blocked_recipients",
            };
        }

        // 3. Amount cap check
        let value_bn = parse_wei(value);
        let max_per_tx = parse_wei(&rules.max_per_tx_wei);
        if value_bn > max_per_tx {
            return PolicyResult::Blocked {
                reason: format!(
                    "Value {} exceeds per-tx cap {}",
                    value, rules.max_per_tx_wei
                ),
                rule: "max_per_tx",
            };
        }

        // 4. Allowlist (if non-empty, reject anything not on it)
        if !rules.allowed_contracts.is_empty()
            && !rules.allowed_contracts.contains(&to.to_lowercase())
        {
            return PolicyResult::Blocked {
                reason: "Contract not in allowlist".to_string(),
                rule: "allowed_contracts",
            };
        }

        // 5. Simulation — predict state changes
        let sim = match self.simulator.simulate(tx).await {
            Ok(s) => s,
            Err(e) => {
                warn!("simulation failed: {}", e);
                return PolicyResult::Escalate {
                    reason: format!("Simulation error: {}", e),
                    transaction: tx.to_string(),
                };
            }
        };

        // 6. Balance drain check — don't let a single tx drain more than daily cap
        if sim.net_balance_change > parse_wei(&rules.max_daily_wei) {
            return PolicyResult::Blocked {
                reason: "Transaction would drain balance beyond daily cap".to_string(),
                rule: "balance_drain_protection",
            };
        }

        PolicyResult::Allowed { simulation: sim }
    }

    pub async fn update_rules(&self, new_rules: PolicyRule) {
        let mut rules = self.rules.write().await;
        *rules = new_rules;
        info!("Policy rules updated");
    }
}

fn parse_wei(value: &str) -> u128 {
    // Handle hex (0x...) or decimal
    if value.starts_with("0x") {
        u64::from_str_radix(&value[2..], 16).unwrap_or(0) as u128
    } else {
        value.parse().unwrap_or(0)
    }
}