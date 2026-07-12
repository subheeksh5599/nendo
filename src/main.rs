//! Nendo — Agent RPC Firewall for Avalanche
//!
//! Intercepts AI agent transactions, evaluates against on-chain policies,
//! and provides an immutable audit trail on Avalanche C-Chain.

use std::sync::Arc;
use anyhow::Result;
use hyper::{Body, Request, Response, StatusCode};
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::{info, error, warn};

pub mod proxy;
pub mod policy;
pub mod simulation;
pub mod logging;
pub mod config;
pub mod sdk;

pub use config::Config;
pub use policy::PolicyEngine;
pub use proxy::RpcProxy;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nendo=info".parse()?)
        )
        .init();

    let config = Config::load()?;

    info!("Nendo v{} starting on {}", env!("CARGO_PKG_VERSION"), config.server_addr());
    info!("RPC target: {}", config.rpc_url);
    info!("Policy contract: {}", config.policy_contract);
    info!("Audit contract: {}", config.audit_contract);

    // Initialize components
    let policy_engine = PolicyEngine::new(config.clone())?;
    let audit_log = logging::AuditLog::open(&config.audit_path)?;
    let proxy = Arc::new(RpcProxy::new(config.clone(), policy_engine, audit_log).await?);

    // Start HTTP server
    let addr = config.server_addr().parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Proxy listening on http://{}", addr);

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let proxy = proxy.clone();

        tokio::spawn(async move {
            let service = hyper::service::service_fn(move |req: Request<Body>| {
                let proxy = proxy.clone();
                async move {
                    proxy.handle(req).await
                }
            });

            if let Err(e) = http1::Builder::new()
                .preserve_header_case_order(true)
                .serve_connection(io, service)
                .await
            {
                error!("connection error from {}: {}", remote_addr, e);
            }
        });
    }
}