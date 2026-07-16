import { TxFeedItem } from '../hooks/useApi'

function fmtTime(iso: string) {
  const d = new Date(iso);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  if (diff < 60000) return 'Just now';
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return d.toLocaleDateString();
}

export function TransactionFeed({ transactions, total }: { transactions: TxFeedItem[]; total: number }) {
  return (
    <div className="glass" style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 4 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
        <div>
          <div style={{ fontFamily: 'var(--font-display)', fontSize: '1rem', fontWeight: 700, color: 'var(--ink)' }}>
            Transaction feed
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--ink3)', marginTop: 1 }}>
            On-chain NendoAudit events from Avalanche Fuji
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, fontSize: '0.7rem', color: 'var(--ink3)' }}>
          <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--coral)', display: 'inline-block' }} />
          POLLING / 5s
          <button className="glass" style={{
            display: 'inline-flex', alignItems: 'center', gap: 4, cursor: 'pointer',
            padding: '0 12px', height: 36, borderRadius: 'var(--radius-chip)',
            border: 'none', fontSize: '0.75rem', color: 'var(--ink2)',
            fontFamily: 'var(--font-body)',
          }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            Export
          </button>
        </div>
      </div>

      <div style={{ overflowX: 'auto', margin: '0 -4px' }}>
        <table style={{ width: '100%', minWidth: 580, borderCollapse: 'collapse' }}>
          <thead>
            <tr>
              {['TIME', 'AGENT', 'RECIPIENT', 'AMOUNT', 'DECISION', 'POLICY', 'TX HASH'].map((h) => (
                <th key={h} style={{
                  fontSize: '0.65rem', textTransform: 'uppercase', letterSpacing: '0.06em',
                  color: 'var(--ink3)', fontWeight: 600, textAlign: 'left',
                  padding: '10px 12px', borderBottom: '1px solid rgba(0,0,0,0.06)',
                }}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {transactions.map((tx, i) => (
              <tr key={i}>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--ink)' }}>
                  {fmtTime(tx.time)}
                </td>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--ink)' }}>
                  {tx.agent}
                </td>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--ink)' }}>
                  {tx.recipient}
                </td>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none', fontFamily: 'var(--font-mono)', fontWeight: 600, textAlign: 'right', fontSize: '0.75rem', color: 'var(--ink)' }}>
                  {tx.amount}
                </td>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none' }}>
                  <StatusPill status={tx.decision} />
                </td>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none', fontSize: '0.75rem', color: tx.decision === 'ALLOWED' ? 'var(--emerald)' : 'var(--rose)' }}>
                  {tx.policy}
                </td>
                <td style={{ padding: 12, borderBottom: i < transactions.length - 1 ? '1px solid rgba(0,0,0,0.04)' : 'none', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: tx.hash ? 'var(--coral)' : 'var(--ink3)' }}>
                  {tx.hash ?? '—'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: 8, fontSize: '0.75rem', color: 'var(--ink3)' }}>
        <span>Showing latest {transactions.length} of {total.toLocaleString()} decisions</span>
        <a href="#" style={{ color: 'var(--coral)', fontWeight: 600, textDecoration: 'none', display: 'flex', alignItems: 'center', gap: 4 }}>
          View full history
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="5" y1="12" x2="19" y2="12" /><polyline points="12 5 19 12 12 19" />
          </svg>
        </a>
      </div>
    </div>
  )
}

function StatusPill({ status }: { status: string }) {
  const isAllowed = status === 'ALLOWED'
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 5,
      padding: '5px 12px', borderRadius: 'var(--radius-pill)',
      fontSize: '0.75rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em',
      background: isAllowed ? 'rgba(16,185,129,0.12)' : 'rgba(251,113,133,0.12)',
      color: isAllowed ? 'var(--emerald)' : 'var(--rose)',
    }}>
      {status}
    </span>
  )
}
