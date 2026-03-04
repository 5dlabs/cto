import { WaitlistForm } from "@/components/waitlist-form";
import { AgentGrid, type AgentSquad } from "@/components/agent-card";
import { Header } from "@/components/header";
import { TechStack } from "@/components/tech-stack";
import { featureFlags } from "@/config/feature-flags";

const squads: AgentSquad[] = [
  {
    title: "Project Management",
    emoji: "🎯",
    agents: [
      {
        name: "Morgan",
        role: "Technical Program Manager",
        avatar: "/agents/morgan-avatar-512.png",
        color: "from-cyan-400 to-pink-500",
        description: "Orchestrates project lifecycles—syncing GitHub with Linear, decomposing PRDs into tasks. Research tools for docs, web, and codebase context.",
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
        description: "Builds high-performance APIs and systems-level infrastructure. When microseconds matter.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Rust Patterns", "Error Handling", "Axum/Tokio", "Compound Engineering"],
      },
      {
        name: "Grizz",
        role: "Go Specialist",
        avatar: "/agents/grizz-avatar-512.png",
        color: "from-amber-500 to-orange-400",
        description: "Ships bulletproof backend services, REST/gRPC APIs, and Kubernetes operators.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Go Patterns", "Concurrency", "gRPC/Chi", "Systematic Debugging"],
      },
      {
        name: "Nova",
        role: "Node.js Engineer",
        avatar: "/agents/nova-avatar-512.png",
        color: "from-purple-500 to-cyan-400",
        description: "Rapid API development and third-party integrations. Speed-to-market specialist.",
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
        description: "Creates stunning web applications with modern component libraries.",
        tools: ["Context7", "shadcn/ui", "AI Elements", "TanStack", "GitHub"],
        skills: ["Frontend Excellence", "React Best Practices", "Anime.js", "Frontend Design"],
      },
      {
        name: "Tap",
        role: "Mobile Developer",
        avatar: "/agents/tap-avatar-512.png",
        color: "from-green-500 to-emerald-400",
        description: "Native-quality iOS and Android apps from a single TypeScript codebase.",
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Expo Patterns", "React Native", "EAS Build", "Frontend Design"],
      },
      {
        name: "Spark",
        role: "Desktop Developer",
        avatar: "/agents/spark-avatar-512.png",
        color: "from-blue-500 to-yellow-400",
        description: "Cross-platform desktop apps with native integrations and offline-first architecture.",
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
        description: "Refactors for maintainability and ensures enterprise-grade code quality.",
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
        description: "Creates comprehensive test suites—unit, integration, and e2e.",
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
        description: "Reviews every PR with surgical precision—catches bugs, suggests improvements.",
        tools: ["Context7", "Octocode", "GitHub"],
        skills: ["PR Review", "Code Review", "Differential Review"],
      },
      {
        name: "Atlas",
        role: "Integration Master",
        avatar: "/agents/atlas-avatar-512.png",
        color: "from-slate-500 to-zinc-500",
        description: "Manages PR merges, rebases stale branches, and ensures clean integration.",
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
        name: "Glitch",
        role: "Game Developer",
        color: "from-fuchsia-500 to-pink-500",
        description: "Builds games and interactive experiences — indie titles, serious games, and browser-based play. Unity, Godot, Unreal, and WebGL.",
        tools: ["Context7", "GitHub", "Firecrawl", "Tavily"],
        skills: ["Unity", "Godot", "Unreal Engine", "WebGL", "Game Physics", "Shader Programming"],
      },
    ],
  },
];

export default function Home() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <div className="fixed inset-0 noise-overlay z-0" />

      {/* Header */}
      <Header />

      {/* Content */}
      <main className="relative z-10">
        {/* Hero Section */}
        <section id="hero" className="min-h-screen flex flex-col items-center justify-center px-6 py-20 pt-24">
          <div className="max-w-4xl mx-auto text-center">
            {/* Badge */}
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8">
              <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
              <span className="text-sm text-cyan font-medium">
                From PRD to Production — Autonomously
              </span>
            </div>

            {/* Headline - LCP element, must be visible immediately */}
            <h1 className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6">
              <span className="gradient-text glow-text-cyan">Your Engineering Team</span>
              <br />
              <span className="text-foreground">Lives Here</span>
            </h1>

            {/* Subheadline */}
            <p className="text-xl sm:text-2xl text-muted-foreground max-w-2xl mx-auto mb-10">
              A growing team of specialized AI agents that ship complete features. From requirements to deployed code—automatically.
            </p>

            {/* CTA Buttons */}
            <div className="flex flex-col sm:flex-row justify-center gap-4 mb-16">
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

            {/* Stats */}
            <div className="flex flex-wrap justify-center gap-8 text-sm text-muted-foreground">
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">60-80%</span>
                <span>cost savings vs cloud</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">17+</span>
                <span>infra providers</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">∞</span>
                <span>faster shipping</span>
              </div>
            </div>
          </div>

          {/* Scroll indicator */}
          <div className="absolute bottom-10 left-1/2 -translate-x-1/2">
            <div className="w-6 h-10 rounded-full border-2 border-muted-foreground/30 flex justify-center pt-2 scroll-bounce">
              <div className="w-1 h-2 rounded-full bg-cyan" />
            </div>
          </div>
        </section>

        {/* Agents Section */}
        <section id="agents" className="py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                We Brought the <span className="gradient-text">Whole Team</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-xl mx-auto">
                Not one generic AI — a full team of domain experts working in parallel across your entire development lifecycle.
              </p>
            </div>

            {/* Agent Grid */}
            <AgentGrid squads={squads} />
          </div>
        </section>

        {/* Tech Stack Section */}
        <TechStack />

        {/* Ecosystem Section */}
        <section id="ecosystem" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Your <span className="gradient-text">Entire Stack</span>, Orchestrated
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                From languages to infrastructure. Bring your own keys, pick your CLIs—we handle everything else.
              </p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Languages & Frameworks</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>Rust, Go, Node.js, TypeScript</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>React, Next.js, Expo, Electron</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>Axum, Elysia/Bun, Chi, gRPC</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>FastAPI (planned), more coming soon</span>
                  </div>
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-magenta/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Infrastructure & Databases</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>Kubernetes, Helm, Terraform</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>PostgreSQL, Redis, Kafka, MongoDB</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>Bolt provisions &amp; monitors bare metal</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>Self-healing infrastructure built-in</span>
                  </div>
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-yellow/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">AI CLIs & Models</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Claude Code, Cursor, Factory</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Codex, Gemini, OpenCode</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>ChatGPT Codex, Gemini, Claude models</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Ollama, vLLM for local inference</span>
                  </div>
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-green/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Integrations</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>GitHub Apps for each agent</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>Linear project management</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>Prometheus, Grafana, Loki</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>Ever-growing MCP ecosystem</span>
                  </div>
                </div>
              </div>
            </div>

            <div className="text-center mt-12">
              <p className="text-muted-foreground">
                Submit a PRD, connect your repo, and watch your AI team ship. No manual handoffs, no context switching.
              </p>
            </div>
          </div>
        </section>

        {/* Infrastructure Providers Section */}
        <section id="infrastructure" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-10">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                One Platform, <span className="gradient-text">17+ Providers</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Frictionless bare metal — no vendor research, no contracts, no provisioning delays. We maintain inventory across 17+ providers worldwide. Pick a region, pick your specs — we handle the rest.
              </p>
            </div>

            {/* Feature callouts */}
            <div className="grid sm:grid-cols-3 gap-4 mb-12">
              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5">
                <p className="text-sm font-semibold text-cyan mb-1">🗺 Region-First Deployment</p>
                <p className="text-xs text-muted-foreground">Pick any region worldwide. We surface available servers from inventory in real time.</p>
              </div>
              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5">
                <p className="text-sm font-semibold text-cyan mb-1">📦 Always-Available Inventory</p>
                <p className="text-xs text-muted-foreground">Because we span 17+ providers, you&apos;re not blocked by one provider&apos;s stock constraints.</p>
              </div>
              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5">
                <p className="text-sm font-semibold text-cyan mb-1">🤝 Fully Managed Contracts</p>
                <p className="text-xs text-muted-foreground">We manage vendor relationships and procurement. You pay us, we handle the rest.</p>
              </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {[
                { name: "5D Labs", region: "Victoria, BC, Canada", desc: "Our own data center", own: true },
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
                    <span className="absolute top-2 right-2 text-[9px] px-1.5 py-0.5 rounded font-medium bg-cyan/20 text-cyan">✦ Our DC</span>
                  )}
                  <h4 className="font-semibold mb-1 text-foreground">{provider.name}</h4>
                  <p className="text-xs text-muted-foreground mb-1">{provider.region}</p>
                  <p className="text-xs text-muted-foreground">{provider.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Multi-Region Cluster Connectivity Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Connected Across Regions & <span className="gradient-text">Providers</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Spin up clusters in different regions — even on different providers — and they stay network-connected. Encrypted tunnels between every node, cross-cluster service discovery, and unified network policy. From your application&apos;s perspective, it&apos;s one flat network.
              </p>
            </div>

            <div className="grid sm:grid-cols-3 gap-6">
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-2">Cross-Provider Mesh</h3>
                <p className="text-sm text-muted-foreground">Encrypted cluster-to-cluster networking. Every node connected regardless of provider or region.</p>
              </div>
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-2">Automatic Failover</h3>
                <p className="text-sm text-muted-foreground">If a data center goes down, traffic shifts to healthy regions automatically. Zero manual intervention.</p>
              </div>
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-2">Transparent Redundancy</h3>
                <p className="text-sm text-muted-foreground">Use up to 5 providers simultaneously. If one has better pricing or availability, we route there. Your app doesn&apos;t notice.</p>
              </div>
            </div>
          </div>
        </section>

        {/* Cloud Exit Journey Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Your Path Out of <span className="gradient-text">the Cloud</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Most teams don&apos;t exit cloud overnight. CTO supports every stage — and we provide the infrastructure AND the tooling to complete the transition.
              </p>
            </div>

            <div className="grid sm:grid-cols-3 gap-6 mb-8">
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="text-xs font-mono text-muted-foreground mb-3 uppercase tracking-wider">[1] Cloud</div>
                <h3 className="font-semibold mb-2 text-foreground">Still on AWS/GCP?</h3>
                <p className="text-sm text-muted-foreground">We meet you there. Start shipping with AI agents now.</p>
              </div>
              <div className="p-6 rounded-xl border border-cyan/20 bg-cyan/5 backdrop-blur-sm relative">
                <div className="absolute -top-3 left-1/2 -translate-x-1/2 text-muted-foreground text-sm">→</div>
                <div className="text-xs font-mono text-cyan mb-3 uppercase tracking-wider">[2] Hosted Bare Metal</div>
                <h3 className="font-semibold mb-2 text-foreground">Dedicated servers at 17+ global providers.</h3>
                <p className="text-sm text-muted-foreground">Your infra, your data, our management layer.</p>
              </div>
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm relative">
                <div className="absolute -top-3 left-1/2 -translate-x-1/2 text-muted-foreground text-sm">→</div>
                <div className="text-xs font-mono text-muted-foreground mb-3 uppercase tracking-wider">[3] On-Prem / Colo</div>
                <h3 className="font-semibold mb-2 text-foreground">Own hardware, your facility or ours.</h3>
                <p className="text-sm text-muted-foreground">Full sovereignty. We still manage it.</p>
              </div>
            </div>

            <div className="p-5 rounded-xl border border-border/30 bg-muted/20 text-center">
              <p className="text-sm text-muted-foreground">We provide the migration tooling and automation — not just the destination.</p>
            </div>
          </div>
        </section>

        {/* Integrations Section */}
        <section id="integrations" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Works With Your <span className="gradient-text">Entire Stack</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Native integrations with the tools your team already uses — from project management to alerting to observability.
              </p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6">
              {/* Project Management */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-1">Project Management</h3>
                <p className="text-xs text-muted-foreground mb-4">Linear is primary — full agent activity sync, PRD intake, and live task updates. Other platforms get task creation and status updates.</p>
                <div className="flex flex-wrap gap-1.5">
                  {[
                    { name: "Linear", primary: true },
                    { name: "GitHub Issues", primary: true },
                    { name: "Jira", primary: false },
                    { name: "Asana", primary: false },
                    { name: "Trello", primary: false },
                    { name: "Monday", primary: false },
                    { name: "Notion", primary: false },
                    { name: "ClickUp", primary: false },
                  ].map(({ name, primary }) => (
                    <span key={name} className={`text-[11px] px-2 py-0.5 rounded-md font-medium ${primary ? "bg-[oklch(0.7_0.25_320)]/15 text-[oklch(0.7_0.25_320)]" : "bg-muted/50 text-muted-foreground"}`}>{name}</span>
                  ))}
                </div>
              </div>

              {/* Communication & Alerting */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-1">Communication & Alerting</h3>
                <p className="text-xs text-muted-foreground mb-4">Agents post progress updates, incident alerts, and deployment notifications to your channels in real time.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["Discord", "Slack", "Microsoft Teams", "PagerDuty", "Email"].map(name => (
                    <span key={name} className="text-[11px] px-2 py-0.5 rounded-md font-medium bg-cyan/10 text-cyan">{name}</span>
                  ))}
                </div>
              </div>

              {/* Observability */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-orange-500/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-orange-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-1">Observability</h3>
                <p className="text-xs text-muted-foreground mb-4">Self-hosted Grafana, Prometheus, and Loki pre-wired. Datadog supported for teams already invested in it.</p>
                <div className="flex flex-wrap gap-1.5">
                  {[
                    { name: "Grafana", primary: true },
                    { name: "Prometheus", primary: true },
                    { name: "Loki", primary: true },
                    { name: "Jaeger", primary: true },
                    { name: "OpenTelemetry", primary: true },
                    { name: "Datadog", primary: false },
                  ].map(({ name, primary }) => (
                    <span key={name} className={`text-[11px] px-2 py-0.5 rounded-md font-medium ${primary ? "bg-orange-500/10 text-orange-400" : "bg-muted/50 text-muted-foreground"}`}>{name}</span>
                  ))}
                </div>
              </div>

              {/* Source Control & CI */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-green-500/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-1">Source Control & CI</h3>
                <p className="text-xs text-muted-foreground mb-4">Each agent has its own GitHub App. PRs, reviews, and deployments are fully automated through Git.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["GitHub Apps", "GitHub Actions", "ArgoCD", "Webhooks", "PR Automation"].map(name => (
                    <span key={name} className="text-[11px] px-2 py-0.5 rounded-md font-medium bg-green-500/10 text-green-400">{name}</span>
                  ))}
                </div>
              </div>

              {/* Security Scanning */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-10 h-10 rounded-lg bg-rose-500/10 flex items-center justify-center mb-4">
                  <svg className="w-5 h-5 text-rose-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <h3 className="font-semibold mb-1">Security Scanning</h3>
                <p className="text-xs text-muted-foreground mb-4">Vulnerability scanning, SCA, AI-native remediation, and supply-chain protection — all surfaced through Cipher&apos;s agent interface.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["Snyk", "Nuclei", "Aikido", "Socket", "Trivy", "Gitleaks", "Datadog", "Dynatrace"].map(name => (
                    <span key={name} className="text-[11px] px-2 py-0.5 rounded-md font-medium bg-rose-500/10 text-rose-400">{name}</span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Platform Features Section */}
        <section id="platform" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                The <span className="gradient-text">Platform</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Everything you need to ship production software—orchestration, project management, and self-healing infrastructure.
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-6">
              {/* MCP Tools */}
              <div
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">MCP Tool Aggregation</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Integrated MCP servers for GitHub, Kubernetes, Linear, Grafana, and more—always expanding.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["GitHub", "K8s", "Linear", "Grafana"].map(tool => (
                    <span key={tool} className="text-xs px-2 py-1 rounded bg-cyan/10 text-cyan">{tool}</span>
                  ))}
                </div>
              </div>

              {/* Linear Integration */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Linear Integration</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Full Linear integration syncing GitHub issues, agent activities, and project boards in real time.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["PRD Intake", "Task Sync", "Live Updates"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-[oklch(0.7_0.25_320)]/10 text-[oklch(0.7_0.25_320)]">{feature}</span>
                  ))}
                </div>
              </div>

              {/* Self-Healing */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-green-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Self-Healing Infrastructure</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Automated incident detection, failure remediation, and workflow restarts — built into the platform. Bolt provisions and monitors bare metal. No on-call rotation needed.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Auto-Remediation", "Incident Detection", "Health Checks", "Auto-Rollback"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-green-500/10 text-green-500">{feature}</span>
                  ))}
                </div>
              </div>

              {/* Kubernetes Operators */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-blue-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Kubernetes Operators</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Replace managed cloud services with open-source operators. CloudNativePG, KubeAI, NVIDIA GPU, Kafka, and more — all self-hosted.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["CloudNativePG", "KubeAI", "NVIDIA GPU", "SeaweedFS", "Kotal"].map(db => (
                    <span key={db} className="text-xs px-2 py-1 rounded bg-blue-500/10 text-blue-500">{db}</span>
                  ))}
                </div>
              </div>

              {/* GitHub-Driven Deployment */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-orange-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-orange-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">GitHub-Driven Deployment</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Every change goes through Git. PRs, reviews, and deployments—all automated, all tracked.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["GitOps", "PRs", "Auto-deploy"].map(tool => (
                    <span key={tool} className="text-xs px-2 py-1 rounded bg-orange-500/10 text-orange-500">{tool}</span>
                  ))}
                </div>
              </div>

              {/* BYOK */}
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-purple-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Managed or Bring Your Own</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  We handle your bare metal relationships — or bring your own API keys and credentials. Stored in OpenBao. Zero lock-in.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Managed Infra", "OpenBao", "Zero Trust"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-purple-500/10 text-purple-500">{feature}</span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Blockchain & AI Section */}
        <section id="web3-ai" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Built for <span className="gradient-text">Blockchain & AI Teams</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Blockchain companies and AI labs share the same problem: they need powerful, sovereign infrastructure without cloud lock-in. CTO ships both.
              </p>
            </div>

            <div className="space-y-20">

              {/* ── Blockchain Node Deployment ── */}
              <div>
                <div className="mb-6 flex items-center gap-3">
                  <div className="w-8 h-8 rounded-lg bg-amber-500/10 flex items-center justify-center shrink-0">
                    <svg className="w-4 h-4 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                    </svg>
                  </div>
                  <h3 className="text-xl font-semibold">Blockchain Node Deployment</h3>
                  <div className="h-px flex-1 bg-border/40" />
                </div>
                <p className="text-muted-foreground mb-8 max-w-3xl">
                  Spin up L1 and L2 validator nodes, RPC endpoints, and archive nodes on dedicated bare metal. Our blockchain operator handles upgrades, peer discovery, and failover automatically — no DevOps team required.
                </p>

                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3 mb-8">
                  {[
                    { name: "Ethereum", types: "Validator · Archive · RPC", status: "live", trading: false },
                    { name: "Solana", types: "Validator · RPC", status: "live", trading: true },
                    { name: "Sui", types: "Full · RPC", status: "live", trading: true },
                    { name: "NEAR", types: "Validator · Archive · RPC", status: "live", trading: true },
                    { name: "BASE", types: "Full · RPC", status: "live", trading: true },
                    { name: "Aptos", types: "Full · Validator", status: "live", trading: false },
                    { name: "Bitcoin", types: "Full · RPC", status: "live", trading: false },
                    { name: "Arbitrum", types: "Full · RPC", status: "live", trading: false },
                    { name: "Optimism", types: "Full · RPC", status: "live", trading: false },
                    { name: "Chainlink", types: "Oracle Nodes", status: "live", trading: false },
                    { name: "The Graph", types: "Indexer Nodes", status: "live", trading: false },
                    { name: "LayerZero", types: "Relayer Nodes", status: "beta", trading: false },
                  ].map((chain) => (
                    <div key={chain.name} className="p-4 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm hover:border-amber-500/30 transition-colors">
                      <div className="flex items-center justify-between mb-1.5">
                        <span className="font-semibold text-sm text-foreground">{chain.name}</span>
                        <span className={`w-2 h-2 rounded-full shrink-0 ${chain.status === "live" ? "bg-green-500" : "bg-amber-500 animate-[pulse_2s_ease-in-out_infinite]"}`} />
                      </div>
                      <p className="text-[11px] text-muted-foreground">{chain.types}</p>
                      {chain.trading && (
                        <span className="mt-1.5 inline-block text-[10px] px-1.5 py-0.5 rounded font-medium bg-amber-500/10 text-amber-400">★ Trading</span>
                      )}
                    </div>
                  ))}
                </div>

                <div className="grid sm:grid-cols-3 gap-4">
                  <div className="p-5 rounded-xl border border-amber-500/20 bg-amber-500/5">
                    <p className="text-sm font-semibold text-amber-400 mb-1">Self-Managing Nodes</p>
                    <p className="text-xs text-muted-foreground">Automatic upgrades, peer discovery, and chain sync. Bolt monitors node health 24/7 and self-heals on failure.</p>
                  </div>
                  <div className="p-5 rounded-xl border border-amber-500/20 bg-amber-500/5">
                    <p className="text-sm font-semibold text-amber-400 mb-1">GitOps-Driven</p>
                    <p className="text-xs text-muted-foreground">Node configs live in Git. Version-controlled chain deployments with one-command rollback to any prior state.</p>
                  </div>
                  <div className="p-5 rounded-xl border border-amber-500/20 bg-amber-500/5">
                    <p className="text-sm font-semibold text-amber-400 mb-1">Sovereign Key Storage</p>
                    <p className="text-xs text-muted-foreground">Validator keys stored in OpenBao on your hardware. Never leave your infrastructure. Zero custody risk.</p>
                  </div>
                </div>
              </div>

              {/* ── Self-Hosted AI on GPU Bare Metal ── */}
              <div>
                <div className="mb-6 flex items-center gap-3">
                  <div className="w-8 h-8 rounded-lg bg-purple-500/10 flex items-center justify-center shrink-0">
                    <svg className="w-4 h-4 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                    </svg>
                  </div>
                  <h3 className="text-xl font-semibold">Self-Hosted AI on GPU Bare Metal</h3>
                  <div className="h-px flex-1 bg-border/40" />
                </div>
                <p className="text-muted-foreground mb-6 max-w-3xl">
                  Run frontier open-weight models on your own infrastructure. Start with shared inference pools during development, scale to dedicated GPU bare metal when volume justifies it — same OpenAI-compatible API throughout.
                </p>

                <div className="grid sm:grid-cols-3 gap-3 mb-8">
                  {[
                    {
                      tier: "Shared Inference",
                      desc: "Serverless GPU pools via Together AI, Fireworks, or Groq. Pay per token, zero commitment — ideal for development and low-volume workloads.",
                      note: "~$0.20–$0.90 / 1M tokens",
                      color: "border-border/50 bg-card/20",
                      labelColor: "text-muted-foreground",
                    },
                    {
                      tier: "GPU VMs",
                      desc: "Single-tenant GPU instances (Latitude vGPU, RunPod). Hourly billing, no noisy neighbours — right-sized for mid-scale production.",
                      note: "~$0.40–$1.70 / hr per GPU",
                      color: "border-purple-500/20 bg-purple-500/5",
                      labelColor: "text-purple-300",
                    },
                    {
                      tier: "Bare Metal GPU",
                      desc: "Full NVIDIA GPU nodes on dedicated servers. No per-token cost, no data leaving your cluster, maximum throughput. Economical at 100k+ tokens/min.",
                      note: "~$0.83–$1.66 / hr (H100 reserved)",
                      color: "border-purple-500/30 bg-purple-500/10",
                      labelColor: "text-purple-200",
                    },
                  ].map((t) => (
                    <div key={t.tier} className={`p-4 rounded-xl border ${t.color} backdrop-blur-sm`}>
                      <p className={`text-xs font-semibold uppercase tracking-wider mb-1 ${t.labelColor}`}>{t.tier}</p>
                      <p className="text-sm text-muted-foreground mb-2">{t.desc}</p>
                      <p className="text-xs font-mono text-muted-foreground/60">{t.note}</p>
                    </div>
                  ))}
                </div>

                <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
                  {[
                    {
                      name: "MiniMax-M2.5",
                      creator: "MiniMax",
                      context: "200K",
                      tags: ["Coding", "Agentic"],
                      note: "80.2% SWE-Bench Verified — best-in-class agentic coding",
                      accent: "pink",
                    },
                    {
                      name: "Qwen3.5",
                      creator: "Alibaba",
                      context: "256K",
                      tags: ["Multilingual", "Multimodal"],
                      note: "201 languages, native multimodal, outperforms GPT-5.2 on IFBench",
                      accent: "orange",
                    },
                    {
                      name: "GLM-4.7",
                      creator: "Z.ai",
                      context: "205K",
                      tags: ["Coding", "Agents"],
                      note: "Rivals Claude Sonnet 4.5 on SWE-bench at 32B active params",
                      accent: "blue",
                    },
                    {
                      name: "Kimi K2.5",
                      creator: "Moonshot AI",
                      context: "256K",
                      tags: ["Multimodal", "Agent Swarm"],
                      note: "1T-param MoE, 74.9% BrowseComp, 100-agent parallel swarm",
                      accent: "cyan",
                    },
                    {
                      name: "Llama 4 Maverick",
                      creator: "Meta",
                      context: "1M",
                      tags: ["Multimodal", "General"],
                      note: "1M token context, open-weight, commercially licensed",
                      accent: "blue",
                    },
                    {
                      name: "Mistral Large",
                      creator: "Mistral AI",
                      context: "256K",
                      tags: ["Coding", "Reasoning"],
                      note: "92% HumanEval — strong coding and reasoning performance",
                      accent: "violet",
                    },
                  ].map((model) => (
                    <div key={model.name} className="p-4 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm hover:border-purple-500/30 transition-colors">
                      <div className="flex items-start justify-between gap-2 mb-2">
                        <div>
                          <p className="font-semibold text-sm text-foreground">{model.name}</p>
                          <p className="text-[11px] text-muted-foreground">{model.creator}</p>
                        </div>
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-300 font-mono shrink-0">
                          {model.context}
                        </span>
                      </div>
                      <div className="flex flex-wrap gap-1 mb-2">
                        {model.tags.map(tag => (
                          <span key={tag} className="text-[10px] px-1.5 py-0.5 rounded-full bg-muted/50 text-muted-foreground">{tag}</span>
                        ))}
                      </div>
                      <p className="text-[11px] text-muted-foreground">{model.note}</p>
                    </div>
                  ))}
                </div>

                <div className="p-6 rounded-xl border border-purple-500/20 bg-purple-500/5 backdrop-blur-sm">
                  <div className="flex flex-col sm:flex-row sm:items-start gap-6">
                    <div className="flex-1">
                      <p className="font-semibold mb-1">Deployment Stack</p>
                      <p className="text-sm text-muted-foreground">
                        Models sourced from Hugging Face, served via KubeAI (Kubernetes-native) or Ollama for development. Use vLLM in production for up to 19× higher throughput. NVIDIA GPU Operator handles driver and plugin lifecycle automatically across your bare metal fleet.
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-2 sm:flex-col sm:items-end shrink-0">
                      {["KubeAI", "Ollama", "vLLM", "Hugging Face", "NVIDIA GPU Op."].map(t => (
                        <span key={t} className="text-xs px-3 py-1 rounded-full border border-purple-500/30 text-purple-300 bg-purple-500/10 whitespace-nowrap">{t}</span>
                      ))}
                    </div>
                  </div>
                </div>
              </div>

            </div>
          </div>
        </section>

        {/* Why CTO Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Why <span className="gradient-text">CTO</span>?
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Other tools help you code. CTO ships complete features—from PRD to production—with specialized agents for every stage of development.
              </p>
            </div>

            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-8">
              <div className="p-6 rounded-xl border border-cyan/30 bg-cyan/5 backdrop-blur-sm text-center h-full">
                <div className="w-14 h-14 rounded-full bg-cyan/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Choose Your CLI</h3>
                <p className="text-sm text-muted-foreground">
                  Claude Code, Cursor, Factory, Codex, Gemini—use what you love. We&apos;re agnostic.
                </p>
              </div>

              <div className="p-6 rounded-xl border border-[oklch(0.7_0.25_320)]/30 bg-[oklch(0.7_0.25_320)]/5 backdrop-blur-sm text-center h-full">
                <div className="w-14 h-14 rounded-full bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Multi-Agent</h3>
                <p className="text-sm text-muted-foreground">
                  A full team of specialists working in parallel. PM, backend, frontend, QA, security, DevOps — all coordinated.
                </p>
              </div>

              <div className="p-6 rounded-xl border border-green-500/30 bg-green-500/5 backdrop-blur-sm text-center h-full">
                <div className="w-14 h-14 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Fully Managed Bare Metal</h3>
                <p className="text-sm text-muted-foreground">
                  Self-healing infrastructure on dedicated servers. Zero cloud tax, zero ops burden.
                </p>
              </div>

              <div className="p-6 rounded-xl border border-yellow-500/30 bg-yellow-500/5 backdrop-blur-sm text-center h-full">
                <div className="w-14 h-14 rounded-full bg-yellow-500/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Bleeding Edge</h3>
                <p className="text-sm text-muted-foreground">
                  Always current. Latest models, newest CLIs, freshest integrations. We stay on the frontier.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <div className="grid md:grid-cols-2 gap-8">
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Bare Metal Advantage</h3>
                <p className="text-muted-foreground">
                  Skip the cloud markup. Direct bare metal pricing with cloud-like reliability.
                </p>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-magenta/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Complete Engineering Team</h3>
                <p className="text-muted-foreground">
                  Not an AI assistant. Specialists across PM, backend, frontend, quality, security, testing, and deployment.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="py-20 px-6">
          <div className="max-w-2xl mx-auto text-center">
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Start <span className="gradient-text">Shipping</span>
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Your AI engineering team is ready. Give it a PRD—get production code.
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
            <div className="flex items-center gap-2">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img src="/5dlabs-logo-header-v2.png" alt="5D Labs" width={64} height={64} className="h-16 w-16 opacity-90" />
            </div>
            <p className="text-sm text-muted-foreground">
              © {new Date().getFullYear()} 5D Labs. From PRD to Production — Autonomously.
            </p>
          </div>
        </footer>
      </main>
    </div>
  );
}
