interface ChainData {
  blockNumber: number | null;
  gasPrice: string | null;
  chainId: number;
  network: string;
}

interface TopBarProps {
  chainData: ChainData | null;
  paused?: boolean;
  policyContract?: string;
  auditContract?: string;
}

export function TopBar({ chainData, paused, policyContract, auditContract }: TopBarProps) {
  return (
    <header
      className="glass"
      style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '12px 20px', minHeight: 56, flexShrink: 0, flexWrap: 'wrap', gap: 8,
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        <span style={{ fontFamily: 'var(--font-display)', fontSize: '1.1rem', fontWeight: 700, color: 'var(--ink)', lineHeight: 1.2 }}>
          Nendo Operations
          {paused !== undefined && (
            <span style={{
              marginLeft: 10, fontSize: '0.65rem', fontWeight: 600, padding: '2px 8px', borderRadius: 4,
              background: paused ? 'var(--rose)' : 'var(--emerald)',
              color: '#fff', verticalAlign: 'middle',
            }}>
              {paused ? '⏸ PAUSED' : '▶ ACTIVE'}
            </span>
          )}
        </span>
        <span style={{ fontSize: '0.75rem', color: 'var(--ink2)' }}>
          Proxy enforcement layer · {chainData?.network === 'fuji' ? 'Avalanche Fuji' : 'Avalanche'}
          {chainData?.gasPrice ? ` · ${Number(chainData.gasPrice).toFixed(0)} gwei` : ''}
        </span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 12, fontSize: '0.75rem', color: 'var(--ink2)' }}>
        {policyContract && (
          <a href={`https://testnet.snowtrace.io/address/${policyContract}`} target="_blank" rel="noopener"
            style={{ color: 'var(--ink2)', textDecoration: 'underline', textUnderlineOffset: 3 }}>
            Policy ↗
          </a>
        )}
        {auditContract && (
          <a href={`https://testnet.snowtrace.io/address/${auditContract}`} target="_blank" rel="noopener"
            style={{ color: 'var(--ink2)', textDecoration: 'underline', textUnderlineOffset: 3 }}>
            Audit ↗
          </a>
        )}
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: chainData?.blockNumber ? 'var(--emerald)' : 'var(--rose)', display: 'inline-block' }} />
        {chainData?.blockNumber ? `Fuji · #${chainData.blockNumber}` : 'RPC offline'}
        {chainData?.gasPrice ? ` · ${Number(chainData.gasPrice).toFixed(0)} gwei` : ''}
      </div>
    </header>
  )
}
