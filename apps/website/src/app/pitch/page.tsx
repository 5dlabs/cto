"use client";

import { motion, MotionConfig } from "framer-motion";
import { InvestorCtaButtons } from "@/components/investor-cta-buttons";
import { AnimatedCounter } from "@/components/animated-counter";
import {
  StatCard,
  PipelineFlow,
  ComparisonMap,
  ConcentricRings,
  AnimatedBarChart,
  CareerTimeline,
  FunnelDiagram,
} from "@/components/charts";
import { DeckToolbar } from "@/components/deck-toolbar";

/* ─── Slide 2: Problem ─── */
const painPoints = [
  {
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
    ),
    value: "Out of sync",
    label: "AI moves faster than your roadmap",
    note: "Training compute for notable models doubles ~every 5 months — Stanford AI Index 2025.",
  },
  {
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    value: "Opaque",
    label: "Cloud bills hide real costs",
    note: "Surveyed orgs report ~29% of cloud spend wasted — Flexera State of the Cloud 2026.",
  },
  {
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    ),
    value: "Sprawl",
    label: "Stack and integration debt",
    note: "More vendors, more glue code, more meetings — not more velocity.",
  },
];

/* ─── Slide 3: Founder ─── */
const founderTimeline = [
  {
    company: "Pocket",
    role: "Head of Infrastructure",
    highlight: "13 engineers. 1B+ requests/day. 50+ network integrations.",
  },
  {
    company: "Coinmiles",
    role: "Senior Engineer → CTO",
    highlight: "Promoted to CTO in 3 months.",
  },
  {
    company: "Blocknative",
    role: "Infrastructure Engineering",
    highlight: "Real-time transaction monitoring at scale.",
  },
  {
    company: "5D Labs",
    role: "Founder & CEO",
    highlight:
      "Solo built: platform, pilot customer, $240K pipeline, 17+ bare-metal deployments.",
    current: true,
  },
];

/* ─── Slide 4: Why Now — structural shift + convergence ─── */
const convergenceFactors = [
  {
    stat: "N×",
    label: "Parallel ventures, not serial bets",
    detail: "Agentic coding collapsed the cost of parallel execution. Run multiple ventures simultaneously on shared infrastructure — fail fast instead of pivoting near the end of your runway.",
    source: "Core thesis",
  },
  {
    stat: "280×",
    label: "Inference cost collapse",
    detail: "API pricing fell >280× in 18 months. Open-weight models (Llama, Mistral, DeepSeek) run on your hardware — no vendor lock-in, owned economics.",
    source: "Stanford AI Index 2025",
  },
  {
    stat: "59%",
    label: "Bare-metal validation",
    detail: "37signals cut cloud from $3.2M to ~$1.3M/yr on bare metal. The economics are proven — now automation makes it accessible.",
    source: "The Register, 2024",
  },
  {
    stat: "On-chain",
    label: "Tokenized assets need infrastructure now",
    detail: "Tokenized securities, real-world assets, and new on-chain instrument classes are going live. The teams building them need infrastructure that isn't rented from the same cloud vendors they're trying to disintermediate.",
    source: "SEC, EU MiCA — 2024–26",
  },
];

/* ─── Slide 5: Solution pipeline ─── */
const pipelineNodes = [
  {
    label: "You write a spec",
    sublabel: "Plain English",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
    ),
  },
  {
    label: "AI plans it",
    sublabel: "Breaks it into tasks",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    ),
  },
  {
    label: "22 AI workers build it",
    sublabel: "Specialized roles",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
      </svg>
    ),
  },
  {
    label: "Automated QA",
    sublabel: "Tests + code review",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
    ),
  },
  {
    label: "Ships + monitors",
    sublabel: "Auto-fixes issues",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M12 5l7 7-7 7" />
      </svg>
    ),
  },
];

/* ─── Slide 6: Platform services (from 5dlabs.ai/cto/services) ─── */
const platformServices = [
  { name: "5D Deploy", desc: "GitOps-driven delivery pipeline", replaces: "CI/CD SaaS" },
  { name: "5D Observe", desc: "Unified monitoring, logs, and traces", replaces: "Datadog, New Relic" },
  { name: "5D Vault", desc: "Secrets management and dynamic credentials", replaces: "AWS Secrets Manager, HashiCorp" },
  { name: "5D Data", desc: "Managed PostgreSQL with HA failover", replaces: "RDS, Cloud SQL, Aurora" },
  { name: "5D Inference", desc: "Managed model runtime on your GPUs", replaces: "SageMaker, Vertex AI" },
  { name: "5D Store", desc: "S3-compatible distributed object storage", replaces: "S3, GCS, Blob Storage" },
  { name: "5D Sentinel", desc: "Continuous security scanning + AI remediation", replaces: "Snyk, Wiz" },
  { name: "5D Edge", desc: "Ingress, TLS, and DNS automation", replaces: "CloudFront, Cloudflare" },
  { name: "5D Stream", desc: "Messaging and event streaming", replaces: "SQS, Pub/Sub, Kafka" },
  { name: "5D Node", desc: "Validator and RPC node operations", replaces: "Alchemy, QuickNode" },
  { name: "5D Mesh", desc: "Service mesh, mTLS, and network policy", replaces: "Istio, Linkerd" },
  { name: "5D Auth", desc: "Identity, SSO, and role-based access", replaces: "Auth0, Okta" },
  { name: "5D Registry", desc: "Private container and artifact registry", replaces: "ECR, GCR, Docker Hub" },
  { name: "5D Cache", desc: "Managed Redis / Valkey with clustering", replaces: "ElastiCache, Memorystore" },
  { name: "5D Queue", desc: "Background job and task processing", replaces: "SQS, Cloud Tasks" },
  { name: "5D Backup", desc: "Automated snapshots and disaster recovery", replaces: "AWS Backup" },
  { name: "5D Network", desc: "VPN, WireGuard, and private networking", replaces: "AWS VPC, Tailscale" },
  { name: "5D Search", desc: "Full-text search and vector indexing", replaces: "Elasticsearch, Algolia" },
  { name: "5D GPU", desc: "GPU scheduling and fractional sharing", replaces: "Lambda Cloud, CoreWeave" },
  { name: "5D Catalog", desc: "Service catalog and developer portal", replaces: "Backstage, Port" },
];

/* ─── Slide 8: Market rings ─── */
const marketRings: [
  { label: string; value: string; description: string },
  { label: string; value: string; description: string },
  { label: string; value: string; description: string },
] = [
  { label: "TAM", value: "$420B+", description: "Global cloud IaaS + PaaS" },
  { label: "SAM", value: "$40-80B", description: "AI-native dev teams + startups" },
  { label: "Beachhead", value: "$3-5B", description: "Teams replacing cloud with bare metal" },
];

/* ─── Slide 9: GTM funnel ─── */
const funnelStages = [
  { label: "Free tier", description: "Developers try a lightweight version of CTO at no cost." },
  { label: "Paid plans", description: "Teams upgrade for the full AI workforce and bare-metal infrastructure." },
  { label: "Recurring revenue", description: "Monthly subscriptions with margins that improve over time." },
];

const revenueStreams = [
  { name: "CTO subscriptions", type: "Recurring", desc: "Monthly plans from Free to Enterprise. CodeRun-based usage." },
  { name: "Bare-metal rev-share", type: "Recurring", desc: "Infrastructure margin on dedicated hardware we provision." },
  { name: "Proprietary trading", type: "Bootstrap", desc: "In-house capital engine. Funds experiments + provides market signal." },
  { name: "Advisory + implementation", type: "Services", desc: "Hands-on setup for early adopters and enterprise pilots." },
];

/* ─── Slide 10: Use of funds ─── */
const fundBars = [
  { label: "Engineering (2 hires, loaded)", value: "$360–420K", percent: 100, colorVar: "--chart-1" },
  { label: "Founder salary (loaded)", value: "$120–140K", percent: 34, colorVar: "--chart-1" },
  { label: "Legal / accounting / 409A", value: "$30–40K", percent: 9, colorVar: "--chart-4" },
  { label: "AI model + inference", value: "$30–50K", percent: 9, colorVar: "--chart-4" },
  { label: "GTM + sales", value: "$20–30K", percent: 6, colorVar: "--chart-4" },
  { label: "Infrastructure (servers)", value: "$30–50K", percent: 9, colorVar: "--chart-4" },
  { label: "Buffer / contingency", value: "$20–30K", percent: 6, colorVar: "--chart-4" },
];

/* ─── Slide divider helper ─── */
function SlideSection({
  id,
  children,
  className = "",
}: {
  id: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section
      id={id}
      className={`py-16 px-6 border-t border-border/30 first:border-t-0 ${className}`}
    >
      <div className="max-w-6xl mx-auto w-full">{children}</div>
    </section>
  );
}

function SlideLabel({ number, title }: { number: string; title: string }) {
  return (
    <div className="flex items-center gap-3 mb-6">
      <span className="text-xs font-mono text-muted-foreground tabular-nums">
        {number}
      </span>
      <span className="text-xs uppercase tracking-[0.2em] text-muted-foreground">
        {title}
      </span>
    </div>
  );
}

/* ═══════════════════════════════════════════════════════════════
   MAIN PAGE
   ═══════════════════════════════════════════════════════════════ */

export default function PitchPage() {
  return (
    <MotionConfig reducedMotion="user">
      <div className="relative min-h-screen overflow-hidden">
        <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
        <div className="fixed inset-0 circuit-bg z-[1]" />
        <div className="fixed inset-0 noise-overlay z-[3]" />

        {/* Deck is standalone — one exit back to the main site */}
        <header className="fixed top-4 left-0 right-0 z-50 flex justify-center px-4 print:hidden">
          <a
            href="https://5dlabs.ai/"
            className="premium-shell inline-flex items-center px-4 py-2 rounded-full backdrop-blur-xl text-sm font-bold text-cyan tracking-tight hover:text-cyan/90 transition-colors"
          >
            5D Labs
          </a>
        </header>

        <main className="relative z-10 pt-20">
          {/* ── SLIDE 1: COVER ── */}
          <section className="min-h-[70vh] flex flex-col items-center justify-center px-6">
            <motion.div
              className="text-center max-w-4xl mx-auto"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.8, ease: [0.25, 0.4, 0, 1] }}
            >
              <span className="inline-flex items-center gap-2 rounded-full border border-cyan/30 bg-cyan/10 px-4 py-1.5 mb-8">
                <span className="size-2 rounded-full bg-cyan animate-[glowPulse_3s_ease-in-out_infinite]" />
                <span className="text-sm text-cyan font-medium">
                  Pre-seed &middot; $750K &middot; Delaware C-Corp
                </span>
              </span>

              <h1 className="text-5xl sm:text-6xl md:text-7xl font-bold tracking-tight mb-6">
                <span className="gradient-text glow-text-cyan">Spec in.</span>{" "}
                <span className="text-foreground">Software out.</span>
              </h1>

              <p className="text-xl text-foreground/70 max-w-2xl mx-auto mb-10">
                You describe what you want built. Our AI builds, tests, and ships it
                &mdash; hyperscale performance, bare-metal economics. Targeting a $420B+ cloud
                market where ~29% of spend is wasted.
              </p>

              {/* Mini pipeline visual */}
              <motion.div
                className="flex items-center justify-center gap-0"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.6, duration: 0.8 }}
              >
                {["Spec", "Plan", "Build", "Test", "Ship"].map((step, i) => (
                  <div key={step} className="flex items-center">
                    <motion.div
                      className="flex flex-col items-center"
                      initial={{ opacity: 0, scale: 0.8 }}
                      animate={{ opacity: 1, scale: 1 }}
                      transition={{ delay: 0.8 + i * 0.12, duration: 0.4 }}
                    >
                      <div className={`w-10 h-10 sm:w-12 sm:h-12 rounded-full border-2 flex items-center justify-center text-xs sm:text-sm font-semibold ${
                        i === 0 ? "border-cyan/60 bg-cyan/15 text-cyan" :
                        i === 4 ? "border-cyan/60 bg-cyan/15 text-cyan" :
                        "border-border/50 bg-card/30 text-muted-foreground"
                      }`}>
                        {step}
                      </div>
                    </motion.div>
                    {i < 4 && (
                      <motion.div
                        className="w-6 sm:w-10 h-px bg-gradient-to-r from-cyan/40 to-cyan/10 mx-1"
                        initial={{ scaleX: 0 }}
                        animate={{ scaleX: 1 }}
                        transition={{ delay: 1 + i * 0.12, duration: 0.3 }}
                      />
                    )}
                  </div>
                ))}
              </motion.div>
            </motion.div>
          </section>

          {/* ── SLIDE 2: PROBLEM ── */}
          <SlideSection id="problem">
            <SlideLabel number="02" title="Problem" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              AI changes faster than teams can ship.
            </h2>
            <div className="grid sm:grid-cols-3 gap-4">
              {painPoints.map((p) => (
                <StatCard
                  key={p.label}
                  icon={p.icon}
                  value={p.value}
                  label={p.label}
                  note={p.note}
                />
              ))}
            </div>
          </SlideSection>

          {/* ── SLIDE 3: FOUNDER ── */}
          <SlideSection id="founder">
            <SlideLabel number="03" title="Founder" />
            <div className="grid lg:grid-cols-[1fr_1.3fr] gap-8 items-start">
              <div>
                <h2 className="text-3xl sm:text-4xl font-bold mb-3">
                  Jonathon Fritz
                </h2>
                <p className="text-lg text-foreground/70">
                  20 years building infrastructure at scale. Built this solo.
                </p>
              </div>

              <div className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
                <CareerTimeline milestones={founderTimeline} />
              </div>
            </div>
          </SlideSection>

          {/* ── SLIDE 4: WHY NOW ── */}
          <SlideSection id="why-now">
            <SlideLabel number="04" title="Why now" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              The old startup model is structurally disadvantaged.
            </h2>
            <p className="text-base text-foreground/70 mb-6 max-w-3xl">
              One bet, 18 months of runway, pivot when you learn too late. Agentic coding + owned infrastructure
              changes the math — run multiple ventures in parallel, fail fast, compound what works.
            </p>

            <div className="grid sm:grid-cols-2 gap-4">
              {convergenceFactors.map((f, i) => (
                <motion.div
                  key={f.label}
                  className="rounded-xl premium-shell p-5 backdrop-blur-sm"
                  initial={{ opacity: 0, y: 14 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-40px" }}
                  transition={{ duration: 0.5, delay: i * 0.1, ease: [0.25, 0.4, 0, 1] }}
                >
                  <p className="text-2xl font-bold gradient-text mb-1">{f.stat}</p>
                  <p className="text-sm font-semibold">{f.label}</p>
                  <p className="text-sm text-foreground/60 mt-1">{f.detail}</p>
                  <p className="text-xs text-muted-foreground/70 mt-2 font-mono">{f.source}</p>
                </motion.div>
              ))}
            </div>
          </SlideSection>

          {/* ── SLIDE 5: SOLUTION + HOW IT WORKS ── */}
          <SlideSection id="solution">
            <SlideLabel number="05" title="Solution" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              One system: spec in, deployed software out.
            </h2>

            <div className="rounded-xl border border-cyan/20 bg-cyan/5 p-6 backdrop-blur-sm mb-5">
              <PipelineFlow nodes={pipelineNodes} />
            </div>

            <div className="grid sm:grid-cols-3 gap-4">
              {[
                {
                  title: "Your servers",
                  desc: "Bare-metal servers we provision through hardware partners (Cherry, Hetzner, Latitude). You control the economics.",
                  icon: (
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
                    </svg>
                  ),
                  emphasis: true,
                },
                {
                  title: "Always current",
                  desc: "AI tools change constantly. We absorb the updates — no context-switching for your team.",
                  icon: (
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                  ),
                  emphasis: false,
                },
                {
                  title: "Self-healing",
                  desc: "Issues fixed before your team wakes up.",
                  icon: (
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                    </svg>
                  ),
                  emphasis: false,
                },
              ].map((card, i) => (
                <motion.div
                  key={card.title}
                  className={`rounded-xl premium-shell p-5 backdrop-blur-sm ${card.emphasis ? "border border-cyan/30 shadow-[0_0_24px_-6px_rgba(34,211,238,0.15)]" : ""}`}
                  initial={{ opacity: 0, y: 14 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-40px" }}
                  transition={{ duration: 0.5, delay: i * 0.1, ease: [0.25, 0.4, 0, 1] }}
                >
                  <div className="w-9 h-9 rounded-lg bg-cyan/10 border border-cyan/20 flex items-center justify-center text-cyan mb-3">
                    {card.icon}
                  </div>
                  <p className="text-base font-semibold mb-1">{card.title}</p>
                  <p className="text-sm text-foreground/80">{card.desc}</p>
                </motion.div>
              ))}
            </div>
          </SlideSection>

          {/* ── SLIDE 6: PRODUCT — AGENTIC PRIVATE CLOUD ── */}
          <SlideSection id="product">
            <SlideLabel number="06" title="Product" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              Agentic private cloud — 20+ services on your hardware.
            </h2>
            <p className="text-base text-foreground/70 mb-6 max-w-3xl">
              Everything a team needs to build, ship, and run software — without renting it from AWS.
              Same capabilities. Your economics.
            </p>

            <div className="grid sm:grid-cols-2 lg:grid-cols-5 gap-3">
              {platformServices.map((svc, i) => (
                <motion.div
                  key={svc.name}
                  className="rounded-xl border border-border/50 bg-card/30 p-4 backdrop-blur-sm"
                  initial={{ opacity: 0, y: 10 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-40px" }}
                  transition={{ duration: 0.4, delay: i * 0.03, ease: [0.25, 0.4, 0, 1] }}
                >
                  <p className="text-sm font-bold gradient-text">{svc.name}</p>
                  <p className="text-sm text-foreground/60 mt-1">{svc.desc}</p>
                  <p className="text-xs text-muted-foreground mt-1.5 font-mono">
                    Replaces {svc.replaces}
                  </p>
                </motion.div>
              ))}
            </div>
            <p className="text-xs text-muted-foreground/70 mt-4 font-mono">
              20 managed services across compute, data, security, networking, AI, and blockchain.
            </p>
          </SlideSection>

          {/* ── SLIDE 7: TRACTION ── */}
          <SlideSection id="traction">
            <SlideLabel number="07" title="Traction" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              Revenue before fundraising.
            </h2>

            <div className="grid grid-cols-2 lg:grid-cols-5 gap-4">
              {[
                { value: "1", label: "Paying customer", note: "Sigma One — live in production" },
                { value: "$240K", label: "Pipeline", note: "Annual contract value in discussions" },
                { value: "17+", label: "Server deployments", note: "Across multiple regions" },
                { value: "22", label: "AI workers", note: "Intake, code gen, testing, deploy, security, self-healing — shipping 24/7" },
                { value: "$0", label: "Outside capital", note: "Self-funded via proprietary trading — zero dilution" },
              ].map((m, i) => (
                <motion.article
                  key={m.label}
                  className={`rounded-xl border border-border/50 bg-card/30 p-5 backdrop-blur-sm text-center ${
                    i === 4 ? "col-span-2 lg:col-span-1" : ""
                  }`}
                  initial={{ opacity: 0, y: 14 }}
                  whileInView={{ opacity: 1, y: 0 }}
                  viewport={{ once: true, margin: "-40px" }}
                  transition={{ duration: 0.5, ease: [0.25, 0.4, 0, 1] }}
                >
                  <AnimatedCounter
                    value={m.value}
                    className="text-3xl font-bold gradient-text block"
                  />
                  <p className="text-sm font-semibold mt-2">{m.label}</p>
                  <p className="text-sm text-foreground/60 mt-1">{m.note}</p>
                </motion.article>
              ))}
            </div>
          </SlideSection>

          {/* ── SLIDE 8: MARKET ── */}
          <SlideSection id="market">
            <SlideLabel number="08" title="Market" />
            <div className="grid lg:grid-cols-[1fr_1fr] gap-8 items-center">
              <div>
                <h2 className="text-3xl sm:text-4xl font-bold mb-3">
                  $420B+ spent on cloud every year.
                </h2>
                <p className="text-base text-foreground/70">
                  We start where teams already feel the pain.
                </p>
              </div>

              <div className="flex justify-center">
                <ConcentricRings
                  rings={marketRings}
                  source="Gartner public-cloud forecast, Nov 2024."
                />
              </div>
            </div>
          </SlideSection>

          {/* ── SLIDE 9: COMPETITIVE LANDSCAPE ── */}
          <SlideSection id="competition">
            <SlideLabel number="09" title="Competition" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-3">
              No one else combines both.
            </h2>
            <p className="text-base text-foreground/70 mb-6 max-w-3xl">
              AI code tools assume cloud. Infrastructure tools assume human engineers. We’re the only player combining AI-native development with bare-metal economics.
            </p>

            <div className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <div className="grid grid-cols-2 gap-px bg-border/20 rounded-lg overflow-hidden">
                <div className="bg-card/40 p-5">
                  <p className="text-xs font-mono text-muted-foreground uppercase tracking-wider mb-3">Cloud-managed + Human-built</p>
                  <div className="space-y-1.5">
                    {["AWS / GCP / Azure", "Heroku / Render", "Vercel / Netlify"].map((name) => (
                      <p key={name} className="text-sm text-foreground/60">{name}</p>
                    ))}
                  </div>
                </div>
                <div className="bg-card/40 p-5">
                  <p className="text-xs font-mono text-muted-foreground uppercase tracking-wider mb-3">Cloud-managed + AI-built</p>
                  <div className="space-y-1.5">
                    {["Replit / Bolt", "GitHub Copilot Workspace", "Vercel v0"].map((name) => (
                      <p key={name} className="text-sm text-foreground/60">{name}</p>
                    ))}
                  </div>
                </div>
                <div className="bg-card/40 p-5">
                  <p className="text-xs font-mono text-muted-foreground uppercase tracking-wider mb-3">Bare metal + Human-built</p>
                  <div className="space-y-1.5">
                    {["Coolify / CapRover", "Hetzner + Terraform", "Oxide Computer"].map((name) => (
                      <p key={name} className="text-sm text-foreground/60">{name}</p>
                    ))}
                  </div>
                </div>
                <motion.div
                  className="relative p-5 border-2 border-cyan/40 bg-cyan/5"
                  initial={{ opacity: 0, scale: 0.95 }}
                  whileInView={{ opacity: 1, scale: 1 }}
                  viewport={{ once: true, margin: "-40px" }}
                  transition={{ duration: 0.6, ease: [0.25, 0.4, 0, 1] }}
                >
                  <p className="text-[10px] font-mono text-cyan uppercase tracking-wider mb-3">Bare metal + AI-built</p>
                  <p className="text-lg font-bold gradient-text">5D Labs</p>
                  <p className="text-sm text-foreground/60 mt-1">Spec in, software out — on your hardware.</p>
                </motion.div>
              </div>
            </div>
          </SlideSection>

          {/* ── SLIDE 10: BUSINESS MODEL + GTM ── */}
          <SlideSection id="business-gtm">
            <SlideLabel number="10" title="Business + GTM" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              Four revenue streams. One stack.
            </h2>

            <div className="grid lg:grid-cols-2 gap-8">
              <div className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
                <p className="text-sm font-semibold mb-4 text-foreground/60 uppercase tracking-wider">
                  How customers find us
                </p>
                <FunnelDiagram stages={funnelStages} />
              </div>

              <div className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
                <p className="text-sm font-semibold mb-4 text-foreground/60 uppercase tracking-wider">
                  Revenue streams
                </p>
                <div className="grid sm:grid-cols-2 gap-3">
                  {revenueStreams.map((s, i) => (
                    <motion.div
                      key={s.name}
                      className="rounded-lg border border-border/40 bg-background/40 p-4"
                      initial={{ opacity: 0, y: 10 }}
                      whileInView={{ opacity: 1, y: 0 }}
                      viewport={{ once: true, margin: "-40px" }}
                      transition={{ duration: 0.4, delay: i * 0.08, ease: [0.25, 0.4, 0, 1] }}
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <span className="text-sm font-semibold">{s.name}</span>
                        <span className="text-[10px] font-mono text-cyan bg-cyan/10 px-1.5 py-0.5 rounded">
                          {s.type}
                        </span>
                      </div>
                      <p className="text-sm text-foreground/60">{s.desc}</p>
                    </motion.div>
                  ))}
                </div>
              </div>
            </div>
          </SlideSection>

          {/* ── SLIDE 11: USE OF FUNDS ── */}
          <SlideSection id="funds">
            <SlideLabel number="11" title="Use of funds" />
            <h2 className="text-3xl sm:text-4xl font-bold mb-6">
              $750K. Two engineers. 18 months.
            </h2>

            <div className="rounded-xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <AnimatedBarChart
                items={fundBars}
                heading="Scaling, not discovery."
                subheading="1 senior backend/infra + 1 full-stack with AI systems experience. All costs loaded (benefits, payroll tax)."
              />
            </div>
          </SlideSection>

          {/* ── SLIDE 11: THE ASK ── */}
          <section className="min-h-[55vh] flex items-center justify-center px-6 border-t border-border/30">
            <motion.div
              className="text-center max-w-3xl mx-auto"
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.8 }}
            >
              <SlideLabel number="12" title="The ask" />

              <p className="text-6xl sm:text-7xl font-bold gradient-text mb-6">
                $750K
              </p>
              <p className="text-2xl font-semibold mb-2">
                Product live. Customer paying. Pipeline in hand.
              </p>
              <p className="text-lg text-foreground/70 mb-4">
                Post-money SAFE. Cap aligned to AI infrastructure comps.
              </p>
              <p className="text-base text-foreground/60 mb-8 font-mono">
                3–5 customers at $5–8K/mo MRR = breakeven at month 15–18.
              </p>

              <InvestorCtaButtons />

              <p className="text-base text-foreground/60 mt-8">
                Live demo available in any meeting.
              </p>
            </motion.div>
          </section>

          {/* Minimal footer — no site links */}
          <div className="py-8 px-6 text-center print:hidden">
            <p className="text-xs text-muted-foreground/60">
              &copy; {new Date().getFullYear()} 5D Labs Inc. &middot; Confidential
            </p>
          </div>
        </main>

        <DeckToolbar />
      </div>
    </MotionConfig>
  );
}
