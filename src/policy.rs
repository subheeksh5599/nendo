//! Policy engine — evaluates transactions against on-chain + local rules.
//!
//! CONCURRENCY: Per-agent Tokio mutex held across check→forward→record.
//! Only one transaction per agent can be in the critical section at a time.
//!
//! FAIL-CLOSED: Every RPC call returns Result; errors propagate as blocked
//! transactions, never as silent forwards.
//!
//! ARCHITECTURE GAP FIX: allowlistMode is enforced BEFORE simulation.
//! If allowlistMode == true and `to` is not in allowedContracts, the tx is
//! rejected WITHOUT calling eth_call. This combined with eth_call's state
//! diff detection closes the estimateGas blindspot completely.

use crate::simulation::{SimulationResult, Simulator};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error};

// ─── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub max_per_tx_wei: u128,
    pub max_daily_wei: u128,
    pub min_interval_ms: u64,
    pub allowed_contracts: Vec<String>,
    pub blocked_recipients: Vec<String>,
    pub paused: bool,
    pub allowlist_mode: bool,
}

impl Default for PolicyRule {
    fn default() -> Self {
        Self {
            max_per_tx_wei: 10_000_000_000_000_000_000, // 10 AVAX
            max_daily_wei: 100_000_000_000_000_000_000, // 100 AVAX
            min_interval_ms: 5_000,
            allowed_contracts: Vec::new(),
            blocked_recipients: Vec::new(),
            paused: false,
            allowlist_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentOverride {
    pub max_per_tx_wei: u128,
    pub max_daily_wei: u128,
    pub min_interval_ms: u64,
}

#[derive(Debug, Clone)]
struct AgentState {
    last_tx_ms: u64,
    daily_spent_wei: u128,
    window_start_ms: u64,
}

impl AgentState {
    fn new() -> Self {
        Self { last_tx_ms: 0, daily_spent_wei: 0, window_start_ms: now_ms() }
    }
    fn refresh_window(&mut self) {
        let now = now_ms();
        if now.saturating_sub(self.window_start_ms) >= 86_400_000 {
            self.daily_spent_wei = 0;
            self.window_start_ms = now;
        }
    }
}

#[derive(Debug)]
pub enum PolicyResult {
    Allowed { simulation: SimulationResult, gas_used: u64 },
    Blocked { reason: String, rule: String },
    Escalate { reason: String },
}

// ─── Engine ──────────────────────────────────────────────────────────────

type AgentLock = Arc<Mutex<()>>;

pub struct PolicyEngine {
    rules: Arc<RwLock<PolicyRule>>,
    agent_overrides: Arc<RwLock<HashMap<String, AgentOverride>>>,
    agent_states: Arc<RwLock<HashMap<String, AgentState>>>,
    agent_locks: Arc<RwLock<HashMap<String, AgentLock>>>,
    simulator: Simulator,
}

impl PolicyEngine {
    pub fn new(rpc_url: &str, rules: PolicyRule, _client: Arc<reqwest::Client>) -> Self {
        Self {
            rules: Arc::new(RwLock::new(rules)),
            agent_overrides: Arc::new(RwLock::new(HashMap::new())),
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            agent_locks: Arc::new(RwLock::new(HashMap::new())),
            simulator: Simulator::new(rpc_url.to_string()),
        }
    }

    async fn get_lock(&self, agent: &str) -> AgentLock {
        let mut locks = self.agent_locks.write().await;
        locks.entry(agent.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Atomic evaluate + record. Holds per-agent lock for the full cycle.
    /// NEVER forwards a transaction blindly — any error returns Blocked.
    pub async fn evaluate_and_record(
        &self,
        from: &str,
        to: &str,
        value_wei: u128,
        data: &str,
    ) -> PolicyResult {
        let from_addr = normalize(from);
        let to_addr = normalize(to);

        // Per-agent lock — serializes all tx for this agent
        let lock = self.get_lock(&from_addr).await;
        let _guard = lock.lock().await;

        // ── 1. Circuit breaker ──
        if self.rules.read().await.paused {
            info!(agent=%from_addr, "blocked: circuit breaker active");
            return PolicyResult::Blocked {
                reason: "firewall_paused".into(),
                rule: "circuit_breaker".into(),
            };
        }

        // ── 2. Recipient blocklist ──
        if self.rules.read().await.blocked_recipients.contains(&to_addr) {
            warn!(agent=%from_addr, recipient=%to_addr, "blocked: recipient blocklisted");
            return PolicyResult::Blocked {
                reason: "recipient_blocklisted".into(),
                rule: "blocklist".into(),
            };
        }

        // ── 3. Contract allowlist (ARCHITECTURE GAP FIX) ──
        // If allowlist mode is on, reject ANY tx to a non-allowlisted contract
        // BEFORE simulation. This closes the estimateGas blindspot entirely.
        {
            let rules = self.rules.read().await;
            if rules.allowlist_mode && !to_addr.is_empty()
                && to_addr != "0x" && to_addr != "0x0000000000000000000000000000000000000000"
                && !rules.allowed_contracts.contains(&to_addr)
            {
                warn!(agent=%from_addr, target=%to_addr, "blocked: contract not in allowlist (allowlist_mode=on)");
                return PolicyResult::Blocked {
                    reason: "contract_not_allowlisted".into(),
                    rule: "allowlist".into(),
                };
            }
        }

        // ── 4. Rate limit ──
        let (max_per_tx, max_daily, min_interval) = {
            let overrides = self.agent_overrides.read().await;
            if let Some(ov) = overrides.get(&from_addr) {
                (ov.max_per_tx_wei, ov.max_daily_wei, ov.min_interval_ms)
            } else {
                let r = self.rules.read().await;
                (r.max_per_tx_wei, r.max_daily_wei, r.min_interval_ms)
            }
        };

        {
            let mut states = self.agent_states.write().await;
            let state = states.entry(from_addr.clone()).or_insert_with(AgentState::new);
            let now = now_ms();
            if now.saturating_sub(state.last_tx_ms) < min_interval {
                let remaining = min_interval - now.saturating_sub(state.last_tx_ms);
                warn!(agent=%from_addr, remaining_ms=remaining, "blocked: rate limit");
                return PolicyResult::Blocked {
                    reason: format!("rate_limit: {}ms remaining", remaining),
                    rule: "rate_limit".into(),
                };
            }
        }

        // ── 5. Daily spend limit ──
        {
            let mut states = self.agent_states.write().await;
            let state = states.entry(from_addr.clone()).or_insert_with(AgentState::new);
            state.refresh_window();
            if state.daily_spent_wei.saturating_add(value_wei) > max_daily {
                warn!(agent=%from_addr, spent=state.daily_spent_wei, cap=max_daily, "blocked: daily limit");
                return PolicyResult::Blocked {
                    reason: format!("daily_limit: spent {} of {}", state.daily_spent_wei, max_daily),
                    rule: "daily_limit".into(),
                };
            }
        }

        // ── 6. Per-tx cap ──
        if value_wei > max_per_tx {
            warn!(agent=%from_addr, value=value_wei, cap=max_per_tx, "blocked: per-tx limit");
            return PolicyResult::Blocked {
                reason: format!("per_tx_limit: {} exceeds cap {}", value_wei, max_per_tx),
                rule: "per_tx_limit".into(),
            };
        }

        // ── 7. Simulation (eth_call + eth_estimateGas) ──
        // FAIL-CLOSED: any simulation error → blocked
        let sim = match self.simulator.simulate(from, to, value_wei, data).await {
            Ok(s) => s,
            Err(e) => {
                error!(agent=%from_addr, error=%e, "simulation RPC error — rejecting (fail-closed)");
                return PolicyResult::Blocked {
                    reason: format!("simulation_error: {}", e),
                    rule: "simulation".into(),
                };
            }
        };

        if !sim.allowed {
            let reason = sim.revert_reason.as_deref().unwrap_or("unknown");
            warn!(agent=%from_addr, reason=%reason, "blocked: simulation failed");
            return PolicyResult::Blocked {
                reason: format!("simulation: {}", reason),
                rule: "simulation".into(),
            };
        }

        // ── 8. Token drain detection ──
        if let Some(ref drain) = sim.unexpected_state_change {
            error!(agent=%from_addr, drain=%drain, "blocked: token drain detected");
            return PolicyResult::Blocked {
                reason: format!("token_drain: {}", drain),
                rule: "token_drain".into(),
            };
        }

        // ── 9. Balance check ──
        if sim.balance_after < value_wei {
            warn!(agent=%from_addr, balance=sim.balance_after, needed=value_wei, "blocked: insufficient balance");
            return PolicyResult::Blocked {
                reason: "insufficient_balance".into(),
                rule: "balance".into(),
            };
        }

        // ── 10. Record spend (still under lock) ──
        {
            let mut states = self.agent_states.write().await;
            let state = states.entry(from_addr.clone()).or_insert_with(AgentState::new);
            state.last_tx_ms = now_ms();
            state.refresh_window();
            state.daily_spent_wei = state.daily_spent_wei.saturating_add(value_wei);
        }

        let gas_used = sim.gas_used;
        info!(agent=%from_addr, to=%to_addr, value=value_wei, gas=gas_used, "allowed");
        PolicyResult::Allowed { simulation: sim, gas_used }
    }

    // ── Admin ─────────────────────────────────────────────────────────

    pub async fn set_paused(&self, paused: bool) {
        self.rules.write().await.paused = paused;
        info!(paused, "circuit breaker toggled");
    }

    pub async fn set_rules(&self, rules: PolicyRule) {
        *self.rules.write().await = rules;
        info!("policy rules updated");
    }

    pub async fn set_agent_override(&self, agent: &str, ov: AgentOverride) {
        self.agent_overrides.write().await.insert(normalize(agent), ov);
        info!(%agent, "agent override set");
    }

    pub async fn add_allowed_contract(&self, contract: &str) {
        self.rules.write().await.allowed_contracts.push(normalize(contract));
        info!(%contract, "contract added to allowlist");
    }

    pub async fn block_recipient(&self, recipient: &str) {
        self.rules.write().await.blocked_recipients.push(normalize(recipient));
        info!(%recipient, "recipient blocklisted");
    }
}

// ─── Cloning ─────────────────────────────────────────────────────────────

impl Clone for PolicyEngine {
    fn clone(&self) -> Self {
        Self {
            rules: self.rules.clone(),
            agent_overrides: self.agent_overrides.clone(),
            agent_states: self.agent_states.clone(),
            agent_locks: self.agent_locks.clone(),
            simulator: self.simulator.clone(),
        }
    }
}

// ─── Utility ─────────────────────────────────────────────────────────────

fn normalize(addr: &str) -> String {
    addr.trim().to_lowercase()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
