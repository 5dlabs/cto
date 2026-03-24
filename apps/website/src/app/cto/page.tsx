import Image from "next/image";
import Link from "next/link";
import { WaitlistForm } from "@/components/cto/waitlist-form";
import { MorganHeroImage } from "@/components/cto/morgan-hero-image";
import { AgentGrid, type AgentSquad } from "@/components/cto/agent-card";
import { Header } from "@/components/cto/header";
import { HeroExperiment } from "@/components/cto/hero-experiment";
import { featureFlags } from "@/config/feature-flags";
import { MeshGradientBg } from "@/components/mesh-gradient-bg";
import { Reveal } from "@/components/reveal";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3000"
    : "https://5dlabs.ai";

const squads: AgentSquad[] = [
  {
    title: "Project Management",
    emoji: "🎯",
    agents: [
      {
        name: "Morgan",
        role: "Technical Program Manager",
        avatar: "/agents/morgan-avatar-512.png?v=20260318",
        color: "from-cyan-400 to-pink-500",
        badge: "Control Agent",
        cta: { label: "Talk to Morgan", href: "/cto/morgan" },
        description: "Your single point of contact. Morgan orchestrates the full project lifecycle — syncing your Git repo with Linear, decomposing PRDs into executable tasks, assigning agents, and tracking progress. Research tools for docs, web, and codebase context keep decisions grounded in evidence, not assumptions.",
        tools: ["Context7", "Firecrawl", "Perplexity", "Tavily", "Exa", "Repomix", "Linear", "GitHub"],
        skills: ["PRD Analysis", "Deep Research", "Multi-Agent", "Brainstorming", "Writing Plans"],
      },
    ],
  },
  {
    title: "Backend Engineering",
    emoji: "🦀",
    agents: [
      {
        name: "Rex",
        role: "Rust Architect",
        avatar: "/agents/rex-avatar-512.png",
        color: "from-orange-500 to-red-500",
        description: "Builds high-performance APIs, CLIs, and systems-level infrastructure in Rust. Handles async runtimes, zero-copy parsing, and memory-safe concurrency. When microseconds matter and correctness is non-negotiable.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Rust Patterns", "Error Handling", "Axum/Tokio", "Compound Engineering"],
      },
      {
        name: "Grizz",
        role: "Go Specialist",
        avatar: "/agents/grizz-avatar-512.png",
        color: "from-amber-500 to-orange-400",
        description: "Ships bulletproof backend services, REST/gRPC APIs, and Kubernetes operators in Go. Specializes in concurrent systems, clean architecture, and production-grade error handling. The go-to agent when you need reliable infrastructure that just works.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Go Patterns", "Concurrency", "gRPC/Chi", "Systematic Debugging"],
      },
      {
        name: "Nova",
        role: "Node.js Engineer",
        avatar: "/agents/nova-avatar-512.png",
        color: "from-purple-500 to-cyan-400",
        description: "Rapid API development, third-party integrations, and real-time services in Node.js and TypeScript. Uses Effect for type-safe error handling, Elysia for fast HTTP, and Drizzle for database access. Speed-to-market without cutting corners.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Effect Patterns", "Elysia", "Drizzle", "Better Auth"],
      },
      {
        name: "Viper",
        role: "Python Specialist",
        avatar: "/agents/viper-avatar-512.png",
        color: "from-yellow-500 to-green-500",
        description: "Data pipelines, ML workflows, automation scripts, and backend services in Python. Fast iteration, clean packages.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["FastAPI", "Pydantic", "Async Python", "Data Pipelines", "ML Tooling"],
      },
    ],
  },
  {
    title: "Frontend Engineering",
    emoji: "🎨",
    agents: [
      {
        name: "Blaze",
        role: "Web App Developer",
        avatar: "/agents/blaze-avatar-512.png",
        color: "from-blue-500 to-cyan-500",
        description: "Creates production-grade web applications with React and Next.js. Obsessive about performance, accessibility, and interaction design. Builds with shadcn/ui, TanStack, and motion libraries to ship interfaces that feel as good as they look.",
        tools: ["Context7", "shadcn/ui", "AI Elements", "TanStack", "GitHub"],
        skills: ["Frontend Excellence", "React Best Practices", "Anime.js", "Frontend Design"],
      },
      {
        name: "Tap",
        role: "Mobile Developer",
        avatar: "/agents/tap-avatar-512.png",
        color: "from-green-500 to-emerald-400",
        description: "Native-quality iOS and Android apps from a single TypeScript codebase using Expo and React Native. Handles navigation, offline storage, push notifications, and EAS builds. One codebase, two app stores, zero compromise on feel.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Expo Patterns", "React Native", "EAS Build", "Frontend Design"],
      },
      {
        name: "Spark",
        role: "Desktop Developer",
        avatar: "/agents/spark-avatar-512.png",
        color: "from-blue-500 to-yellow-400",
        description: "Cross-platform desktop apps with native OS integrations and offline-first architecture. Builds with Tauri and Electron — system tray utilities, local-first databases, file system access, and native menus. Desktop-grade UX from web technologies.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Electron Patterns", "Tauri", "Frontend Design"],
      },
    ],
  },
  {
    title: "Quality & Security",
    emoji: "🛡️",
    agents: [
      {
        name: "Cleo",
        role: "Quality Guardian",
        avatar: "/agents/cleo-avatar-512.png",
        color: "from-emerald-500 to-teal-500",
        description: "Enforces enterprise-grade code quality across the entire codebase. Refactors for maintainability, enforces naming conventions, reduces complexity, and ensures consistent patterns. Every PR leaves the codebase cleaner than it found it.",
        tools: ["Context7", "Firecrawl", "Repomix", "GitHub"],
        skills: ["Code Review", "Evaluation", "Code Maturity", "Advanced Evaluation"],
      },
      {
        name: "Cipher",
        role: "Security Sentinel",
        avatar: "/agents/cipher-avatar-512.png",
        color: "from-red-500 to-rose-500",
        description: "Runs security audits, dependency scans, and pen tests. Scans at code level then attacks deployed apps to find what static analysis misses.",
        tools: ["Context7", "Firecrawl", "Tavily", "OpenCode", "GitHub", "Snyk", "Nuclei", "Aikido"],
        skills: ["Semgrep", "CodeQL", "Pen Testing", "Red Teaming", "SARIF", "Audit Prep", "Supply Chain"],
      },
      {
        name: "Tess",
        role: "Testing Genius",
        avatar: "/agents/tess-avatar-512.png",
        color: "from-violet-500 to-purple-500",
        description: "Creates comprehensive test suites — unit, integration, and end-to-end. Uses Playwright for browser testing, property-based testing for edge cases, and TDD workflows that catch regressions before they reach production.",
        tools: ["Context7", "Kubernetes", "GitHub"],
        skills: ["Testing Strategies", "Playwright", "TDD", "Property-Based Testing"],
      },
    ],
  },
  {
    title: "Operations",
    emoji: "🚀",
    agents: [
      {
        name: "Stitch",
        role: "Code Reviewer",
        avatar: "/agents/stitch-avatar-512.png",
        color: "from-orange-500 to-blue-400",
        description: "Reviews every PR with surgical precision. Catches bugs, race conditions, and security issues that automated linters miss. Suggests architectural improvements and ensures code meets team standards before it merges.",
        tools: ["Context7", "Octocode", "GitHub"],
        skills: ["PR Review", "Code Review", "Differential Review"],
      },
      {
        name: "Atlas",
        role: "Integration Master",
        avatar: "/agents/atlas-avatar-512.png",
        color: "from-slate-500 to-zinc-500",
        description: "Manages the merge gate — rebases stale branches, resolves conflicts, runs final checks, and ensures clean integration into main. Keeps the commit history legible and the CI pipeline green.",
        tools: ["Context7", "Repomix", "GitHub"],
        skills: ["Git Integration", "Git Worktrees", "Multi-Agent", "Finishing Branch"],
      },
      {
        name: "Bolt",
        role: "Infrastructure & SRE",
        avatar: "/agents/bolt-avatar-512.png",
        color: "from-yellow-500 to-amber-500",
        description: "Your always-on SRE. Provisions bare metal, deploys services, monitors health, and triggers self-healing — so you never get paged.",
        tools: ["Context7", "Kubernetes", "GitHub"],
        skills: ["Kubernetes Operators", "ArgoCD/GitOps", "Secrets Mgmt", "Observability", "MCP Builder"],
      },
    ],
  },
  {
    title: "Specialists",
    emoji: "🔬",
    agents: [
      {
        name: "Block",
        role: "Blockchain Specialist",
        avatar: "/agents/block-avatar-512.png",
        color: "from-amber-500 to-orange-500",
        description: "Deploys and operates blockchain nodes across every supported chain. Validator setup, RPC endpoints, archive nodes — all on bare metal.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Ethereum", "NEAR", "Solana", "Node Ops", "Smart Contracts"],
      },
      {
        name: "Vex",
        role: "VR/Unity Developer",
        avatar: "/agents/vex-avatar-512.png",
        color: "from-violet-500 to-indigo-500",
        description: "Builds cross-platform VR and XR experiences with Unity and OpenXR. From Quest to PC to spatial web.",
        tools: ["Context7", "Octocode", "Firecrawl", "GitHub"],
        skills: ["Unity", "OpenXR", "Meta XR", "Three.js", "Cross-Platform XR"],
      },
      {
        name: "Angie",
        role: "Agent Builder",
        avatar: "/agents/angie-avatar-512.png?v=20260314",
        color: "from-indigo-500 to-cyan-400",
        description: "Designs OpenClaw-first agent systems, including orchestration, runtime patterns, and MCP-connected toolchains.",
        tools: ["OpenClaw", "MCP", "Context7", "Octocode", "GitHub"],
        skills: ["Agent Architecture", "OpenClaw", "LiveKit", "ElevenLabs", "LangGraph", "CrewAI", "AutoGen"],
      },
      {
        name: "Glitch",
        role: "Game Developer",
        avatar: "/agents/glitch-avatar-512.png",
        color: "from-fuchsia-500 to-pink-500",
        description: "Builds games and interactive experiences — indie titles, serious games, and browser-based play. Unity, Godot, Unreal, and WebGL.",
        tools: ["Context7", "GitHub", "Firecrawl", "Tavily"],
        skills: ["Unity", "Godot", "Unreal Engine", "WebGL", "Game Physics", "Shader Programming"],
      },
    ],
  },
  {
    title: "Business Team",
    emoji: "🏢",
    agents: [
      {
        name: "Lex",
        role: "Legal Counsel",
        avatar: "/agents/lex-avatar-512.png",
        color: "from-blue-600 to-indigo-600",
        description: "Contract review, compliance checks, and legal risk assessment. Flags clauses, suggests redlines, and surfaces precedent — trained on your jurisdiction and standards.",
        tools: ["Context7", "Firecrawl", "Ironclad", "Harvey AI", "Lexis+", "GitHub"],
        skills: ["Contract Review", "Risk Assessment", "Compliance", "Due Diligence", "Legal Research"],
      },
      {
        name: "Hype",
        role: "Marketing Strategist",
        avatar: "/agents/hype-avatar-512.png",
        color: "from-orange-500 to-rose-500",
        description: "Campaign strategy, copy, and analytics. From brand voice to conversion — SEO, competitor intel, and content that moves as fast as your product.",
        tools: ["Context7", "Firecrawl", "Surfer SEO", "Jasper", "Brand24", "Paradigm AI"],
        skills: ["Campaign Strategy", "Copywriting", "SEO", "Competitor Intel", "Brand Voice"],
      },
      {
        name: "Tally",
        role: "Accounting Specialist",
        avatar: "/agents/tally-avatar-512.png",
        color: "from-emerald-600 to-teal-600",
        description: "Bookkeeping, reconciliation, and financial reporting. Automates categorization, month-end close, and P&L — always accurate, always current.",
        tools: ["Context7", "Firecrawl", "Vic.ai", "QuickBooks", "Excel", "GitHub"],
        skills: ["Bookkeeping", "Reconciliation", "Financial Reporting", "Tax Prep", "Month-End Close"],
      },
      {
        name: "Chase",
        role: "Sales Agent",
        avatar: "/agents/chase-avatar-512.png",
        color: "from-amber-500 to-yellow-500",
        description: "Outreach, pipeline management, and closing. Handles prospecting, discovery, follow-ups, and deal tracking so your team stays focused on building.",
        tools: ["Context7", "Firecrawl", "Outreach", "Clay", "Salesforce", "Cognism"],
        skills: ["Prospecting", "Pipeline Mgmt", "CRM", "Discovery", "Follow-up"],
      },
    ],
  },
];

export default function Home() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <MeshGradientBg />
      <div className="fixed inset-0 noise-overlay z-[3]" />

      {/* Header */}
      <Header />

      {/* Content */}
      <main className="relative z-10">
        <HeroExperiment />

        {/* Meet Morgan Section */}
        <section id="morgan" className="section-frame py-20 px-6 scroll-mt-24">
          <Reveal>
          <div className="max-w-3xl mx-auto flex flex-col items-center text-center">
            <MorganHeroImage />
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Meet <span className="gradient-text-stitch">Morgan</span>
            </h2>
            <p className="text-lg text-muted-foreground mb-6 max-w-xl mx-auto">
              Your single point of contact. Chat, voice, or video—from any device. You only talk to Morgan; Morgan coordinates the rest.
            </p>
            <Link
              href="/cto/morgan"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-lg bg-gradient-to-r from-violet-500 via-indigo-500 to-cyan-500 text-white font-semibold transition-all shadow-lg shadow-indigo-500/35 hover:brightness-110"
            >
              Talk to Morgan
            </Link>
            <div className="premium-shell mt-12 w-full rounded-xl p-8 min-h-[280px] flex flex-col items-center justify-center gap-4">
              <p className="text-muted-foreground text-center">
                Chat, voice, or video with Morgan — your single point of contact.
              </p>
              <Link
                href="/cto/morgan"
                className="premium-chip rounded-full px-4 py-2 text-cyan hover:text-cyan-400 font-medium transition-colors"
              >
                Try Morgan live →
              </Link>
            </div>
          </div>
          </Reveal>
        </section>

        {/* Agents Section */}
        <section id="agents" className="py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <Reveal>
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                We Brought the <span className="gradient-text">Whole Team</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-xl mx-auto">
                Not one generic AI — a full team of domain experts working in parallel across your entire development lifecycle.
              </p>
            </div>
            </Reveal>

            {/* Agent Grid */}
            <AgentGrid squads={squads} />
          </div>
        </section>


        {/* Infrastructure Providers */}
        <section id="infrastructure" className="section-frame py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                <span className="gradient-text-stitch">17+</span> Bare Metal Providers
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Frictionless bare metal — no vendor research, no contracts, no provisioning delays. We maintain inventory across 17+ providers worldwide. Pick a region, pick your specs — we handle the rest.
              </p>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
              {[
                { name: "On-Premises", region: "Your hardware", desc: "USB key — plug in and run", own: true },
                { name: "Latitude.sh", region: "Americas, Europe, Asia", desc: "Global bare-metal cloud" },
                { name: "Hetzner", region: "Germany, Finland", desc: "European dedicated servers" },
                { name: "OVH", region: "Europe, Americas, Asia", desc: "Global bare-metal & cloud" },
                { name: "Vultr", region: "25+ locations", desc: "Worldwide infrastructure" },
                { name: "Scaleway", region: "France, Netherlands", desc: "European cloud provider" },
                { name: "Cherry Servers", region: "Lithuania, Netherlands", desc: "High-performance bare-metal" },
                { name: "DigitalOcean", region: "Americas, Europe, Asia", desc: "Developer-friendly droplets" },
                { name: "Servers.com", region: "Americas, Europe, Asia", desc: "Hybrid bare-metal cloud" },
                { name: "PhoenixNAP", region: "Americas, Europe, Asia", desc: "Dedicated servers" },
                { name: "i3D.net", region: "60+ locations, 6 continents", desc: "Low-latency bare metal" },
                { name: "Hivelocity", region: "50+ locations", desc: "Instant dedicated servers" },
                { name: "Denvr", region: "Canada, USA", desc: "GPU & AI compute" },
                { name: "Zenlayer", region: "360+ edge locations", desc: "Distributed edge bare metal" },
                { name: "NetActuate", region: "40+ locations", desc: "Edge bare metal" },
                { name: "HOSTKEY", region: "Europe, USA, Turkey", desc: "GPU & dedicated servers" },
                { name: "Leaseweb", region: "Global", desc: "Dedicated servers" },
              ].map((provider) => (
                <div
                  key={provider.name}
                  className={`p-4 rounded-xl border bg-card/30 backdrop-blur-sm text-center hover:border-cyan/30 transition-colors relative ${
                    (provider as { own?: boolean }).own
                      ? "border-cyan/40 bg-cyan/5"
                      : "border-border/50"
                  }`}
                >
                  {(provider as { own?: boolean }).own && (
                    <span className="absolute top-2 right-2 text-[9px] px-1.5 py-0.5 rounded font-medium bg-cyan/20 text-cyan">Our DC</span>
                  )}
                  <h4 className="font-semibold mb-1 text-foreground">{provider.name}</h4>
                  <p className="text-xs text-muted-foreground mb-1">{provider.region}</p>
                  <p className="text-xs text-muted-foreground">{provider.desc}</p>
                </div>
              ))}
            </div>

            <div className="grid sm:grid-cols-3 gap-4 mt-8">
              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5">
                <p className="text-sm font-semibold text-cyan mb-1">Always-Available Inventory</p>
                <p className="text-xs text-muted-foreground">Because we span 17+ providers, you&apos;re not blocked by one provider&apos;s stock constraints.</p>
              </div>
              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5">
                <p className="text-sm font-semibold text-cyan mb-1">Cross-Provider Mesh</p>
                <p className="text-xs text-muted-foreground">Encrypted cluster-to-cluster networking. Every node connected regardless of provider or region.</p>
              </div>
              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5">
                <p className="text-sm font-semibold text-cyan mb-1">Transparent Redundancy</p>
                <p className="text-xs text-muted-foreground">Use multiple providers simultaneously. If one has better pricing or availability, we route there.</p>
              </div>
            </div>
          </div>
        </section>

        {/* Startups Section */}
        <section id="web3-ai" className="section-frame py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Built for <span className="gradient-text-stitch">Startups</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Any startup. Reliable execution, strong operational controls, and infrastructure that doesn&apos;t turn into a second product to manage. We&apos;re especially strong for teams building in blockchain and AI.
              </p>
            </div>

            <div className="grid lg:grid-cols-2 gap-8">
              <div className="p-8 rounded-2xl border border-amber-500/20 bg-card/30 backdrop-blur-sm">
                <div className="mb-5 flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center shrink-0">
                    <svg className="w-5 h-5 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                    </svg>
                  </div>
                  <h3 className="text-xl font-semibold">On-Chain Infrastructure</h3>
                </div>
                <p className="text-muted-foreground mb-5">
                  Run validator nodes, RPC endpoints, archive infrastructure, and execution systems on dedicated hardware with managed operations underneath.
                </p>
                <div className="flex flex-wrap gap-2 mb-5">
                  {["Solana", "Base", "NEAR", "Sui", "Ethereum", "and more"].map((item) => (
                    <span key={item} className="text-xs px-2 py-1 rounded bg-amber-500/10 text-amber-400 font-medium">
                      {item}
                    </span>
                  ))}
                </div>
                <div className="space-y-3 text-sm text-muted-foreground">
                  <p>Managed upgrades, failover, and health monitoring.</p>
                  <p>Dedicated infrastructure for latency-sensitive workloads.</p>
                  <p>Key isolation and operational controls designed for serious teams.</p>
                </div>
              </div>

              <div className="p-8 rounded-2xl border border-purple-500/20 bg-card/30 backdrop-blur-sm">
                <div className="mb-5 flex items-center gap-3">
                  <div className="w-10 h-10 rounded-lg bg-purple-500/10 flex items-center justify-center shrink-0">
                    <svg className="w-5 h-5 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                    </svg>
                  </div>
                  <h3 className="text-xl font-semibold">AI Inference, Without the Ops Burden</h3>
                </div>
                <p className="text-muted-foreground mb-5">
                  Start with hosted providers, move to dedicated GPU infrastructure when usage justifies it, and keep the same product surface the entire way through.
                </p>
                <div className="flex flex-wrap gap-2 mb-5">
                  {["Hosted models", "Dedicated GPU", "Open-weight support", "Consistent APIs"].map((item) => (
                    <span key={item} className="text-xs px-2 py-1 rounded bg-purple-500/10 text-purple-300 font-medium">
                      {item}
                    </span>
                  ))}
                </div>
                <div className="space-y-3 text-sm text-muted-foreground">
                  <p>Use leading hosted models now or self-host later.</p>
                  <p>Keep one API contract as teams scale up.</p>
                  <p>Focus on product behavior, not inference plumbing.</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="section-frame py-20 px-6">
          <div className="max-w-2xl mx-auto text-center">
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Start <span className="gradient-text-stitch">Shipping</span>
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Other tools help you write code. CTO ships complete products — planned, designed, built, tested, and deployed by a coordinated team of specialists.
            </p>
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              {featureFlags.showStartNowButton && (
                <a
                  href="https://app.5dlabs.ai"
                  className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
                >
                  Start Now
                </a>
              )}
              <WaitlistForm />
            </div>
          </div>
        </section>

        {/* Footer */}
        <footer className="py-8 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
            <a href={homeHref} className="flex items-center gap-2" aria-label="Back to 5D Labs">
              <Image
                src="/5dlabs-logo-3d.jpg"
                alt="5D Labs"
                width={200}
                height={50}
                className="h-12 w-auto opacity-95"
              />
            </a>
            <p className="text-sm text-muted-foreground">
              © {new Date().getFullYear()} 5D Labs. Idea to Production — Autonomously.
            </p>
          </div>
        </footer>
      </main>
    </div>
  );
}
