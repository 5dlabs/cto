import { Header } from "@/components/header";
import Image from "next/image";
import Link from "next/link";

const homeHref =
  process.env.NODE_ENV === "development"
    ? "http://localhost:3001"
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
    title: "Delivery & Observability",
    description:
      "GitOps-driven release pipelines, unified monitoring, and self-healing operations — so the platform stays healthy and delivery stays fast.",
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
              CTO manages databases, storage, inference, secrets, delivery, networking, and
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
              href="/"
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
              © {new Date().getFullYear()} 5D Labs. From PRD to Production — Autonomously.
            </p>
          </div>
        </footer>
      </main>
    </div>
  );
}
