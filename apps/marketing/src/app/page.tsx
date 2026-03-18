import Image from "next/image";
import Link from "next/link";
import { WaitlistForm } from "@/components/waitlist-form";
import { MorganHeroImage } from "@/components/morgan-hero-image";
import { AgentGrid, type AgentSquad } from "@/components/agent-card";
import { Header } from "@/components/header";
import { TechStack } from "@/components/tech-stack";
import { HeroExperiment } from "@/components/hero-experiment";
import { LemonSliceWidget } from "@/components/lemon-slice-widget";
import { featureFlags } from "@/config/feature-flags";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3001"
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
        cta: { label: "Talk to Morgan", href: "/morgan" },
        description: "Orchestrates project lifecycles—syncing your Git repo (GitHub, GitLab, or Gitea) with Linear, decomposing PRDs into tasks. Research tools for docs, web, and codebase context.",
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
      <div className="fixed inset-0 circuit-bg z-0" />
      <div className="fixed inset-0 noise-overlay z-0" />

      {/* Header */}
      <Header />

      {/* Content */}
      <main className="relative z-10">
        <HeroExperiment />

        {/* Meet Morgan Section */}
        <section id="morgan" className="py-20 px-6 border-t border-border/30 scroll-mt-24">
          <div className="max-w-3xl mx-auto text-center">
            <MorganHeroImage />
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Meet <span className="gradient-text">Morgan</span>
            </h2>
            <p className="text-lg text-muted-foreground mb-6 max-w-xl mx-auto">
              Your single point of contact. Chat, voice, or video—from any device. You only talk to Morgan; Morgan coordinates the rest.
            </p>
            <Link
              href="/morgan"
              className="inline-flex items-center gap-2 px-6 py-3 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold hover:from-cyan-600 hover:to-blue-600 transition-all"
            >
              Talk to Morgan
            </Link>
            <div className="mt-12 rounded-xl border border-border bg-card/50 p-4 min-h-[320px] flex items-center justify-center">
              <LemonSliceWidget
                initialState="minimized"
                className="w-full min-h-[280px]"
              />
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
                Works With the Way Your <span className="gradient-text">Team Already Ships</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                You don&apos;t have to think about operators, runtimes, or platform plumbing. Point CTO at a repo and it handles the rest — tooling, infrastructure, and delivery are already included.
              </p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Your Stack, Supported</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>Backend, frontend, mobile, and desktop workflows</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>Works with modern repos, frameworks, and APIs</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>No rebuild of your stack required</span>
                  </div>
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-magenta/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Infrastructure, Abstracted</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>Managed databases, storage, source control, and deployments</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>Self-healing infrastructure underneath the product</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-magenta/60"></span>
                    <span>Cloud today, bare metal when you are ready</span>
                  </div>
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-yellow/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Use the CLI You Like</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Claude Code, Cursor, Factory, Codex</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Gemini, OpenCode, GitHub Copilot, Kimi</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Keys and runtime managed for you</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Intelligent routing across Anthropic, OpenAI, Google, Bedrock, OpenRouter, xAI, DeepSeek, and 50+ more</span>
                  </div>
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-green/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Your Workflow, Connected</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>GitHub, GitLab, Gitea, Linear, Slack, Teams, and more</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>Planning, progress, alerts, and releases in one loop</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-green/60"></span>
                    <span>No context-switch tax across the team</span>
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

        {/* Infrastructure Section */}
        <section id="infrastructure" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Infrastructure, <span className="gradient-text">Abstracted</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                CTO is designed so teams do not have to think about providers,
                data-center inventory, or migration mechanics. Choose the
                outcome you need and the platform handles the operational path
                underneath it.
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-6">
              <div className="p-6 rounded-xl border border-cyan/20 bg-cyan/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest text-cyan mb-3">
                  Launch Where You Need It
                </p>
                <h3 className="text-lg font-semibold mb-2 text-foreground">
                  Cloud, dedicated hardware, or your own footprint
                </h3>
                <p className="text-sm text-muted-foreground">
                  Start in the environment that matches your current stage and
                  compliance needs. CTO keeps the product surface consistent as
                  the underlying infrastructure changes.
                </p>
              </div>
              <div className="p-6 rounded-xl border border-[oklch(0.7_0.25_320)]/20 bg-[oklch(0.7_0.25_320)]/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest text-[oklch(0.7_0.25_320)] mb-3">
                  Managed Migration Path
                </p>
                <h3 className="text-lg font-semibold mb-2 text-foreground">
                  Move without re-platforming the team
                </h3>
                <p className="text-sm text-muted-foreground">
                  You do not need to pause delivery to move off the cloud or
                  into a more efficient setup. We handle the migration path,
                  release flow, and operational cutover behind the scenes.
                </p>
              </div>
              <div className="p-6 rounded-xl border border-green-500/20 bg-green-500/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest text-green-400 mb-3">
                  Quiet Redundancy
                </p>
                <h3 className="text-lg font-semibold mb-2 text-foreground">
                  Reliability without an infrastructure control room
                </h3>
                <p className="text-sm text-muted-foreground">
                  Region placement, failover strategy, and operational
                  resilience are handled as part of the platform so your team
                  can stay focused on shipping product.
                </p>
              </div>
            </div>

            <div className="mt-8 p-5 rounded-xl border border-border/30 bg-muted/20 text-center">
              <p className="text-sm text-muted-foreground max-w-3xl mx-auto">
                One stable platform that runs wherever it needs to run. The
                operational complexity is abstracted away so your team stays
                focused on what they shipped here to build.
              </p>
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
                <p className="text-xs text-muted-foreground mb-4">Each agent integrates with your Git host. PRs, reviews, and deployments are fully automated—whether you use GitHub, self-hosted GitLab, or Gitea.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["Git Apps", "CI/CD", "ArgoCD", "Webhooks", "PR Automation"].map(name => (
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
                Everything needed to move from idea to production — assembled, tested, and ready to go.
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-6">
              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Plan</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  PRDs become structured plans through deliberation — optimist and pessimist agents challenge each decision point before committing, the same way a real team would debate scope, risk, and tradeoffs. The result is a plan that has already survived scrutiny.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["PRD intake", "Deliberation", "Decision gates", "Task decomposition"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-[oklch(0.7_0.25_320)]/10 text-[oklch(0.7_0.25_320)]">{feature}</span>
                  ))}
                </div>
              </div>

              <div
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Code</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  OpenClaw&apos;s ACP harness manages all supported CLIs and makes intelligent routing decisions based on available token usage across providers. Multiple CLIs — Claude Code, Cursor, Codex, OpenCode, Gemini, Copilot, Kimi, Pi — and 60+ model providers (Anthropic, OpenAI, Google, AWS Bedrock, Azure Foundry, OpenRouter, xAI, DeepSeek, MiniMax, DeepInfra, and more) in one consistent experience.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["ACP harness", "Multi-CLI", "Token-aware routing", "60+ providers"].map(tool => (
                    <span key={tool} className="text-xs px-2 py-1 rounded bg-cyan/10 text-cyan">{tool}</span>
                  ))}
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-green-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Pulse</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  The platform monitors its own vitals and fixes what breaks — before it becomes an incident. Automated detection, remediation, and restart logic keep everything running without turning your team into a 24/7 ops desk.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Self-healing", "Auto-remediation", "Health checks", "Auto-rollback"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-green-500/10 text-green-500">{feature}</span>
                  ))}
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-rose-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-rose-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Sentinel</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Continuous vulnerability scanning, dependency analysis, and AI-native remediation running across every service. Cipher doesn&apos;t just flag issues — it ships the fix through the same agent pipeline as everything else.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Vuln scanning", "Dependency audit", "AI remediation", "Supply chain"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-rose-500/10 text-rose-400">{feature}</span>
                  ))}
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-blue-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Infrastructure</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Databases, storage, and inference — the same managed services teams expect from cloud providers, already running and ready to use. No setup, no assembly, no surprise bills.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["5D Data", "5D Store", "5D Volume", "5D Inference"].map(db => (
                    <span key={db} className="text-xs px-2 py-1 rounded bg-blue-500/10 text-blue-500">{db}</span>
                  ))}
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-orange-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-orange-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Deploy</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Every change moves through a tracked release flow, from review to deploy, with clear auditability and fast rollback when needed.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Tracked releases", "Rollbacks", "Automation"].map(tool => (
                    <span key={tool} className="text-xs px-2 py-1 rounded bg-orange-500/10 text-orange-500">{tool}</span>
                  ))}
                </div>
              </div>

              <div className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                <div className="w-12 h-12 rounded-lg bg-purple-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">5D Vault</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  API keys, credentials, and provider access are fully managed behind a secure control layer. Everything is included — nothing to configure, nothing to wire up.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Fully managed", "Secure by default", "Zero config"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-purple-500/10 text-purple-500">{feature}</span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Startups Section */}
        <section id="web3-ai" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Built for <span className="gradient-text">Startups</span>
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

        {/* Why CTO Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Why <span className="gradient-text">CTO</span>?
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Other tools help you write code. CTO ships complete products — planned, built, tested, and deployed by a coordinated team of specialists.
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
                  Claude Code, Cursor, Factory, Codex, Gemini, GitHub Copilot, Kimi—use what you love. We&apos;re agnostic.
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
              Describe what you want to build. Watch it ship.
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
                width={160}
                height={40}
                className="h-10 w-auto opacity-90"
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
