import Link from "next/link";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";

const pillars = [
  {
    title: "Revenue Engine",
    description:
      "Generating our own capital gives the studio more room to experiment, build, and move without waiting on a single outside funding event.",
    color: "text-cyan",
  },
  {
    title: "Precision Execution",
    description:
      "Systematic, data-driven strategies across multiple chains. Built for consistent performance, not speculation. Guardrails, monitoring, and risk controls are part of the architecture from day one.",
    color: "text-[oklch(0.7_0.25_320)]",
  },
  {
    title: "Market Intelligence",
    description:
      "Running real systems with real stakes keeps the team sharp. We stay close to where liquidity is moving, which chains are gaining institutional adoption, and what execution conditions actually look like.",
    color: "text-yellow-500",
  },
];

const solanaThesis = [
  {
    title: "Tokenized Equities Are Already Live",
    description:
      "Ondo Finance and xStocks are running 200+ tokenized U.S. stocks and ETFs on Solana with direct connectivity to NASDAQ and NYSE. Over $3B in transaction volume as of early 2026.",
  },
  {
    title: "NYSE Is Building On-Chain Settlement",
    description:
      "The New York Stock Exchange is developing a tokenized securities platform with 24/7 trading and on-chain settlement — with Solana listed as a supported settlement layer alongside Ethereum.",
  },
  {
    title: "T+0 Settlement Changes the Game",
    description:
      "Traditional equity markets settle in T+2. On-chain settlement is instant. That compression creates new execution opportunities that don't exist in traditional market microstructure.",
  },
  {
    title: "Infrastructure Built for Speed",
    description:
      "Solana's throughput and sub-second finality make it the natural home for high-frequency settlement workflows as institutional capital moves on-chain.",
  },
];

const chains = [
  {
    name: "Solana",
    role: "Primary execution layer",
    detail: "Low-latency settlement, tokenized equities, high-throughput DeFi. The emerging institutional settlement layer.",
    color: "text-[oklch(0.7_0.25_320)]",
    border: "border-[oklch(0.7_0.25_320)]/30",
    bg: "bg-[oklch(0.7_0.25_320)]/5",
  },
  {
    name: "Base",
    role: "Ethereum-aligned distribution",
    detail: "Ethereum composability and ecosystem depth without the gas overhead. Strong for DeFi primitives and broader distribution.",
    color: "text-blue-400",
    border: "border-blue-400/30",
    bg: "bg-blue-400/5",
  },
  {
    name: "Polygon",
    role: "Ethereum scaling & institutional adoption",
    detail: "Enterprise-grade Ethereum scaling with strong institutional adoption. Real-world asset rails and traditional finance integration.",
    color: "text-purple-400",
    border: "border-purple-400/30",
    bg: "bg-purple-400/5",
  },
  {
    name: "Near",
    role: "AI-native contract surfaces",
    detail: "Chain abstraction and AI-native smart contracts. A natural fit for agents operating across multiple chains.",
    color: "text-green-400",
    border: "border-green-400/30",
    bg: "bg-green-400/5",
  },
  {
    name: "Sui",
    role: "Object-centric execution",
    detail: "Move-based, object-centric model with strong parallelism. Well-suited for complex state management and DeFi composability.",
    color: "text-cyan",
    border: "border-cyan/30",
    bg: "bg-cyan/5",
  },
];

export default function TradingPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10 pt-24">
        {/* Hero */}
        <section className="min-h-[70vh] flex flex-col items-center justify-center px-6 py-20 text-center">
          <div className="max-w-4xl mx-auto">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-[oklch(0.7_0.25_320)]/20 bg-[oklch(0.7_0.25_320)]/5 mb-8">
              <span className="w-2 h-2 rounded-full bg-[oklch(0.7_0.25_320)] animate-[pulse_3s_ease-in-out_infinite]" />
              <span className="text-sm text-[oklch(0.7_0.25_320)] font-semibold tracking-wide">
                In-House Capital Engine
              </span>
            </div>

            <h1 className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6">
              <span className="gradient-text">Precision Execution</span>
              <br />
              <span className="text-foreground">Across On-Chain Markets</span>
            </h1>

            <p className="text-xl sm:text-2xl text-muted-foreground max-w-3xl mx-auto mb-10">
              5D Labs operates its own trading engine across Solana, Base, Polygon, Near, and Sui.
              It finances the studio, sharpens our market intelligence, and gives us
              a front-row seat as{" "}
              <span className="text-foreground">on-chain finance becomes the new market infrastructure</span>.
            </p>

            <div className="flex flex-col sm:flex-row justify-center gap-4">
              <Link
                href="/#operating-model"
                className="px-8 py-4 rounded-lg bg-gradient-to-r from-purple-500 to-[oklch(0.7_0.25_320)] text-white font-semibold text-lg hover:opacity-90 transition-all shadow-xl shadow-purple-500/20 hover:scale-105"
              >
                See How It Fits
              </Link>
              <a
                href="https://cto.5dlabs.ai"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all"
              >
                Explore CTO
              </a>
            </div>
          </div>
        </section>

        {/* Why It Exists */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Why It <span className="gradient-text">Exists</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
                The trading engine is the studio&apos;s capital and
                intelligence layer — designed to keep 5D Labs funded, informed, and
                operating close to where markets are actually moving.
              </p>
            </div>

            <div className="grid md:grid-cols-3 gap-8">
              {pillars.map((pillar) => (
                <div
                  key={pillar.title}
                  className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
                >
                  <h3 className={`text-xl font-semibold mb-3 ${pillar.color}`}>
                    {pillar.title}
                  </h3>
                  <p className="text-sm text-muted-foreground">{pillar.description}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Solana Thesis */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="text-center mb-16">
              <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-[oklch(0.7_0.25_320)]/20 bg-[oklch(0.7_0.25_320)]/5 mb-6">
                <span className="text-xs text-[oklch(0.7_0.25_320)] font-medium uppercase tracking-wider">
                  Institutional Thesis
                </span>
              </div>
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Solana Is Becoming the{" "}
                <span className="gradient-text">New Market Infrastructure</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
                Tokenized equities, on-chain settlement, and institutional liquidity are
                converging on Solana. We positioned early. The NYSE is now following.
              </p>
            </div>

            <div className="grid md:grid-cols-2 gap-6 mb-10">
              {solanaThesis.map((item) => (
                <div
                  key={item.title}
                  className="p-6 rounded-xl border border-[oklch(0.7_0.25_320)]/20 bg-[oklch(0.7_0.25_320)]/5 backdrop-blur-sm"
                >
                  <h3 className="text-lg font-semibold mb-2 text-foreground">{item.title}</h3>
                  <p className="text-sm text-muted-foreground">{item.description}</p>
                </div>
              ))}
            </div>

            <div className="p-5 rounded-xl border border-border/30 bg-muted/20 text-center">
              <p className="text-sm text-muted-foreground max-w-3xl mx-auto">
                Ondo Finance and xStocks are already running 200+ tokenized U.S. equities on Solana
                with direct NASDAQ/NYSE connectivity. The NYSE itself is building on-chain settlement
                infrastructure with Solana listed as a supported layer.{" "}
                <span className="text-foreground">This is not a future bet — it is already in motion.</span>
              </p>
            </div>
          </div>
        </section>

        {/* Multi-Chain */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Multi-Chain by <span className="gradient-text">Design</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Each chain has a distinct role. We match the execution environment to the opportunity,
                with Solana as the primary layer for precision, high-throughput workflows.
              </p>
            </div>

            <div className="grid sm:grid-cols-2 gap-6">
              {chains.map((chain) => (
                <div
                  key={chain.name}
                  className={`p-4 sm:p-6 rounded-xl border ${chain.border} ${chain.bg} backdrop-blur-sm`}
                >
                  <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-1 sm:gap-2 mb-3">
                    <h3 className={`text-base sm:text-xl font-bold ${chain.color}`}>{chain.name}</h3>
                    <span className={`text-[10px] sm:text-xs font-medium uppercase tracking-widest ${chain.color} opacity-70 sm:shrink-0`}>
                      {chain.role}
                    </span>
                  </div>
                  <p className="text-xs sm:text-sm text-muted-foreground">{chain.detail}</p>
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
