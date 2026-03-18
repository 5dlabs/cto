import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { InvestorCtaButtons } from "@/components/investor-cta-buttons";

const highlights = [
  {
    metric: "1",
    label: "Operating Stack",
    description: "CTO and the internal trading engine working as one coordinated system to launch ventures",
  },
  {
    metric: "5+",
    label: "Venture Themes",
    description: "Multiple venture directions explored from a shared operating model",
  },
  {
    metric: "17+",
    label: "Infra Providers",
    description: "Global bare metal providers across every major region",
  },
  {
    metric: "4",
    label: "On-Chain Environments",
    description: "Internal trading infrastructure active across Solana, Base, Polygon, Near, and Sui",
  },
];

const differentiators = [
  {
    title: "One Operating Model",
    description:
      "CTO and the internal trading engine are not separate company theses. They are coordinated parts of one venture creation system, each reinforcing the other.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
  },
  {
    title: "CTO as Commercial Wedge",
    description:
      "CTO is the build engine we use internally and the first service we can sell externally. It proves the system can win in the open market.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
      </svg>
    ),
  },
  {
    title: "In-House Capital Engine",
    description:
      "The trading engine helps finance experimentation and generate live market intelligence — giving the studio its own source of capital and real-time signal.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
      </svg>
    ),
  },
  {
    title: "Venture Acceleration",
    description:
      "Because the studio runs its own build engine and capital engine, new ventures start with infrastructure already in place — dramatically compressing the time from idea to market.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
];

export default function InvestorsPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      {/* GridPulse at z-[2] from layout */}
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10 pt-24">
        {/* Hero */}
        <section className="py-20 px-6">
          <div className="max-w-4xl mx-auto text-center">
            <div>
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8">
                <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
                <span className="text-sm text-cyan font-medium">
                  Investor Relations
                </span>
              </div>

              <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight mb-6">
                Invest in the <span className="gradient-text glow-text-cyan">Engine</span>{" "}
                Behind the Ventures
              </h1>

              <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
                5D Labs is building the operating stack behind an AI-native venture
                studio. CTO is the commercial build engine. The internal trading engine
                is the capital engine. Together they power venture creation.
              </p>
            </div>
          </div>
        </section>

        {/* Key Metrics */}
        <section className="py-12 px-6">
          <div className="max-w-5xl mx-auto">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
              {highlights.map((item) => (
                <div
                  key={item.label}
                  className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
                >
                  <p className="text-3xl font-bold gradient-text mb-1">
                    {item.metric}
                  </p>
                  <p className="text-sm font-semibold text-foreground mb-1">
                    {item.label}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {item.description}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Mission */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <div>
              <h2 className="text-3xl sm:text-4xl font-bold mb-8 text-center">
                The <span className="gradient-text">Opportunity</span>
              </h2>

              <div className="space-y-6 text-lg text-muted-foreground max-w-3xl mx-auto">
                <p>
                  The cost of building software is collapsing. AI coding agents
                  are becoming good enough to ship production features
                  autonomously. This means the bottleneck is no longer
                  engineering headcount — it&apos;s <span className="text-foreground font-medium">ideas, capital,
                  direction, and the infrastructure to orchestrate execution at scale</span>.
                </p>
                <p>
                  5D Labs is building an operating stack for venture creation.
                  <span className="text-foreground font-medium"> CTO </span>
                  is the build engine — an operating system for software delivery.
                  The
                  <span className="text-foreground font-medium"> internal trading engine </span>
                  is the capital and market-intelligence engine that finances experimentation and keeps the team close to real markets.
                </p>
                <p>
                  CTO can be commercialized and sold as a service. The trading
                  engine is intentionally kept in-house because it is part of the
                  firm&apos;s edge rather than a product we want to distribute.
                </p>
                <p>
                  This isn&apos;t a bet on one app. It&apos;s a bet on a repeatable{" "}
                  <span className="text-foreground font-medium">
                    system for discovering, financing, and shipping AI-native ventures
                  </span>.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Differentiators */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Why <span className="gradient-text">5D Labs</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                What makes us different from every other AI company you&apos;ll
                hear a pitch from this year.
              </p>
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              {differentiators.map((item) => (
                <div
                  key={item.title}
                  className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
                >
                  <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4 text-cyan">
                    {item.icon}
                  </div>
                  <h3 className="text-xl font-semibold mb-2">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">
                    {item.description}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Ventures Overview */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                How the Studio <span className="gradient-text">Works</span>
              </h2>
            </div>

            <div className="space-y-6">
              <div className="p-8 rounded-xl border border-cyan/30 bg-cyan/5 backdrop-blur-sm">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-2xl font-bold">CTO</h3>
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-cyan/10 text-cyan">
                    <span className="w-1.5 h-1.5 rounded-full bg-cyan animate-pulse" />
                    Building
                  </span>
                </div>
                <p className="text-cyan text-sm font-medium mb-2">
                  Commercial Build Engine
                </p>
                <p className="text-muted-foreground mb-4">
                  CTO is the system we use internally to build software and the
                  first service we can sell externally. It is both a product and
                  the clearest proof that the operating model works.
                </p>
                <a
                  href="https://cto.5dlabs.ai"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-cyan hover:text-cyan/80 underline underline-offset-4 transition-colors"
                >
                  Visit cto.5dlabs.ai
                </a>
              </div>

              <div className="p-8 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-2xl font-bold">Internal Trading Engine</h3>
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-[oklch(0.7_0.25_320)]/10 text-[oklch(0.7_0.25_320)]">
                    <span className="w-1.5 h-1.5 rounded-full bg-[oklch(0.7_0.25_320)] animate-pulse" />
                    Internal
                  </span>
                </div>
                <p className="text-[oklch(0.7_0.25_320)] text-sm font-medium mb-2">
                  In-House Capital + Market-Intelligence Engine
                </p>
                <p className="text-muted-foreground">
                  This system helps 5D Labs generate internal revenue, stay close
                  to live on-chain markets, and finance venture creation. It is
                  built to keep the studio funded and informed with real market
                  signal.
                </p>
              </div>

              <div className="p-8 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-2xl font-bold">Venture Pipeline</h3>
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-yellow-500/10 text-yellow-400">
                    <span className="w-1.5 h-1.5 rounded-full bg-yellow-400 animate-pulse" />
                    Exploring
                  </span>
                </div>
                <p className="text-yellow-400 text-sm font-medium mb-2">
                  The Products That Emerge from the System
                </p>
                <p className="text-muted-foreground">
                  The long-term outcome is not just CTO or trading. The stack exists
                  to help 5D Labs discover, validate, and launch new customer-facing
                  ventures with more speed and better economics than a traditional
                  startup model.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Investor CTA */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-2xl mx-auto text-center">
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Let&apos;s <span className="gradient-text">Talk</span>
            </h2>
            <p className="text-lg text-muted-foreground mb-10 max-w-xl mx-auto">
              Interested in learning more? Schedule a conversation or meet the
              founder.
            </p>

            <InvestorCtaButtons />
            <p className="text-sm text-muted-foreground mt-6">
              Full pitch deck available upon request.
            </p>
          </div>
        </section>

        <Footer />
      </main>
    </div>
  );
}
