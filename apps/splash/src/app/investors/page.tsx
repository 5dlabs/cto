import { Header } from "@/components/header";
import { Footer } from "@/components/footer";

const highlights = [
  {
    metric: "13",
    label: "AI Agents",
    description: "Specialized autonomous agents powering our operations",
  },
  {
    metric: "5+",
    label: "Active Ventures",
    description: "Parallel initiatives across AI, crypto, open-source tooling, smart home, and infrastructure",
  },
  {
    metric: "3",
    label: "Blockchains",
    description: "Agentic trading on Solana, Base, Near, and Sui",
  },
  {
    metric: "100%",
    label: "Open Source Core",
    description: "OpenClaw agent orchestration, built transparently",
  },
];

const differentiators = [
  {
    title: "AI-Native Startup Studio",
    description:
      "We don't just use AI — it's our core operating model. Every venture is built by autonomous AI agents from day one, enabling us to move faster and iterate cheaper than traditional startups.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
  },
  {
    title: "OpenClaw Platform",
    description:
      "Our proprietary agent orchestration layer is the engine behind everything. It coordinates AI agents across multiple ventures, CLIs, and infrastructure providers. This is the unfair advantage.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
      </svg>
    ),
  },
  {
    title: "Parallel Venture Validation",
    description:
      "Traditional startups bet everything on one idea. We run multiple ventures simultaneously, identifying product-market fit faster and cheaper. When something works, we double down.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
      </svg>
    ),
  },
  {
    title: "Crypto + AI Convergence",
    description:
      "We operate at the intersection of two of the largest technological shifts in a generation. Our agentic trading platform and AI engineering tools sit at this convergence point.",
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
                <span className="gradient-text glow-text-cyan">Invest</span>{" "}
                in the Future of Building
              </h1>

              <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
                5D Labs is raising its first round. We&apos;re building the
                infrastructure for AI-native startups — and proving the model
                with our own ventures.
              </p>
            </div>
          </div>
        </section>

        {/* Key Metrics */}
        <section className="py-12 px-6">
          <div className="max-w-5xl mx-auto">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
              {highlights.map((item, i) => (
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
                  engineering headcount — it&apos;s <span className="text-foreground font-medium">ideas, direction, and the
                  infrastructure to orchestrate AI agents at scale</span>.
                </p>
                <p>
                  5D Labs is positioned at this inflection point. We&apos;ve
                  built OpenClaw — an agent orchestration platform that
                  coordinates 13 specialized AI agents across the full software
                  development lifecycle. And we&apos;re using it to run a startup
                  studio that validates multiple ventures in parallel.
                </p>
                <p>
                  This isn&apos;t a bet on one product. It&apos;s a bet on the{" "}
                  <span className="text-foreground font-medium">
                    operating system for AI-native companies
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
              {differentiators.map((item, i) => (
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
                Current <span className="gradient-text">Portfolio</span>
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
                  Multi-Agent AI Engineering Platform
                </p>
                <p className="text-muted-foreground mb-4">
                  Thirteen specialized AI agents that ship complete features from
                  PRD to production. Self-healing bare metal infrastructure with
                  60-80% cost savings vs. cloud. Our flagship product and the
                  first proof that the model works.
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
                  <h3 className="text-2xl font-bold">Agentic Trading</h3>
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-cyan/10 text-cyan">
                    <span className="w-1.5 h-1.5 rounded-full bg-cyan animate-pulse" />
                    Building
                  </span>
                </div>
                <p className="text-[oklch(0.7_0.25_320)] text-sm font-medium mb-2">
                  HFT-Grade Autonomous Trading on Solana, Base, Near, and Sui
                </p>
                <p className="text-muted-foreground">
                  Hedge-fund-grade trading technology for individuals. AI-driven
                  strategy, execution, and risk management operating autonomously
                  across DeFi markets 24/7. The same speed and precision that Wall
                  Street uses — now accessible to everyone. Built on the same
                  OpenClaw orchestration layer as CTO.
                </p>
              </div>

              <div className="p-8 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-2xl font-bold">OpenClaw Platform</h3>
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-cyan/10 text-cyan">
                    <span className="w-1.5 h-1.5 rounded-full bg-cyan animate-pulse" />
                    Building
                  </span>
                </div>
                <p className="text-orange-400 text-sm font-medium mb-2">
                  Open-Source Agent Orchestration for Kubernetes
                </p>
                <p className="text-muted-foreground mb-4">
                  Kubernetes-native platform for deploying and managing AI agent
                  fleets. One-command TUI installer for desktop KinD clusters or
                  enterprise EKS, with GitOps via ArgoCD, NATS inter-agent messaging,
                  Grafana observability, and integrated secrets management. Will be
                  fully open-sourced and free.
                </p>
                <a
                  href="https://github.com/5dlabs/openclaw-platform"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-sm text-cyan hover:text-cyan/80 underline underline-offset-4 transition-colors"
                >
                  View on GitHub
                </a>
              </div>

              <div className="p-8 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-2xl font-bold">Sanctuary</h3>
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-cyan/10 text-cyan">
                    <span className="w-1.5 h-1.5 rounded-full bg-cyan animate-pulse" />
                    Building
                  </span>
                </div>
                <p className="text-emerald-400 text-sm font-medium mb-2">
                  AI Life Architect for Smart Homes
                </p>
                <p className="text-muted-foreground">
                  A privacy-forward AI system that turns multimodal life signals
                  into textural orchestration — shaping lighting, sound, temperature,
                  and microinterventions to improve wellbeing and household flow.
                  Demonstrates the studio model extending beyond crypto and dev tools.
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
              Interested in learning more? Download the one-pager, schedule a
              conversation, or meet the founder.
            </p>

            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <a
                href="/5dlabs-investor-one-pager.pdf"
                download
                className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 text-center"
              >
                Download One-Pager (PDF)
              </a>
              <a
                href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
                target="_blank"
                rel="noopener noreferrer"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all text-center"
              >
                Schedule a Call
              </a>
              <a
                href="/founder"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all text-center"
              >
                Meet the Founder
              </a>
            </div>
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
