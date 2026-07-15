//! Policy engine — evaluates transactions against on-chain and local rules
//!
//! Evaluation order (fail-fast):
//!   1. Circuit breaker (paused flag)
//!   2. Recipient blocklist
//!   3. Per-agent rate limit (sliding window)
//!   4. Per-agent daily spend limit
//!   5. Per-tx amount cap
//!   6. Contract allowlist
//!   7. eth_estimateGas simulation
//!   8. Balance drain check
//!
//! Each agent has its own state (spend, rate) tracked in-memory.
//! The owner can set per-agent overrides via setAgentPolicy.

use crate::simulation::{SimulationResult, Simulator};
use crate::config::PolicyDefaults;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub max_per_tx_wei: u128,
    pub max_daily_wei: u128,
    pub min_interval_ms: u64,
    pub allowed_contracts: Vec<String>,
    pub blocked_recipients: Vec<String>,
    pub paused: bool,
}

impl Default for PolicyRule {
    fn default() -> Self {
        Self {
            max_per_tx_wei: 10 * 10u128.pow(18), // 10 AVAX
            max_daily_wei: 100 * 10u128.pow(18),  // 100 AVAX
            min_interval_ms: 5000,                // 5 seconds
            allowed_contracts: vec![],
            blocked_recipients: vec![],
            paused: false,
        }
    }
}

impl From<PolicyDefaults> for PolicyRule {
    fn from(p: PolicyDefaults) -> Self {
        Self {
            max_per_tx_wei: parse_wei_hex(&p.max_avax_per_tx),
            max_daily_wei: parse_wei_hex(&p.max_avax_daily),
            min_interval_ms: p.min_interval_seconds * 1000,
            allowed_contracts: p.allowed_contracts
                .into_iter()
                .map(|a| normalize_addr(&a))
                .collect(),
            blocked_recipients: p.blocked_recipients
                .into_iter()
                .map(|a| normalize_addr(&a))
                .collect(),
            paused: p.paused,
        }
    }
}

/// Per-agent transaction state for rate limiting and spend tracking.
#[derive(Debug, Clone)]
struct AgentState {
    /// Wall clock time (ms) of the last accepted transaction.
    last_tx_ms: u64,
    /// Total AVAX spent in the current 24-hour window.
    daily_spent_wei: u128,
    /// Start of the current 24-hour window (Unix ms).
    window_start_ms: u64,
}

impl AgentState {
    fn new() -> Self {
        Self {
            last_tx_ms: 0,
            daily_spent_wei: 0,
            window_start_ms: now_ms(),
        }
    }

    fn refresh_window(&mut self) {
        let now = now_ms();
        let day_ms = 86_400_000;
        if now - self.window_start_ms >= day_ms {
            self.daily_spent_wei = 0;
            self.window_start_ms = now;
        }
    }
}

/// Per-agent policy override (set by owner).
#[derive(Debug, Clone)]
pub struct AgentOverride {
    max_per_tx_wei: u128,
    max_daily_wei: u128,
    min_interval_ms: u64,
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
    },
}

pub struct PolicyEngine {
    /// Global fallback rules.
    global_rules: Arc<RwLock<PolicyRule>>,
    /// Per-agent policy overrides (higher priority than global).
    agent_overrides: Arc<RwLock<HashMap<String, AgentOverride>>>,
    /// Per-agent transaction state (rate limit + spend).
    agent_states: Arc<RwLock<HashMap<String, AgentState>>>,
    /// eth_estimateGas simulator.
    simulator: Simulator,
}

impl Clone for PolicyEngine {
    fn clone(&self) -> Self {
        Self {
            global_rules: self.global_rules.clone(),
            agent_overrides: self.agent_overrides.clone(),
            agent_states: self.agent_states.clone(),
            simulator: self.simulator.clone(),
        }
    }
}

impl PolicyEngine {
    pub fn new(rules: PolicyDefaults) -> Self {
        Self {
            global_rules: Arc::new(RwLock::new(rules.into())),
            agent_overrides: Arc::new(RwLock::new(HashMap::new())),
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            simulator: Simulator::new(),
        }
    }

    /// Evaluate a transaction. `from` and `to` must be checksummed or lowercase addresses.
    pub async fn evaluate(
        &self,
        from: &str,
        to: &str,
        value_wei: u128,
        data: &str,
    ) -> PolicyResult {
        let from_addr = normalize_addr(from);
        let to_addr = normalize_addr(to);

        // 1. Global circuit breaker
        {
            let rules = self.global_rules.read().await;
            if rules.paused {
                return PolicyResult::Blocked {
                    reason: "firewall_paused".to_string(),
                    rule: "circuit_breaker",
                };
            }
        }

        // 2. Recipient blocklist
        {
            let rules = self.global_rules.read().await;
            if rules.blocked_recipients.contains(&to_addr) {
                return PolicyResult::Blocked {
                    reason: "recipient_blocklisted".to_string(),
                    rule: "blocked_recipients",
                };
            }
        }

        // 3. Get effective rules for this agent (override or global)
        let (max_per_tx, max_daily, min_interval) = {
            let overrides = self.agent_overrides.read().await;
            if let Some(ov) = overrides.get(&from_addr) {
                (ov.max_per_tx_wei, ov.max_daily_wei, ov.min_interval_ms)
            } else {
                let rules = self.global_rules.read().await;
                (rules.max_per_tx_wei, rules.max_daily_wei, rules.min_interval_ms)
            }
        };

        // 4. Per-agent rate limit
        {
            let mut states = self.agent_states.write().await;
            let state = states.entry(from_addr.clone()).or_insert_with(AgentState::new);
            let now = now_ms();
            if now.saturating_sub(state.last_tx_ms) < min_interval {
                return PolicyResult::Blocked {
                    reason: format!("rate_limit_exceeded_min_interval_{}ms", min_interval),
                    rule: "rate_limit",
                };
            }
        }

        // 5. Per-agent daily spend limit
        {
            let mut states = self.agent_states.write().await;
            let state = states.entry(from_addr.clone()).or_insert_with(AgentState::new);
            state.refresh_window();
            if state.daily_spent_wei + value_wei > max_daily {
                return PolicyResult::Blocked {
                    reason: format!(
                        "daily_limit_exceeded_needed_{}_cap_{}",
                        wei_to_avax_str(state.daily_spent_wei + value_wei),
                        wei_to_avax_str(max_daily)
                    ),
                    rule: "daily_spend_limit",
                };
            }
        }

        // 6. Per-tx amount cap
        if value_wei > max_per_tx {
            return PolicyResult::Blocked {
                reason: format!(
                    "per_tx_limit_exceeded_{}_cap_{}",
                    wei_to_avax_str(value_wei),
                    wei_to_avax_str(max_per_tx)
                ),
                rule: "per_tx_limit",
            };
        }

        // 7. Contract allowlist (if non-empty, everything else is rejected)
        {
            let rules = self.global_rules.read().await;
            if !rules.allowed_contracts.is_empty() && !rules.allowed_contracts.contains(&to_addr) {
                return PolicyResult::Blocked {
                    reason: "contract_not_allowlisted".to_string(),
                    rule: "allowlist",
                };
            }
        }

        // 8. Simulation via eth_estimateGas — detect reverts before forwarding
        let sim = match self.simulator.simulate(from, to, value_wei, data).await {
            Ok(s) => s,
            Err(e) => {
                warn!("simulation failed (will escalate): {}", e);
                return PolicyResult::Escalate {
                    reason: format!("simulation_error_{}", e),
                };
            }
        };

        if !sim.allowed {
            return PolicyResult::Blocked {
                reason: format!("simulation_revert_{}", sim.revert_reason.as_deref().unwrap_or("unknown")),
                rule: "simulation",
            };
        }

        // 9. Balance drain check — ensure agent has enough balance
        if sim.balance_after < value_wei {
            return PolicyResult::Blocked {
                reason: "insufficient_balance".to_string(),
                rule: "balance_check",
            };
        }

        PolicyResult::Allowed { simulation: sim }
    }

    /// Record that a transaction was accepted. Call this after successful forwarding.
    pub async fn record(&self, from: &str, value_wei: u128) {
        let from_addr = normalize_addr(from);
        let mut states = self.agent_states.write().await;
        let state = states.entry(from_addr).or_insert_with(AgentState::new);
        state.last_tx_ms = now_ms();
        state.refresh_window();
        state.daily_spent_wei += value_wei;
    }

    /// Update global rules (owner only — in production, gate with ownership check).
    pub async fn update_global_rules(&self, rules: PolicyRule) {
        let mut global = self.global_rules.write().await;
        *global = rules;
        debug!("global policy rules updated");
    }

    /// Set a per-agent override (owner only).
    pub async fn set_agent_override(&self, agent: &str, override_: AgentOverride) {
        let addr = normalize_addr(agent);
        let mut overrides = self.agent_overrides.write().await;
        overrides.insert(addr, override_);
        debug!("agent policy override set");
    }

    /// Pause / unpause the firewall (owner only).
    pub async fn set_paused(&self, paused: bool) {
        let mut rules = self.global_rules.write().await;
        rules.paused = paused;
        warn!("firewall paused={}", paused);
    }
}

// ─── Utility functions ────────────────────────────────────────────────────────

/// Parse a wei value string. Handles "0x" prefix and decimal.
fn parse_wei_hex(value: &str) -> u128 {
    let v = value.trim();
    if v.starts_with("0x") || v.starts_with("0X") {
        u128::from_str_radix(&v[2..], 16).unwrap_or(u128::MAX)
    } else {
        v.parse::<u128>().unwrap_or(u128::MAX)
    }
}

/// Normalize an address to lowercase for consistent comparisons.
fn normalize_addr(addr: &str) -> String {
    addr.trim().to_lowercase()
}

/// Get current Unix time in milliseconds.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Format wei as AVAX string (e.g. "10.5 AVAX").
fn wei_to_avax_str(wei: u128) -> String {
    let avax = wei as f64 / 10f64.powi(18);
    format!("{:.4} AVAX", avax)
}