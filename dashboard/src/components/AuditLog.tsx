import { AuditEntry } from '../hooks/useApi'

export function AuditLog({ entries }: { entries: AuditEntry[] }) {
  return (
    <div className="glass" style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 4 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
        <div>
          <div style={{ fontFamily: 'var(--font-display)', fontSize: '1rem', fontWeight: 700, color: 'var(--ink)', display: 'flex', alignItems: 'center', gap: 8 }}>
            Audit log
            <span style={{
              fontSize: '0.6rem', textTransform: 'uppercase', letterSpacing: '0.05em', fontWeight: 700,
              padding: '3px 8px', borderRadius: 'var(--radius-pill)',
              background: 'rgba(16,185,129,0.12)', color: 'var(--emerald)',
            }}>FUJI INDEXED</span>
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--ink3)', marginTop: 1 }}>
            Local sled DB + immutable NendoAudit events
          </div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginTop: 12, flexWrap: 'wrap' }}>
        <input
          type="text"
          placeholder="Search hash, agent, recipient…"
          style={{
            flex: 1, minWidth: 200, height: 40, borderRadius: 'var(--radius-chip)', padding: '0 14px',
            border: '1px solid rgba(0,0,0,0.1)', background: 'rgba(255,255,255,0.6)',
            fontSize: '0.8125rem', color: 'var(--ink)', outline: 'none', fontFamily: 'var(--font-body)',
          }}
        />
        <select style={{
          height: 40, borderRadius: 'var(--radius-chip)', padding: '0 12px',
          border: '1px solid rgba(0,0,0,0.1)', background: 'rgba(255,255,255,0.6)',
          fontSize: '0.8125rem', color: 'var(--ink)', outline: 'none', fontFamily: 'var(--font-body)',
        }}>
          <option>All decisions</option>
          <option>Allowed</option>
          <option>Blocked</option>
        </select>
      </div>

      <div style={{ marginTop: 12, display: 'flex', flexDirection: 'column', gap: 8 }}>
        {entries.map((entry, i) => (
          <div key={i} style={{
            display: 'flex', alignItems: 'center', gap: 12, padding: '10px 14px',
            borderRadius: 'var(--radius-chip)', background: 'rgba(0,0,0,0.02)', flexWrap: 'wrap',
          }}>
            <AuditStatusPill status={entry.type} />
            <div style={{ flex: 1, minWidth: 0 }}>
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: 'var(--ink)' }}>
                {entry.agent} → {entry.recipient}
              </span>
              <div style={{ fontSize: '0.7rem', fontFamily: 'var(--font-mono)', color: 'var(--ink3)', marginTop: 2 }}>
                {entry.reason}
              </div>
            </div>
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.7rem', color: 'var(--ink3)', flexShrink: 0 }}>
              {entry.time}
            </span>
            {entry.hash && (
              <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.7rem', color: 'var(--coral)', flexShrink: 0 }}>
                {entry.hash}
              </span>
            )}
          </div>
        ))}
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: 8, fontSize: '0.75rem', color: 'var(--ink3)' }}>
        <span>Showing {entries.length} of 1,301 events</span>
        <a href="#" style={{ color: 'var(--coral)', fontWeight: 600, textDecoration: 'none', display: 'flex', alignItems: 'center', gap: 4 }}>
          Open explorer
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15 3 21 3 21 9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </svg>
        </a>
      </div>
    </div>
  )
}

function AuditStatusPill({ status }: { status: 'ALLOWED' | 'BLOCKED' }) {
  const isAllowed = status === 'ALLOWED'
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 5, flexShrink: 0,
      padding: '5px 12px', borderRadius: 'var(--radius-pill)',
      fontSize: '0.75rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em',
      background: isAllowed ? 'rgba(16,185,129,0.12)' : 'rgba(251,113,133,0.12)',
      color: isAllowed ? 'var(--emerald)' : 'var(--rose)',
    }}>
      {status}
    </span>
  )
}
