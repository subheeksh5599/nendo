import Navbar from "./components/Navbar";
import HeroSection from "./components/HeroSection";
import InfoSection from "./components/InfoSection";
import BackedBySection from "./components/BackedBySection";
import UseCasesSection from "./components/UseCasesSection";
import CTASection from "./components/CTASection";
import Footer from "./components/Footer";

function LandingPage() {
  return (
    <div className="flex flex-col bg-[#F5F5F5]">
      <div className="relative h-screen overflow-hidden">
        <Navbar />
        <HeroSection />
      </div>
      <InfoSection />
      <BackedBySection />
      <UseCasesSection />
      <CTASection />
      <Footer />
    </div>
  );
}

export default LandingPage;
