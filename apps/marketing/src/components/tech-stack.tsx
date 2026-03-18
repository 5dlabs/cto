const cliTools = [
  "Claude Code",
  "Cursor",
  "Codex",
  "Factory",
  "Gemini CLI",
  "OpenCode",
  "GitHub Copilot",
  "Kimi CLI",
];

const services = [
  {
    name: "5D Deploy",
    tagline: "Git-to-production delivery",
    description:
      "Deployments, rollbacks, and release coordination handled for you so the team can stay focused on shipping.",
    highlights: ["Git-driven", "Automated rollbacks", "Managed environments"],
    color: "cyan",
  },
  {
    name: "5D Observe",
    tagline: "Unified monitoring and alerts",
    description:
      "Metrics, logs, traces, and incident signals surfaced in one place without asking teams to assemble an observability stack first.",
    highlights: ["Metrics", "Logs", "Alerts"],
    color: "orange",
  },
  {
    name: "5D Vault",
    tagline: "Secrets and access control",
    description:
      "API keys, credentials, and environment secrets fully managed behind a secure layer. Everything is included — nothing to configure.",
    highlights: ["Fully managed", "Access controls", "Secret rotation"],
    color: "green",
  },
  {
    name: "5D Store",
    tagline: "Object and file storage",
    description:
      "A managed storage layer for assets, artifacts, backups, and durable application data across the platform.",
    highlights: ["Object storage", "Backups", "Artifacts"],
    color: "blue",
  },
  {
    name: "5D Volume",
    tagline: "Replicated high-speed volumes",
    description:
      "Persistent NVMe-backed volumes designed for databases and stateful workloads that need performance without manual storage engineering.",
    highlights: ["Replicated", "NVMe-backed", "Stateful workloads"],
    color: "purple",
  },
  {
    name: "5D Git",
    tagline: "Self-hosted GitLab or Gitea",
    description:
      "Enterprise Git hosting on your infrastructure — GitLab or Gitea, your choice. Full CI/CD, issues, and MRs without vendor lock-in.",
    highlights: ["GitLab or Gitea", "Self-hosted", "No per-seat pricing"],
    color: "purple",
  },
  {
    name: "5D Inference",
    tagline: "Managed model runtime",
    description:
      "Use hosted providers or self-hosted inference behind a consistent interface while the platform handles the operational complexity.",
    highlights: ["Hosted or self-hosted", "Consistent runtime", "GPU-ready"],
    color: "yellow",
  },
];

const colorMap: Record<string, { bg: string; text: string; border: string }> = {
  cyan: { bg: "bg-cyan/10", text: "text-cyan", border: "border-cyan/20" },
  orange: { bg: "bg-orange-500/10", text: "text-orange-400", border: "border-orange-500/20" },
  purple: { bg: "bg-purple-500/10", text: "text-purple-400", border: "border-purple-500/20" },
  green: { bg: "bg-green-500/10", text: "text-green-400", border: "border-green-500/20" },
  blue: { bg: "bg-blue-500/10", text: "text-blue-400", border: "border-blue-500/20" },
  yellow: { bg: "bg-yellow-500/10", text: "text-yellow-400", border: "border-yellow-500/20" },
};

export function TechStack() {
  return (
    <section id="stack" className="py-24 border-t border-border/30 overflow-hidden">
      <div className="max-w-6xl mx-auto px-6">
        <div className="text-center mb-12">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan/20 bg-cyan/5 mb-6">
            <span className="text-xs text-cyan font-medium uppercase tracking-wider">
              Managed Foundation
            </span>
          </div>
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            The Hard Parts Are{" "}
            <span className="gradient-text">Already Handled</span>
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Pick any CLI — CTO runs the
            delivery, storage, observability, secrets, and inference layer
            underneath it.
          </p>
        </div>

        <div className="mb-14 rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
          <p className="mb-4 text-sm font-medium uppercase tracking-widest text-muted-foreground">
            Supported AI CLIs
          </p>
          <div className="flex flex-wrap justify-center gap-3">
            {cliTools.map((tool) => (
              <span
                key={tool}
                className="inline-flex items-center rounded-full border border-border/60 bg-background/60 px-4 py-2 text-sm font-medium text-foreground"
              >
                {tool}
              </span>
            ))}
          </div>
          <p className="mt-4 text-sm text-muted-foreground text-center">
            Use your preferred interface. We abstract the underlying infrastructure so your team does not have to think about it day to day.
          </p>
        </div>

        <div className="mb-14 rounded-2xl border border-border/50 bg-card/30 p-6 backdrop-blur-sm">
          <p className="mb-4 text-sm font-medium uppercase tracking-widest text-muted-foreground">
            Model Providers (Token-Aware Routing)
          </p>
          <p className="mb-4 text-sm text-muted-foreground text-center">
            OpenClaw&apos;s ACP harness routes intelligently across 60+ providers based on available token usage — no single vendor lock-in.
          </p>
          <div className="flex flex-wrap justify-center gap-3">
            {[
              "Anthropic",
              "OpenAI",
              "Google (Vertex)",
              "AWS Bedrock",
              "Azure Foundry",
              "OpenRouter",
              "xAI",
              "DeepSeek",
              "MiniMax",
              "DeepInfra",
              "NovitaAI",
              "NVIDIA",
            ].map((provider) => (
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

        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-5">
          {services.map((service) => {
            const colors = colorMap[service.color] ?? colorMap.cyan;
            return (
              <div
                key={service.name}
                className={`p-5 rounded-xl border ${colors.border} bg-card/30 backdrop-blur-sm`}
              >
                <p className={`text-xs font-medium uppercase tracking-widest mb-2 ${colors.text}`}>
                  {service.name}
                </p>
                <h3 className="text-lg font-semibold mb-1">{service.tagline}</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  {service.description}
                </p>
                <div className="flex flex-wrap gap-1.5">
                  {service.highlights.map((item) => (
                    <span
                      key={item}
                      className={`text-[11px] px-2 py-0.5 rounded-md ${colors.bg} ${colors.text} font-medium`}
                    >
                      {item}
                    </span>
                  ))}
                </div>
              </div>
            );
          })}
        </div>

        <div className="mt-10 text-center">
          <p className="text-sm text-muted-foreground max-w-3xl mx-auto">
            Behind the scenes, CTO manages databases, storage, secrets, source control, deployments,
            and inference as productized platform services so teams can stay focused
            on product delivery instead of infrastructure assembly.
          </p>
        </div>
      </div>
    </section>
  );
}
