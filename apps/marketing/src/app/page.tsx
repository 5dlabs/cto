"use client";

import { motion } from "framer-motion";
import { GridPulse } from "@/components/grid-pulse";
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
        description: "Orchestrates project lifecycles—syncing GitHub with Linear, decomposing PRDs into tasks.",
        stack: ["Linear", "GitHub", "PRDs"],
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
        stack: ["Rust", "Tokio", "Axum"],
      },
      {
        name: "Grizz",
        role: "Go Specialist",
        avatar: "/agents/grizz-avatar-512.png",
        color: "from-amber-500 to-orange-400",
        description: "Ships bulletproof backend services, REST/gRPC APIs, and Kubernetes operators.",
        stack: ["Go", "gRPC", "PostgreSQL"],
      },
      {
        name: "Nova",
        role: "Node.js Engineer",
        avatar: "/agents/nova-avatar-512.png",
        color: "from-purple-500 to-cyan-400",
        description: "Rapid API development and third-party integrations. Speed-to-market specialist.",
        stack: ["Node.js", "TypeScript", "Fastify"],
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
        stack: ["React", "Next.js", "shadcn/ui"],
      },
      {
        name: "Tap",
        role: "Mobile Developer",
        avatar: "/agents/tap-avatar-512.png",
        color: "from-green-500 to-emerald-400",
        description: "Native-quality iOS and Android apps from a single TypeScript codebase.",
        stack: ["Expo", "React Native", "NativeWind"],
      },
      {
        name: "Spark",
        role: "Desktop Developer",
        avatar: "/agents/spark-avatar-512.png",
        color: "from-blue-500 to-yellow-400",
        description: "Cross-platform desktop apps with native integrations and offline-first architecture.",
        stack: ["Electron", "Tauri", "React"],
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
        stack: ["Code Review", "Patterns", "Best Practices"],
      },
      {
        name: "Cipher",
        role: "Security Sentinel",
        avatar: "/agents/cipher-avatar-512.png",
        color: "from-red-500 to-rose-500",
        description: "Runs security audits, dependency scans, and ensures OWASP compliance.",
        stack: ["Trivy", "Gitleaks", "OWASP"],
      },
      {
        name: "Tess",
        role: "Testing Genius",
        avatar: "/agents/tess-avatar-512.png",
        color: "from-violet-500 to-purple-500",
        description: "Creates comprehensive test suites—unit, integration, and e2e.",
        stack: ["Jest", "Playwright", "Vitest"],
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
        stack: ["PR Review", "Linting", "Standards"],
      },
      {
        name: "Atlas",
        role: "Integration Master",
        avatar: "/agents/atlas-avatar-512.png",
        color: "from-slate-500 to-zinc-500",
        description: "Manages PR merges, rebases stale branches, and ensures clean integration.",
        stack: ["Git", "Rebasing", "CI/CD"],
      },
      {
        name: "Bolt",
        role: "Infrastructure & SRE",
        avatar: "/agents/bolt-avatar-512.png",
        color: "from-yellow-500 to-amber-500",
        description: "Your always-on SRE. Provisions bare metal, deploys services, monitors health, and triggers self-healing — so you never get paged.",
        stack: ["Kubernetes", "Bare Metal", "GitOps"],
      },
      {
        name: "Healer",
        role: "Self-Healing Agent",
        color: "from-green-500 to-emerald-500",
        description: "Detects failures, remediates incidents, restarts stuck workflows, and fixes CI — automatically, before you notice.",
        stack: ["Incident Response", "Auto-Remediation", "Monitoring"],
      },
    ],
  },
];

export default function Home() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background layers */}
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.06_0.03_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <GridPulse />
      <div className="fixed inset-0 noise-overlay z-0" />

      {/* Header */}
      <Header />

      {/* Content */}
      <main className="relative z-10">
        {/* Hero Section */}
        <section id="hero" className="min-h-screen flex flex-col items-center justify-center px-6 py-20 pt-24">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8 }}
            className="max-w-4xl mx-auto text-center"
          >
            {/* Badge */}
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.2, duration: 0.8, ease: "easeOut" }}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8"
            >
              <span className="w-2 h-2 rounded-full bg-cyan animate-[pulse_3s_ease-in-out_infinite]" />
              <span className="text-sm text-cyan font-medium">
                From PRD to Production — Autonomously
              </span>
            </motion.div>

            {/* Headline */}
            <motion.h1
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3, duration: 1, ease: "easeOut" }}
              className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6"
            >
              <span className="gradient-text glow-text-cyan">Your Engineering Team</span>
              <br />
              <span className="text-foreground">Lives Here</span>
            </motion.h1>

            {/* Subheadline */}
            <motion.p
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5, duration: 1, ease: "easeOut" }}
              className="text-xl sm:text-2xl text-muted-foreground max-w-2xl mx-auto mb-10"
            >
              Thirteen specialized AI agents that ship complete features. From requirements to deployed code—automatically.
            </motion.p>

            {/* CTA Buttons */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7, duration: 0.8 }}
              className="flex flex-col sm:flex-row justify-center gap-4 mb-16"
            >
              {featureFlags.showStartNowButton && (
                <a
                  href="https://app.5dlabs.ai"
                  className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
                >
                  Start Now
                </a>
              )}
              <WaitlistForm />
            </motion.div>

            {/* Stats */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.9, duration: 0.8 }}
              className="flex flex-wrap justify-center gap-8 text-sm text-muted-foreground"
            >
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">60-80%</span>
                <span>cost savings vs cloud</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">13</span>
                <span>specialized agents</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold text-foreground">∞</span>
                <span>faster shipping</span>
              </div>
            </motion.div>
          </motion.div>

          {/* Scroll indicator */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 1.2, duration: 1 }}
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

        {/* Agents Section */}
        <section id="agents" className="py-20 px-6">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 12 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 1, ease: "easeOut" }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                We Brought the <span className="gradient-text">Whole Team</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-xl mx-auto">
                Not one generic AI—thirteen domain experts working in parallel across your entire development lifecycle.
              </p>
            </motion.div>

            {/* Agent Grid */}
            <AgentGrid squads={squads} />
          </div>
        </section>

        {/* Tech Stack Section */}
        <TechStack />

        {/* Ecosystem Section */}
        <section id="ecosystem" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Your <span className="gradient-text">Entire Stack</span>, Orchestrated
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                From languages to infrastructure. Bring your own keys, pick your CLIs—we handle everything else.
              </p>
            </motion.div>

            <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
              {/* Languages & Frameworks */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-3">Languages & Frameworks</h3>
                <div className="space-y-2 text-sm text-muted-foreground">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>Rust, Go, Node.js, Python</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>React, Vue, Svelte, Next.js</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>FastAPI, Express, Axum, Chi</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-cyan/60"></span>
                    <span>Electron, Expo, Unity</span>
                  </div>
                </div>
              </motion.div>

              {/* Infrastructure & Databases */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
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
                    <span>Healer auto-remediates incidents</span>
                  </div>
                </div>
              </motion.div>

              {/* AI CLIs & Models */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.2 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
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
                    <span>GPT-4, Claude, Gemini models</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-yellow/60"></span>
                    <span>Ollama, vLLM for local inference</span>
                  </div>
                </div>
              </motion.div>

              {/* Integrations */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.3 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
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
              </motion.div>
            </div>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8, delay: 0.4 }}
              className="text-center mt-12"
            >
              <p className="text-muted-foreground">
                Submit a PRD, connect your repo, and watch your AI team ship. No manual handoffs, no context switching.
              </p>
            </motion.div>
          </div>
        </section>

        {/* Infrastructure Providers Section */}
        <section id="infrastructure" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Bare Metal <span className="gradient-text">Everywhere</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Deploy on dedicated servers worldwide. Skip the cloud tax and own your infrastructure.
              </p>
            </motion.div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {[
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
              ].map((provider, index) => (
                <motion.div
                  key={provider.name}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: index * 0.05 }}
                  className="p-4 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm text-center hover:border-cyan/30 transition-colors"
                >
                  <h4 className="font-semibold mb-1 text-foreground">{provider.name}</h4>
                  <p className="text-xs text-muted-foreground mb-1">{provider.region}</p>
                  <p className="text-xs text-muted-foreground">{provider.desc}</p>
                </motion.div>
              ))}
            </div>
          </div>
        </section>

        {/* Platform Features Section */}
        <section id="platform" className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                The <span className="gradient-text">Platform</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Everything you need to ship production software—orchestration, project management, and self-healing infrastructure.
              </p>
            </motion.div>

            <div className="grid md:grid-cols-3 gap-6">
              {/* MCP Tools */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
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
              </motion.div>

              {/* Linear Integration */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Linear Agent API</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Native Linear integration with real-time agent activities. Watch your AI team work in your project board.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["PRD Intake", "Task Sync", "Live Updates"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-[oklch(0.7_0.25_320)]/10 text-[oklch(0.7_0.25_320)]">{feature}</span>
                  ))}
                </div>
              </motion.div>

              {/* Self-Healing */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.2 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-green-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Self-Healing Infrastructure</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Healer detects incidents, remediates failures, and restarts stuck workflows — automatically. Bolt provisions and monitors bare metal. No on-call rotation needed.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["Auto-Remediation", "Incident Detection", "Health Checks", "Auto-Rollback"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-green-500/10 text-green-500">{feature}</span>
                  ))}
                </div>
              </motion.div>

              {/* Kubernetes Operators */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.3 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-blue-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Database Operators</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Replace managed cloud services with open-source Kubernetes operators. 60-80% cost savings.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["PostgreSQL", "Redis", "Kafka", "MongoDB"].map(db => (
                    <span key={db} className="text-xs px-2 py-1 rounded bg-blue-500/10 text-blue-500">{db}</span>
                  ))}
                </div>
              </motion.div>

              {/* GitHub-Driven Deployment */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.4 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
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
              </motion.div>

              {/* BYOK */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.5 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-purple-500/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Bring Your Own Keys</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Your API keys, your infrastructure credentials. Stored in OpenBao (HashiCorp Vault fork). Zero vendor lock-in.
                </p>
                <div className="flex flex-wrap gap-2">
                  {["OpenBao", "Zero Trust", "BYOK"].map(feature => (
                    <span key={feature} className="text-xs px-2 py-1 rounded bg-purple-500/10 text-purple-500">{feature}</span>
                  ))}
                </div>
              </motion.div>
            </div>
          </div>
        </section>

        {/* Why CTO Section */}
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
                Why <span className="gradient-text">CTO</span>?
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Other tools help you code. CTO ships complete features—from PRD to production—with specialized agents for every stage of development.
              </p>
            </motion.div>

            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-8">
              {/* CLI Agnostic */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5 }}
                className="p-6 rounded-xl border border-cyan/30 bg-cyan/5 backdrop-blur-sm text-center h-full"
              >
                <div className="w-14 h-14 rounded-full bg-cyan/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Choose Your CLI</h3>
                <p className="text-sm text-muted-foreground">
                  Claude Code, Cursor, Factory, Codex, Gemini—use what you love. We&apos;re agnostic.
                </p>
              </motion.div>

              {/* Multi-Agent */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: 0.1 }}
                className="p-6 rounded-xl border border-[oklch(0.7_0.25_320)]/30 bg-[oklch(0.7_0.25_320)]/5 backdrop-blur-sm text-center h-full"
              >
                <div className="w-14 h-14 rounded-full bg-[oklch(0.7_0.25_320)]/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Multi-Agent</h3>
                <p className="text-sm text-muted-foreground">
                  13 specialists working in parallel. PM, backend, frontend, QA, security, DevOps—all coordinated.
                </p>
              </motion.div>

              {/* Bare Metal */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: 0.2 }}
                className="p-6 rounded-xl border border-green-500/30 bg-green-500/5 backdrop-blur-sm text-center h-full"
              >
                <div className="w-14 h-14 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Fully Managed Bare Metal</h3>
                <p className="text-sm text-muted-foreground">
                  Self-healing infrastructure on dedicated servers. Zero cloud tax, zero ops burden.
                </p>
              </motion.div>

              {/* Bleeding Edge */}
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: 0.3 }}
                className="p-6 rounded-xl border border-yellow-500/30 bg-yellow-500/5 backdrop-blur-sm text-center h-full"
              >
                <div className="w-14 h-14 rounded-full bg-yellow-500/10 flex items-center justify-center mx-auto mb-4">
                  <svg className="w-7 h-7 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <h3 className="text-lg font-semibold mb-2">Bleeding Edge</h3>
                <p className="text-sm text-muted-foreground">
                  Always current. Latest models, newest CLIs, freshest integrations. We stay on the frontier.
                </p>
              </motion.div>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <div className="grid md:grid-cols-2 gap-8">
              {/* Feature 1 */}
              <motion.div
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-cyan/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-cyan" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Bare Metal Advantage</h3>
                <p className="text-muted-foreground">
                  Skip the cloud markup. Direct bare metal pricing with cloud-like reliability.
                </p>
              </motion.div>

              {/* Feature 2 */}
              <motion.div
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-lg bg-magenta/10 flex items-center justify-center mb-4">
                  <svg className="w-6 h-6 text-[oklch(0.7_0.25_320)]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold mb-2">Complete Engineering Team</h3>
                <p className="text-muted-foreground">
                  Not an AI assistant. Thirteen specialists across PM, backend, frontend, quality, security, testing, and deployment.
                </p>
              </motion.div>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section className="py-20 px-6">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8 }}
            className="max-w-2xl mx-auto text-center"
          >
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
          </motion.div>
        </section>

        {/* Footer */}
        <footer className="py-8 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img src="/5dlabs-logo-header-v2.png" alt="5D Labs" className="h-16 opacity-90" />
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
