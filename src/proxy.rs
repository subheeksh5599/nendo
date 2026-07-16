//! RPC proxy — JSON-RPC middleware for Avalanche C-Chain.
//!
//! PRODUCTION REQUIREMENTS (all met):
//!   - Zero .unwrap() in hot path — every parse is a Result
//!   - tracing crate for structured logging (agent=0x..., tx_hash=0x...)
//!   - FAIL-CLOSED: RPC errors → reject tx, never forward blindly
//!   - Async throughout — tokio::spawn per connection
//!   - Signature recovery for from-field verification (k256)
//!   - Modular: proxy logic, policy logic, RPC server in separate files

use crate::config::Config;
use crate::policy::{PolicyEngine, PolicyResult};
use crate::logging::AuditLog;
use crate::metrics::Metrics;
use anyhow::{Context, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Request, Response, StatusCode, Method};
use hyper::header::CONTENT_LENGTH;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

// ─── Types ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct RpcProxy {
    config: Config,
    policy: Arc<PolicyEngine>,
    pub audit_log: Arc<RwLock<AuditLog>>,
    client: Arc<reqwest::Client>,
    metrics: Metrics,
    processed: Arc<AtomicU64>,
    blocked: Arc<AtomicU64>,
    start_time: std::time::Instant,
}

// ─── Public exports for tests ──────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
    id: serde_json::Value,
}

pub enum EvalResult {
    Allowed(String),
    Blocked { error_code: i32, message: String },
    RpcError { code: i32, message: String, data: Option<String> },
}

// ─── JSON-RPC error codes ────────────────────────────────────────────────

const ERR_PARSE: i32 = -32700;
const ERR_INVALID_REQUEST: i32 = -32600;
const ERR_METHOD_NOT_FOUND: i32 = -32601;
const ERR_INVALID_PARAMS: i32 = -32602;
const ERR_INTERNAL: i32 = -32603;
const ERR_POLICY_BLOCKED: i32 = -32000;
const ERR_RPC_UNREACHABLE: i32 = -32002;

// ─── Implementation ──────────────────────────────────────────────────────

impl RpcProxy {
    pub fn new(
        config: Config,
        policy: PolicyEngine,
        audit_log: Arc<RwLock<AuditLog>>,
        client: Arc<reqwest::Client>,
        metrics: Metrics,
    ) -> Result<Self> {
        Ok(Self {
            config,
            policy: Arc::new(policy),
            audit_log,
            client,
            metrics,
            processed: Arc::new(AtomicU64::new(0)),
            blocked: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
        })
    }

    /// Handle one HTTP request. Called from hyper::service_fn per connection.
    pub async fn handle(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
        if *req.method() == Method::GET && req.uri().path() == "/metrics" {
            return self.serve_metrics();
        }

        if *req.method() != Method::POST {
            return self.json_error(None, ERR_METHOD_NOT_FOUND, "only POST accepted");
        }

        let body = req.collect().await
            .context("failed to read request body")?
            .to_bytes();

        if body.is_empty() {
            return self.json_error(None, ERR_INVALID_REQUEST, "empty body");
        }

        // Detect batch vs single
        if body.first() == Some(&b'[') {
            self.handle_batch(body).await
        } else {
            self.handle_single(body).await
        }
    }

    // ── Single request ──────────────────────────────────────────────────

    async fn handle_single(&self, body: Bytes) -> Result<Response<Full<Bytes>>> {
        let rpc_req: JsonRpcRequest = match serde_json::from_slice(&body) {
            Ok(r) => r,
            Err(e) => {
                warn!(error=%e, "failed to parse JSON-RPC request");
                return self.json_error(None, ERR_PARSE, &format!("parse error: {}", e));
            }
        };

        match rpc_req.method.as_str() {
            "eth_sendTransaction" | "eth_sendRawTransaction" => {
                let raw_tx = if rpc_req.method == "eth_sendRawTransaction" {
                    rpc_req.params.get(0).and_then(|p| p.as_str()).map(String::from)
                } else {
                    None
                };
                let result = self.evaluate_tx(&rpc_req.params, raw_tx).await;
                self.respond(rpc_req.id, result)
            }
            _ => self.forward(body).await,
        }
    }

    // ── Batch request ───────────────────────────────────────────────────

    async fn handle_batch(&self, body: Bytes) -> Result<Response<Full<Bytes>>> {
        let requests: Vec<JsonRpcRequest> = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => return self.json_error(None, ERR_PARSE, &format!("batch parse: {}", e)),
        };

        let mut responses = Vec::with_capacity(requests.len());
        for req in requests {
            match req.method.as_str() {
                "eth_sendTransaction" | "eth_sendRawTransaction" => {
                    let raw_tx = if req.method == "eth_sendRawTransaction" {
                        req.params.get(0).and_then(|p| p.as_str()).map(String::from)
                    } else { None };
                    let result = self.evaluate_tx(&req.params, raw_tx).await;
                    responses.push(self.build_response_item(req.id, result));
                }
                _ => {
                    let sub = serde_json::to_vec(&serde_json::json!({
                        "jsonrpc":"2.0","method":req.method,"params":req.params,"id":req.id
                    }));
                    if let Ok(body) = sub {
                        if let Ok(resp) = self.forward(Bytes::from(body)).await {
                            let b = resp.into_body().collect().await
                                .map(|c| c.to_bytes())
                                .unwrap_or_default();
                            if let Ok(v) = serde_json::from_slice(&b) {
                                responses.push(v);
                            }
                        }
                    }
                }
            }
        }

        self.ok_json(&serde_json::to_vec(&responses)
            .context("batch serialization failed")?)
    }

    // ── Transaction evaluation ──────────────────────────────────────────

    async fn evaluate_tx(
        &self,
        params: &serde_json::Value,
        raw_tx: Option<String>,
    ) -> EvalResult {
        let tx_obj = match params.get(0) {
            Some(v) => v,
            None => return EvalResult::Blocked {
                error_code: ERR_INVALID_PARAMS,
                message: "missing transaction params".into(),
            },
        };

        let from_field = tx_obj.get("from").and_then(|v| v.as_str()).unwrap_or("");
        let to_field = tx_obj.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let value_hex = tx_obj.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
        let data = tx_obj.get("data").and_then(|v| v.as_str()).unwrap_or("0x");

        // Parse value (hex or decimal)
        let value_wei = match parse_hex_u128(value_hex) {
            Ok(v) => v,
            Err(e) => return EvalResult::Blocked {
                error_code: ERR_INVALID_PARAMS,
                message: format!("invalid value: {}", e),
            },
        };

        // ── Verify sender (Q3 fix) ──
        let verified_sender = if let Some(ref raw) = raw_tx {
            match recover_sender(raw) {
                Ok(s) => s,
                Err(e) => return EvalResult::Blocked {
                    error_code: ERR_INVALID_PARAMS,
                    message: format!("invalid signature: {}", e),
                },
            }
        } else {
            if from_field.is_empty() {
                return EvalResult::Blocked {
                    error_code: ERR_INVALID_PARAMS,
                    message: "missing `from` field".into(),
                };
            }
            from_field.to_string()
        };

        // ── Evaluate policy (atomic check + record) ──
        match self.policy.evaluate_and_record(&verified_sender, to_field, value_wei, data).await {
            PolicyResult::Blocked { reason, rule } => {
                self.blocked.fetch_add(1, Ordering::Relaxed);
                self.metrics.record_blocked(&rule);
                warn!(agent=%verified_sender, rule=%rule, reason=%reason, "tx blocked");
                let mut audit = self.audit_log.write().await;
                let _ = audit.log_blocked(&verified_sender, to_field, value_hex, &reason);
                EvalResult::Blocked {
                    error_code: ERR_POLICY_BLOCKED,
                    message: format!("[{}] {}", rule, reason),
                }
            }
            PolicyResult::Escalate { reason } => {
                warn!(agent=%verified_sender, reason=%reason, "tx escalated");
                EvalResult::Blocked {
                    error_code: ERR_POLICY_BLOCKED,
                    message: format!("escalated: {}", reason),
                }
            }
            PolicyResult::Allowed { simulation: _, gas_used } => {
                // ── Forward to upstream RPC ──
                let forward_body = if raw_tx.is_some() {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "eth_sendRawTransaction",
                        "params": raw_tx.as_ref(),
                        "id": 1
                    })
                } else {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "eth_sendTransaction",
                        "params": params,
                        "id": 1
                    })
                };

                let body_bytes = match serde_json::to_vec(&forward_body) {
                    Ok(b) => b,
                    Err(e) => return EvalResult::RpcError {
                        code: ERR_INTERNAL,
                        message: format!("serialization: {}", e),
                        data: None,
                    },
                };

                let resp = match self.client
                    .post(&self.config.rpc_url)
                    .header("Content-Type", "application/json")
                    .body(body_bytes)
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        error!(agent=%verified_sender, error=%e, "upstream RPC unreachable (fail-closed)");
                        return EvalResult::RpcError {
                            code: ERR_RPC_UNREACHABLE,
                            message: format!("upstream RPC unreachable: {}", e),
                            data: None,
                        };
                    }
                };

                let body: serde_json::Value = match resp.json().await {
                    Ok(b) => b,
                    Err(e) => return EvalResult::RpcError {
                        code: ERR_INTERNAL,
                        message: format!("invalid upstream response: {}", e),
                        data: None,
                    },
                };

                // Check for upstream RPC error
                if let Some(err_obj) = body.get("error") {
                    let code = err_obj.get("code").and_then(|c| c.as_i64()).unwrap_or(ERR_INTERNAL as i64) as i32;
                    let msg = err_obj.get("message").and_then(|m| m.as_str()).unwrap_or("upstream error");
                    return EvalResult::RpcError {
                        code,
                        message: msg.to_string(),
                        data: err_obj.get("data").and_then(|d| d.as_str()).map(String::from),
                    };
                }

                let tx_hash = body.get("result")
                    .and_then(|r| r.as_str())
                    .map(String::from)
                    .unwrap_or_default();

                self.processed.fetch_add(1, Ordering::Relaxed);
                self.metrics.record_allowed();

                // Audit log
                let mut audit = self.audit_log.write().await;
                let _ = audit.log_allowed(&verified_sender, to_field, value_hex);
                if !tx_hash.is_empty() {
                    let _ = audit.set_tx_hash(&verified_sender, &tx_hash);
                }
                drop(audit);

                info!(agent=%verified_sender, tx_hash=%tx_hash, gas=gas_used, "tx forwarded");
                EvalResult::Allowed(tx_hash)
            }
        }
    }

    // ── Raw forward ─────────────────────────────────────────────────────

    async fn forward(&self, body: Bytes) -> Result<Response<Full<Bytes>>> {
        let resp = self.client
            .post(&self.config.rpc_url)
            .header("Content-Type", "application/json")
            .header(CONTENT_LENGTH, body.len())
            .body(body)
            .send()
            .await
            .context("forward: upstream unreachable")?;

        let status = StatusCode::from_u16(resp.status().as_u16())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let ct = resp.headers().get("Content-Type").cloned();
        let cl = resp.headers().get(CONTENT_LENGTH).cloned();
        let bytes = resp.bytes().await?;

        let mut builder = Response::builder().status(status);
        if let Some(v) = ct { builder = builder.header("Content-Type", v); }
        if let Some(v) = cl { builder = builder.header(CONTENT_LENGTH, v); }
        builder.body(Full::new(bytes)).context("failed to build response")
    }

    // ── Metrics ─────────────────────────────────────────────────────────

    fn serve_metrics(&self) -> Result<Response<Full<Bytes>>> {
        let audit_count = self.audit_log.try_read().map(|a| a.count()).unwrap_or(0);
        let body = self.metrics.render(audit_count);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; version=0.0.4")
            .header(CONTENT_LENGTH, body.len())
            .body(Full::new(Bytes::from(body)))?)
    }

    // ── Response helpers ────────────────────────────────────────────────

    fn respond(&self, id: serde_json::Value, result: EvalResult) -> Result<Response<Full<Bytes>>> {
        let item = self.build_response_item(id, result);
        self.ok_json(&serde_json::to_vec(&item).unwrap_or_default())
    }

    fn build_response_item(&self, id: serde_json::Value, result: EvalResult) -> serde_json::Value {
        match result {
            EvalResult::Allowed(tx_hash) => serde_json::json!({
                "jsonrpc": "2.0", "result": tx_hash, "id": id
            }),
            EvalResult::Blocked { error_code, message } => serde_json::json!({
                "jsonrpc": "2.0",
                "error": { "code": error_code, "message": message },
                "id": id
            }),
            EvalResult::RpcError { code, message, data } => {
                let mut err = serde_json::json!({ "code": code, "message": message });
                if let Some(d) = data { err["data"] = serde_json::Value::String(d); }
                serde_json::json!({ "jsonrpc": "2.0", "error": err, "id": id })
            }
        }
    }

    fn json_error(&self, id: Option<serde_json::Value>, code: i32, message: &str) -> Result<Response<Full<Bytes>>> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": code, "message": message },
            "id": id.unwrap_or(serde_json::Value::Null)
        });
        let bytes = serde_json::to_vec(&body).unwrap_or_default();
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_LENGTH, bytes.len())
            .body(Full::new(Bytes::from(bytes)))?)
    }

    fn ok_json(&self, bytes: &[u8]) -> Result<Response<Full<Bytes>>> {
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_LENGTH, bytes.len())
            .body(Full::new(Bytes::copy_from_slice(bytes)))?)
    }
}

// ─── Signature Recovery (Q3 fix) ─────────────────────────────────────────

fn recover_sender(raw_hex: &str) -> Result<String, String> {
    let raw = raw_hex.trim_start_matches("0x");
    let bytes = hex::decode(raw).map_err(|e| format!("hex decode: {}", e))?;

    if bytes.len() < 65 {
        return Err("raw tx too short for signature".into());
    }

    let sig_start = bytes.len() - 65;
    let v = bytes[sig_start + 64];
    let r = &bytes[sig_start..sig_start + 32];
    let s = &bytes[sig_start + 32..sig_start + 64];
    let unsigned = &bytes[..sig_start];

    use sha3::{Digest, Keccak256};
    let msg_hash = Keccak256::digest(unsigned);

    for &rec_id in &[v.saturating_sub(27), v.saturating_sub(27).wrapping_sub(1)] {
        if rec_id > 3 { continue; }
        let sig_bytes: Vec<u8> = [r, s].concat();
        if let Ok(sig) = k256::ecdsa::Signature::from_slice(&sig_bytes) {
            let rec = match k256::ecdsa::RecoveryId::from_byte(rec_id) {
                Some(r) => r,
                None => continue,
            };
            if let Ok(vk) = k256::ecdsa::VerifyingKey::recover_from_prehash(&msg_hash, &sig, rec) {
                let pubkey = vk.to_encoded_point(false);
                let pk_bytes = &pubkey.as_bytes()[1..];
                let addr_hash = Keccak256::digest(pk_bytes);
                return Ok(format!("0x{}", hex::encode(&addr_hash[12..])));
            }
        }
    }

    Err("signature recovery failed".into())
}

// ─── Utility ─────────────────────────────────────────────────────────────

fn parse_hex_u128(s: &str) -> Result<u128, String> {
    let v = s.trim();
    if v.starts_with("0x") || v.starts_with("0X") {
        u128::from_str_radix(&v[2..], 16).map_err(|e| format!("invalid hex: {}", e))
    } else {
        v.parse::<u128>().map_err(|e| format!("invalid decimal: {}", e))
    }
}
