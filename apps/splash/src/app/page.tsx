"use client";

import { motion } from "framer-motion";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { VentureGrid, type Venture } from "@/components/venture-card";
import { WaitlistForm } from "@/components/waitlist-form";

const ventures: Venture[] = [
  {
    name: "CTO",
    tagline: "The thing to build the thing.",
    description:
      "A multi-agent AI engineering platform. Thirteen specialized agents ship complete features — from PRD to production — autonomously on self-healing bare metal infrastructure.",
    tags: ["Multi-Agent", "Kubernetes", "OpenClaw", "Bare Metal"],
    color: "from-cyan-500 to-blue-500",
    href: "https://cto.5dlabs.ai",
    status: "building",
  },
  {
    name: "Agentic Trading",
    tagline: "Your own personal hedge fund.",
    description:
      "HFT-grade autonomous trading agents on Solana, Base, Near, and Sui — putting the same technology that powers Wall Street hedge funds into the hands of individuals. AI-driven strategy, execution, and risk management across DeFi markets, 24/7.",
    tags: ["HFT", "Solana", "Base", "Near", "Sui", "DeFi", "AI Agents"],
    color: "from-purple-500 to-magenta",
    status: "building",
  },
  {
    name: "OpenClaw Platform",
    tagline: "Deploy your own agent swarm.",
    description:
      "Open-source Kubernetes-native platform for orchestrating AI agent fleets. One-command install via TUI, runs on desktop KinD clusters or enterprise EKS — with GitOps, NATS messaging, observability, and secrets management built in.",
    tags: ["Open Source", "Kubernetes", "GitOps", "ArgoCD", "Helm"],
    color: "from-orange-500 to-amber-500",
    href: "https://github.com/5dlabs/openclaw-platform",
    status: "building",
  },
  {
    name: "Sanctuary",
    tagline: "Your home becomes a supportive director.",
    description:
      "A privacy-forward AI Life Architect that turns multimodal life signals — sensors, devices, calendar, purchases — into textural orchestration: shaping your environment, pacing, and microinterventions for wellbeing and household flow.",
    tags: ["Smart Home", "AI", "IoT", "Wellness", "Privacy-First"],
    color: "from-emerald-500 to-teal-500",
    status: "building",
  },
  {
    name: "What's Next?",
    tagline: "Always building.",
    description:
      "The beauty of a startup studio is that we're always exploring. By validating many ideas in parallel, we find what sticks — faster than anyone else.",
    tags: ["OpenClaw", "Crypto", "AI", "Web3"],
    color: "from-yellow-500 to-orange-500",
    status: "exploring",
  },
];

export default function Home() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.06_0.03_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      {/* GridPulse at z-[2] from layout */}
      <div className="fixed inset-0 noise-overlay z-[3]" />

      {/* Header */}
      <Header />

      {/* Content */}
      <main className="relative z-10">
        {/* Hero Section */}
        <section className="min-h-screen flex flex-col items-center justify-center px-6 py-20 pt-24">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 1.2 }}
            className="max-w-4xl mx-auto text-center"
          >
            {/* Badge */}
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.4, duration: 1.2, ease: "easeOut" }}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8"
            >
              <span className="text-sm text-cyan font-semibold tracking-wide">
                Agentic &middot; Fully Autonomous &middot; On-Chain
              </span>
            </motion.div>

            {/* Headline */}
            <motion.h1
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7, duration: 1.4, ease: "easeOut" }}
              className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6"
            >
              <span className="gradient-text glow-text-cyan">Build in Parallel.</span>
              <br />
              <span className="text-foreground">Fail Fast. Double Down.</span>
            </motion.h1>

            {/* Subheadline */}
            <motion.p
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 1.0, duration: 1.4, ease: "easeOut" }}
              className="text-xl sm:text-2xl text-muted-foreground max-w-2xl mx-auto mb-10"
            >
              5D Labs is a startup studio that leverages OpenClaw and autonomous
              AI agents to validate many ideas simultaneously — transcending the
              old &ldquo;pivot or perish&rdquo; model.
            </motion.p>

            {/* CTA Buttons */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 1.3, duration: 1.2 }}
              className="flex flex-col sm:flex-row justify-center gap-4 mb-16"
            >
              <a
                href="https://cto.5dlabs.ai"
                className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
              >
                Explore CTO
              </a>
              <a
                href="/investors"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all"
              >
                Investor Relations
              </a>
            </motion.div>

            {/* Stats */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 1.6, duration: 1.2 }}
              className="flex flex-wrap justify-center gap-8 text-sm text-muted-foreground"
            >
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">5+</span>
                <span>ventures in flight</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">13</span>
                <span>AI agents</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">3</span>
                <span>blockchains</span>
              </div>
            </motion.div>
          </motion.div>

          {/* Scroll indicator */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 2.0, duration: 1.2 }}
            className="absolute bottom-10 left-1/2 -translate-x-1/2"
          >
            <motion.div
              animate={{ y: [0, 6, 0] }}
              transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
              className="w-6 h-10 rounded-full border-2 border-muted-foreground/30 flex justify-center pt-2"
            >
              <div className="w-1 h-2 rounded-full bg-cyan" />
            </motion.div>
          </motion.div>
        </section>

        {/* Disruptor Manifesto */}
        <section className="py-16 px-6">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8 }}
            className="max-w-3xl mx-auto text-center"
          >
            <p className="text-xl sm:text-2xl text-muted-foreground leading-relaxed">
              One idea. One team. One shot at product-market fit.
              <br className="hidden sm:block" />
              We thought there might be a{" "}
              <span className="text-foreground font-semibold">better way</span>.
            </p>
          </motion.div>
        </section>

        {/* Thesis Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                The <span className="gradient-text">Startup Studio</span> Model, Reimagined
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Traditional startups burn runway trying to find product-market
                fit with a single idea. We take a different approach.
              </p>
            </motion.div>

            <div className="grid md:grid-cols-3 gap-8">
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
              >
                <div className="w-14 h-14 rounded-full bg-cyan/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Parallel Validation</h3>
                <p className="text-sm text-muted-foreground">
                  Instead of one bet, we explore multiple ventures simultaneously.
                  OpenClaw and AI agents let us move at startup speed across
                  every idea.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
              >
                <div className="w-14 h-14 rounded-full bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Faster Product-Market Fit</h3>
                <p className="text-sm text-muted-foreground">
                  Find what works before the money runs out. Our tooling lets us
                  validate or kill ideas in weeks, not quarters.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.2 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
              >
                <div className="w-14 h-14 rounded-full bg-yellow-500/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">AI-Native Infrastructure</h3>
                <p className="text-sm text-muted-foreground">
                  Every venture is built with autonomous AI agents from day one.
                  OpenClaw powers the orchestration layer across our entire
                  portfolio.
                </p>
              </motion.div>
            </div>
          </div>
        </section>

        {/* Ventures Section */}
        <section id="ventures" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Our <span className="gradient-text">Ventures</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                From AI engineering platforms to on-chain trading agents, open-source
                tooling, and smart homes — here&apos;s what we&apos;re building.
              </p>
            </motion.div>

            <VentureGrid ventures={ventures} />
          </div>
        </section>

        {/* Crypto Vision Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-5xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ delay: 0.1, duration: 0.6 }}
                className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-[oklch(0.7_0.25_320)]/20 bg-[oklch(0.7_0.25_320)]/5 mb-6"
              >
                <span className="text-xs text-[oklch(0.7_0.25_320)] font-medium uppercase tracking-wider">
                  Crypto Native
                </span>
              </motion.div>
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Building for the <span className="gradient-text">New Economy</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                We believe AI agents and crypto rails are converging into
                something bigger — an agent economy where software transacts,
                earns, and operates autonomously on-chain.
              </p>
            </motion.div>

            <div className="grid md:grid-cols-3 gap-8">
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
              >
                <div className="w-14 h-14 rounded-full bg-yellow-500/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Solana · Base · Near · Sui</h3>
                <p className="text-sm text-muted-foreground">
                  Four chains, one thesis. Solana for raw speed and HFT,
                  Base for Ethereum-grade composability, Near for AI-native
                  smart contracts, Sui for Move and object-centric DeFi. We&apos;re building where the action is.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
              >
                <div className="w-14 h-14 rounded-full bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Multi-Chain</h3>
                <p className="text-sm text-muted-foreground">
                  Solana for speed, Ethereum and Base for composability, Near
                  for AI-native smart contracts, Sui for Move and object-centric DeFi. We go where the opportunity
                  is.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.2 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center"
              >
                <div className="w-14 h-14 rounded-full bg-cyan/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">The Agent Economy</h3>
                <p className="text-sm text-muted-foreground">
                  Agents that don&apos;t just write code — they trade, settle
                  payments, and coordinate on-chain. AI and crypto aren&apos;t
                  separate bets; they&apos;re one thesis.
                </p>
              </motion.div>
            </div>
          </div>
        </section>

        {/* OpenClaw Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-12"
            >
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                whileInView={{ opacity: 1, scale: 1 }}
                viewport={{ once: true }}
                transition={{ delay: 0.1, duration: 0.6 }}
                className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan/20 bg-cyan/5 mb-6"
              >
                <span className="text-xs text-cyan font-medium uppercase tracking-wider">
                  Our Foundation
                </span>
              </motion.div>
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Built with <span className="gradient-text">OpenClaw</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto mb-8">
                OpenClaw is our open-source agent orchestration layer.
                Self-hosted models, every major provider, and infrastructure
                we own — optimized for cost, speed, and staying ahead of an
                industry that moves daily.
              </p>
            </motion.div>

            <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
              <motion.div
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <h3 className="text-xl font-semibold mb-2">Self-Hosted Models</h3>
                <p className="text-muted-foreground text-sm">
                  Run open-weight models on our own hardware. Full control
                  over inference, zero vendor lock-in, dramatically lower
                  costs.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <h3 className="text-xl font-semibold mb-2">Every Major Model</h3>
                <p className="text-muted-foreground text-sm">
                  We use them all — frontier and open-weight. Best model for
                  each task, swapped in as new ones drop.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.2 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <h3 className="text-xl font-semibold mb-2">Cost Optimized</h3>
                <p className="text-muted-foreground text-sm">
                  Bare metal over cloud. Self-hosted inference over API calls.
                  60-80% savings on infrastructure without sacrificing
                  performance.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.3 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <h3 className="text-xl font-semibold mb-2">Always Current</h3>
                <p className="text-muted-foreground text-sm">
                  New model? Integrated within days. New CLI? Already
                  supported. We move at the speed of the industry — or faster.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.4 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <h3 className="text-xl font-semibold mb-2">Agent Orchestration</h3>
                <p className="text-muted-foreground text-sm">
                  Thirteen specialized agents coordinated across development,
                  trading, and operations. Each with its own identity, skills,
                  and tools.
                </p>
              </motion.div>

              <motion.div
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.5 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <h3 className="text-xl font-semibold mb-2">Open at the Core</h3>
                <p className="text-muted-foreground text-sm">
                  Key infrastructure and tooling released as open source.
                  We contribute upstream and build on open standards wherever
                  possible.
                </p>
              </motion.div>
            </div>
          </div>
        </section>

        {/* Waitlist + CTA Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8 }}
            className="max-w-2xl mx-auto"
          >
            <WaitlistForm
              source="waitlist"
              heading="Stay in the Loop"
              subheading="Join the waitlist to get early access to OpenClaw Platform, trading tools, and studio updates. No spam, ever."
            />

            <div className="flex flex-col sm:flex-row justify-center gap-4 mt-10">
              <a
                href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
                target="_blank"
                rel="noopener noreferrer"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all text-center"
              >
                Schedule a Call
              </a>
              <a
                href="https://github.com/5dlabs"
                target="_blank"
                rel="noopener noreferrer"
                className="px-8 py-4 rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm text-foreground font-semibold text-lg hover:border-cyan/30 hover:bg-cyan/5 transition-all text-center"
              >
                View on GitHub
              </a>
            </div>
          </motion.div>
        </section>

        {/* Footer */}
        <Footer />
      </main>
    </div>
  );
}
