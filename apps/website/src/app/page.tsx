import Link from "next/link";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { VentureGrid, type Venture } from "@/components/venture-card";

const operatingStack: Venture[] = [
  {
    name: "CTO",
    tagline: "The build engine.",
    description:
      "Our AI engineering platform. It is the system we use internally to ship software and the first commercial wedge we can offer to external teams.",
    tags: ["Commercial Service", "Build Engine", "Multi-Agent", "AI OS"],
    color: "from-cyan-500 to-blue-500",
    href: "/cto",
    status: "building",
  },
  {
    name: "Trading Engine",
    tagline: "The capital engine.",
    description:
      "Our in-house system that generates revenue, sharpens market intelligence, and finances venture creation across Solana, Base, Polygon, Near, and Sui.",
    tags: ["Revenue Engine", "Market Intelligence", "Multi-Chain", "In-House"],
    color: "from-purple-500 to-magenta",
    href: "/trading",
    status: "building",
  },
  {
    name: "Venture Pipeline",
    tagline: "The products we bring to market.",
    description:
      "These systems exist to help 5D Labs discover, validate, and launch customer-facing ventures faster than a traditional startup can.",
    tags: ["Studio", "Validation", "Launch", "Product-Market Fit"],
    color: "from-yellow-500 to-orange-500",
    status: "exploring",
  },
];

const operatingModel = [
  {
    title: "5D Labs",
    text: "The studio. We choose where to focus, allocate resources, and decide what earns a deeper build cycle.",
  },
  {
    title: "CTO",
    text: "The build engine. It is both our internal production system and the first commercial service we can take to market.",
  },
  {
    title: "Trading Engine",
    text: "The capital engine. Our in-house system that generates revenue and live market intelligence to finance and inform the work.",
  },
  {
    title: "Ventures",
    text: "The products we bring to market. The studio exists to discover, validate, and launch customer-facing ventures at a pace a traditional startup cannot match.",
  },
];

const capitalEnginePoints = [
  {
    title: "Revenue for Experimentation",
    text: "Generating our own revenue gives the studio more room to test, learn, and build without relying on a single outside funding event.",
  },
  {
    title: "Live Market Intelligence",
    text: "Operating in real markets keeps the team close to execution quality, liquidity conditions, and where on-chain demand is actually moving.",
  },
  {
    title: "Focused on Venture Creation",
    text: "The right role for this system is as leverage inside the studio — helping finance the work and sharpen decisions as new ventures are built.",
  },
];

const northStarMetrics = [
  { value: "1", label: "operating stack" },
  { value: "5+", label: "venture themes in flight" },
  { value: "17+", label: "infra providers" },
  { value: "4", label: "chains in production" },
];

const proofSignals = [
  "Multi-agent delivery loop running in production",
  "Infrastructure, model routing, and release flow in one stack",
  "Built for rapid venture validation without context-switch tax",
];

const executionLoop = [
  {
    title: "Decide",
    text: "Pick one thesis and constrain scope to a buildable wedge with explicit success signals.",
  },
  {
    title: "Deploy",
    text: "Ship through CTO and the OpenClaw platform with specialist agents coordinated across the same delivery loop.",
  },
  {
    title: "Learn",
    text: "Fold runtime and market signal back into roadmap decisions instead of shipping blind.",
  },
  {
    title: "Compound",
    text: "Reuse infrastructure, prompts, and workflows so each venture starts with leverage, not from zero.",
  },
];

export default function Home() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10">
        {/* Hero Section */}
        <section className="min-h-[100dvh] flex flex-col items-center justify-center px-6 py-20 pt-24">
          <div className="max-w-4xl mx-auto text-center">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full premium-chip mb-8">
                <span className="size-2 rounded-full bg-cyan animate-[glowPulse_3s_ease-in-out_infinite]" />
                <span className="text-sm text-cyan font-semibold tracking-wide">
                  AI-Native Venture Studio
                </span>
            </div>

            <h1 className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6">
                <span className="gradient-text-stitch glow-text-cyan">
                  We Built the Stack
                </span>
                <br />
                <span className="text-foreground">
                  That Helps Us Build Companies.
                </span>
            </h1>

            <p className="text-xl sm:text-2xl text-muted-foreground max-w-3xl mx-auto mb-10">
                5D Labs is an AI-native venture studio. <span className="text-foreground">CTO</span> is the build
                engine — an operating system for software delivery, not just a coding tool. Our{" "}
                <span className="text-foreground">trading engine</span> helps finance and
                inform the work. The ventures are what we bring to market.
            </p>

            <div className="flex flex-col sm:flex-row justify-center gap-4 mb-8">
                <a
                  href="/cto"
                  className="relative overflow-hidden px-8 py-4 rounded-lg bg-gradient-to-r from-violet-500 via-indigo-500 to-cyan-500 text-white font-semibold text-lg transition-all shadow-xl shadow-indigo-500/35 hover:shadow-indigo-500/55 hover:scale-105 hover:brightness-110 active:translate-y-px"
                >
                  <span className="relative z-10">Explore CTO Platform</span>
                  <span className="absolute inset-0 w-[30%] h-full bg-gradient-to-r from-transparent via-white/25 to-transparent animate-[ctaShimmer_3s_ease-in-out_infinite]" />
                </a>
                <Link
                  href="/opportunities"
                  className="px-8 py-4 rounded-lg bg-gradient-to-r from-purple-500 to-pink-500 text-white font-semibold text-lg hover:from-purple-600 hover:to-pink-600 transition-all shadow-xl shadow-purple-500/30 hover:shadow-purple-500/50 hover:scale-105 active:translate-y-px"
                >
                  Explore Opportunities
                </Link>
                <Link
                  href="/#operating-model"
                  className="px-8 py-4 rounded-lg premium-chip text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all active:translate-y-px"
                >
                  See How It Works
                </Link>
            </div>

            <div className="premium-shell rounded-2xl p-5 md:p-6 mb-6">
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3 max-w-4xl mx-auto">
                  {northStarMetrics.map((metric) => (
                    <div
                      key={metric.label}
                      className="rounded-xl premium-chip p-4 text-left"
                    >
                      <p className="text-2xl font-bold text-foreground font-mono tabular-nums">
                        {metric.value}
                      </p>
                      <p className="text-xs uppercase tracking-wider text-muted-foreground mt-1">
                        {metric.label}
                      </p>
                    </div>
                  ))}
                </div>
            </div>

            <div className="flex flex-wrap justify-center gap-2 max-w-3xl mx-auto">
                {proofSignals.map((signal, i) => (
                  <span
                    key={signal}
                    className="premium-chip rounded-full px-3 py-1.5 text-xs text-muted-foreground"
                    style={{
                      animation: `chipFloat 3s ease-in-out ${i * 0.4}s infinite`,
                    }}
                  >
                    {signal}
                  </span>
                ))}
            </div>
          </div>

          <div className="absolute bottom-10 left-1/2 -translate-x-1/2">
            <div className="w-6 h-10 rounded-full border-2 border-muted-foreground/30 flex justify-center pt-2 scroll-bounce">
              <div className="w-1 h-2 rounded-full bg-cyan" />
            </div>
          </div>
        </section>

        {/* Disruptor Manifesto */}
        <section className="py-16 px-6">
          <div className="max-w-3xl mx-auto text-center">
            <p className="text-xl sm:text-2xl text-muted-foreground leading-relaxed">
              One idea. One team. One shot at product-market fit.
              <br className="hidden sm:block" />
              We thought there{" "}
              <span className="text-foreground font-semibold">must be a better way</span>.{" "}
              <span className="text-foreground font-bold">We built it.</span>
            </p>
          </div>
        </section>

        <section className="section-frame py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                The <span className="gradient-text">Execution Loop</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
                We keep strategy, shipping, and market signal in one tight loop so venture decisions are made with live evidence.
              </p>
            </div>
            <div className="grid md:grid-cols-2 xl:grid-cols-4 gap-6">
              {executionLoop.map((item, index) => (
                <div key={item.title} className="p-6 rounded-xl premium-shell backdrop-blur-sm relative">
                  <p className="text-xs uppercase tracking-wider text-cyan mb-3">
                    Step {index + 1}
                  </p>
                  <h3 className="text-lg font-semibold mb-2">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.text}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Operating Model Section */}
        <section id="operating-model" className="section-frame py-20 px-6">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                How <span className="gradient-text">5D Labs</span> Works
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                One studio, one operating stack, multiple venture outcomes.
              </p>
            </div>

            <div className="grid md:grid-cols-2 xl:grid-cols-4 gap-6">
              {operatingModel.map((item) => (
                <div key={item.title} className="p-6 rounded-xl premium-shell backdrop-blur-sm">
                  <h3 className="text-lg font-semibold mb-3">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.text}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Operating Stack Section */}
        <section id="stack" className="section-frame py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                The <span className="gradient-text">Stack</span> Behind the Studio
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                CTO, the trading engine, and the venture pipeline are not separate company theses.
                They are coordinated parts of one operating model.
              </p>
            </div>

            <VentureGrid ventures={operatingStack} />
          </div>
        </section>

        {/* Capital Engine Section */}
        <section className="section-frame py-20 px-6">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                The <span className="gradient-text">Trading Engine</span> in the Model
              </h2>
              <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
                It funds experimentation, keeps the studio close to live on-chain markets,
                and compounds market intelligence that only comes from operating with real stakes.
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-8">
              {capitalEnginePoints.map((item) => (
                <div key={item.title} className="p-6 rounded-xl premium-shell backdrop-blur-sm">
                  <h3 className="text-lg font-semibold mb-2">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.text}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* OpenClaw Section */}
        <section id="openclaw" className="section-frame py-20 px-6">
          <div className="max-w-4xl mx-auto text-center">
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full premium-chip mb-6">
              <span className="size-2 rounded-full bg-cyan" />
              <span className="text-xs text-cyan font-medium uppercase tracking-wider">
                The Engine Underneath
              </span>
            </div>
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Powered by the <span className="gradient-text">OpenClaw Platform</span>
            </h2>
            <p className="text-lg text-muted-foreground max-w-2xl mx-auto mb-10">
              The OpenClaw platform is the orchestration layer beneath CTO and the rest of the operating stack.
              It coordinates AI agents across multiple CLIs, bare-metal Kubernetes infrastructure,
              and a growing ecosystem of MCP tools — turning a collection of specialized agents
              into a coherent, self-healing system.
            </p>
            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6 text-left">
              {[
                {
                  title: "Multi-CLI Agnostic",
                  text: "Works with Claude Code, Cursor, Codex, Factory, Gemini, and OpenCode. The orchestration layer doesn't care which model or interface runs the task.",
                },
                {
                  title: "Bare-Metal Native",
                  text: "Built from the ground up for Kubernetes on dedicated hardware. No cloud dependencies, no managed service lock-in.",
                },
                {
                  title: "Coming Soon",
                  text: "The OpenClaw platform will be open-sourced. When it ships publicly, it will be the reusable foundation any team can use to run the same kind of agent infrastructure we run internally.",
                },
                {
                  title: "Specialist Builders",
                  text: "Specialists like Angie focus on agent architecture for the OpenClaw platform: MCP tool routing, runtime integration, and practical voice-agent patterns on top of the core orchestration layer.",
                },
              ].map((item) => (
                <div key={item.title} className="p-5 rounded-xl premium-shell backdrop-blur-sm">
                  <h3 className="text-base font-semibold mb-2 text-foreground">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.text}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <Footer />
      </main>
    </div>
  );
}
