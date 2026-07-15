function Footer() {
  return (
    <footer className="bg-[#1A1A1A] text-white/60 px-6 py-16">
      <div className="max-w-[88rem] mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-10 mb-12">
          <div className="md:col-span-2">
            <h3 className="text-white text-xl font-medium mb-4">Nendo</h3>
            <p className="text-white/50 text-sm leading-relaxed max-w-sm">
              On-chain security middleware for autonomous AI agents. RPC proxy
              firewall with on-chain policy enforcement and immutable audit
              trail on Avalanche C-Chain.
            </p>
          </div>
          <div>
            <h4 className="text-white text-sm font-medium mb-3">Protocol</h4>
            <ul className="space-y-2 text-sm">
              <li><a href="https://github.com/subheeksh5599/nendo#readme" className="hover:text-white transition-colors">Docs</a></li>
              <li><a href="https://github.com/subheeksh5599/nendo/tree/main/contracts" className="hover:text-white transition-colors">Contracts</a></li>
              <li><a href="https://github.com/subheeksh5599/nendo/tree/main/sdk" className="hover:text-white transition-colors">SDK</a></li>
              <li><a href="#/dashboard" className="hover:text-white transition-colors">Dashboard</a></li>
            </ul>
          </div>
          <div>
            <h4 className="text-white text-sm font-medium mb-3">Stack</h4>
            <ul className="space-y-2 text-sm">
              <li><span>Avalanche C-Chain</span></li>
              <li><span>Rust + Hyper</span></li>
              <li><span>Solidity + Foundry</span></li>
              <li><span>TypeScript SDK</span></li>
            </ul>
          </div>
        </div>

        <div className="border-t border-white/10 pt-8 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
          <p className="text-white/40 text-sm">
            &copy; {new Date().getFullYear()} Nendo. MIT Licensed.
          </p>
          <div className="flex items-center gap-6 text-sm">
            <a href="https://github.com/subheeksh5599/nendo" target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors">GitHub</a>
            <a href="https://github.com/subheeksh5599/nendo#readme" className="hover:text-white transition-colors">Docs</a>
          </div>
        </div>
      </div>
    </footer>
  );
}

export default Footer;
