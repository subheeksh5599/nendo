import { useEffect } from 'react'
import { TopBar } from './components/TopBar'
import { KpiCard } from './components/KpiCard'
import { DonutChart } from './components/DonutChart'
import { TransactionFeed } from './components/TransactionFeed'
import { AuditLog } from './components/AuditLog'
import { useStats, useFeed, useAudit, usePolicy } from './hooks/useApi'

export default function DashboardPage() {
  const { stats, loading: statsLoading } = useStats()
  const { transactions, total: feedTotal } = useFeed()
  const { entries: auditEntries, total: auditTotal } = useAudit()
  const { policy } = usePolicy()

  const KPIS = [
    {
      label: 'Proxy Uptime',
      value: stats?.uptime ?? '--',
      delta: stats?.chain.blockNumber ? `Block #${stats.chain.blockNumber}` : '',
      deltaUp: true, icon: 'server', color: 'emerald' as const,
      sparkline: stats?.proxyOnline ? 'M0,22 L20,21 L40,21 L60,20 L80,21 L100,20 L120,21 L140,20 L160,21 L180,20 L200,20' : 'M0,22 L200,22',
    },
    {
      label: 'Processed',
      value: stats ? stats.processedToday.toLocaleString() : '--',
      delta: stats?.blockRatio ? `${(100 - parseFloat(stats.blockRatio)).toFixed(1)}%` : '',
      deltaUp: true, icon: 'activity', color: 'coral' as const,
      sparkline: stats && stats.processedToday > 0 ? 'M0,22 L20,18 L40,20 L60,15 L80,18 L100,14 L120,17 L140,12 L160,15 L180,10 L200,13' : 'M0,22 L200,22',
    },
    {
      label: 'Blocked',
      value: stats ? stats.blockedToday.toString() : '--',
      delta: stats?.blockRatio ? `${stats.blockRatio}%` : '',
      deltaUp: false, icon: 'shield', color: 'rose' as const,
      sparkline: stats && stats.blockedToday > 0 ? 'M0,22 L20,20 L40,18 L60,21 L80,17 L100,20 L120,16 L140,19 L160,14 L180,18 L200,15' : 'M0,22 L200,22',
    },
    {
      label: 'Circuit Breaker',
      value: stats?.paused ? 'PAUSED' : 'ACTIVE',
      delta: policy ? `Cap: ${policy.maxPerTx}` : '',
      deltaUp: !stats?.paused, icon: 'shield', color: (stats?.paused ? 'rose' : 'emerald') as const,
      sparkline: stats?.paused ? 'M0,22 L200,22' : 'M0,22 L20,20 L40,21 L60,19 L80,20 L100,18 L120,20 L140,18 L160,19 L180,17 L200,19',
    },
  ]

  useEffect(() => {
    if (stats?.chain.blockNumber) {
      document.title = `Nendo Operations — Block #${stats.chain.blockNumber}`
    }
  }, [stats])

  return (
    <div className="dashboard-scope" style={{
      fontFamily: 'var(--font-body)', color: 'var(--ink)',
      background: 'var(--bg-peach)', minHeight: '100vh',
      backgroundImage: `
        radial-gradient(ellipse 80% 60% at 20% 10%, #ffd9a8 0%, transparent 55%),
        radial-gradient(ellipse 60% 70% at 85% 15%, #ff9d7a 0%, transparent 55%),
        radial-gradient(ellipse 50% 50% at 85% 85%, var(--amber) 0%, transparent 60%),
        radial-gradient(ellipse 55% 60% at 15% 90%, #ff8fa0 0%, transparent 55%),
        linear-gradient(180deg, #fff2e6 0%, #ffd9c4 100%)
      `,
    }}>
      <div style={{ maxWidth: 1280, margin: '0 auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <TopBar
          chainData={stats?.chain ?? null}
          paused={stats?.paused}
          policyContract={stats?.policyContract}
          auditContract={stats?.auditContract}
        />

        {/* KPI row */}
        <div className="kpi-grid">
          {KPIS.map((kpi) => (
            <KpiCard key={kpi.label} {...kpi} />
          ))}
        </div>

        {/* On-chain policy bar */}
        {policy && (
          <div style={{
            display: 'flex', gap: 16, flexWrap: 'wrap', padding: '10px 16px',
            background: 'rgba(255,255,255,0.6)', borderRadius: 10,
            fontSize: '0.78rem', color: 'var(--ink2)', alignItems: 'center',
            backdropFilter: 'blur(10px)', border: '1px solid rgba(0,0,0,0.06)',
          }}>
            <strong style={{ color: 'var(--ink)' }}>On-chain policy:</strong>
            <span>Max/Tx: <strong style={{ color: 'var(--ink)' }}>{policy.maxPerTx}</strong></span>
            <span>·</span>
            <span>Daily: <strong style={{ color: 'var(--ink)' }}>{policy.maxDaily}</strong></span>
            <span>·</span>
            <span>Rate: <strong style={{ color: 'var(--ink)' }}>{policy.minInterval}</strong></span>
            <span>·</span>
            <span>Allowlist: <strong style={{ color: policy.allowlistMode ? 'var(--rose)' : 'var(--ink)' }}>{policy.allowlistMode ? 'ON' : 'OFF'}</strong></span>
            <span style={{ marginLeft: 'auto', fontSize: '0.7rem', opacity: 0.5 }}>
              Verified on-chain · {policy.contract.slice(0,6)}...
            </span>
          </div>
        )}

        {/* Feed + Donut */}
        <div className="grid-2-1">
          <TransactionFeed transactions={transactions} total={feedTotal} />
          <DonutChart blockedToday={stats?.blockedToday ?? 0} processedToday={stats?.processedToday ?? 0} />
        </div>

        {/* Audit log */}
        <AuditLog entries={auditEntries} total={auditTotal} />
      </div>
    </div>
  )
}
