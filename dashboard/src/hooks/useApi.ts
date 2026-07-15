import { useState, useEffect, useCallback } from "react";

const API_BASE = import.meta.env.PROD ? "" : (import.meta.env.VITE_API_URL || "http://localhost:4000");

interface ChainData {
  blockNumber: number | null;
  gasPrice: string | null;
  chainId: number;
  network: string;
}

export interface Stats {
  uptime: string;
  processedToday: number;
  blockedToday: number;
  blockRatio: string;
  chain: ChainData;
  proxyOnline: boolean;
}

export interface TxFeedItem {
  time: string;
  agent: string;
  recipient: string;
  amount: string;
  decision: "ALLOWED" | "BLOCKED";
  policy: string;
  hash: string | null;
}

export interface AuditEntry {
  type: "ALLOWED" | "BLOCKED";
  agent: string;
  recipient: string;
  reason: string;
  time: string;
  hash: string | null;
}

export interface PolicyData {
  maxPerTx: string;
  maxDaily: string;
  minInterval: string;
  circuitBreaker: boolean;
  contract: string;
  auditContract: string;
  lastUpdated: string;
}

async function fetchJson<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) throw new Error(`API ${res.status}`);
  return res.json();
}

export function useStats(refreshMs = 5000) {
  const [stats, setStats] = useState<Stats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = useCallback(async () => {
    try {
      const data = await fetchJson<Stats>("/api/stats");
      setStats(data);
      setError(null);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, refreshMs);
    return () => clearInterval(interval);
  }, [fetchStats, refreshMs]);

  return { stats, loading, error, refetch: fetchStats };
}

export function useFeed(refreshMs = 5000) {
  const [transactions, setTransactions] = useState<TxFeedItem[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);

  const fetchFeed = useCallback(async () => {
    try {
      const data = await fetchJson<{ transactions: TxFeedItem[]; total: number }>("/api/feed");
      setTransactions(data.transactions);
      setTotal(data.total);
    } catch {
      // API may be offline
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchFeed();
    const interval = setInterval(fetchFeed, refreshMs);
    return () => clearInterval(interval);
  }, [fetchFeed, refreshMs]);

  return { transactions, total, loading, refetch: fetchFeed };
}

export function useAudit(refreshMs = 10000) {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);

  const fetchAudit = useCallback(async () => {
    try {
      const data = await fetchJson<{ entries: AuditEntry[]; total: number }>("/api/audit");
      setEntries(data.entries);
      setTotal(data.total);
    } catch {
      // API may be offline
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAudit();
    const interval = setInterval(fetchAudit, refreshMs);
    return () => clearInterval(interval);
  }, [fetchAudit, refreshMs]);

  return { entries, total, loading, refetch: fetchAudit };
}

export function usePolicy() {
  const [policy, setPolicy] = useState<PolicyData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchJson<PolicyData>("/api/policy")
      .then(setPolicy)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  return { policy, loading };
}
