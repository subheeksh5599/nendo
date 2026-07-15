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
      sparkline: 'M0,22 L20,21 L40,21 L60,20 L80,21 L100,20 L120,21 L140,20 L160,21 L180,20 L200,20',
    },
    {
      label: 'Processed Today',
      value: stats ? stats.processedToday.toLocaleString() : '--',
      delta: stats?.blockRatio ? `${(100 - parseFloat(stats.blockRatio)).toFixed(1)}%` : '',
      deltaUp: true, icon: 'activity', color: 'coral' as const,
      sparkline: 'M0,22 L20,18 L40,14 L60,16 L80,10 L100,12 L120,8 L140,10 L160,6 L180,8 L200,4',
    },
    {
      label: 'Blocked Today',
      value: stats ? stats.blockedToday.toString() : '--',
      delta: stats?.blockRatio ? `${stats.blockRatio}%` : '',
      deltaUp: false, icon: 'shield', color: 'rose' as const,
      sparkline: 'M0,10 L20,14 L40,10 L60,16 L80,12 L100,18 L120,14 L140,20 L160,16 L180,20 L200,18',
    },
    {
      label: 'Registered Agents',
      value: stats ? stats.registeredAgents.toString() : '--',
      delta: stats?.chain.network === 'fuji' ? 'Fuji testnet' : '',
      deltaUp: true, icon: 'users', color: 'amber' as const,
      sparkline: 'M0,24 L20,22 L40,20 L60,18 L80,16 L100,14 L120,14 L140,12 L160,10 L180,10 L200,8',
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
          <DonutChart blockedToday={stats?.blockedToday ?? 17} processedToday={stats?.processedToday ?? 1284} />
        </div>

        {/* Audit log */}
        <AuditLog entries={auditEntries} />
      </div>
    </div>
  )
}
