//! RPC proxy — intercepts Ethereum JSON-RPC calls and routes through policy engine

use crate::config::Config;
use crate::policy::{PolicyEngine, PolicyResult};
use crate::logging::AuditLog;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use hyper::header::HeaderValue;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct RpcProxy {
    config: Config,
    policy: Arc<PolicyEngine>,
    audit_log: Arc<RwLock<AuditLog>>,
    rpc_client: reqwest::Client,
}

impl RpcProxy {
    pub async fn new(
        config: Config,
        policy: PolicyEngine,
        audit_log: AuditLog,
    ) -> Result<Self> {
        Ok(Self {
            config,
            policy: Arc::new(policy),
            audit_log: Arc::new(RwLock::new(audit_log)),
            rpc_client: reqwest::Client::new(),
        })
    }

    pub async fn handle(&self, req: Request<Body>) -> Result<Response<Body>> {
        // Only handle POST (JSON-RPC)
        if req.method() != hyper::Method::POST {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::empty())?);
        }

        let body = hyper::body::to_bytes(req.into_body()).await?;
        let body_str = String::from_utf8_lossy(&body);

        // Parse JSON-RPC request
        let rpc_request: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(v) => v,
            Err(_) => {
                return self.forward_to_rpc(&body_str).await;
            }
        };

        let method = rpc_request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = rpc_request.get("params");

        match method {
            "eth_sendTransaction" | "eth_sendRawTransaction" => {
                self.handle_send_transaction(method, params, &body_str).await
            }
            _ => self.forward_to_rpc(&body_str).await,
        }
    }

    async fn handle_send_transaction(
        &self,
        method: &str,
        params: Option<&serde_json::Value>,
        raw_body: &str,
    ) -> Result<Response<Body>> {
        // Extract from/to/value from params
        let (from, to, value) = match params.and_then(|p| p.get(0)) {
            Some(tx_params) => {
                let from = tx_params.get("from").and_then(|v| v.as_str()).unwrap_or("0x");
                let to = tx_params.get("to").and_then(|v| v.as_str()).unwrap_or("0x");
                let value = tx_params.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
                (from, to, value)
            }
            None => {
                return self.forward_to_rpc(raw_body).await;
            }
        };

        info!("Nendo intercepting {} from={} to={} value={}", method, from, to, value);

        // Evaluate against policy engine
        let result = self.policy.evaluate(raw_body, from, to, value).await;

        match result {
            PolicyResult::Allowed { simulation } => {
                // Log to audit trail
                let mut audit = self.audit_log.write().await;
                audit.log_allowed(from, to, value).ok();

                info!("✅ Transaction ALLOWED (simulation: {:?})", simulation);
                self.forward_to_rpc(raw_body).await
            }
            PolicyResult::Blocked { reason, rule } => {
                let mut audit = self.audit_log.write().await;
                audit.log_blocked(from, to, value, &reason).ok();

                warn!("🚫 Transaction BLOCKED [{}]: {}", rule, reason);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": format!("Nendo firewall blocked: {}", reason)
                    },
                    "id": 1
                });
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(response.to_string()))?)
            }
            PolicyResult::Escalate { reason, transaction } => {
                warn!("⏸ Transaction ESCALATED: {}", reason);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32001,
                        "message": format!("Nendo requires manual approval: {}", reason)
                    },
                    "id": 1
                });
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(response.to_string()))?)
            }
        }
    }

    async fn forward_to_rpc(&self, body: &str) -> Result<Response<Body>> {
        let resp = self.rpc_client
            .post(&self.config.rpc_url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        let status = StatusCode::from_u16(resp.status().as_u16())?;
        let body = resp.bytes().await?;

        let mut builder = Response::builder().status(status);
        if let Some(ct) = resp.headers().get("Content-Type") {
            builder = builder.header("Content-Type", ct);
        }

        Ok(builder.body(Body::from(body))?)
    }
}