export function DonutChart({ blockedToday, processedToday }: { blockedToday: number; processedToday: number }) {
  const total = blockedToday + processedToday || 1
  const allowedPct = total > 0 ? ((processedToday / total) * 100).toFixed(1) : "0.0"
  const blockedPct = ((blockedToday / total) * 100).toFixed(1)

  // Arc lengths for SVG dasharray (circumference = 2 * PI * 58 ≈ 364.4)
  const circumference = 364.4
  const allowedLen = (parseFloat(allowedPct) / 100) * circumference
  const blockedLen = (parseFloat(blockedPct) / 100) * circumference

  return (
    <div className="glass" style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 4 }}>
      <div style={{ marginBottom: 8 }}>
        <div style={{ fontFamily: 'var(--font-display)', fontSize: '1rem', fontWeight: 700, color: 'var(--ink)' }}>
          Decision breakdown
        </div>
        <div style={{ fontSize: '0.75rem', color: 'var(--ink3)', marginTop: 1 }}>
          Today's allow vs block ratio
        </div>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12, paddingTop: 4 }}>
        <svg viewBox="0 0 160 160" style={{ width: 160, height: 160 }}>
          {/* Track */}
          <circle cx="80" cy="80" r="58" fill="none" stroke="rgba(0,0,0,0.04)" strokeWidth="22" />
          {/* Allowed */}
          <circle cx="80" cy="80" r="58" fill="none" stroke="var(--emerald)" strokeWidth="22"
            strokeDasharray={`${allowedLen} ${circumference - allowedLen}`} strokeDashoffset="90" strokeLinecap="round"
            transform="rotate(-90 80 80)" />
          {/* Blocked */}
          <circle cx="80" cy="80" r="58" fill="none" stroke="var(--rose)" strokeWidth="22"
            strokeDasharray={`${blockedLen} ${circumference - blockedLen}`} strokeDashoffset={90 - allowedLen} strokeLinecap="round"
            transform="rotate(-90 80 80)" />
          <text x="80" y="74" textAnchor="middle" fontFamily="var(--font-mono)" fontSize="22" fontWeight="700" fill="var(--ink)">{allowedPct}%</text>
          <text x="80" y="94" textAnchor="middle" fontFamily="var(--font-body)" fontSize="9" fill="var(--ink3)">allowed</text>
        </svg>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, width: '100%' }}>
          <LegendRow color="var(--emerald)" label="Allowed" value={processedToday.toLocaleString()} />
          <LegendRow color="var(--rose)" label="Blocked" value={blockedToday.toLocaleString()} />
        </div>
      </div>
    </div>
  )
}

function LegendRow({ color, label, value }: { color: string; label: string; value: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '0.8125rem' }}>
      <span style={{ display: 'flex', alignItems: 'center', gap: 8, color: 'var(--ink2)' }}>
        <span style={{ width: 8, height: 8, borderRadius: '50%', background: color, display: 'inline-block' }} />
        {label}
      </span>
      <span style={{ fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--ink)' }}>{value}</span>
    </div>
  )
}
