import { ArrowRight } from "lucide-react";

function InfoSection() {
  return (
    <section className="bg-[#F5F5F5] px-6 py-24">
      <div className="max-w-[88rem] mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-12 mb-16 items-start">
          <div>
            <h2 className="text-black text-4xl md:text-5xl font-medium leading-tight mb-8" style={{ letterSpacing: "-0.03em" }}>
              Meet Nendo.
            </h2>
            <a
              href="#/dashboard"
              className="inline-flex items-center gap-3 bg-black text-white text-base font-medium pl-8 pr-2 py-2 rounded-full hover:bg-gray-800 transition-colors duration-200"
            >
              Launch Dashboard
              <span className="bg-white rounded-full p-2">
                <ArrowRight className="w-5 h-5 text-black" />
              </span>
            </a>
          </div>
          <p className="text-black/70 text-2xl md:text-3xl leading-relaxed">
            An RPC proxy firewall that intercepts every transaction from your
            AI agents, validates it against on-chain policies, and logs every
            decision immutably on Avalanche C-Chain.
          </p>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <div
            className="rounded-2xl p-7 min-h-80 flex flex-col justify-between lg:col-span-2"
            style={{
              backgroundImage:
                "url(https://images.higgs.ai/?default=1&output=webp&url=https%3A%2F%2Fd8j0ntlcm91z4.cloudfront.net%2Fuser_38xzZboKViGWJOttwIXH07lWA1P%2Fhf_20260423_164207_f243351d-ed59-48ec-83a0-a5e996bdbe3c.png&w=1280&q=85)",
              backgroundSize: "cover",
              backgroundPosition: "center",
            }}
          >
            <h3 className="text-black text-2xl font-medium leading-snug" style={{ letterSpacing: "-0.02em" }}>
              Per-Transaction Caps
            </h3>
            <p className="text-black/70 text-base max-w-xs mt-4">
              Set max AVAX per single transaction. Any transfer exceeding
              the limit is blocked before it reaches the network.
            </p>
          </div>

          <div className="bg-[#2B2644] rounded-2xl p-7 min-h-80 flex flex-col justify-between">
            <h3 className="text-white text-2xl font-medium leading-snug">
              Pre-Flight
              <br />
              Simulation
            </h3>
            <p className="text-white/60 text-base mt-4">
              Every transaction is simulated against your policies using
              eth_estimateGas before forwarding.
            </p>
          </div>

          <div className="bg-[#2B2644] rounded-2xl p-7 min-h-80 flex flex-col justify-between">
            <h3 className="text-white text-2xl font-medium leading-snug">
              Immutable
              <br />
              Audit Trail
            </h3>
            <p className="text-white/60 text-base mt-4">
              Every decision is emitted as an event on Avalanche C-Chain
              via NendoAudit.sol. Queryable on Snowscan.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}

export default InfoSection;
