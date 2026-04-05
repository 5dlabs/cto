import { Header } from "@/components/header";
import { Footer } from "@/components/footer";

export default function PrivacyPolicyPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10 pt-28 pb-12 px-6">
        <section className="section-frame py-16 px-6">
          <div className="max-w-4xl mx-auto">
            <div className="text-center mb-10">
              <h1 className="text-3xl sm:text-4xl font-bold mb-4">
                Privacy <span className="gradient-text">Policy</span>
              </h1>
              <p className="text-sm text-muted-foreground">Last updated: April 2026</p>
            </div>

            <div className="premium-shell rounded-2xl p-6 sm:p-8 space-y-6">
              <p className="text-sm text-muted-foreground leading-relaxed">
                This Privacy Policy describes how 5D Labs collects, uses, and protects personal information submitted through our brand website.
              </p>

              <div>
                <h2 className="text-base font-semibold mb-2">Information We Collect</h2>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  We may collect information you provide directly, such as your name, email address, company details, and other contact information submitted through forms or outreach requests.
                </p>
              </div>

              <div>
                <h2 className="text-base font-semibold mb-2">How We Use Information</h2>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  We use this information to respond to inquiries, provide requested services, communicate updates relevant to your request, and operate and improve our website and offerings.
                </p>
              </div>

              <div>
                <h2 className="text-base font-semibold mb-2">Data Sharing and Selling</h2>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  <span className="text-foreground font-semibold">We do not sell consumer personal information.</span> We also do not share consumer personal information, including phone numbers, with third parties or affiliates for their marketing or lead generation purposes.
                </p>
              </div>

              <div>
                <h2 className="text-base font-semibold mb-2">Contact</h2>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  For privacy questions or requests, contact us at{" "}
                  <a href="mailto:privacy@5dlabs.ai" className="text-cyan hover:text-cyan-400 transition-colors">
                    privacy@5dlabs.ai
                  </a>
                  .
                </p>
              </div>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </div>
  );
}
