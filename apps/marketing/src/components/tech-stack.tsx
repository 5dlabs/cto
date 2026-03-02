interface TechItem {
  name: string;
  color: string;
  category: string;
}

// Row 1: Managed operators & databases
const row1: TechItem[] = [
  { name: "PostgreSQL", color: "#336791", category: "Operator" },
  { name: "Redis", color: "#DC382D", category: "Operator" },
  { name: "MongoDB", color: "#47A248", category: "Operator" },
  { name: "Kafka", color: "#231F20", category: "Operator" },
  { name: "Grafana", color: "#F46800", category: "Dashboards" },
  { name: "Prometheus", color: "#E6522C", category: "Metrics" },
  { name: "Loki", color: "#F4D03F", category: "Logs" },
  { name: "Tempo", color: "#2EB67D", category: "Traces" },
  { name: "Datadog", color: "#632CA6", category: "Observability" },
  { name: "Mayastor", color: "#00BFFF", category: "Block Storage" },
  { name: "SeaweedFS", color: "#4CAF50", category: "Object Storage" },
];

// Row 2: Delivery, security, integrations
const row2: TechItem[] = [
  { name: "ArgoCD", color: "#EF7B4D", category: "Continuous Delivery" },
  { name: "OpenBao", color: "#FFEC6E", category: "Secret Management" },
  { name: "External Secrets", color: "#6C5CE7", category: "BYOK Sync" },
  { name: "Snyk", color: "#4C4CFF", category: "SCA" },
  { name: "Nuclei", color: "#00BCD4", category: "Pentest CLI" },
  { name: "Aikido", color: "#6C63FF", category: "AI Security" },
  { name: "Trivy", color: "#1904DA", category: "Vulnerability Scanning" },
  { name: "Gitleaks", color: "#FF4444", category: "Secret Detection" },
  { name: "GitHub", color: "#FFFFFF", category: "Source Control" },
  { name: "Linear", color: "#5E6AD2", category: "Project Mgmt" },
  { name: "Slack", color: "#4A154B", category: "Alerting" },
  { name: "Discord", color: "#5865F2", category: "Alerting" },
  { name: "PagerDuty", color: "#06AC38", category: "Incidents" },
  { name: "Jira", color: "#0052CC", category: "Project Mgmt" },
  { name: "Asana", color: "#F06A6A", category: "Project Mgmt" },
];

// Row 3: Supported AI CLIs & models
const row3: TechItem[] = [
  { name: "Claude Code", color: "#D4A574", category: "AI CLI" },
  { name: "Cursor", color: "#00D4FF", category: "AI CLI" },
  { name: "Codex", color: "#10A37F", category: "AI CLI" },
  { name: "Factory", color: "#FF6B6B", category: "AI CLI" },
  { name: "Gemini CLI", color: "#4285F4", category: "AI CLI" },
  { name: "OpenCode", color: "#FFFFFF", category: "AI CLI" },
  { name: "Claude", color: "#D4A574", category: "Model" },
  { name: "ChatGPT", color: "#10A37F", category: "Model" },
  { name: "Gemini", color: "#4285F4", category: "Model" },
  { name: "DeepSeek", color: "#00BFFF", category: "Self-Hosted" },
  { name: "Qwen3", color: "#FF6B00", category: "Self-Hosted" },
  { name: "Ollama", color: "#FFFFFF", category: "Self-Hosted" },
  { name: "vLLM", color: "#FF6B6B", category: "Self-Hosted" },
];

function TechBadge({ item }: { item: TechItem }) {
  return (
    <div className="flex items-center gap-3 px-5 py-3 rounded-xl border border-border/50 bg-card/40 backdrop-blur-sm whitespace-nowrap select-none">
      <span
        className="w-3 h-3 rounded-full shrink-0"
        style={{
          backgroundColor: item.color,
          boxShadow: `0 0 8px ${item.color}40, 0 0 0 2px ${item.color}30`,
        }}
      />
      <span className="font-medium text-sm text-foreground">{item.name}</span>
      <span className="text-[10px] text-muted-foreground/60 uppercase tracking-wider font-mono">
        {item.category}
      </span>
    </div>
  );
}

function MarqueeRow({
  items,
  direction = "left",
  speed = 40,
}: {
  items: TechItem[];
  direction?: "left" | "right";
  speed?: number;
}) {
  const doubled = [...items, ...items];
  const duration = items.length * speed;

  return (
    <div className="relative overflow-hidden">
      <div className="absolute left-0 top-0 bottom-0 w-24 z-10 bg-gradient-to-r from-background to-transparent pointer-events-none" />
      <div className="absolute right-0 top-0 bottom-0 w-24 z-10 bg-gradient-to-l from-background to-transparent pointer-events-none" />

      <div
        className="flex gap-4 marquee-track"
        style={{
          animationDuration: `${duration}s`,
          animationDirection: direction === "right" ? "reverse" : "normal",
        }}
      >
        {doubled.map((item, i) => (
          <TechBadge key={`${item.name}-${i}`} item={item} />
        ))}
      </div>
    </div>
  );
}

const categories = [
  {
    title: "Database Operators",
    description:
      "Fully managed operators with automated provisioning, backups, failover, and horizontal scaling",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
        />
      </svg>
    ),
    items: ["PostgreSQL", "Redis", "MongoDB", "Kafka"],
    color: "cyan",
  },
  {
    title: "Telemetry & Observability",
    description:
      "Complete metrics, logging, tracing, and dashboards—pre-wired and ready from day one",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
        />
      </svg>
    ),
    items: ["Grafana", "Prometheus", "Loki", "Tempo", "Datadog", "OpenTelemetry"],
    color: "orange",
  },
  {
    title: "Continuous Delivery",
    description:
      "GitOps deployments with ArgoCD — continuously reconciled, automated rollouts, and instant rollback",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
        />
      </svg>
    ),
    items: ["ArgoCD"],
    color: "purple",
  },
  {
    title: "Security & Secret Management",
    description:
      "Vault-grade BYOK secret management, vulnerability scanning, and leak detection built in",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
        />
      </svg>
    ),
    items: ["OpenBao", "External Secrets", "Snyk", "Nuclei", "Aikido", "Trivy", "Gitleaks"],
    color: "green",
  },
  {
    title: "Persistent Storage",
    description:
      "High-performance block and S3-compatible object storage managed for your workloads",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"
        />
      </svg>
    ),
    items: ["Mayastor", "SeaweedFS"],
    color: "blue",
  },
  {
    title: "AI CLI & Model Support",
    description:
      "Bring your own CLI and model API keys—every major provider supported out of the box",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
        />
      </svg>
    ),
    items: [
      "Claude Code",
      "Cursor",
      "Codex",
      "Factory",
      "Gemini CLI",
      "OpenCode",
    ],
    color: "yellow",
  },
  {
    title: "Integrations",
    description:
      "Native connections to your project management, communication, and alerting tools",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
        />
      </svg>
    ),
    items: ["GitHub", "Linear", "Slack", "Discord", "Jira", "Asana", "Teams", "PagerDuty"],
    color: "rose",
  },
];

const colorMap: Record<string, { bg: string; text: string; border: string }> = {
  cyan: {
    bg: "bg-cyan/10",
    text: "text-cyan",
    border: "border-cyan/20",
  },
  orange: {
    bg: "bg-orange-500/10",
    text: "text-orange-400",
    border: "border-orange-500/20",
  },
  purple: {
    bg: "bg-purple-500/10",
    text: "text-purple-400",
    border: "border-purple-500/20",
  },
  green: {
    bg: "bg-green-500/10",
    text: "text-green-400",
    border: "border-green-500/20",
  },
  blue: {
    bg: "bg-blue-500/10",
    text: "text-blue-400",
    border: "border-blue-500/20",
  },
  yellow: {
    bg: "bg-yellow-500/10",
    text: "text-yellow-400",
    border: "border-yellow-500/20",
  },
  rose: {
    bg: "bg-rose-500/10",
    text: "text-rose-400",
    border: "border-rose-500/20",
  },
};

export function TechStack() {
  return (
    <section id="stack" className="py-24 border-t border-border/30 overflow-hidden">
      <div className="max-w-6xl mx-auto px-6">
        <div className="text-center mb-6">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-cyan/20 bg-cyan/5 mb-6">
            <span className="text-xs text-cyan font-medium uppercase tracking-wider">
              Managed Services
            </span>
          </div>
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            Everything You Need,{" "}
            <span className="gradient-text">Already Running</span>
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Operators, telemetry, delivery pipelines, secret management, and
            storage—all pre-configured and managed so you can focus on shipping
            code.
          </p>
        </div>

        {/* Stats bar */}
        <div className="flex flex-wrap justify-center gap-8 text-sm text-muted-foreground mb-14">
          <div className="flex items-center gap-2">
            <span className="text-2xl font-bold text-foreground">50+</span>
            <span>managed services</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-2xl font-bold text-foreground">7</span>
            <span>service categories</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-2xl font-bold text-foreground">100%</span>
            <span>open source</span>
          </div>
        </div>
      </div>

      {/* Marquee rows - full width */}
      <div className="space-y-4 mb-16">
        <MarqueeRow items={row1} direction="left" speed={3} />
        <MarqueeRow items={row2} direction="right" speed={3.5} />
        <MarqueeRow items={row3} direction="left" speed={2.8} />
      </div>

      {/* Category breakdown grid */}
      <div className="max-w-6xl mx-auto px-6">
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
          {categories.map((cat) => {
            const colors = colorMap[cat.color] ?? colorMap.cyan;
            return (
              <div
                key={cat.title}
                className={`p-5 rounded-xl border ${colors.border} bg-card/30 backdrop-blur-sm`}
              >
                <div
                  className={`w-10 h-10 rounded-lg ${colors.bg} flex items-center justify-center mb-3`}
                >
                  <span className={colors.text}>{cat.icon}</span>
                </div>
                <h3 className="text-base font-semibold mb-1">{cat.title}</h3>
                <p className="text-xs text-muted-foreground mb-3">
                  {cat.description}
                </p>
                <div className="flex flex-wrap gap-1.5">
                  {cat.items.map((item) => (
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
      </div>
    </section>
  );
}
