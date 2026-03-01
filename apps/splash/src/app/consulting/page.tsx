"use client";

import { motion } from "framer-motion";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { WaitlistForm } from "@/components/waitlist-form";
import { colorMap } from "@/lib/utils";

const expertise = [
  {
    title: "Multi-Agent AI Systems",
    description:
      "OpenClaw deployment and configuration, agent orchestration, MCP server integration, Claude Code workflows, autonomous development pipelines, custom skill development.",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
    color: "cyan",
  },
  {
    title: "Blockchain & Solana",
    description:
      "Solana program development, on-chain trading agents, DeFi protocol integration, Base/Near/Sui/Ethereum node operations, RPC infrastructure at scale.",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
    color: "purple",
  },
  {
    title: "Infrastructure & DevOps",
    description:
      "Bare-metal Kubernetes, Talos Linux, ArgoCD GitOps, Cilium networking, zero-trust architecture. Proven 60\u201380% cloud cost reduction.",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
      </svg>
    ),
    color: "blue",
  },
  {
    title: "Platform Engineering",
    description:
      "Rust systems programming, custom Kubernetes operators and CRDs, event-driven architectures, Argo Workflows, high-performance async services.",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
      </svg>
    ),
    color: "orange",
  },
  {
    title: "Observability & Reliability",
    description:
      "Prometheus, Grafana, Loki, OpenTelemetry stacks. Self-healing systems, incident response automation, SRE practices for high-availability workloads.",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    ),
    color: "yellow",
  },
  {
    title: "Security & Networking",
    description:
      "Zero-trust networking with Cloudflare Tunnels and WireGuard. HashiCorp Vault and OpenBao secret management. ZTNA design and implementation.",
    icon: (
      <svg className="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    ),
    color: "emerald",
  },
];

const rates = [
  {
    name: "Hourly Advisory",
    price: "$225",
    unit: "/ hour",
    details: "Ad-hoc consulting, architecture guidance, troubleshooting. 1-hour minimum.",
  },
  {
    name: "Daily Intensive",
    price: "$1,600",
    unit: "/ day",
    details: "Full-day working sessions. Ideal for workshops, hands-on setup, or team training.",
  },
  {
    name: "Monthly Retainer",
    price: "$3,500",
    unit: "/ month",
    details: "~20 hours included. Dedicated availability, async Slack support, scheduled calls. Overage at $200/hr.",
    recommended: true,
  },
  {
    name: "Fixed-Scope Project",
    price: "Varies",
    unit: "",
    details: "Defined deliverables with clear timeline and milestones. Scoped during discovery call.",
  },
];

const phases = [
  {
    week: "Week 1",
    title: "Discovery & Architecture",
    description:
      "Audit current workflows, tooling, and infrastructure. Define the target architecture, hosting approach, and security requirements.",
  },
  {
    week: "Week 2",
    title: "Deployment & Configuration",
    description:
      "Infrastructure setup, platform configuration, authentication, networking, and access controls. Production-ready environments.",
  },
  {
    week: "Weeks 3\u20134",
    title: "Custom Development & Integrations",
    description:
      "Custom tooling, CI/CD automation, agent workflows, monitoring integrations, and platform-specific development.",
  },
  {
    week: "Week 5",
    title: "Training, Security & Handoff",
    description:
      "Hands-on training, security audit, documentation, runbooks, and transition to ongoing support or retainer.",
  },
];

const stats = [
  { value: "20+", label: "years in infrastructure & engineering" },
  { value: "1B+", label: "daily requests managed (Pocket Network)" },
  { value: "50+", label: "blockchain clients operated" },
  { value: "13", label: "AI agents shipping production code" },
];

const consultingJsonLd = {
  "@context": "https://schema.org",
  "@type": "ProfessionalService",
  "@id": "https://5dlabs.ai/consulting/#service",
  name: "5D Labs Consulting",
  description:
    "Expert consulting for AI agent systems, blockchain and Solana development, Kubernetes infrastructure, and DevOps. Hands-on expertise from 20+ years in infrastructure and software engineering.",
  provider: {
    "@id": "https://5dlabs.ai/#organization",
  },
  serviceType: [
    "AI Agent Consulting",
    "OpenClaw Deployment",
    "Blockchain Consulting",
    "Solana Development",
    "Kubernetes Consulting",
    "DevOps Consulting",
    "Infrastructure Architecture",
  ],
  areaServed: {
    "@type": "Country",
    name: "US",
  },
  priceRange: "$225-$3500/mo",
  url: "https://5dlabs.ai/consulting/",
  hasOfferCatalog: {
    "@type": "OfferCatalog",
    name: "Consulting Engagements",
    itemListElement: [
      {
        "@type": "Offer",
        name: "Hourly Advisory",
        price: "225",
        priceCurrency: "USD",
        description: "Ad-hoc consulting, architecture guidance, troubleshooting.",
      },
      {
        "@type": "Offer",
        name: "Daily Intensive",
        price: "1600",
        priceCurrency: "USD",
        description: "Full-day working sessions for workshops, hands-on setup, or team training.",
      },
      {
        "@type": "Offer",
        name: "Monthly Retainer",
        price: "3500",
        priceCurrency: "USD",
        description: "~20 hours included with dedicated availability and async support.",
      },
      {
        "@type": "Offer",
        name: "Fixed-Scope Project",
        description: "Defined deliverables with clear timeline and milestones. Price varies by project scope.",
      },
    ],
  },
};

export default function ConsultingPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.06_0.03_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-0" />
      <div className="fixed inset-0 noise-overlay z-0" />

      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(consultingJsonLd) }}
      />

      <Header />

      <main className="relative z-10 pt-24">
        {/* Hero */}
        <section className="py-20 px-6">
          <div className="max-w-4xl mx-auto text-center">
            <motion.div
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.2, duration: 0.8 }}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-cyan/30 bg-cyan/5 mb-8"
            >
              <span className="text-sm text-cyan font-semibold tracking-wide">
                Available for Engagements
              </span>
            </motion.div>

            <motion.h1
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4, duration: 1 }}
              className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight mb-6"
            >
              <span className="gradient-text glow-text-cyan">AI, Blockchain &</span>
              <br />
              <span className="text-foreground">Infrastructure Consulting</span>
            </motion.h1>

            <motion.p
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7, duration: 1 }}
              className="text-xl text-muted-foreground max-w-2xl mx-auto mb-10"
            >
              From multi-agent AI platforms to Solana trading systems to bare-metal
              Kubernetes &mdash; hands-on expertise from someone who builds it daily.
            </motion.p>

            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 1.0, duration: 0.8 }}
              className="flex flex-col sm:flex-row justify-center gap-4"
            >
              <a
                href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
                target="_blank"
                rel="noopener noreferrer"
                className="px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105"
              >
                Schedule a Discovery Call
              </a>
            </motion.div>
          </div>
        </section>

        {/* Expertise Grid */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                What I <span className="gradient-text">Bring</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Deep, production-tested expertise across AI, blockchain, and
                infrastructure &mdash; not theory, but systems I build and operate
                every day.
              </p>
            </motion.div>

            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {expertise.map((item, i) => {
                const colors = colorMap[item.color] || colorMap.cyan;
                return (
                  <motion.div
                    key={item.title}
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.5, delay: i * 0.08 }}
                    className="p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
                  >
                    <div
                      className={`w-14 h-14 rounded-full ${colors.bg} flex items-center justify-center mb-4`}
                    >
                      <div className={colors.text}>{item.icon}</div>
                    </div>
                    <h3 className="text-lg font-semibold mb-2">{item.title}</h3>
                    <p className="text-sm text-muted-foreground">
                      {item.description}
                    </p>
                  </motion.div>
                );
              })}
            </div>
          </div>
        </section>

        {/* Rate Card */}
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
                <span className="gradient-text">Engagement</span> Options
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                Flexible structures to match your team&apos;s needs and budget. All
                engagements include a complimentary 30-minute discovery call.
              </p>
            </motion.div>

            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
              {rates.map((rate, i) => (
                <motion.div
                  key={rate.name}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: i * 0.1 }}
                  className={`p-6 rounded-xl border backdrop-blur-sm relative ${
                    rate.recommended
                      ? "border-cyan/40 bg-cyan/5"
                      : "border-border/50 bg-card/30"
                  }`}
                >
                  {rate.recommended && (
                    <div className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-0.5 rounded-full bg-cyan text-background text-xs font-semibold">
                      Recommended
                    </div>
                  )}
                  <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider mb-3">
                    {rate.name}
                  </h3>
                  <div className="mb-4">
                    <span className="text-3xl font-bold text-foreground">
                      {rate.price}
                    </span>
                    <span className="text-muted-foreground ml-1">
                      {rate.unit}
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground">{rate.details}</p>
                </motion.div>
              ))}
            </div>

            <motion.p
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ delay: 0.5, duration: 0.6 }}
              className="text-center text-sm text-muted-foreground mt-8"
            >
              All rates quoted in USD. Payment accepted via wire transfer, ACH,
              or major credit card.
            </motion.p>
          </div>
        </section>

        {/* Sample Engagement */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-4xl mx-auto">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
              className="text-center mb-16"
            >
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Sample <span className="gradient-text">Engagement</span>
              </h2>
              <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
                A typical fixed-scope engagement for teams looking to deploy AI
                agents, blockchain infrastructure, or modernize their platform.
              </p>
            </motion.div>

            <div className="space-y-6">
              {phases.map((phase, i) => (
                <motion.div
                  key={phase.title}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: i * 0.1 }}
                  className="flex gap-6 items-start"
                >
                  <div className="shrink-0 w-20 text-center">
                    <div className="text-xs font-semibold text-cyan uppercase tracking-wider">
                      {phase.week}
                    </div>
                  </div>
                  <div className="flex-1 p-5 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm">
                    <h3 className="text-lg font-semibold mb-1">{phase.title}</h3>
                    <p className="text-sm text-muted-foreground">
                      {phase.description}
                    </p>
                  </div>
                </motion.div>
              ))}
            </div>

            <motion.div
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ delay: 0.5, duration: 0.6 }}
              className="text-center mt-10 p-4 rounded-lg border border-border/30 bg-card/20"
            >
              <p className="text-muted-foreground">
                Estimated investment:{" "}
                <span className="text-foreground font-semibold">
                  $10,000 &ndash; $15,000 USD
                </span>{" "}
                depending on team size, scope, and infrastructure complexity.
              </p>
            </motion.div>
          </div>
        </section>

        {/* Track Record */}
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
                Track <span className="gradient-text">Record</span>
              </h2>
            </motion.div>

            <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
              {stats.map((stat, i) => (
                <motion.div
                  key={stat.label}
                  initial={{ opacity: 0, y: 20 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: i * 0.1 }}
                  className="text-center p-6 rounded-xl border border-border/50 bg-card/30 backdrop-blur-sm"
                >
                  <div className="text-3xl font-bold gradient-text mb-2">
                    {stat.value}
                  </div>
                  <p className="text-sm text-muted-foreground">{stat.label}</p>
                </motion.div>
              ))}
            </div>

          </div>
        </section>

        {/* CTA */}
        <section className="py-20 px-6 border-t border-border/30">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8 }}
            className="max-w-2xl mx-auto text-center"
          >
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Ready to <span className="gradient-text">Get Started</span>?
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Every engagement begins with a complimentary 30-minute discovery
              call to understand your needs and scope the work.
            </p>
            <a
              href="https://cal.com/jonathon-fritz-2uhdqe/discovery"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-block px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold text-lg hover:from-cyan-600 hover:to-blue-600 transition-all shadow-xl shadow-cyan-500/30 hover:shadow-cyan-500/50 hover:scale-105 mb-10"
            >
              Schedule a Discovery Call
            </a>

            <div className="border-t border-border/30 pt-8">
              <p className="text-sm text-muted-foreground mb-4">
                Not ready for a call? Leave your email and we&apos;ll follow up.
              </p>
              <WaitlistForm source="consulting" compact />
            </div>
          </motion.div>
        </section>

        <Footer />
      </main>
    </div>
  );
}
