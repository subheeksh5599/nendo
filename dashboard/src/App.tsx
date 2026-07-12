import { useState, useEffect, useCallback } from 'react'

type LogEntry = {
  type: 'ALLOWED' | 'BLOCKED' | 'ESCALATED'
  from: string
  to: string
  value: string
  reason?: string
  timestamp: string
}

type PolicyState = {
  maxPerTx: string
  maxDaily: string
  minInterval: number
  paused: boolean
  allowedContracts: number
  blockedRecipients: number
}

export default function App() {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [policy, setPolicy] = useState<PolicyState>({
    maxPerTx: '10 AVAX',
    maxDaily: '100 AVAX',
    minInterval: 5,
    paused: false,
    allowedContracts: 3,
    blockedRecipients: 0,
  })
  const [stats, setStats] = useState({ allowed: 0, blocked: 0, escalated: 0 })
  const [activeTab, setActiveTab] = useState<'feed' | 'policy' | 'audit'>('feed')

  // Simulate live feed for demo
  useEffect(() => {
    const demoLogs: LogEntry[] = [
      { type: 'ALLOWED', from: '0xABCD...1234', to: '0xJUP...swap', value: '2.5 AVAX', timestamp: new Date(Date.now() - 120000).toISOString() },
      { type: 'ALLOWED', from: '0xABCD...1234', to: '0xUSDC...transfer', value: '500 USDC', timestamp: new Date(Date.now() - 60000).toISOString() },
      { type: 'BLOCKED', from: '0xABCD...1234', to: '0xDEAD...DRAIN', value: '50 AVAX', reason: 'Exceeds per-tx cap (max 10 AVAX)', timestamp: new Date(Date.now() - 30000).toISOString() },
      { type: 'ALLOWED', from: '0xABCD...1234', to: '0xJUP...swap', value: '1.0 AVAX', timestamp: new Date().toISOString() },
    ]
    setLogs(demoLogs)
    setStats({ allowed: 142, blocked: 3, escalated: 1 })
  }, [])

  const formatTime = (ts: string) => {
    const d = new Date(ts)
    return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }

  return (
    <div style={{ fontFamily: 'Inter, system-ui, sans-serif', background: '#FAF8F4', minHeight: '100vh', color: '#0F1729' }}>
      {/* Header */}
      <header style={{ background: '#0F1729', color: '#FAF8F4', padding: '16px 32px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <div style={{ width: 32, height: 32, background: '#C8102E', borderRadius: 6, display: 'flex', alignItems: 'center', justifyContent: 'center', fontFamily: 'monospace', fontWeight: 'bold', fontSize: 18 }}>N</div>
          <div>
            <div style={{ fontWeight: 700, fontSize: 16, fontFamily: 'monospace' }}>NENDO</div>
            <div style={{ fontSize: 11, opacity: 0.6, fontFamily: 'monospace' }}>Agent RPC Firewall</div>
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <div style={{ textAlign: 'right' }}>
            <div style={{ fontSize: 11, opacity: 0.6 }}>AVALANCHE FUJI TESTNET</div>
            <div style={{ fontSize: 11, fontFamily: 'monospace' }}>proxy: 127.0.0.1:8545</div>
          </div>
          <div style={{ width: 8, height: 8, borderRadius: '50%', background: '#2A7A3B', boxShadow: '0 0 8px #2A7A3B' }} />
        </div>
      </header>

      {/* Stats bar */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, padding: '24px 32px', background: '#0F1729' }}>
        {[
          { label: 'Transactions Allowed', value: stats.allowed, color: '#2A7A3B' },
          { label: 'Blocked', value: stats.blocked, color: '#C8102E' },
          { label: 'Escalated', value: stats.escalated, color: '#F59E0B' },
          { label: 'Firewall Status', value: policy.paused ? 'PAUSED' : 'ACTIVE', color: policy.paused ? '#C8102E' : '#2A7A3B' },
        ].map(stat => (
          <div key={stat.label} style={{ background: '#1A2332', borderRadius: 8, padding: '16px 20px' }}>
            <div style={{ fontSize: 28, fontWeight: 700, fontFamily: 'monospace', color: stat.color }}>{stat.value}</div>
            <div style={{ fontSize: 11, color: '#8A8580', marginTop: 4 }}>{stat.label}</div>
          </div>
        ))}
      </div>

      {/* Navigation */}
      <div style={{ display: 'flex', gap: 0, padding: '0 32px', background: '#F0EDE7', borderBottom: '1px solid #E0DDD7' }}>
        {(['feed', 'policy', 'audit'] as const).map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            style={{
              padding: '12px 24px',
              border: 'none',
              background: 'transparent',
              cursor: 'pointer',
              fontFamily: 'monospace',
              fontSize: 13,
              fontWeight: 600,
              color: activeTab === tab ? '#C8102E' : '#8A8580',
              borderBottom: activeTab === tab ? '2px solid #C8102E' : '2px solid transparent',
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
            }}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ padding: 32 }}>
        {activeTab === 'feed' && (
          <div style={{ maxWidth: 800 }}>
            <h2 style={{ fontFamily: 'monospace', fontSize: 13, color: '#8A8580', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 16 }}>Live Transaction Feed</h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {logs.map((log, i) => (
                <div
                  key={i}
                  style={{
                    background: 'white',
                    border: `1px solid ${log.type === 'BLOCKED' ? '#C8102E20' : log.type === 'ESCALATED' ? '#F59E0B20' : '#2A7A3B20'}`,
                    borderLeft: `3px solid ${log.type === 'BLOCKED' ? '#C8102E' : log.type === 'ESCALATED' ? '#F59E0B' : '#2A7A3B'}`,
                    borderRadius: 6,
                    padding: '12px 16px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                    <span style={{
                      fontFamily: 'monospace',
                      fontSize: 11,
                      fontWeight: 700,
                      padding: '2px 8px',
                      borderRadius: 4,
                      background: log.type === 'BLOCKED' ? '#C8102E15' : log.type === 'ESCALATED' ? '#F59E0B15' : '#2A7A3B15',
                      color: log.type === 'BLOCKED' ? '#C8102E' : log.type === 'ESCALATED' ? '#F59E0B' : '#2A7A3B',
                    }}>
                      {log.type}
                    </span>
                    <div>
                      <div style={{ fontFamily: 'monospace', fontSize: 13, fontWeight: 600 }}>{log.value}</div>
                      <div style={{ fontSize: 11, color: '#8A8580', fontFamily: 'monospace' }}>to {log.to}</div>
                    </div>
                  </div>
                  <div style={{ textAlign: 'right' }}>
                    <div style={{ fontFamily: 'monospace', fontSize: 11, color: '#8A8580' }}>{formatTime(log.timestamp)}</div>
                    {log.reason && (
                      <div style={{ fontSize: 11, color: '#C8102E', marginTop: 2, maxWidth: 280, textAlign: 'right' }}>{log.reason}</div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'policy' && (
          <div style={{ maxWidth: 600 }}>
            <h2 style={{ fontFamily: 'monospace', fontSize: 13, color: '#8A8580', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 16 }}>Current Policy</h2>
            <div style={{ background: 'white', borderRadius: 8, padding: 24, border: '1px solid #E0DDD7' }}>
              {[
                { label: 'Max per transaction', value: policy.maxPerTx },
                { label: 'Max daily spending', value: policy.maxDaily },
                { label: 'Min interval between txs', value: `${policy.minInterval}s` },
                { label: 'Allowed contracts', value: `${policy.allowedContracts} whitelisted` },
                { label: 'Blocked recipients', value: `${policy.blockedRecipients} blocked` },
              ].map(row => (
                <div key={row.label} style={{ display: 'flex', justifyContent: 'space-between', padding: '10px 0', borderBottom: '1px solid #F0EDE7' }}>
                  <span style={{ color: '#8A8580', fontSize: 13 }}>{row.label}</span>
                  <span style={{ fontFamily: 'monospace', fontSize: 13, fontWeight: 600 }}>{row.value}</span>
                </div>
              ))}
              <div style={{ marginTop: 20 }}>
                <button style={{
                  background: '#C8102E',
                  color: 'white',
                  border: 'none',
                  padding: '10px 20px',
                  borderRadius: 6,
                  fontFamily: 'monospace',
                  fontSize: 13,
                  fontWeight: 600,
                  cursor: 'pointer',
                }}>
                  Update Policy (Owner)
                </button>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'audit' && (
          <div style={{ maxWidth: 800 }}>
            <h2 style={{ fontFamily: 'monospace', fontSize: 13, color: '#8A8580', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 16 }}>On-Chain Audit Log</h2>
            <div style={{ background: 'white', borderRadius: 8, padding: 24, border: '1px solid #E0DDD7', fontFamily: 'monospace', fontSize: 12 }}>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr 2fr 1fr', gap: 8, padding: '8px 0', borderBottom: '2px solid #E0DDD7', fontWeight: 700, color: '#8A8580', fontSize: 11, textTransform: 'uppercase' }}>
                <span>Type</span><span>Agent</span><span>Recipient</span><span>Reason / Amount</span><span>Time</span>
              </div>
              {[
                { type: 'ALLOWED', agent: '0xABCD...1234', recipient: '0xJUP...swap', amount: '2.5 AVAX', time: '14:32:01' },
                { type: 'ALLOWED', agent: '0xABCD...1234', recipient: '0xUSDC...', amount: '500 USDC', time: '14:31:00' },
                { type: 'BLOCKED', agent: '0xABCD...1234', recipient: '0xDEAD...DRAIN', amount: '50 AVAX', time: '14:30:33', reason: 'Exceeds per-tx cap' },
              ].map((row, i) => (
                <div key={i} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr 2fr 1fr', gap: 8, padding: '10px 0', borderBottom: '1px solid #F0EDE7', alignItems: 'center' }}>
                  <span style={{ color: row.type === 'BLOCKED' ? '#C8102E' : '#2A7A3B', fontWeight: 700 }}>{row.type}</span>
                  <span style={{ color: '#8A8580' }}>{row.agent}</span>
                  <span style={{ color: '#8A8580' }}>{row.recipient}</span>
                  <span style={{ color: '#0F1729' }}>{row.reason || row.amount}</span>
                  <span style={{ color: '#8A8580' }}>{row.time}</span>
                </div>
              ))}
            </div>
            <p style={{ fontSize: 11, color: '#8A8580', marginTop: 12, fontFamily: 'monospace' }}>
              Full audit trail available on Snowscan: <span style={{ color: '#0F1729' }}>View NendoAudit contract events</span>
            </p>
          </div>
        )}
      </div>

      {/* Footer */}
      <footer style={{ padding: '20px 32px', borderTop: '1px solid #E0DDD7', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontFamily: 'monospace', fontSize: 11, color: '#8A8580' }}>NENDO v0.1.0 — Built for Avalanche Team1</span>
        <span style={{ fontFamily: 'monospace', fontSize: 11, color: '#8A8580' }}>
          Based on SUDONT — Colosseum Frontier Hackathon Top 25 Winner (June 2026)
        </span>
      </footer>
    </div>
  )
}