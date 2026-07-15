const navLinks = ["Docs", "Contracts", "SDK", "Dashboard"];

function Navbar() {
  return (
    <nav className="fixed top-0 left-0 right-0 z-20 px-6 py-4 bg-black/30 border-b border-white/10" style={{ boxShadow: "inset 0 1px 0 rgba(255,255,255,0.1)" }}>
      <div className="max-w-[88rem] mx-auto flex items-center justify-between">
        <span className="text-2xl font-medium tracking-tight text-white">
          Nendo
        </span>

        <div className="hidden md:flex items-center gap-8">
          {navLinks.map((link) => (
            <a
              key={link}
              href={link === "Dashboard" ? "#/dashboard" : `#${link.toLowerCase()}`}
              className="text-base text-white/70 hover:text-white font-medium transition-colors duration-200"
            >
              {link}
            </a>
          ))}
        </div>

        <a
          href="#/dashboard"
          className="bg-white text-black text-base font-medium px-7 py-2.5 rounded-full hover:bg-gray-200 transition-colors duration-200"
        >
          Launch Dashboard
        </a>
      </div>
    </nav>
  );
}

export default Navbar;
