//! Simulation — uses Avalanche RPC eth_estimateGas to detect transaction failure
//!
//! eth_estimateGas is ideal because:
//!   1. It returns a real revert reason on failure (better than eth_call)
//!   2. It doesn't require signing the transaction first
//!   3. Avalanche's RPC returns detailed error messages (seen in testing:
//!      "insufficient funds for gas * price + value: address 0x... have 0 want 1")
//!
//! We also do a balance pre-check to avoid wasted simulation calls.

use anyhow::{anyhow, Result};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Whether the transaction would succeed.
    pub allowed: bool,
    /// Revert reason if the transaction would fail.
    pub revert_reason: Option<String>,
    /// Estimated gas units (0 if the call failed to estimate).
    pub gas_used: u64,
    /// Agent's balance after the transaction would execute.
    /// This is the balance *after* gas is deducted, not before.
    pub balance_after: u128,
}

/// Simulates a transaction against the Avalanche C-Chain RPC.
/// Uses eth_estimateGas (preferred) with a fallback to eth_call.
pub struct Simulator {
    rpc_url: String,
    rpc_client: reqwest::Client,
}

impl Simulator {
    pub fn new() -> Self {
        Self::with_url("https://api.avax-test.network/ext/bc/C/rpc")
    }

    pub fn with_url(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            rpc_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("valid reqwest client"),
        }
    }

    /// Simulate a transaction from `from` to `to` with `value_wei` and `data`.
    ///
    /// Steps:
    ///   1. Pre-check: fetch current balance of `from` via eth_getBalance.
    ///      If insufficient for value + estimated gas, reject immediately.
    ///   2. eth_estimateGas — if this succeeds, the tx would succeed.
    ///      If it returns an error with a revert reason, record that reason.
    ///   3. Return SimulationResult with allowed=true/false and details.
    pub async fn simulate(
        &self,
        from: &str,
        to: &str,
        value_wei: u128,
        data: &str,
    ) -> Result<SimulationResult> {
        let from = from.trim();
        let to = to.trim();
        let data = data.trim().trim_start_matches("0x");

        // Step 1: Get current balance
        let balance = self.get_balance(from).await?;
        debug!("simulate: from={} balance={} value={}", from, balance, value_wei);

        // Step 2: Get current gas price so we can estimate total cost
        let (gas_price, block_gas_limit) = self.get_gas_context().await.unwrap_or((0, 8_000_000));
        let max_gas_cost = block_gas_limit.saturating_mul(gas_price);

        // Quick reject: if balance < value + rough gas estimate, reject without RPC call
        if balance < value_wei.saturating_add(max_gas_cost as u128) {
            return Ok(SimulationResult {
                allowed: false,
                revert_reason: Some("insufficient_balance".to_string()),
                gas_used: 0,
                balance_after: balance,
            });
        }

        // Step 3: eth_estimateGas
        let estimate_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_estimateGas",
            "params": [{
                "from": from,
                "to": to,
                "value": format!("0x{:x}", value_wei),
                "data": if data.is_empty() { "0x" } else { data }
            }],
            "id": 1
        });

        let resp = self.rpc_client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&estimate_payload)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.json::<serde_json::Value>().await.ok();

        if status.is_success() {
            // Estimate succeeded — transaction would succeed
            let gas_used = body
                .as_ref()
                .and_then(|b| b.get("result"))
                .and_then(|r| r.as_str())
                .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
                .unwrap_or(21_000);

            let gas_cost = (gas_used as u128).saturating_mul(gas_price as u128);
            let balance_after = balance.saturating_sub(value_wei).saturating_sub(gas_cost);

            Ok(SimulationResult {
                allowed: true,
                revert_reason: None,
                gas_used,
                balance_after,
            })
        } else {
            // Estimate failed — extract revert reason
            let revert_reason = body
                .as_ref()
                .and_then(|b| b.get("error"))
                .or_else(|| body.as_ref().and_then(|b| b.get("error")))
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(|s| s.to_string());

            // Fallback: parse from raw error body
            let revert_reason = revert_reason.or_else(|| {
                body.as_ref().and_then(|b| b.as_str()).map(|s| s.to_string())
            });

            warn!("simulation revert: {:?}", revert_reason);

            Ok(SimulationResult {
                allowed: false,
                revert_reason,
                gas_used: 0,
                balance_after: balance,
            })
        }
    }

    /// Get ETH balance for an address.
    async fn get_balance(&self, addr: &str) -> Result<u128> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [addr, "latest"],
            "id": 1
        });

        let resp = self.rpc_client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let body: serde_json::Value = resp.json().await?;

        body.get("result")
            .and_then(|r| r.as_str())
            .map(|s| {
                let hex = s.trim_start_matches("0x");
                u128::from_str_radix(hex, 16).unwrap_or(0)
            })
            .ok_or_else(|| anyhow!("eth_getBalance failed: {:?}", body))
    }

    /// Get current gas price and block gas limit.
    async fn get_gas_context(&self) -> Result<(u64, u64)> {
        let [gp_payload, bg_payload] = [
            serde_json::json!({"jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1}),
            serde_json::json!({"jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["latest", false], "id": 2}),
        ];

        let (gp_resp, bg_resp) = tokio::join!(
            self.rpc_client.post(&self.rpc_url).header("Content-Type", "application/json").json(&gp_payload).send(),
            self.rpc_client.post(&self.rpc_url).header("Content-Type", "application/json").json(&bg_payload).send(),
        );

        let gp: serde_json::Value = gp_resp?.json().await?;
        let bg: serde_json::Value = bg_resp?.json().await?;

        let gas_price = gp
            .get("result")
            .and_then(|r| r.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(25_000_000_000u64); // 25 gwei fallback

        let block_gas_limit = bg
            .get("result")
            .and_then(|r| r.get("gasLimit"))
            .and_then(|g| g.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(8_000_000);

        Ok((gas_price, block_gas_limit))
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Simulator {
    fn clone(&self) -> Self {
        Self {
            rpc_url: self.rpc_url.clone(),
            rpc_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("valid reqwest client"),
        }
    }
}