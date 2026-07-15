import { useEffect } from 'react'
import { TopBar } from './components/TopBar'
import { KpiCard } from './components/KpiCard'
import { DonutChart } from './components/DonutChart'
import { TransactionFeed } from './components/TransactionFeed'
import { AuditLog } from './components/AuditLog'
import { useStats, useFeed, useAudit } from './hooks/useApi'

export default function DashboardPage() {
  const { stats, loading: statsLoading } = useStats()
  const { transactions, total: feedTotal } = useFeed()
  const { entries: auditEntries } = useAudit()

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
      sparkline: 'M0,22 L200,22',
    },
    {
      label: 'Blocked',
      value: stats ? stats.blockedToday.toString() : '--',
      delta: stats?.blockRatio ? `${stats.blockRatio}%` : '',
      deltaUp: false, icon: 'shield', color: 'rose' as const,
      sparkline: 'M0,22 L200,22',
    },
    {
      label: 'Chain',
      value: stats?.chain.chainId === 43113 ? 'Fuji' : stats?.chain.chainId.toString() ?? '--',
      delta: stats?.chain.blockNumber ? `#${stats.chain.blockNumber}` : '',
      deltaUp: true, icon: 'users', color: 'amber' as const,
      sparkline: 'M0,22 L200,22',
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
        <TopBar chainData={stats?.chain ?? null} />

        {/* KPI row */}
        <div className="kpi-grid">
          {KPIS.map((kpi) => (
            <KpiCard key={kpi.label} {...kpi} />
          ))}
        </div>

        {/* Feed + Donut */}
        <div className="grid-2-1">
          <TransactionFeed transactions={transactions} total={feedTotal} />
          <DonutChart blockedToday={stats?.blockedToday ?? 0} processedToday={stats?.processedToday ?? 0} />
        </div>

        {/* Audit log */}
        <AuditLog entries={auditEntries} />
      </div>
    </div>
  )
}
