//! Nendo — Agent RPC Firewall for Avalanche
//!
//! Intercepts AI agent transactions, evaluates against on-chain and local rules,
//! and provides an immutable audit trail on Avalanche C-Chain.
//!
//! Architecture:
//!   [Agent] → (HTTP) → [Nendo Proxy] → (policy check) → [Avalanche RPC]
//!                         ↓
//!                   [Policy Engine] ← → [Audit Log (sled)]
//!                         ↓
//!                  [Simulator (eth_estimateGas)]

pub mod proxy;
pub mod policy;
pub mod simulation;
pub mod logging;
pub mod config;
pub mod sdk;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("nendo=info".parse()?),
        )
        .with_target(false)
        .init();

    let config = Config::load().unwrap_or_else(|_| Config::default_for_demo());

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        rpc = %config.rpc_url,
        listen = %config.server_addr(),
        "Nendo starting"
    );

    // Extract values from config before moving it
    let server_addr = config.server_addr();

    let policy_engine = policy::PolicyEngine::new(config.policy.clone());
    let audit_log = logging::AuditLog::open(&config.audit_path)?;
    let proxy = proxy::RpcProxy::new(
        config,
        policy_engine,
        audit_log,
    )?;

    let addr: std::net::SocketAddr = server_addr.parse()?;
    tracing::info!(%addr, "proxy listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let proxy = proxy.clone();

        tokio::spawn(async move {
            let io = hyper_util::rt::TokioIo::new(stream);

            let service = hyper::service::service_fn(move |req| {
                let p = proxy.clone();
                async move { p.handle(req).await }
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                tracing::error!(%remote_addr, %e, "connection error");
            }
        });
    }
}

// Re-export for convenience
pub use config::Config;