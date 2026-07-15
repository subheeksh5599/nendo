interface ChainData {
  blockNumber: number | null;
  gasPrice: string | null;
  chainId: number;
  network: string;
}

export function TopBar({ chainData }: { chainData: ChainData | null }) {
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
        </span>
        <span style={{ fontSize: '0.75rem', color: 'var(--ink2)' }}>
          Proxy enforcement layer · {chainData?.network === 'fuji' ? 'Avalanche Fuji' : 'Avalanche'}
          {chainData?.gasPrice ? ` · ${Number(chainData.gasPrice).toFixed(0)} gwei` : ''}
        </span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 12, fontSize: '0.75rem', color: 'var(--ink3)' }}>
        <span style={{ width: 6, height: 6, borderRadius: '50%', background: chainData ? 'var(--emerald)' : 'var(--rose)', display: 'inline-block' }} />
        {chainData ? 'Live' : 'Offline'}
        <span style={{ fontFamily: 'var(--font-mono)' }}>
          {chainData?.blockNumber ? `#${chainData.blockNumber}` : '--'}
        </span>
      </div>
    </header>
  )
}
