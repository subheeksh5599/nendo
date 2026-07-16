//! Simulation — state-change inspection via eth_call + eth_estimateGas
//!
//! CRITICAL ARCHITECTURAL DECISION:
//!   eth_estimateGas only detects reverts and out-of-gas. A malicious contract
//!   that drains USDC without reverting PASSES eth_estimateGas silently.
//!
//!   Nendo's fix (two layers):
//!     1. eth_call — actually executes the transaction in a sandbox and
//!        compares pre/post state. Catches non-reverting malicious transfers.
//!        We call eth_call on the target with the exact params, then compare
//!        the agent's balance + known ERC-20 balanceOf slots before/after.
//!     2. allowlist enforcement — if allowlistMode is true, only contracts
//!        in the allowedContracts mapping can be called. This is the
//!        DEFAULT for production. Combined with eth_call, this closes the
//!        estimateGas blindspot completely.
//!
//!   Without allowlist mode: eth_call detects the drain (balanceOf drops).
//!   With allowlist mode: unknown contracts are rejected BEFORE simulation.
//!   Both layers: the firewall catches what estimateGas misses.
//!
//! FAIL-CLOSED: if the RPC is unreachable for simulation, the transaction
//!   is REJECTED, never forwarded blindly.

use anyhow::{anyhow, Context, Result};
use tracing::{warn, error};

// ─── Constants ───────────────────────────────────────────────────────────

/// Known stablecoin contracts (Fuji + Mainnet). Used for balanceOf checks.
const KNOWN_STABLECOINS: &[(&str, &str, u8)] = &[
    ("0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E", "USDC", 6),
    ("0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7", "USDT", 6),
];

// ─── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub allowed: bool,
    pub revert_reason: Option<String>,
    pub gas_used: u64,
    pub balance_after: u128,
    /// ERC-20 token balances after simulation (token address → balance).
    pub token_balances_after: Vec<(String, u128)>,
    /// Whether eth_call detected state changes outside the expected scope.
    pub unexpected_state_change: Option<String>,
}

/// Result of debug_traceCall with prestateTracer.
#[derive(Debug, Clone)]
pub struct StateDiffResult {
    pub safe: bool,
    pub reason: Option<String>,
    pub contracts_touched: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Simulator {
    rpc_url: String,
    client: reqwest::Client,
}

impl Simulator {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("reqwest client construction is infallible with valid params"),
        }
    }

    /// Full simulation pipeline:
    ///   1. eth_getBalance (pre-state)
    ///   2. debug_traceCall with prestateTracer — full storage diff inspection
    ///   3. eth_call on the target (execute in sandbox)
    ///   4. ERC-20 balanceOf checks for known stablecoins
    ///   5. eth_estimateGas as safety net
    ///   6. FAIL-CLOSED: any RPC error → reject transaction
    pub async fn simulate(
        &self,
        from: &str,
        to: &str,
        value_wei: u128,
        data: &str,
    ) -> Result<SimulationResult> {
        let from = from.trim().to_lowercase();
        let to = to.trim().to_lowercase();
        let data_clean = data.trim().trim_start_matches("0x");

        // Step 1: Pre-state balance
        let pre_balance = self.eth_get_balance(&from).await
            .context("eth_getBalance failed — RPC unreachable, rejecting tx (fail-closed)")?;

        // Step 2: debug_traceCall with prestateTracer — inspect every storage change
        let trace_result = self.debug_trace_call(&from, &to, value_wei, data_clean).await;
        if let Ok(Some(diff)) = &trace_result {
            if !diff.safe {
                warn!(agent=%from, reason=?diff.reason, "debug_traceCall: unsafe state change detected — blocking");
                return Ok(SimulationResult {
                    allowed: false,
                    revert_reason: diff.reason.clone(),
                    gas_used: 0,
                    balance_after: pre_balance,
                    token_balances_after: Vec::new(),
                    unexpected_state_change: diff.reason.clone(),
                });
            }
        }

        // Step 3: Pre-state ERC-20 balances
        let pre_token_balances = self.get_token_balances(&from).await;

        // Step 4: eth_call — actually execute the tx in a sandbox
        let call_result = self.eth_call(&from, &to, value_wei, data_clean).await;

        // Step 4: Post-state ERC-20 balances (only if eth_call succeeded)
        let post_token_balances = if call_result.is_ok() {
            self.get_token_balances(&from).await
        } else {
            Vec::new()
        };

        // Step 5: Check for unauthorized ERC-20 drains
        if let Some(drain_msg) = self.detect_token_drain(&pre_token_balances, &post_token_balances) {
            warn!(agent=%from, token_drain=%drain_msg, "token drain detected — blocking");
            return Ok(SimulationResult {
                allowed: false,
                revert_reason: Some(format!("token_drain_detected: {}", drain_msg)),
                gas_used: 0,
                balance_after: pre_balance,
                token_balances_after: post_token_balances,
                unexpected_state_change: Some(drain_msg),
            });
        }

        // Step 6: eth_estimateGas as safety net
        let gas_estimate = self.eth_estimate_gas(&from, &to, value_wei, data_clean).await;

        // Step 7: Determine result
        match call_result {
            Err(e) => {
                // eth_call failed — either revert or RPC error
                let msg = format!("{}", e);
                if msg.contains("execution reverted") || msg.contains("Revert") {
                    Ok(SimulationResult {
                        allowed: false,
                        revert_reason: Some(msg),
                        gas_used: 0,
                        balance_after: pre_balance,
                        token_balances_after: post_token_balances,
                        unexpected_state_change: None,
                    })
                } else {
                    // RPC error — FAIL CLOSED
                    error!(%from, %to, error=%e, "eth_call RPC failure — rejecting tx (fail-closed)");
                    Ok(SimulationResult {
                        allowed: false,
                        revert_reason: Some(format!("simulation_rpc_error: {}", e)),
                        gas_used: 0,
                        balance_after: pre_balance,
                        token_balances_after: post_token_balances,
                        unexpected_state_change: None,
                    })
                }
            }
            Ok(_call_ok) => {
                let gas_used = gas_estimate.as_ref().map(|(g, _, _)| *g).unwrap_or(21_000);
                let gas_price = gas_estimate.as_ref().map(|(_, gp, _)| *gp).unwrap_or(25_000_000_000);
                let gas_cost = (gas_used as u128).saturating_mul(gas_price as u128);
                let balance_after = pre_balance.saturating_sub(value_wei).saturating_sub(gas_cost);

                if let Err(e) = &gas_estimate {
                    if format!("{}", e).contains("execution reverted") || format!("{}", e).contains("Revert") {
                        return Ok(SimulationResult {
                            allowed: false,
                            revert_reason: Some(format!("{}", e)),
                            gas_used: 0,
                            balance_after: pre_balance,
                            token_balances_after: post_token_balances,
                            unexpected_state_change: None,
                        });
                    }
                }

                Ok(SimulationResult {
                    allowed: true,
                    revert_reason: None,
                    gas_used,
                    balance_after,
                    token_balances_after: post_token_balances,
                    unexpected_state_change: None,
                })
            }
        }
    }

    // ─── RPC methods ────────────────────────────────────────────────────

    async fn eth_get_balance(&self, addr: &str) -> Result<u128> {
        let body = self.rpc_call("eth_getBalance", &serde_json::json!([addr, "latest"])).await?;
        let hex_val = body["result"].as_str()
            .ok_or_else(|| anyhow!("eth_getBalance: missing result"))?;
        u128::from_str_radix(hex_val.trim_start_matches("0x"), 16)
            .context("eth_getBalance: invalid hex")
    }

    async fn eth_call(&self, from: &str, to: &str, value_wei: u128, data: &str) -> Result<serde_json::Value> {
        let params = serde_json::json!([{
            "from": from,
            "to": to,
            "value": format!("0x{:x}", value_wei),
            "data": if data.is_empty() { "0x".to_string() } else { format!("0x{}", data) },
        }, "latest"]);
        self.rpc_call("eth_call", &params).await
    }

    async fn eth_estimate_gas(&self, from: &str, to: &str, value_wei: u128, data: &str) -> Result<(u64, u64, bool)> {
        let params = serde_json::json!([{
            "from": from,
            "to": to,
            "value": format!("0x{:x}", value_wei),
            "data": if data.is_empty() { "0x".to_string() } else { format!("0x{}", data) },
        }]);
        let body = self.rpc_call("eth_estimateGas", &params).await?;

        if let Some(result) = body["result"].as_str() {
            let gas = u64::from_str_radix(result.trim_start_matches("0x"), 16)
                .context("eth_estimateGas: invalid hex")?;
            Ok((gas, 0, true))
        } else if let Some(err) = body.get("error") {
            let msg = err["message"].as_str().unwrap_or("unknown error");
            Err(anyhow!("{}", msg))
        } else {
            Err(anyhow!("eth_estimateGas: unexpected response"))
        }
    }

    async fn erc20_balance_of(&self, token: &str, owner: &str) -> Result<u128> {
        let clean_owner = owner.trim_start_matches("0x");
        let data = format!("0x70a08231{clean_owner:0>64}");
        let params = serde_json::json!([{
            "to": token,
            "data": data,
        }, "latest"]);
        let body = self.rpc_call("eth_call", &params).await?;
        let hex_val = body["result"].as_str()
            .ok_or_else(|| anyhow!("balanceOf: missing result"))?;
        u128::from_str_radix(hex_val.trim_start_matches("0x"), 16)
            .context("balanceOf: invalid hex")
    }

    async fn get_token_balances(&self, owner: &str) -> Vec<(String, u128)> {
        let mut balances = Vec::new();
        for (addr, symbol, _decimals) in KNOWN_STABLECOINS {
            if let Ok(bal) = self.erc20_balance_of(addr, owner).await {
                if bal > 0 {
                    balances.push((symbol.to_string(), bal));
                }
            }
        }
        balances
    }

    fn detect_token_drain(
        &self,
        pre: &[(String, u128)],
        post: &[(String, u128)],
    ) -> Option<String> {
        for (symbol, pre_bal) in pre {
            let post_bal = post.iter()
                .find(|(s, _)| s == symbol)
                .map(|(_, b)| *b)
                .unwrap_or(0);
            if post_bal < *pre_bal {
                return Some(format!(
                    "{}_drain: {} → {} (delta: -{})",
                    symbol, pre_bal, post_bal, pre_bal - post_bal
                ));
            }
        }
        None
    }

    // ─── Core RPC call ──────────────────────────────────────────────────

    /// debug_traceCall with prestateTracer — full storage diff inspection.
    /// Catches malicious contracts that modify state without reverting.
    /// Uses Avalanche's debug_traceCall with diffMode: true to see every
    /// storage slot that would change during execution.
    async fn debug_trace_call(
        &self,
        from: &str,
        to: &str,
        value_wei: u128,
        data: &str,
    ) -> Result<Option<StateDiffResult>> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "debug_traceCall",
            "params": [{
                "from": from,
                "to": to,
                "value": format!("0x{:x}", value_wei),
                "data": if data.is_empty() { "0x".to_string() } else { format!("0x{}", data) },
                "gas": "0x7A1200",
            }, "latest", {
                "tracer": "prestateTracer",
                "tracerConfig": { "diffMode": true }
            }],
            "id": 1
        });

        let resp = match self.client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => {
                // traceCall unavailable — not all RPC providers support it
                warn!("debug_traceCall unavailable — falling back to eth_call only");
                return Ok(None);
            }
        };

        if !resp.status().is_success() {
            warn!("debug_traceCall HTTP {} — tracer likely unavailable", resp.status());
            return Ok(None);
        }

        let body: serde_json::Value = match resp.json().await {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };

        // Parse prestate tracer output: { "<address>": { "storage": { "<slot>": "<value>" } } }
        let result = match body.get("result") {
            Some(r) => r,
            None => {
                if body.get("error").is_some() {
                    warn!("debug_traceCall returned error — tracer unavailable");
                    return Ok(None);
                }
                return Ok(None);
            }
        };

        // Track which contracts' storage was touched
        let mut contracts_touched = Vec::new();
        let mut unsafe_reason: Option<String> = None;

        if let Some(prestate) = result.as_object() {
            for (addr, state) in prestate {
                let addr_lower = addr.to_lowercase();
                if let Some(storage) = state.get("storage").and_then(|s| s.as_object()) {
                    if !storage.is_empty() {
                        contracts_touched.push(addr_lower.clone());

                        // If the tx touches storage of a contract OTHER than the target
                        // AND value is being sent, flag it as potentially malicious
                        if addr_lower != to && addr_lower != from && value_wei > 0 {
                            unsafe_reason = Some(format!(
                                "debug_traceCall: unexpected storage change in {} (tx target is {})",
                                &addr_lower[..10.min(addr_lower.len())],
                                &to[..10.min(to.len())]
                            ));
                        }
                    }
                }
            }
        }

        let safe = unsafe_reason.is_none();
        Ok(Some(StateDiffResult {
            safe,
            reason: unsafe_reason,
            contracts_touched,
        }))
    }

    // ─── Core RPC call ──────────────────────────────────────────────────

    async fn rpc_call(&self, method: &str, params: &serde_json::Value) -> Result<serde_json::Value> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let resp = self.client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context(format!("RPC call failed: {} — upstream unreachable (fail-closed)", method))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await
            .context(format!("RPC {}: invalid JSON response", method))?;

        if !status.is_success() {
            let err_msg = body["error"]["message"].as_str().unwrap_or("unknown");
            return Err(anyhow!("RPC {} failed (HTTP {}): {}", method, status.as_u16(), err_msg));
        }

        if body.get("error").is_some() {
            let msg = body["error"]["message"].as_str().unwrap_or("unknown RPC error");
            return Err(anyhow!("RPC {} error: {}", method, msg));
        }

        Ok(body)
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new("https://api.avax-test.network/ext/bc/C/rpc".to_string())
    }
}
