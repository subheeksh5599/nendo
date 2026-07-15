interface Props {
  label: string
  value: string
  delta: string
  deltaUp: boolean
  icon: string
  color: 'coral' | 'emerald' | 'rose' | 'amber'
  sparkline: string
}

const COLOR_MAP = {
  coral: { stroke: 'var(--coral)', bg: 'rgba(242,82,27,0.12)', pill: 'var(--emerald)' },
  emerald: { stroke: 'var(--emerald)', bg: 'rgba(16,185,129,0.12)', pill: 'var(--emerald)' },
  rose: { stroke: 'var(--rose)', bg: 'rgba(251,113,133,0.12)', pill: 'var(--rose)' },
  amber: { stroke: 'var(--amber)', bg: 'rgba(239,143,42,0.12)', pill: 'var(--emerald)' },
}

const ICON_PATHS: Record<string, string> = {
  server: 'M2 7h20v14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V7zm14-2V3a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2',
  activity: 'M22 12h-4l-3 9L9 3l-3 9H2',
  shield: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z M15 9l-6 6 M9 9l6 6',
  users: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M23 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75',
}

export function KpiCard({ label, value, delta, deltaUp, icon, color, sparkline }: Props) {
  const c = COLOR_MAP[color]

  return (
    <div className="glass" style={{ padding: '18px 20px', display: 'flex', flexDirection: 'column', gap: 6 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{
          width: 36, height: 36, borderRadius: 'var(--radius-chip)', background: c.bg,
          display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0,
        }}>
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke={c.stroke} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d={ICON_PATHS[icon] ?? ''} />
          </svg>
        </div>
        <span style={{
          display: 'inline-flex', alignItems: 'center', gap: 4,
          padding: '3px 10px', borderRadius: 'var(--radius-pill)',
          fontSize: '0.75rem', fontWeight: 600,
          background: deltaUp ? 'rgba(16,185,129,0.12)' : 'rgba(251,113,133,0.12)',
          color: deltaUp ? 'var(--emerald)' : 'var(--rose)',
          fontFamily: 'var(--font-mono)',
        }}>
          {deltaUp ? '↑' : '↓'} {delta}
        </span>
      </div>
      <div style={{ fontSize: '0.7rem', textTransform: 'uppercase', letterSpacing: '0.06em', color: 'var(--ink3)', fontWeight: 600 }}>
        {label}
      </div>
      <div style={{ fontFamily: 'var(--font-mono)', fontSize: '1.65rem', fontWeight: 700, color: 'var(--ink)', lineHeight: 1.1 }}>
        {value}
      </div>
      <div style={{ width: '100%', height: 32, marginTop: 2 }}>
        <svg viewBox="0 0 200 32" preserveAspectRatio="none" style={{ width: '100%', height: '100%' }}>
          <polyline fill="none" stroke={c.stroke} strokeWidth="1.5" points={sparkline} />
        </svg>
      </div>
    </div>
  )
}
