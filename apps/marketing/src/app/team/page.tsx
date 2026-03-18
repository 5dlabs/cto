import Image from "next/image";
import Link from "next/link";
import { Header } from "@/components/header";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

type TeamAgent = {
  name: string;
  role: string;
  avatar?: string;
  color: string;
  personality: string;
  description: string;
  badge?: string;
  stack?: string[];
  tools?: string[];
  skills?: string[];
};

type TeamSquad = {
  title: string;
  emoji: string;
  agents: TeamAgent[];
};

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3001"
    : "https://5dlabs.ai";

const squads: TeamSquad[] = [
  {
    title: "Project Management",
    emoji: "🎯",
    agents: [
      {
        name: "Morgan",
        role: "Technical Program Manager",
        avatar: "/agents/morgan-avatar-512.png?v=20260318",
        color: "from-cyan-400 to-pink-500",
        personality: "Keeps the trains running. Decomposes PRDs, assigns tasks, tracks progress. Runs a tight ship.",
        description: "Orchestrates project lifecycles, syncs your Git repo (GitHub, GitLab, or Gitea) with Linear, and turns PRDs into coordinated execution plans with strong research context.",
        stack: ["Linear", "GitHub/GitLab/Gitea", "PRDs"],
        tools: ["Context7", "Firecrawl", "Perplexity", "Tavily", "Exa", "Repomix"],
        skills: ["PRD Analysis", "Deep Research", "Multi-Agent", "Writing Plans"],
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
        personality: "Lives for zero-cost abstractions. If it compiles, it ships. Opinions on lifetimes are... strong.",
        description: "Builds high-performance APIs and systems-level infrastructure when correctness, throughput, and tight control matter.",
        stack: ["Rust", "Tokio", "Axum"],
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Rust Patterns", "Error Handling", "Axum/Tokio", "Compound Engineering"],
      },
      {
        name: "Grizz",
        role: "Go Specialist",
        avatar: "/agents/grizz-avatar-512.png",
        color: "from-amber-500 to-orange-400",
        personality: "Pragmatic. Ships clean Go services without overengineering. Fan of simplicity and goroutines.",
        description: "Ships bulletproof backend services, REST/gRPC APIs, and Kubernetes operators with production-minded simplicity.",
        stack: ["Go", "gRPC", "PostgreSQL"],
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Go Patterns", "Concurrency", "gRPC/Chi", "Systematic Debugging"],
      },
      {
        name: "Nova",
        role: "Node.js Engineer",
        avatar: "/agents/nova-avatar-512.png",
        color: "from-purple-500 to-cyan-400",
        personality: "The speed demon. Gets APIs up and running faster than you can write the spec.",
        description: "Rapid API development and third-party integrations for teams optimizing for speed to market without losing structure.",
        stack: ["Node.js", "TypeScript", "Fastify"],
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Effect Patterns", "Elysia", "Drizzle", "Better Auth"],
      },
      {
        name: "Viper",
        role: "Python Specialist",
        avatar: "/agents/viper-avatar-512.png",
        color: "from-yellow-500 to-green-500",
        personality: "Automates the repetitive work and ships Python systems with zero drama.",
        description: "Builds data pipelines, ML workflows, automation scripts, and backend services with clean packaging and fast iteration.",
        stack: ["FastAPI", "Pydantic", "Async Python"],
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Data Pipelines", "ML Tooling", "Async Python", "FastAPI"],
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
        personality: "Pixel-perfect or bust. React components so clean they belong in a museum.",
        description: "Creates polished web applications with modern component systems and strong frontend craft.",
        stack: ["React", "Next.js", "shadcn/ui"],
        tools: ["Context7", "shadcn/ui", "AI Elements", "TanStack", "GitHub"],
        skills: ["Frontend Excellence", "React Best Practices", "Anime.js", "Frontend Design"],
      },
      {
        name: "Tap",
        role: "Mobile Developer",
        avatar: "/agents/tap-avatar-512.png",
        color: "from-green-500 to-emerald-400",
        personality: "One codebase, two platforms. Makes cross-platform feel native because it is.",
        description: "Builds native-quality iOS and Android apps from a single TypeScript codebase.",
        stack: ["Expo", "React Native", "NativeWind"],
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Expo Patterns", "React Native", "EAS Build", "Frontend Design"],
      },
      {
        name: "Spark",
        role: "Desktop Developer",
        avatar: "/agents/spark-avatar-512.png",
        color: "from-blue-500 to-yellow-400",
        personality: "Bringing the web to the desktop, with native superpowers. Offline-first evangelist.",
        description: "Builds cross-platform desktop apps with native integrations and offline-first architecture.",
        stack: ["Electron", "Tauri", "React"],
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
        personality: "Will refactor your code and you'll thank her. Best practices aren't optional.",
        description: "Refactors for maintainability and pushes code toward enterprise-grade quality standards.",
        stack: ["Code Review", "Patterns", "Standards"],
        tools: ["Context7", "Firecrawl", "Repomix", "GitHub"],
        skills: ["Evaluation", "Code Maturity", "Advanced Evaluation", "Code Review"],
      },
      {
        name: "Cipher",
        role: "Security Sentinel",
        avatar: "/agents/cipher-avatar-512.png",
        color: "from-red-500 to-rose-500",
        personality: "Paranoid by design. If there's a vulnerability, Cipher already found it.",
        description: "Runs security audits, dependency scans, and pen tests, combining code-level review with attack-minded validation.",
        stack: ["Trivy", "Gitleaks", "OWASP"],
        tools: ["Context7", "Firecrawl", "OpenCode", "GitHub", "Snyk", "Nuclei"],
        skills: ["Semgrep", "CodeQL", "Pen Testing", "Supply Chain"],
      },
      {
        name: "Tess",
        role: "Testing Genius",
        avatar: "/agents/tess-avatar-512.png",
        color: "from-violet-500 to-purple-500",
        personality: "100% coverage or she's not done. Writes tests you didn't know you needed.",
        description: "Builds comprehensive test suites across unit, integration, and end-to-end layers.",
        stack: ["Jest", "Playwright", "Vitest"],
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
        personality: "Reviews every PR with surgical precision. Catches what others miss. Politely.",
        description: "Reviews pull requests with a focus on correctness, risk, and meaningful improvement opportunities.",
        stack: ["PR Review", "Linting", "Standards"],
        tools: ["Context7", "Octocode", "GitHub"],
        skills: ["Code Review", "Differential Review", "PR Review"],
      },
      {
        name: "Atlas",
        role: "Integration Master",
        avatar: "/agents/atlas-avatar-512.png",
        color: "from-slate-500 to-zinc-500",
        personality: "The gatekeeper. If it merges, Atlas approved it. If it conflicts, Atlas fixed it.",
        description: "Owns merge flow, rebases stale branches, and keeps the path to clean integration moving.",
        stack: ["Git", "Rebasing", "CI/CD"],
        tools: ["Context7", "Repomix", "GitHub"],
        skills: ["Git Integration", "Git Worktrees", "Finishing Branch"],
      },
      {
        name: "Bolt",
        role: "Infrastructure & SRE",
        avatar: "/agents/bolt-avatar-512.png",
        color: "from-yellow-500 to-amber-500",
        personality: "Your always-on SRE team. Provisions bare metal, deploys services, detects incidents, and self-heals — so you never get paged at 3 AM.",
        description: "Builds and operates the self-healing infrastructure layer beneath the entire platform.",
        stack: ["Kubernetes", "Bare Metal", "GitOps", "Monitoring"],
        tools: ["Context7", "Kubernetes", "GitHub"],
        skills: ["Kubernetes Operators", "ArgoCD/GitOps", "Observability", "MCP Builder"],
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
        personality: "Keeps validators online, nodes synced, and chain infrastructure humming.",
        description: "Deploys and operates blockchain nodes across supported chains, from validators and RPC endpoints to archive infrastructure.",
        stack: ["Ethereum", "NEAR", "Solana"],
        tools: ["Context7", "Firecrawl", "Tavily", "GitHub"],
        skills: ["Node Ops", "Smart Contracts", "Validator Ops", "RPC Infrastructure"],
      },
      {
        name: "Vex",
        role: "VR/Unity Developer",
        avatar: "/agents/vex-avatar-512.png",
        color: "from-violet-500 to-indigo-500",
        personality: "Builds immersive experiences that feel native across headsets and spatial surfaces.",
        description: "Builds cross-platform VR and XR experiences with Unity and OpenXR, from Quest to PC to spatial web.",
        stack: ["Unity", "OpenXR", "Meta XR"],
        tools: ["Context7", "Octocode", "Firecrawl", "GitHub"],
        skills: ["Three.js", "Cross-Platform XR", "Unity", "OpenXR"],
      },
      {
        name: "Angie",
        role: "Agent Builder",
        avatar: "/agents/angie-avatar-512.png?v=20260314",
        color: "from-indigo-500 to-cyan-400",
        personality: "Thinks in agent systems first: orchestration, tool routing, and runtime behavior before UI polish.",
        description: "Designs OpenClaw-first agent architecture, including MCP tool integration and multi-agent execution patterns.",
        stack: ["OpenClaw", "MCP", "LiveKit", "ElevenLabs"],
        tools: ["OpenClaw", "Context7", "Octocode", "GitHub"],
        skills: ["Agent Architecture", "Orchestration", "Tooling Design", "OpenClaw", "LangGraph", "CrewAI", "AutoGen"],
      },
      {
        name: "Glitch",
        role: "Game Developer",
        avatar: "/agents/glitch-avatar-512.png",
        color: "from-fuchsia-500 to-pink-500",
        personality: "Prototypes fast, tunes the feel, and ships interactive experiences people remember.",
        description: "Builds games and interactive experiences across Unity, Godot, Unreal, and WebGL.",
        stack: ["Unity", "Godot", "Unreal"],
        tools: ["Context7", "GitHub", "Firecrawl", "Tavily"],
        skills: ["WebGL", "Game Physics", "Shader Programming", "Gameplay Systems"],
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
        personality: "Reads the fine print, flags the risk, and keeps the deal clean.",
        description: "Contract review, compliance checks, and legal risk assessment. Trained on your jurisdiction, your agreements, and your standards.",
        badge: "In Development",
        stack: ["Contracts", "Compliance", "Risk Review"],
      },
      {
        name: "Hype",
        role: "Marketing Strategist",
        avatar: "/agents/hype-avatar-512.png",
        color: "from-orange-500 to-rose-500",
        personality: "Sharpens the story, tests the angle, and turns attention into pipeline.",
        description: "Campaign strategy, copy, and analytics. From brand voice to conversion, built to move as fast as the product.",
        badge: "In Development",
        stack: ["Campaigns", "Copy", "Analytics"],
      },
      {
        name: "Tally",
        role: "Accounting Specialist",
        avatar: "/agents/tally-avatar-512.png",
        color: "from-emerald-600 to-teal-600",
        personality: "Keeps the books clean, the numbers current, and the close under control.",
        description: "Bookkeeping, reconciliation, and financial reporting with always-current operating visibility.",
        badge: "In Development",
        stack: ["Bookkeeping", "Reconciliation", "Reporting"],
      },
      {
        name: "Chase",
        role: "Sales Agent",
        avatar: "/agents/chase-avatar-512.png",
        color: "from-amber-500 to-yellow-500",
        personality: "Relentless follow-up, crisp discovery, and no dropped deals.",
        description: "Outreach, pipeline management, and closing so the team can stay focused on building.",
        badge: "In Development",
        stack: ["Outreach", "Pipeline", "Follow-up"],
      },
    ],
  },
];

function TeamCard({ agent }: { agent: TeamAgent }) {
  return (
    <div className="group relative min-h-[280px] rounded-xl border border-border bg-card/60 p-4 sm:p-5 backdrop-blur-sm transition-colors hover:border-primary/50">
      <div
        className={cn(
          "absolute inset-0 rounded-xl bg-gradient-to-br opacity-0 transition-opacity duration-300 group-hover:opacity-15",
          agent.color
        )}
        aria-hidden
      />

      <div className="relative z-10 flex items-start gap-4">
        <Avatar className="size-[72px] sm:size-[88px] lg:size-[112px] shrink-0 ring-2 ring-border">
          {agent.avatar ? <AvatarImage src={agent.avatar} alt={agent.name} /> : null}
          <AvatarFallback
            className={cn("bg-gradient-to-br text-white text-2xl sm:text-3xl font-bold", agent.color)}
          >
            {agent.name.charAt(0)}
          </AvatarFallback>
        </Avatar>

        <div className="min-w-0 flex-1">
          <p className="font-semibold text-base sm:text-lg text-foreground">{agent.name}</p>
          <p className="text-xs sm:text-sm text-cyan">{agent.role}</p>
          {agent.badge ? (
            <p className="mt-1 text-[10px] uppercase tracking-widest text-[oklch(0.7_0.25_320)]/80 font-medium">
              {agent.badge}
            </p>
          ) : null}
          <p className="mt-3 text-sm text-foreground/90">{agent.personality}</p>
          <p className="mt-2 text-sm text-muted-foreground">{agent.description}</p>
        </div>
      </div>

      <div className="relative z-10 mt-4 space-y-3">
        {agent.stack && agent.stack.length > 0 ? (
          <div className="flex flex-wrap gap-1.5">
            {agent.stack.map((item) => (
              <span
                key={item}
                className="inline-flex items-center rounded-md bg-muted/60 px-2 py-1 text-[10px] font-medium uppercase tracking-wide text-muted-foreground"
              >
                {item}
              </span>
            ))}
          </div>
        ) : null}

        {agent.tools && agent.tools.length > 0 ? (
          <div>
            <p className="mb-1 text-[10px] font-medium uppercase tracking-widest text-muted-foreground">
              Tools
            </p>
            <div className="flex flex-wrap gap-1.5">
              {agent.tools.slice(0, 6).map((tool) => (
                <span
                  key={tool}
                  className="inline-flex items-center rounded-md border border-border/60 px-2 py-1 text-[10px] font-mono text-foreground/90"
                >
                  {tool}
                </span>
              ))}
            </div>
          </div>
        ) : null}

        {agent.skills && agent.skills.length > 0 ? (
          <div>
            <p className="mb-1 text-[10px] font-medium uppercase tracking-widest text-muted-foreground">
              Skills
            </p>
            <div className="flex flex-wrap gap-1.5">
              {agent.skills.slice(0, 6).map((skill) => (
                <span
                  key={skill}
                  className="inline-flex items-center rounded-md border border-border/60 px-2 py-1 text-[10px] font-mono text-foreground/90"
                >
                  {skill}
                </span>
              ))}
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}

export default function TeamPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <div className="fixed inset-0 noise-overlay z-0" />

      <Header />

      <main className="relative z-10 pt-24">
        <section className="py-20 px-6">
          <div className="max-w-6xl mx-auto text-center">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8">
              <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
              <span className="text-sm text-cyan font-medium">AI Agents. One coordinated team.</span>
            </div>

            <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight mb-6">
              Meet the <span className="gradient-text glow-text-cyan">Team</span>
            </h1>

            <p className="text-xl text-muted-foreground max-w-3xl mx-auto">
              Not one generic AI assistant. A full team of specialists across planning,
              backend, frontend, security, operations, and business functions, each with
              a distinct working style, toolchain, and area of expertise.
            </p>
          </div>
        </section>

        <section className="pb-20 px-6">
          <div className="max-w-6xl mx-auto space-y-10">
            {squads.map((squad) => (
              <div key={squad.title}>
                <div className="mb-4 flex items-center gap-3">
                  <span className="text-xl">{squad.emoji}</span>
                  <h2 className="text-sm font-medium uppercase tracking-wider text-muted-foreground">
                    {squad.title}
                  </h2>
                  <div className="h-px flex-1 bg-border/50" />
                </div>

                <div
                  className={cn(
                    "grid gap-4",
                    squad.agents.length === 1
                      ? "grid-cols-1 max-w-2xl mx-auto"
                      : squad.agents.length === 4
                        ? "grid-cols-1 sm:grid-cols-2"
                        : "grid-cols-1 md:grid-cols-2 xl:grid-cols-3"
                  )}
                >
                  {squad.agents.map((agent) => (
                    <TeamCard key={agent.name} agent={agent} />
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>

        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-2xl mx-auto text-center">
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Ready to <span className="gradient-text">ship</span> with them?
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Give the platform a PRD and watch a coordinated team turn it into production code.
            </p>
            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <a
                href="https://app.5dlabs.ai"
                className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
              >
                Start Now
              </a>
              <Link
                href="/"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all"
              >
                Back to CTO
              </Link>
            </div>
          </div>
        </section>

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
              © {new Date().getFullYear()} 5D Labs. From PRD to Production — Autonomously.
            </p>
          </div>
        </footer>
      </main>
    </div>
  );
}
