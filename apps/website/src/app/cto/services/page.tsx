import { Header } from "@/components/cto/header";
import Image from "next/image";
import Link from "next/link";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3000"
    : "https://5dlabs.ai";

type ServiceCategory = {
  title: string;
  description: string;
  color: string;
  border: string;
  bg: string;
  services: {
    name: string;
    tagline: string;
    description: string;
    poweredBy: string;
  }[];
};

const categories: ServiceCategory[] = [
  {
    title: "Planning & Design",
    description:
      "From PRD to production-ready designs — before a single line of code is written. Structured deliberation and automated design generation so your team starts building from a vetted plan and approved visuals.",
    color: "text-violet-400",
    border: "border-violet-400/20",
    bg: "bg-violet-400/5",
    services: [
      {
        name: "5D Plan",
        tagline: "PRD intake and structured deliberation",
        description:
          "PRDs become structured plans through deliberation — optimist and pessimist agents challenge each decision point before committing, the same way a real team would debate scope, risk, and tradeoffs. The result is a plan that has already survived scrutiny.",
        poweredBy: "Lobster workflow engine + deliberation agents",
      },
      {
        name: "5D Design",
        tagline: "AI-generated design variants before code",
        description:
          "Before a single line of code is written, the platform generates production-ready UI options powered by Google Stitch. Multiple design directions — layout, color, typography — are presented so you choose the look before the team starts building.",
        poweredBy: "Google Stitch API",
      },
    ],
  },
  {
    title: "Development",
    description:
      "Multi-CLI agent harness with token-aware routing across 60+ model providers. One consistent development experience regardless of which tool or model your team prefers.",
    color: "text-cyan",
    border: "border-cyan/20",
    bg: "bg-cyan/5",
    services: [
      {
        name: "5D Code",
        tagline: "Multi-CLI harness with intelligent routing",
        description:
          "The platform's ACP harness manages every supported CLI — Claude Code, Cursor, Codex, Factory, Gemini CLI, OpenCode, GitHub Copilot, Kimi CLI — and makes intelligent routing decisions based on available token usage across providers. One consistent experience, regardless of which tool or model your team prefers.",
        poweredBy: "ACP harness + OpenRouter + direct provider integrations",
      },
    ],
  },
  {
    title: "Security",
    description:
      "Continuous vulnerability scanning, dependency analysis, and AI-native remediation running across every service — integrated into the same agent pipeline as everything else.",
    color: "text-rose-400",
    border: "border-rose-400/20",
    bg: "bg-rose-400/5",
    services: [
      {
        name: "5D Sentinel",
        tagline: "Continuous security scanning and AI remediation",
        description:
          "Continuous vulnerability scanning, dependency analysis, and AI-native remediation running across every service. Cipher doesn't just flag issues — it ships the fix through the same agent pipeline as everything else.",
        poweredBy: "Snyk + Nuclei + Aikido + Semgrep + CodeQL",
      },
    ],
  },
  {
    title: "Data & Storage",
    description:
      "Managed databases, object storage, and high-performance block volumes — operated for you so your team never has to think about storage engineering.",
    color: "text-cyan",
    border: "border-cyan/20",
    bg: "bg-cyan/5",
    services: [
      {
        name: "5D Data",
        tagline: "Managed PostgreSQL",
        description:
          "Production-grade PostgreSQL clusters with automated backups, point-in-time recovery, connection pooling, and high-availability failover. Zero manual DBA work.",
        poweredBy: "CloudNativePG operator",
      },
      {
        name: "5D Cache",
        tagline: "High-performance in-memory data layer",
        description:
          "Redis-compatible caching and pub/sub infrastructure with sub-millisecond latency. Ideal for session state, rate limiting, leaderboards, and real-time pipelines.",
        poweredBy: "Redis operator (Valkey)",
      },
      {
        name: "5D Store",
        tagline: "S3-compatible object storage",
        description:
          "Fast distributed object storage for assets, artifacts, model weights, backups, and durable application data. S3-compatible API — no vendor lock-in.",
        poweredBy: "SeaweedFS operator",
      },
      {
        name: "5D Volume",
        tagline: "NVMe-backed block volumes",
        description:
          "High-performance persistent block storage with synchronous replication. Built for databases, message queues, and stateful workloads that need speed and durability on bare metal.",
        poweredBy: "Mayastor (OpenEBS)",
      },
    ],
  },
  {
    title: "AI & Inference",
    description:
      "Managed model runtimes across hosted providers and dedicated GPU infrastructure — with a consistent API surface regardless of where the model runs.",
    color: "text-[oklch(0.7_0.25_320)]",
    border: "border-[oklch(0.7_0.25_320)]/20",
    bg: "bg-[oklch(0.7_0.25_320)]/5",
    services: [
      {
        name: "5D Inference",
        tagline: "Managed model runtime",
        description:
          "Run open-weight models on dedicated GPU infrastructure or route to hosted providers (OpenAI, Anthropic, Google) behind a single OpenAI-compatible API. Scale from zero. Hot-swap models without code changes.",
        poweredBy: "KubeAI operator (vLLM, Ollama, FasterWhisper) + NVIDIA GPU operator",
      },
      {
        name: "5D LlamaStack",
        tagline: "Meta LlamaStack inference and agents",
        description:
          "Deploy and manage Meta's LlamaStack distributions for agentic inference workflows. Purpose-built for teams building on Llama models with structured tool use and memory.",
        poweredBy: "LlamaStack Kubernetes operator",
      },
    ],
  },
  {
    title: "Messaging & Events",
    description:
      "High-throughput, durable messaging for agent-to-agent communication, event-driven services, and real-time workloads.",
    color: "text-orange-400",
    border: "border-orange-400/20",
    bg: "bg-orange-400/5",
    services: [
      {
        name: "5D Stream",
        tagline: "Cloud-native messaging and event streaming",
        description:
          "High-performance publish/subscribe, request-reply, and persistent JetStream messaging. The connective tissue between agents, services, and systems — with at-least-once and exactly-once delivery.",
        poweredBy: "NATS with JetStream",
      },
    ],
  },
  {
    title: "Secrets & Identity",
    description:
      "Secrets management, dynamic credentials, and automatic synchronization — hardened by default, fully managed, nothing to configure.",
    color: "text-green-400",
    border: "border-green-400/20",
    bg: "bg-green-400/5",
    services: [
      {
        name: "5D Vault",
        tagline: "Secrets management and dynamic credentials",
        description:
          "API keys, credentials, and environment secrets fully managed behind a secure, audited control layer. Dynamic secret generation, automatic rotation, and lease management included. Kubernetes-native sync keeps secrets fresh without manual intervention.",
        poweredBy: "OpenBao (open-source Vault) + External Secrets Operator",
      },
    ],
  },
  {
    title: "Source Control",
    description:
      "Self-hosted Git hosting with full CI/CD, issues, and merge requests — no vendor lock-in, no per-seat pricing. Choose the stack that fits your workflow.",
    color: "text-purple-400",
    border: "border-purple-400/20",
    bg: "bg-purple-400/5",
    services: [
      {
        name: "5D Git",
        tagline: "Self-hosted GitLab or Gitea",
        description:
          "Enterprise-grade Git hosting on your infrastructure. Run GitLab or Gitea as your default — full CI/CD, issues, merge requests, and repository management. All features included. Integrates seamlessly with CTO agents and 5D Deploy.",
        poweredBy: "GitLab Helm chart / Gitea Helm chart",
      },
    ],
  },
  {
    title: "Delivery & Observability",
    description:
      "GitOps-driven release pipelines, unified monitoring, self-healing operations, and automated remediation — so the platform stays healthy and delivery stays fast.",
    color: "text-blue-400",
    border: "border-blue-400/20",
    bg: "bg-blue-400/5",
    services: [
      {
        name: "5D Deploy",
        tagline: "GitOps-driven delivery pipeline",
        description:
          "Every change moves through a tracked, automated release flow — from PR merge to production deployment. Automated rollbacks, health checks, and full auditability. Agents can ship without touching deployment tooling.",
        poweredBy: "ArgoCD + ArgoCD Image Updater",
      },
      {
        name: "5D Observe",
        tagline: "Unified monitoring, logs, and traces",
        description:
          "Metrics, logs, distributed traces, and incident signals surfaced in one place. Pre-wired dashboards for every platform service. OpenTelemetry-native — everything is included and ready to go.",
        poweredBy: "Prometheus + Grafana + Loki + Fluent Bit + Jaeger + OpenTelemetry Collector",
      },
      {
        name: "5D Pulse",
        tagline: "Self-healing and automated remediation",
        description:
          "The platform monitors its own vitals and fixes what breaks — before it becomes an incident. Automated detection, remediation, and restart logic keep everything running without turning your team into a 24/7 ops desk.",
        poweredBy: "Healer agent + health check controllers + auto-rollback",
      },
    ],
  },
  {
    title: "Networking & Connectivity",
    description:
      "eBPF-powered service mesh, zero-trust access, TLS automation, and DNS management — networking that just works across bare metal and cloud.",
    color: "text-yellow-400",
    border: "border-yellow-400/20",
    bg: "bg-yellow-400/5",
    services: [
      {
        name: "5D Mesh",
        tagline: "eBPF networking and zero-trust access",
        description:
          "High-performance eBPF-based networking with network policy enforcement and cluster connectivity. Zero-trust private access for agents and services — no VPN required for internal tooling.",
        poweredBy: "Cilium + Twingate operator + Headscale/Tailscale",
      },
      {
        name: "5D Edge",
        tagline: "Ingress, TLS, and DNS automation",
        description:
          "Managed ingress routing with automatic TLS certificate provisioning and renewal. External DNS automation keeps records in sync as services move. Custom domain support out of the box.",
        poweredBy: "ingress-nginx + cert-manager + external-dns",
      },
    ],
  },
  {
    title: "Blockchain Infrastructure",
    description:
      "Managed node operations and on-chain data infrastructure for teams building in Web3 — across L1s, L2s, and interoperability protocols, on dedicated hardware.",
    color: "text-[oklch(0.7_0.25_320)]",
    border: "border-[oklch(0.7_0.25_320)]/20",
    bg: "bg-[oklch(0.7_0.25_320)]/5",
    services: [
      {
        name: "5D Node",
        tagline: "Validator and RPC node operations",
        description:
          "Managed node deployment across Solana, Sui, Aptos, NEAR, Base, Ethereum (Reth), Berachain, Monad, Arbitrum, Optimism, and LayerZero. Managed upgrades, health monitoring, and failover on dedicated hardware.",
        poweredBy: "CTO Blockchain Operator (Rust) + Kotal (5dlabs fork)",
      },
      {
        name: "5D Index",
        tagline: "On-chain data indexing and explorer infrastructure",
        description:
          "Real-time indexing of on-chain events, account states, and transaction history. Includes BlockScout explorer deployments and Cloudflare R2-backed storage for archive data — so your application always has reliable, low-latency chain data.",
        poweredBy: "CTO Blockchain Operator — indexing and explorer CRDs (in development)",
      },
    ],
  },
];

export default function ServicesPage() {
  return (
    <div className="relative min-h-screen overflow-hidden">
      <div className="fixed inset-0 bg-gradient-to-b from-background via-background to-[oklch(0.04_0.02_260)] z-0" />
      <div className="fixed inset-0 circuit-bg z-[1]" />
      <div className="fixed inset-0 noise-overlay z-[3]" />

      <Header />

      <main className="relative z-10 pt-24">
        {/* Hero */}
        <section className="py-20 px-6">
          <div className="max-w-4xl mx-auto text-center">
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan/20 bg-cyan/5 mb-6">
              <span className="text-xs text-cyan font-medium uppercase tracking-wider">
                Platform Services
              </span>
            </div>
            <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight mb-6">
              Everything Your Stack{" "}
              <span className="gradient-text">Needs to Run</span>
            </h1>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-8">
              CTO manages databases, storage, inference, secrets, source control, delivery, networking, and
              blockchain infrastructure as productized platform services — all branded,
              all operated for you.
            </p>
            <div className="max-w-2xl mx-auto p-5 rounded-xl border border-cyan/20 bg-cyan/5 text-center">
              <p className="text-base text-foreground font-medium mb-2">
                Every managed service your team relies on from AWS, GCP, or Azure — databases,
                storage, secrets, inference, observability, networking — already runs here.
              </p>
              <p className="text-sm text-muted-foreground">
                Same capabilities. Predictable costs. No egress fees, no surprise invoices,
                no vendor lock-in.
              </p>
            </div>

            <div className="mt-12 rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <p className="mb-4 text-sm font-medium uppercase tracking-widest text-muted-foreground">
                Supported AI CLIs
              </p>
              <div className="flex flex-wrap justify-center gap-3">
                {["Claude Code", "Cursor", "Codex", "Factory", "Gemini CLI", "OpenCode", "GitHub Copilot", "Kimi CLI"].map((tool) => (
                  <span
                    key={tool}
                    className="inline-flex items-center rounded-full border border-border/60 bg-background/60 px-4 py-2 text-sm font-medium text-foreground"
                  >
                    {tool}
                  </span>
                ))}
              </div>
            </div>

            <div className="mt-6 rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
              <p className="mb-4 text-sm font-medium uppercase tracking-widest text-muted-foreground">
                Model Providers (Token-Aware Routing)
              </p>
              <p className="mb-4 text-sm text-muted-foreground text-center">
                Token-aware routing across 60+ providers based on available usage — no single vendor lock-in.
              </p>
              <div className="flex flex-wrap justify-center gap-3">
                {["Anthropic", "OpenAI", "Google (Vertex)", "Azure Foundry", "OpenRouter", "xAI", "DeepSeek", "MiniMax", "DeepInfra", "NovitaAI", "NVIDIA"].map((provider) => (
                  <span
                    key={provider}
                    className="inline-flex items-center rounded-full border border-border/60 bg-background/60 px-4 py-2 text-sm font-medium text-foreground"
                  >
                    {provider}
                  </span>
                ))}
              </div>
              <p className="mt-4 text-xs text-muted-foreground text-center">
                Plus 50+ more via OpenRouter and direct integrations
              </p>
            </div>
          </div>
        </section>

        {/* Service Categories */}
        {categories.map((cat) => (
          <section
            key={cat.title}
            className="py-16 px-6 border-t border-border/30"
          >
            <div className="max-w-6xl mx-auto">
              <div className="mb-10">
                <h2 className={`text-2xl font-bold mb-2 ${cat.color}`}>{cat.title}</h2>
                <p className="text-muted-foreground max-w-2xl">{cat.description}</p>
              </div>

              <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-5">
                {cat.services.map((svc) => (
                  <div
                    key={svc.name}
                    className={`p-5 rounded-xl border ${cat.border} ${cat.bg} backdrop-blur-sm`}
                  >
                    <p className={`text-xs font-medium uppercase tracking-widest mb-2 ${cat.color}`}>
                      {svc.name}
                    </p>
                    <h3 className="text-base font-semibold mb-2 text-foreground">
                      {svc.tagline}
                    </h3>
                    <p className="text-sm text-muted-foreground mb-3">
                      {svc.description}
                    </p>
                    <p className="text-[11px] text-muted-foreground/40 italic">
                      {svc.poweredBy}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </section>
        ))}

        {/* Integrations */}
        <section className="py-16 px-6 border-t border-border/30">
          <div className="max-w-6xl mx-auto">
            <div className="mb-10">
              <h2 className="text-2xl font-bold mb-2 text-foreground">Integrations</h2>
              <p className="text-muted-foreground max-w-2xl">
                Native integrations with the tools your team already uses — from project management to alerting to observability.
              </p>
            </div>

            <div className="grid md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-5">
              <div className="p-5 rounded-xl border border-[oklch(0.7_0.25_320)]/20 bg-[oklch(0.7_0.25_320)]/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest mb-2 text-[oklch(0.7_0.25_320)]">Project Management</p>
                <p className="text-sm text-muted-foreground mb-3">Linear is primary — full agent activity sync, PRD intake, and live task updates. Other platforms get task creation and status updates.</p>
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

              <div className="p-5 rounded-xl border border-cyan/20 bg-cyan/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest mb-2 text-cyan">Communication & Alerting</p>
                <p className="text-sm text-muted-foreground mb-3">Agents post progress updates, incident alerts, and deployment notifications to your channels in real time.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["Discord", "Slack", "Microsoft Teams", "PagerDuty", "Email"].map(name => (
                    <span key={name} className="text-[11px] px-2 py-0.5 rounded-md font-medium bg-cyan/10 text-cyan">{name}</span>
                  ))}
                </div>
              </div>

              <div className="p-5 rounded-xl border border-orange-400/20 bg-orange-400/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest mb-2 text-orange-400">Observability</p>
                <p className="text-sm text-muted-foreground mb-3">Self-hosted Grafana, Prometheus, and Loki pre-wired. Datadog supported for teams already invested in it.</p>
                <div className="flex flex-wrap gap-1.5">
                  {[
                    { name: "Grafana", primary: true },
                    { name: "Prometheus", primary: true },
                    { name: "Loki", primary: true },
                    { name: "Jaeger", primary: true },
                    { name: "OpenTelemetry", primary: true },
                    { name: "Datadog", primary: false },
                  ].map(({ name, primary }) => (
                    <span key={name} className={`text-[11px] px-2 py-0.5 rounded-md font-medium ${primary ? "bg-orange-400/10 text-orange-400" : "bg-muted/50 text-muted-foreground"}`}>{name}</span>
                  ))}
                </div>
              </div>

              <div className="p-5 rounded-xl border border-green-400/20 bg-green-400/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest mb-2 text-green-400">Source Control & CI</p>
                <p className="text-sm text-muted-foreground mb-3">Each agent integrates with your Git host. PRs, reviews, and deployments are fully automated.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["Git Apps", "CI/CD", "ArgoCD", "Webhooks", "PR Automation"].map(name => (
                    <span key={name} className="text-[11px] px-2 py-0.5 rounded-md font-medium bg-green-400/10 text-green-400">{name}</span>
                  ))}
                </div>
              </div>

              <div className="p-5 rounded-xl border border-rose-400/20 bg-rose-400/5 backdrop-blur-sm">
                <p className="text-xs font-medium uppercase tracking-widest mb-2 text-rose-400">Security Scanning</p>
                <p className="text-sm text-muted-foreground mb-3">Vulnerability scanning, SCA, AI-native remediation, and supply-chain protection.</p>
                <div className="flex flex-wrap gap-1.5">
                  {["Snyk", "Nuclei", "Aikido", "Socket", "Trivy", "Gitleaks", "Datadog", "Dynatrace"].map(name => (
                    <span key={name} className="text-[11px] px-2 py-0.5 rounded-md font-medium bg-rose-400/10 text-rose-400">{name}</span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* CTA */}
        <section className="py-20 px-6 border-t border-border/30">
          <div className="max-w-2xl mx-auto text-center">
            <h2 className="text-3xl font-bold mb-4">
              The Platform is <span className="gradient-text">Ready</span>
            </h2>
            <p className="text-lg text-muted-foreground mb-8">
              Every service above runs as part of the CTO platform. Your team ships product — we run the stack.
            </p>
            <Link
              href="/cto"
              className="inline-block px-8 py-4 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-semibold hover:opacity-90 transition-all"
            >
              Back to CTO
            </Link>
          </div>
        </section>

        {/* Footer */}
        <footer className="border-t border-border/30 py-8 px-6">
          <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
            <a href={homeHref} className="inline-flex items-center">
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
