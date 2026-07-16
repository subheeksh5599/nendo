//! Nendo metrics — Prometheus-compatible counters for observability.
//!
//! Exposed at GET /metrics on the proxy. Scrapable by Prometheus/Grafana.
//! Shows: tx_allowed_count, tx_blocked_count, tx_blocked_by_rule,
//!        proxy_uptime_seconds, audit_entries_count.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct Metrics {
    pub tx_allowed: Arc<AtomicU64>,
    pub tx_blocked: Arc<AtomicU64>,
    pub tx_blocked_circuit_breaker: Arc<AtomicU64>,
    pub tx_blocked_rate_limit: Arc<AtomicU64>,
    pub tx_blocked_daily_limit: Arc<AtomicU64>,
    pub tx_blocked_per_tx_limit: Arc<AtomicU64>,
    pub tx_blocked_allowlist: Arc<AtomicU64>,
    pub tx_blocked_simulation: Arc<AtomicU64>,
    pub tx_blocked_token_drain: Arc<AtomicU64>,
    pub tx_blocked_balance: Arc<AtomicU64>,
    pub start_time: std::time::Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            tx_allowed: Arc::new(AtomicU64::new(0)),
            tx_blocked: Arc::new(AtomicU64::new(0)),
            tx_blocked_circuit_breaker: Arc::new(AtomicU64::new(0)),
            tx_blocked_rate_limit: Arc::new(AtomicU64::new(0)),
            tx_blocked_daily_limit: Arc::new(AtomicU64::new(0)),
            tx_blocked_per_tx_limit: Arc::new(AtomicU64::new(0)),
            tx_blocked_allowlist: Arc::new(AtomicU64::new(0)),
            tx_blocked_simulation: Arc::new(AtomicU64::new(0)),
            tx_blocked_token_drain: Arc::new(AtomicU64::new(0)),
            tx_blocked_balance: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_allowed(&self) {
        self.tx_allowed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_blocked(&self, rule: &str) {
        self.tx_blocked.fetch_add(1, Ordering::Relaxed);
        match rule {
            "circuit_breaker" => self.tx_blocked_circuit_breaker.fetch_add(1, Ordering::Relaxed),
            "rate_limit" => self.tx_blocked_rate_limit.fetch_add(1, Ordering::Relaxed),
            "daily_limit" => self.tx_blocked_daily_limit.fetch_add(1, Ordering::Relaxed),
            "per_tx_limit" => self.tx_blocked_per_tx_limit.fetch_add(1, Ordering::Relaxed),
            "allowlist" => self.tx_blocked_allowlist.fetch_add(1, Ordering::Relaxed),
            "simulation" => self.tx_blocked_simulation.fetch_add(1, Ordering::Relaxed),
            "token_drain" => self.tx_blocked_token_drain.fetch_add(1, Ordering::Relaxed),
            "balance" => self.tx_blocked_balance.fetch_add(1, Ordering::Relaxed),
            _ => 0u64
        };
    }

    /// Render Prometheus text format.
    pub fn render(&self, audit_entries: usize) -> String {
        let uptime = self.start_time.elapsed().as_secs();
        let mut out = String::new();

        out.push_str(&format!("# HELP nendo_tx_allowed_total Total allowed transactions\n"));
        out.push_str(&format!("# TYPE nendo_tx_allowed_total counter\n"));
        out.push_str(&format!("nendo_tx_allowed_total {}\n", self.tx_allowed.load(Ordering::Relaxed)));

        out.push_str(&format!("# HELP nendo_tx_blocked_total Total blocked transactions\n"));
        out.push_str(&format!("# TYPE nendo_tx_blocked_total counter\n"));
        out.push_str(&format!("nendo_tx_blocked_total {}\n", self.tx_blocked.load(Ordering::Relaxed)));

        let mut emit = |name: &str, val: u64| {
            out.push_str(&format!("# HELP nendo_{} Blocked by {} rule\n", name, name));
            out.push_str(&format!("# TYPE nendo_{} counter\n", name));
            out.push_str(&format!("nendo_{} {}\n", name, val));
        };
        emit("tx_blocked_circuit_breaker", self.tx_blocked_circuit_breaker.load(Ordering::Relaxed));
        emit("tx_blocked_rate_limit", self.tx_blocked_rate_limit.load(Ordering::Relaxed));
        emit("tx_blocked_daily_limit", self.tx_blocked_daily_limit.load(Ordering::Relaxed));
        emit("tx_blocked_per_tx_limit", self.tx_blocked_per_tx_limit.load(Ordering::Relaxed));
        emit("tx_blocked_allowlist", self.tx_blocked_allowlist.load(Ordering::Relaxed));
        emit("tx_blocked_simulation", self.tx_blocked_simulation.load(Ordering::Relaxed));
        emit("tx_blocked_token_drain", self.tx_blocked_token_drain.load(Ordering::Relaxed));
        emit("tx_blocked_balance", self.tx_blocked_balance.load(Ordering::Relaxed));

        out.push_str(&format!("# HELP nendo_uptime_seconds Proxy uptime in seconds\n"));
        out.push_str(&format!("# TYPE nendo_uptime_seconds gauge\n"));
        out.push_str(&format!("nendo_uptime_seconds {}\n", uptime));

        out.push_str(&format!("# HELP nendo_audit_entries Audit log entry count\n"));
        out.push_str(&format!("# TYPE nendo_audit_entries gauge\n"));
        out.push_str(&format!("nendo_audit_entries {}\n", audit_entries));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_records_blocked_tx() {
        let m = Metrics::new();
        m.record_blocked("circuit_breaker");
        m.record_blocked("rate_limit");
        m.record_allowed();

        let rendered = m.render(42);
        assert!(rendered.contains("nendo_tx_allowed_total 1"));
        assert!(rendered.contains("nendo_tx_blocked_total 2"));
        assert!(rendered.contains("nendo_tx_blocked_circuit_breaker 1"));
        assert!(rendered.contains("nendo_tx_blocked_rate_limit 1"));
        assert!(rendered.contains("nendo_audit_entries 42"));
    }

    #[test]
    fn test_metrics_uptime_tracks() {
        let m = Metrics::new();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let rendered = m.render(0);
        assert!(rendered.contains("nendo_uptime_seconds "));
        // Should be >= 0 (uptime at least 0 seconds)
        assert!(rendered.contains("nendo_uptime_seconds 0") || rendered.contains("nendo_uptime_seconds 1"));
    }
}
