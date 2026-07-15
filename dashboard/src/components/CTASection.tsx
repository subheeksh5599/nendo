import { ArrowRight, Copy, Check } from "lucide-react";
import { useState } from "react";

const configToml = `[nendo]
proxy_host = "127.0.0.1"
proxy_port = 8545
avalanche_rpc = "https://api.avax-test.network/ext/bc/C/rpc"

[policy]
max_per_tx = "1.0 AVAX"
max_daily = "10.0 AVAX"
min_interval_secs = 60
circuit_breaker = true

[audit]
db_path = "./nendo_audit"
on_chain = true`;

function CTASection() {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(configToml);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section id="playground" className="bg-[#F5F5F5] px-6 py-24">
      <div className="max-w-[88rem] mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-start">
          <div>
            <p className="text-black/60 text-sm mb-2">Get Started</p>
            <h2
              className="text-5xl md:text-6xl font-medium leading-none mb-6"
              style={{ letterSpacing: "-0.04em" }}
            >
              Secure Your Agents
            </h2>
            <p className="text-black/60 text-base leading-relaxed max-w-md mb-8">
              Nendo runs as a local proxy on port 8545. Point your AI agent's
              Avalanche RPC to Nendo and every transaction is validated,
              simulated, and audited automatically. Zero code changes on the
              agent side.
            </p>

            <div className="flex flex-wrap gap-4">
              <a
                href="https://github.com/subheeksh5599/nendo#readme"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-3 bg-black text-white text-base font-medium pl-8 pr-2 py-2 rounded-full hover:bg-gray-800 transition-colors duration-200"
              >
                Read the Docs
                <span className="bg-white rounded-full p-2">
                  <ArrowRight className="w-5 h-5 text-black" />
                </span>
              </a>
              <button
                type="button"
                onClick={handleCopy}
                className="inline-flex items-center gap-2 border border-black/20 text-black text-base font-medium px-6 py-2.5 rounded-full hover:bg-black/5 transition-colors duration-200"
              >
                {copied ? (
                  <Check className="w-4 h-4 text-green-600" />
                ) : (
                  <Copy className="w-4 h-4" />
                )}
                {copied ? "Copied!" : "Copy Config"}
              </button>
            </div>
          </div>

          <div className="bg-[#1A1A1A] rounded-2xl p-8 overflow-x-auto">
            <p className="text-white/50 text-xs mb-3 font-mono">
              config.toml
            </p>
            <pre className="text-white/80 text-sm font-mono leading-relaxed whitespace-pre">
              {configToml}
            </pre>
          </div>
        </div>

        <div className="mt-20 grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div className="bg-white rounded-2xl p-7">
            <div className="w-10 h-10 rounded-full bg-black/5 flex items-center justify-center mb-4">
              <span className="text-black font-bold text-lg">1</span>
            </div>
            <h3 className="text-black text-lg font-medium mb-2">Deploy</h3>
            <p className="text-black/60 text-sm leading-relaxed">
              Deploy NendoPolicy + NendoAudit contracts on Avalanche C-Chain
              in one transaction via Foundry. On-chain policy enforcement from
              day zero.
            </p>
          </div>
          <div className="bg-white rounded-2xl p-7">
            <div className="w-10 h-10 rounded-full bg-black/5 flex items-center justify-center mb-4">
              <span className="text-black font-bold text-lg">2</span>
            </div>
            <h3 className="text-black text-lg font-medium mb-2">Configure</h3>
            <p className="text-black/60 text-sm leading-relaxed">
              Point your agent's RPC URL to Nendo's proxy. Set caps,
              allowlists, rate limits per agent or globally via the dashboard
              or TypeScript SDK.
            </p>
          </div>
          <div className="bg-white rounded-2xl p-7">
            <div className="w-10 h-10 rounded-full bg-black/5 flex items-center justify-center mb-4">
              <span className="text-black font-bold text-lg">3</span>
            </div>
            <h3 className="text-black text-lg font-medium mb-2">Ship</h3>
            <p className="text-black/60 text-sm leading-relaxed">
              Every transaction is validated, simulated, and audited. Circuit
              breaker pauses all agent outflow in one click. Prompt injections
              can't drain your wallet.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}

export default CTASection;
