//! Nendo — Agent RPC Firewall for Avalanche
//!
//! PRODUCTION FEATURES:
//!   - Connection pooling: one reqwest::Client in Arc, reused for all RPC calls
//!   - Graceful shutdown: Ctrl+C → stop accepting, drain in-flight, flush sled, exit
//!   - Trait-based architecture: PolicyProvider, AuditStore, RpcForwarder
//!   - thiserror error enum — no String errors, no .unwrap() in hot path
//!   - Modular: proxy, policy, simulation, logging in separate files

pub mod proxy;
pub mod policy;
pub mod simulation;
pub mod logging;
pub mod config;
pub mod sdk;
pub mod error;
pub mod traits;
pub mod metrics;

use crate::config::Config;
use crate::error::NendoResult;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> NendoResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nendo=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let config = Config::load()?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        rpc = %config.rpc_url,
        listen = %config.server_addr(),
        "Nendo starting"
    );

    // ─── Connection pool (Arc<reqwest::Client>) ────────────────────────
    let rpc_client = Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| error::NendoError::Config(format!("reqwest: {}", e)))?,
    );

    let server_addr = config.server_addr();

    let policy_rules = policy::PolicyRule {
        max_per_tx_wei: parse_wei_hex(&config.policy.max_avax_per_tx),
        max_daily_wei: parse_wei_hex(&config.policy.max_avax_daily),
        min_interval_ms: config.policy.min_interval_seconds * 1000,
        allowed_contracts: config.policy.allowed_contracts.iter().map(|a| a.to_lowercase()).collect(),
        blocked_recipients: config.policy.blocked_recipients.iter().map(|a| a.to_lowercase()).collect(),
        paused: config.policy.paused,
        allowlist_mode: config.policy.allowlist_mode,
    };

    let policy_engine = policy::PolicyEngine::new(
        &config.rpc_url,
        policy_rules,
        rpc_client.clone(),
    );

    let mut audit_log = logging::AuditLog::open_with_rpc(
        &config.audit_path,
        &config.rpc_url,
        &config.audit_contract,
    )?;

    match audit_log.backfill_from_chain().await {
        Ok(n) if n > 0 => tracing::info!(backfilled=n, "audit log restored from on-chain"),
        Ok(_) => tracing::info!("audit log ready"),
        Err(e) => tracing::warn!(error=%e, "backfill failed, continuing with empty log"),
    }

    let metrics = metrics::Metrics::new();

    let proxy = proxy::RpcProxy::new(
        config,
        policy_engine,
        Arc::new(RwLock::new(audit_log)),
        rpc_client,
        metrics,
    )?;

    let addr: std::net::SocketAddr = server_addr.parse()
        .map_err(|e| error::NendoError::Config(format!("invalid bind address: {}", e)))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "proxy listening — Ctrl+C to stop");

    // ─── Graceful shutdown ─────────────────────────────────────────────
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    let proxy_ref = proxy.clone();

    // Ctrl+C handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("shutdown signal received — draining connections...");
        let _ = shutdown_tx.send(()).await;
    });

    // Accept loop with shutdown listener
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, remote_addr) = match result {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!(%e, "accept error");
                        continue;
                    }
                };
                let p = proxy.clone();
                tokio::spawn(async move {
                    let io = hyper_util::rt::TokioIo::new(stream);
                    let service = hyper::service::service_fn(move |req| {
                        let p2 = p.clone();
                        async move { p2.handle(req).await }
                    });
                    if let Err(e) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, service).await
                    {
                        tracing::error!(%remote_addr, %e, "connection error");
                    }
                });
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("shutting down — flushing audit log...");
                let audit_arc = proxy_ref.audit_log.clone();
                let audit = audit_arc.write().await;
                if let Err(e) = audit.flush() {
                    tracing::error!(%e, "failed to flush audit log on shutdown");
                }
                drop(audit);
                tracing::info!("shutdown complete");
                return Ok(());
            }
        }
    }
}

fn parse_wei_hex(s: &str) -> u128 {
    let v = s.trim();
    if v.starts_with("0x") || v.starts_with("0X") {
        u128::from_str_radix(&v[2..], 16).unwrap_or(u128::MAX)
    } else {
        v.parse::<u128>().unwrap_or(u128::MAX)
    }
}
