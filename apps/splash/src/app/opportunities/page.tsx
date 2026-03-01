import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { colorMap } from "@/lib/utils";

const stats = [
  { value: "20+", label: "years in production infra" },
  { value: "10,600+", label: "GitHub contributions (last year)" },
  { value: "1B+", label: "daily requests managed" },
  { value: "13", label: "AI agents shipping code" },
  { value: "60–80%", label: "cost savings on migrations" },
  { value: "SE→CTO", label: "in 3 months at Coinmiles" },
];

const opportunities = [
  {
    title: "Technical Co-Founder",
    badge: "Equity Partnership",
    badgeColor: "cyan",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    ),
    description:
      "Partnering with a non-technical or business-focused founder who has strong domain expertise, early traction, or a funded concept in AI, blockchain, or developer tooling.",
    listHeading: "Ideal fit:",
    items: [
      "Pre-seed to Seed stage, equity-based partnership",
      "AI-native products: agent platforms, autonomous workflows, LLM-powered tools",
      "Blockchain infrastructure: DeFi, on-chain agents, RPC/node operations",
      "Developer platforms and tooling",
      "You have conviction about the problem — I'll build the machine that solves it",
    ],
    footer: "Full-stack architecture from day one. I've gone from raw idea to production platform multiple times. Promoted SE → CTO in 3 months at Coinmiles.",
  },
  {
    title: "Fractional CTO / Advisory",
    badge: "Negotiable",
    badgeColor: "purple",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
    description:
      "Senior technical leadership without a full-time CTO hire. I embed part-time and own your technology strategy, architecture, and engineering culture — writing code, reviewing PRs, deploying infrastructure alongside your team.",
    listHeading: "What I typically own:",
    items: [
      "Technology strategy and roadmap aligned to business milestones",
      "Architecture decisions: build vs. buy, stack selection, scalability planning",
      "Engineering team hiring, onboarding, and process design",
      "Infrastructure cost optimization — proven 60–80% reductions on cloud-to-bare-metal migrations",
      "Investor-facing technical narratives for fundraising",
      "Security posture, observability, and incident response",
    ],
    footer: null,
    cta: { label: "See full consulting options →", href: "/consulting/" },
  },
  {
    title: "Infrastructure & Engineering Roles",
    badge: "Full-Time Remote",
    badgeColor: "blue",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
      </svg>
    ),
    description:
      "Full-time remote roles at US-based startups where I can lead infrastructure, platform engineering, or AI systems teams. Most interested in early-to-growth-stage companies where I can have outsized impact.",
    listHeading: "Roles that fit:",
    items: [
      "VP / Director of Infrastructure",
      "Head of Platform Engineering",
      "Infrastructure Engineering Manager",
      "AI Systems / Agent Platform Lead",
      "Principal Engineer, Infrastructure",
    ],
    footer: "Must-haves: Remote-friendly, building something technically ambitious, small-to-mid-size team where individual contributors still matter.",
  },
];

const technicalDepth = [
  {
    title: "AI & Agent Systems",
    color: "cyan",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
    description:
      "Multi-agent orchestration with OpenClaw, MCP server with 60+ tools, NATS-based inter-agent messaging. Model-agnostic: commercial LLM APIs and self-hosted open-weight models with hot-swapping. I build the infrastructure that makes AI agents reliable, observable, and production-grade.",
  },
  {
    title: "Blockchain & Solana",
    color: "purple",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
    description:
      "Solana program development, on-chain trading agents, DeFi protocol integration. RPC infrastructure at scale — 50+ blockchain clients including validators and archival nodes. Low-latency systems where milliseconds matter.",
  },
  {
    title: "Infrastructure & DevOps",
    color: "blue",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
      </svg>
    ),
    description:
      "Bare-metal Kubernetes on Talos Linux, ArgoCD GitOps, Cilium networking, zero-trust architecture. Led migrations from EC2/Docker Compose to fully GitOps Kubernetes across 16 global regions. Infrastructure that's secure by default and costs a fraction of cloud.",
  },
  {
    title: "Platform Engineering",
    color: "orange",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
      </svg>
    ),
    description:
      "Rust systems programming (Tokio, Axum, Serde), Go, TypeScript. Custom Kubernetes operators and CRDs, event-driven architectures. Replaced 15+ managed cloud services with self-hosted operators — CloudNative-PG, Strimzi Kafka, SeaweedFS, ClickHouse.",
  },
  {
    title: "Observability & Reliability",
    color: "yellow",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    ),
    description:
      "Prometheus, Grafana, Loki, OpenTelemetry, Fluent-bit. Built self-healing systems with automated incident remediation — 9 alert types with agents that autonomously diagnose and fix failures. Migrated from DataDog to open-source stacks.",
  },
  {
    title: "Team Leadership",
    color: "emerald",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
      </svg>
    ),
    description:
      "Led and reorganized 13 infrastructure engineers into specialized functional teams across 16 global regions. Regular 1:1s, mentorship, career development, and hiring. Built streamlined onboarding enabling new engineers to contribute from week one.",
  },
];

const caseStudies = [
  {
    company: "CTO Platform",
    role: "Founder & Architect",
    color: "cyan",
    problem:
      "Early-stage startups can't afford full engineering teams, but need production-grade software infrastructure from day one.",
    what: "Built a Cognitive Task Orchestrator deploying 13 specialized AI agents — Rex (Orchestrator), Blaze (Implementer), Morgan (Architect), Sentinel (Security), Pixel (Frontend), Echo (QA) — across the full SDLC. Runs on bare-metal Kubernetes with Talos Linux, ArgoCD, Cilium, and a full observability stack. Custom MCP server with 60+ tools and a self-healing system with 9 automated remediation types.",
    result:
      "\"CTO as a Service\" platform replacing early engineering hires at 60–80% lower infrastructure cost than cloud. Open-sourced under AGPL-3.0.",
  },
  {
    company: "Pocket Network",
    role: "Head of Infrastructure Engineering",
    color: "purple",
    problem:
      "Decentralized RPC infrastructure serving 50+ blockchain networks needed to handle massive throughput with high reliability across a global footprint.",
    what: "Promoted from Sr. DevOps Engineer to DevOps Team Lead to Head of Infrastructure within 12 months. Led migration from EC2/Docker Compose to GitOps with Kubernetes and ArgoCD across 16 global regions. Reorganized 13 engineers into specialized functional teams. Migrated from DataDog to VictoriaMetrics/Loki/Grafana.",
    result:
      "Infrastructure serving 1B+ daily requests. Significant cost reductions across compute and observability. A well-structured team that could operate independently.",
  },
  {
    company: "Blocknative",
    role: "Senior Reliability Engineer",
    color: "blue",
    problem:
      "Mixed systemd/unikernel architecture needed modernization to support high-performance blockchain gas estimation infrastructure.",
    what: "Led strategic transformation achieving 100% Kubernetes adoption with ArgoCD-based GitOps. Deployed bare-metal Kubernetes on Latitude hardware with Cilium CNI. Built Gas Network infrastructure — a distributed oracle providing real-time gas price data across 35+ blockchain networks. Implemented nOPs and Kubecost for cost analysis.",
    result:
      "Fully containerized, GitOps-driven infrastructure with self-service developer workflows and a 40% cost reduction.",
  },
];

const ventures = [
  { name: "CTO Platform", desc: "Multi-agent AI engineering platform", href: "https://cto.5dlabs.ai", status: "Pre-launch" },
  { name: "OpenClaw Platform", desc: "Open-source Kubernetes-native agent orchestration", href: "https://github.com/5dlabs/openclaw-platform", status: "Building" },
  { name: "Agentic Trading", desc: "Autonomous on-chain trading agents on Solana, Base, Near, and Sui", href: null, status: "Building" },
  { name: "Sanctuary", desc: "AI-powered smart home orchestration", href: null, status: "Building" },
];

export default function OpportunitiesPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.06_0.03_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      {/* GridPulse at z-[2] from layout */}
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10 pt-24">

        {/* Hero */}
        <section className="min-h-[60vh] flex flex-col items-center justify-center px-6 py-20 text-center">
          <div
            className="max-w-4xl mx-auto fade-in-up"
          >
            <div
              className="inline-flex items-center gap-2 px-4 py-2 rounded-full glass-badge mb-8 fade-in-up"
            >
              <span className="w-2 h-2 rounded-full bg-cyan animate-pulse" />
              <span className="text-sm text-cyan font-semibold tracking-wide">
                Builder · Operator · Technical Co-Founder
              </span>
            </div>

            <h1
              className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6 fade-in-up"
            >
              <span className="gradient-text glow-text-cyan">Let&apos;s Build</span>
              <br />
              <span className="text-foreground">Something Real.</span>
            </h1>

            <p
              className="text-xl sm:text-2xl text-muted-foreground max-w-3xl mx-auto mb-10 leading-relaxed fade-in-up"
            >
              I build AI agent platforms and bare-metal infrastructure for a living —{" "}
              <span className="text-foreground font-semibold">10,600+ GitHub contributions</span> in
              the last year alone. I&apos;m looking to do more of it: as a technical co-founder, a
              fractional CTO, or the infrastructure lead at an early-stage AI or blockchain startup.
            </p>

            <div
              className="flex flex-col sm:flex-row justify-center gap-4 fade-in-up"
            >
              <a
                href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
                target="_blank"
                rel="noopener noreferrer"
                className="px-8 py-4 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all glass-cta hover:scale-105 shadow-xl shadow-cyan-500/30"
                data-umami-event="opportunities-hero-schedule-call"
              >
                Schedule a Discovery Call
              </a>
              <a
                href="/consulting/"
                className="px-8 py-4 rounded-xl glass text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all"
              >
                See Consulting Options
              </a>
            </div>
          </div>
        </section>

        {/* What I'm Looking For */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div
              className="text-center mb-16 reveal-on-scroll"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                What I&apos;m{" "}
                <span className="gradient-text">Looking For</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Three types of engagement where I can make the most impact.
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-8">
              {opportunities.map((opp, i) => {
                const colors = colorMap[opp.badgeColor];
                return (
                  <div
                    key={opp.title}
                    className="relative p-6 rounded-2xl glass-card glass-shimmer overflow-hidden flex flex-col reveal-on-scroll"
                  >
                    <div className="flex items-start justify-between mb-4">
                      <div className={`w-12 h-12 rounded-xl ${colors.bg} border ${colors.border} flex items-center justify-center`}>
                        <span className={colors.text}>{opp.icon}</span>
                      </div>
                      <span className={`text-xs font-semibold px-3 py-1 rounded-full ${colors.bg} ${colors.text} border ${colors.border}`}>
                        {opp.badge}
                      </span>
                    </div>

                    <h3 className="text-xl font-bold mb-3">{opp.title}</h3>
                    <p className="text-sm text-muted-foreground mb-4 leading-relaxed">
                      {opp.description}
                    </p>

                    <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                      {opp.listHeading}
                    </p>
                    <ul className="space-y-1.5 flex-1">
                      {opp.items.map((item) => (
                        <li key={item} className="flex items-start gap-2 text-sm text-muted-foreground">
                          <span className={`mt-1 w-1.5 h-1.5 rounded-full ${colors.bg.replace("/10", "")} shrink-0`} />
                          {item}
                        </li>
                      ))}
                    </ul>

                    {opp.footer && (
                      <p className="mt-4 pt-4 border-t border-border/30 text-sm text-muted-foreground italic">
                        {opp.footer}
                      </p>
                    )}
                    {opp.cta && (
                      <a
                        href={opp.cta.href}
                        className={`mt-4 pt-4 border-t border-border/30 text-sm ${colors.text} hover:underline`}
                      >
                        {opp.cta.label}
                      </a>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </section>

        {/* What I Bring — stats */}
        <section className="py-20 px-6 border-t border-border/30 glass-section">
          <div className="max-w-6xl mx-auto">
            <div
              className="text-center mb-12 reveal-on-scroll"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                What I <span className="gradient-text">Bring</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                20+ years shipping production systems. I&apos;ve held every role on the infrastructure
                ladder: network engineer → DevOps → SRE → team lead → Head of Infra → CTO → founder.
              </p>
            </div>

            {/* Stats row */}
            <div
              className="flex flex-wrap justify-center gap-4 mb-16 reveal-on-scroll"
            >
              {stats.map((stat) => (
                <div key={stat.value} className="flex items-center gap-2 px-5 py-3 rounded-xl glass-subtle">
                  <span className="text-xl font-bold text-foreground">{stat.value}</span>
                  <span className="text-sm text-muted-foreground">{stat.label}</span>
                </div>
              ))}
            </div>

            {/* Technical depth grid */}
            <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
              {technicalDepth.map((area, i) => {
                const colors = colorMap[area.color];
                return (
                  <div
                    key={area.title}
                    className="relative p-6 rounded-2xl glass-card glass-shimmer overflow-hidden reveal-on-scroll"
                  >
                    <div className={`w-11 h-11 rounded-xl ${colors.bg} border ${colors.border} flex items-center justify-center mb-4`}>
                      <span className={colors.text}>{area.icon}</span>
                    </div>
                    <h3 className="text-base font-semibold mb-2">{area.title}</h3>
                    <p className="text-sm text-muted-foreground leading-relaxed">{area.description}</p>
                  </div>
                );
              })}
            </div>
          </div>
        </section>

        {/* Case Studies */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <div
              className="text-center mb-16 reveal-on-scroll"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                How I&apos;ve <span className="gradient-text">Done It</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Three case studies.                 Same pattern every time: inherit a hard problem, build the solution, ship results.
              </p>
            </div>

            <div className="space-y-8">
              {caseStudies.map((cs, i) => {
                const colors = colorMap[cs.color];
                return (
                  <div
                    key={cs.company}
                    className="relative p-8 rounded-2xl glass-card glass-shimmer overflow-hidden reveal-on-scroll"
                  >
                    <div className="flex items-start gap-4 mb-6">
                      <div className={`shrink-0 w-12 h-12 rounded-xl ${colors.bg} border ${colors.border} flex items-center justify-center`}>
                        <span className={`text-lg font-bold ${colors.text}`}>{i + 1}</span>
                      </div>
                      <div>
                        <h3 className="text-xl font-bold">{cs.company}</h3>
                        <p className={`text-sm ${colors.text} font-medium`}>{cs.role}</p>
                      </div>
                    </div>

                    <div className="grid md:grid-cols-3 gap-6">
                      <div>
                        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Problem</p>
                        <p className="text-sm text-muted-foreground leading-relaxed">{cs.problem}</p>
                      </div>
                      <div>
                        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">What I Did</p>
                        <p className="text-sm text-muted-foreground leading-relaxed">{cs.what}</p>
                      </div>
                      <div>
                        <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Result</p>
                        <p className={`text-sm leading-relaxed font-medium ${colors.text}`}>{cs.result}</p>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </section>

        {/* Currently Building */}
        <section className="py-20 px-6 border-t border-border/30 glass-section">
          <div className="max-w-4xl mx-auto">
            <div
              className="text-center mb-12 reveal-on-scroll"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Currently <span className="gradient-text">Building</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                <a href="https://5dlabs.ai" className="text-cyan hover:underline">5D Labs</a>{" "}
                — an AI-first startup studio operating multiple ventures in parallel.
              </p>
            </div>

            <div className="grid sm:grid-cols-2 gap-4">
              {ventures.map((v, i) => (
                <div
                  key={v.name}
                  className="relative p-5 rounded-2xl glass-card glass-shimmer overflow-hidden reveal-on-scroll"
                >
                  <div className="flex items-start justify-between mb-2">
                    <h3 className="text-base font-semibold">
                      {v.href ? (
                        <a href={v.href} target="_blank" rel="noopener noreferrer" className="text-cyan hover:underline">
                          {v.name}
                        </a>
                      ) : (
                        <span>{v.name}</span>
                      )}
                    </h3>
                    <span className="text-xs text-cyan bg-cyan/10 border border-cyan/20 px-2 py-0.5 rounded-full">
                      {v.status}
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground">{v.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Let's Talk */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-2xl mx-auto text-center">
            <div
              className="reveal-on-scroll"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Let&apos;s <span className="gradient-text">Talk</span>
              </h2>
              <p className="text-lg text-muted-foreground mb-8">
                30 minutes, no commitment. If there&apos;s a fit, we&apos;ll figure it out fast.
              </p>

              <a
                href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-block px-10 py-4 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all glass-cta hover:scale-105 shadow-xl shadow-cyan-500/30 mb-10"
                data-umami-event="opportunities-cta-schedule-call"
              >
                Schedule a Discovery Call
              </a>

              <div className="flex justify-center gap-6 text-sm text-muted-foreground mb-8">
                <a
                  href="https://x.com/5dlabs"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-foreground transition-colors"
                  data-umami-event="opportunities-twitter"
                >
                  @5dlabs on X
                </a>
                <a
                  href="https://github.com/5dlabs"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-foreground transition-colors"
                  data-umami-event="opportunities-github"
                >
                  GitHub
                </a>
                <a
                  href="https://discord.gg/r334tFP87Y"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="hover:text-foreground transition-colors"
                  data-umami-event="opportunities-discord"
                >
                  Discord
                </a>
              </div>

              <p className="text-sm text-muted-foreground">
                Based in Victoria, BC. Open to remote roles with US-based companies.
              </p>
            </div>
          </div>
        </section>

        <Footer />
      </main>
    </div>
  );
}
