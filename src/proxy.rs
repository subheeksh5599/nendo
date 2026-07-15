//! RPC proxy — transparent JSON-RPC middleware for Avalanche C-Chain
//!
//! Agents send transactions here instead of directly to the Avalanche RPC.
//! Every eth_sendTransaction / eth_sendRawTransaction is evaluated by the
//! policy engine before being forwarded.
//!
//! Supported:
//!   - eth_sendTransaction     — intercepted, policy-checked
//!   - eth_sendRawTransaction  — intercepted, policy-checked (sender via from field)
//!   - All other methods        — forwarded without modification
//!   - JSON-RPC batch           — each item processed individually

use crate::config::Config;
use crate::policy::{PolicyEngine, PolicyResult};
use crate::logging::AuditLog;
use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode, Method};
use hyper::header::CONTENT_LENGTH;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// RPC proxy server. Receives JSON-RPC requests, intercepts transaction
/// methods for policy evaluation, and forwards everything to the upstream RPC.
#[derive(Clone)]
pub struct RpcProxy {
    config: Config,
    policy: Arc<PolicyEngine>,
    audit_log: Arc<RwLock<AuditLog>>,
    rpc_client: reqwest::Client,
    processed: Arc<AtomicU64>,
    blocked: Arc<AtomicU64>,
    start_time: std::time::Instant,
}

impl RpcProxy {
    /// Create a new proxy with the given config, policy engine, and audit log.
    pub fn new(
        config: Config,
        policy: PolicyEngine,
        audit_log: AuditLog,
    ) -> Result<Self> {
        Ok(Self {
            config,
            policy: Arc::new(policy),
            audit_log: Arc::new(RwLock::new(audit_log)),
            rpc_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| anyhow::anyhow!("reqwest init: {}", e))?,
            processed: Arc::new(AtomicU64::new(0)),
            blocked: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
        })
    }

    /// Handle an incoming HTTP request.
    pub async fn handle(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
        // GET /metrics — management endpoint for dashboard
        if *req.method() == Method::GET && req.uri().path() == "/metrics" {
            return self.handle_metrics();
        }

        // Only POST is valid for JSON-RPC
        if *req.method() != hyper::Method::POST {
            return method_not_allowed();
        }

        // Collect the entire body as bytes. We forward the same bytes to
        // the RPC, so we must not consume the body more than once.
        let body_bytes = req.collect().await?.to_bytes();
        let body_len = body_bytes.len();

        // Detect JSON-RPC batch (array) vs single request
        if body_bytes.first() == Some(&b'[') {
            self.handle_batch(body_bytes, body_len).await
        } else {
            self.handle_single(body_bytes, body_len).await
        }
    }

    /// Handle a single JSON-RPC request.
    async fn handle_single(&self, body_bytes: Bytes, _body_len: usize) -> Result<Response<Full<Bytes>>> {
        // Try to parse as JSON-RPC request to extract the method
        let Ok(rpc_req) = serde_json::from_slice::<JsonRpcRequest>(&body_bytes) else {
            // Not valid JSON — forward as-is
            return self.forward_raw(body_bytes.clone()).await;
        };

        match rpc_req.method.as_str() {
            "eth_sendTransaction" | "eth_sendRawTransaction" => {
                let raw_tx = if rpc_req.method == "eth_sendRawTransaction" {
                    rpc_req.params.get(0).and_then(|p| p.as_str()).map(String::from)
                } else {
                    None
                };
                self.handle_send_transaction(&rpc_req.params, rpc_req.id, raw_tx).await
            }
            _ => self.forward_raw(body_bytes.clone()).await,
        }
    }

    /// Handle a JSON-RPC batch request.
    async fn handle_batch(&self, body_bytes: Bytes, _body_len: usize) -> Result<Response<Full<Bytes>>> {
        let requests: Vec<JsonRpcRequest> = match serde_json::from_slice(&body_bytes) {
            Ok(v) => v,
            Err(_) => return self.forward_raw(body_bytes.clone()).await,
        };

        if requests.is_empty() {
            return ok_json(b"[]".as_slice());
        }

        let mut responses: Vec<serde_json::Value> = Vec::with_capacity(requests.len());

        for req in requests {
            match req.method.as_str() {
                "eth_sendTransaction" | "eth_sendRawTransaction" => {
                    let raw_tx = if req.method == "eth_sendRawTransaction" {
                        req.params.get(0).and_then(|p| p.as_str()).map(String::from)
                    } else {
                        None
                    };
                    let result = self.evaluate_transaction(&req.params, raw_tx).await;
                    responses.push(self.build_response(req.id, result));
                }
                _ => {
                    // Forward via a sub-request and collect the response
                    let sub_body = serde_json::to_vec(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": req.method,
                        "params": req.params,
                        "id": req.id
                    }))?;
                    match self.forward_raw(Bytes::copy_from_slice(&sub_body)).await {
                        Ok(resp) => {
                            let body = resp.into_body().collect().await?.to_bytes();
                            if let Ok(v) = serde_json::from_slice(&body) {
                                responses.push(v);
                            }
                        }
                        Err(_) => {
                            responses.push(serde_json::json!({
                                "jsonrpc": "2.0",
                                "error": { "code": -32603, "message": "batch forward failed" },
                                "id": req.id
                            }));
                        }
                    }
                }
            }
        }

        let body = serde_json::to_vec(&responses)?;
        ok_json(&body)
    }

    /// Build a JSON-RPC response from an evaluation result.
    fn build_response(&self, id: serde_json::Value, result: EvaluateResult) -> serde_json::Value {
        match result {
            EvaluateResult::Allowed(tx_hash) => serde_json::json!({
                "jsonrpc": "2.0",
                "result": tx_hash,
                "id": id
            }),
            EvaluateResult::Blocked(code, message) => serde_json::json!({
                "jsonrpc": "2.0",
                "error": { "code": code, "message": message },
                "id": id
            }),
            EvaluateResult::RpcError(code, message, data) => serde_json::json!({
                "jsonrpc": "2.0",
                "error": { "code": code, "message": message, "data": data },
                "id": id
            }),
        }
    }

    /// Intercept eth_sendTransaction or eth_sendRawTransaction.
    async fn handle_send_transaction(
        &self,
        params: &serde_json::Value,
        id: serde_json::Value,
        raw_tx: Option<String>,
    ) -> Result<Response<Full<Bytes>>> {
        let result = self.evaluate_transaction(params, raw_tx).await;
        let body = serde_json::to_vec(&self.build_response(id, result))?;
        ok_json(&body)
    }

    /// Evaluate a transaction through policy and simulation.
    /// Returns an EvaluateResult with tx hash on success, or block/error reason.
    async fn evaluate_transaction(
        &self,
        params: &serde_json::Value,
        raw_tx: Option<String>,
    ) -> EvaluateResult {
        // Extract from/to/value from params[0]
        let Some(tx_params) = params.get(0) else {
            return EvaluateResult::Blocked(-32000, "missing tx params".into());
        };

        let from = tx_params.get("from").and_then(|v| v.as_str()).unwrap_or("0x");
        let to = tx_params.get("to").and_then(|v| v.as_str()).unwrap_or("0x");
        let value_str = tx_params.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
        let data = tx_params.get("data").and_then(|v| v.as_str()).unwrap_or("0x");

        let value_wei = parse_value_hex(value_str);

        // For eth_sendRawTransaction, sender must come from the raw tx itself.
        // Without k-of-n signature recovery we can't do this properly yet,
        // so we require the `from` field to be populated.
        let sender = if from == "0x" && raw_tx.is_some() {
            return EvaluateResult::Blocked(
                -32000,
                "eth_sendRawTransaction requires `from` field for policy evaluation".into(),
            );
        } else {
            from
        };

        // Policy + simulation evaluation
        match self.policy.evaluate(sender, to, value_wei, data).await {
            PolicyResult::Blocked { reason, rule } => {
                self.blocked.fetch_add(1, Ordering::Relaxed);
                let msg = format!("[{}] {}", rule, reason);
                let mut audit = self.audit_log.write().await;
                let _ = audit.log_blocked(sender, to, value_str, &reason);
                EvaluateResult::Blocked(-32000, msg)
            }
            PolicyResult::Escalate { reason } => {
                EvaluateResult::Blocked(-32001, format!("escalated: {reason}"))
            }
            PolicyResult::Allowed { simulation: _ } => {
                // Forward to upstream RPC
                let forward_body = if raw_tx.is_some() {
                    serde_json::to_vec(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "eth_sendRawTransaction",
                        "params": raw_tx.as_ref(),
                        "id": 1
                    }))
                } else {
                    serde_json::to_vec(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "eth_sendTransaction",
                        "params": params,
                        "id": 1
                    }))
                };

                let body_bytes = match forward_body {
                    Ok(b) => b,
                    Err(e) => {
                        return EvaluateResult::RpcError(
                            -32603,
                            format!("serialization error: {e}"),
                            None,
                        );
                    }
                };

                let resp = match self.rpc_client
                    .post(&self.config.rpc_url)
                    .header("Content-Type", "application/json")
                    .body(body_bytes)
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        return EvaluateResult::RpcError(
                            -32002,
                            format!("upstream unreachable: {e}"),
                            None,
                        );
                    }
                };

                let body: serde_json::Value = match resp.json().await {
                    Ok(b) => b,
                    Err(e) => {
                        return EvaluateResult::RpcError(
                            -32003,
                            format!("invalid RPC response: {e}"),
                            None,
                        );
                    }
                };

                // If the RPC itself returned a JSON-RPC error, propagate it
                if let Some(err_obj) = body.get("error") {
                    let code = err_obj
                        .get("code")
                        .and_then(|c| c.as_i64())
                        .unwrap_or(-32000) as i32;
                    let msg = err_obj
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("RPC error")
                        .to_string();
                    let data = err_obj
                        .get("data")
                        .and_then(|d| d.as_str())
                        .map(String::from);
                    return EvaluateResult::RpcError(code, msg, data);
                }

                let tx_hash = body
                    .get("result")
                    .and_then(|r| r.as_str())
                    .map(String::from)
                    .unwrap_or_default();

                // Record to audit log
                let mut audit = self.audit_log.write().await;
                let _ = audit.log_allowed(sender, to, value_str);
                if !tx_hash.is_empty() {
                    let _ = audit.set_tx_hash(sender, &tx_hash);
                }
                drop(audit);

                // Update per-agent spend tracking
                self.policy.record(sender, value_wei).await;
                self.processed.fetch_add(1, Ordering::Relaxed);

                EvaluateResult::Allowed(tx_hash)
            }
        }
    }

    /// Forward raw bytes to the upstream RPC exactly as received.
    async fn forward_raw(&self, body_bytes: Bytes) -> Result<Response<Full<Bytes>>> {
        let resp = self
            .rpc_client
            .post(&self.config.rpc_url)
            .header("Content-Type", "application/json")
            .header(CONTENT_LENGTH, body_bytes.len())
            .body(body_bytes)
            .send()
            .await?;

        let status = StatusCode::from_u16(resp.status().as_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Extract headers before consuming the body
        let content_type = resp.headers().get("Content-Type").cloned();
        let content_length = resp.headers().get(CONTENT_LENGTH).cloned();

        let body_bytes = resp.bytes().await?;

        let mut builder = Response::builder().status(status);
        if let Some(ct) = content_type {
            builder = builder.header("Content-Type", ct);
        }
        if let Some(cl) = content_length {
            builder = builder.header(CONTENT_LENGTH, cl);
        }
        Ok(builder.body(Full::new(body_bytes))?)
    }

    /// GET /metrics — returns proxy stats for the dashboard
    fn handle_metrics(&self) -> Result<Response<Full<Bytes>>> {
        let uptime = self.start_time.elapsed().as_secs();
        let metrics = serde_json::json!({
            "uptime_secs": uptime,
            "processed": self.processed.load(Ordering::Relaxed),
            "blocked": self.blocked.load(Ordering::Relaxed),
            "audit_entries": self.audit_log.try_read().map(|a| a.count()).unwrap_or(0),
        });
        let body = serde_json::to_vec(&metrics)?;
        ok_json(&body)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn ok_json(body: &[u8]) -> Result<Response<Full<Bytes>>> {
    let len = body.len();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_LENGTH, len)
        .body(Full::new(Bytes::copy_from_slice(body)))?)
}

fn method_not_allowed() -> Result<Response<Full<Bytes>>> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Full::new(Bytes::from_static(b"")))?)
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
    id: serde_json::Value,
}

/// Result of transaction evaluation.
#[derive(Debug)]
enum EvaluateResult {
    Allowed(String),
    Blocked(i32, String),
    RpcError(i32, String, Option<String>),
}

/// Parse a wei value string. Accepts hex (0x...) or decimal.
/// Returns u128::MAX on parse failure (effectively blocking the tx).
fn parse_value_hex(value: &str) -> u128 {
    let v = value.trim();
    if v.starts_with("0x") || v.starts_with("0X") {
        u128::from_str_radix(&v[2..], 16).unwrap_or(u128::MAX)
    } else {
        v.parse::<u128>().unwrap_or(u128::MAX)
    }
}

#[allow(dead_code)]
fn wei_to_avax_str(wei: u128) -> String {
    format!("{:.4} AVAX", wei as f64 / 10f64.powi(18))
}

impl From<String> for EvaluateResult {
    fn from(s: String) -> Self { EvaluateResult::Blocked(-32000, s) }
}

impl From<&str> for EvaluateResult {
    fn from(s: &str) -> Self { EvaluateResult::Blocked(-32000, s.to_string()) }
}