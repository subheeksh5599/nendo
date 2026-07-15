import { ArrowRight } from "lucide-react";

const stack = ["Avalanche C-Chain", "Solidity", "Foundry", "Rust", "TypeScript", "Hyper", "sled DB", "Vercel"];

function HeroSection() {
  return (
    <div className="absolute inset-0">
      <video
        autoPlay
        muted
        loop
        playsInline
        className="object-cover absolute inset-0 w-full h-full"
        src="https://d8j0ntlcm91z4.cloudfront.net/user_38xzZboKViGWJOttwIXH07lWA1P/hf_20260423_161253_c72b1869-400f-45ed-ac0c-52f68c2ed5bd.mp4"
      />

      <div className="relative z-10 flex flex-col items-start justify-end h-full p-12 pb-12">
        <h1 className="text-black text-5xl md:text-6xl font-medium leading-tight max-w-xl mb-4" style={{ letterSpacing: "-0.04em" }}>
          On-Chain Security
          <br />
          for AI Agents
        </h1>
        <p className="text-black/70 text-base md:text-lg max-w-md mb-8 leading-relaxed" style={{ fontFamily: "'Inter', ui-sans-serif, system-ui, sans-serif" }}>
          Nendo sits between your AI agents and Avalanche, validating every
          transaction against on-chain policies before it reaches the network.
        </p>

        <a
          href="#/dashboard"
          className="inline-flex items-center gap-3 bg-black text-white text-base md:text-lg font-medium pl-8 pr-2 py-2 rounded-full hover:bg-gray-800 transition-colors duration-200"
        >
          Launch Dashboard
          <span className="bg-white rounded-full p-2">
            <ArrowRight className="w-5 h-5 text-black" />
          </span>
        </a>

        <div className="mt-16 w-full max-w-md overflow-hidden">
          <style>{`
            .marquee-track {
              display: flex;
              width: max-content;
              animation: marquee 22s linear infinite;
            }
          `}</style>
          <div className="marquee-track">
            {[...stack, ...stack].map((item, i) => (
              <span key={i} className="mx-7 shrink-0 text-black/60 whitespace-nowrap text-sm font-medium">
                {item}
              </span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default HeroSection;
