const stack = ["Avalanche C-Chain", "Solidity", "Rust", "Foundry", "TypeScript", "Hyper", "Vercel", "sled DB"];

function BackedBySection() {
  return (
    <section className="bg-[#F5F5F5] px-6 py-16">
      <div className="max-w-[88rem] mx-auto grid grid-cols-1 md:grid-cols-4 gap-8 items-center">
        <div>
          <p className="text-black/70 text-base leading-relaxed">
            Built on Avalanche
          </p>
          <p className="text-black/40 text-sm mt-1">
            Fuji testnet
          </p>
        </div>

        <div className="md:col-span-3 overflow-hidden">
          <style>{`
            .backers-track {
              display: flex;
              width: max-content;
              animation: backers-marquee 30s linear infinite;
            }
          `}</style>
          <div className="backers-track">
            {[...stack, ...stack].map((item, i) => (
              <span key={i} className="mx-10 shrink-0 text-black/50 whitespace-nowrap text-sm font-medium">
                {item}
              </span>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

export default BackedBySection;
