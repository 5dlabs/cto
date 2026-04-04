/**
 * Single source of truth for pitch deck content.
 * Shared by the web page, PDF exporter, and PPTX exporter.
 */

/* ─── Types ─── */

export interface SlideTable {
  headers: string[];
  rows: string[][];
}

export interface DeckSlideStat {
  value: string;
  label: string;
}

export type DeckSlideLayout = "default" | "hero" | "impact";

export interface DeckSlide {
  id: string;
  label: string;
  headline: string;
  subhead?: string;
  body?: string;
  bullets?: string[];
  stats?: DeckSlideStat[];
  callout?: string;
  table?: SlideTable;
  footnote?: string;
  layout?: DeckSlideLayout;
}

/* ─── Brand ─── */

export const BRAND = {
  company: "5D Labs",
  round: "Pre-seed",
  ask: "$750K",
  entity: "Delaware C-Corp",
  header: "5D Labs \u00b7 Pre-seed \u00b7 Confidential",
} as const;

/* ─── Theme colors ─── */

export type RGB = [number, number, number];

export const THEME = {
  rgb: {
    bg: [10, 12, 22] as RGB,
    accent: [34, 211, 238] as RGB,
    topBar: [6, 182, 212] as RGB,
    text: [248, 250, 252] as RGB,
    muted: [163, 163, 163] as RGB,
    dim: [120, 130, 150] as RGB,
    calloutBg: [15, 40, 45] as RGB,
    calloutText: [204, 251, 241] as RGB,
    footnote: [140, 150, 170] as RGB,
  },
  /** Hex (no #) for pptxgenjs */
  hex: {
    bg: "0A0E1A",
    cyan: "22D3EE",
    white: "F0F0F0",
    muted: "8A8A8A",
    dim: "78829A",
    calloutBg: "0F282D",
    calloutText: "CCFBF1",
    headerBg: "162330",
    altRowBg: "0E121E",
    border: "374155",
  },
};

/* ─── Platform services (Slide 6: Product) ─── */

export interface PlatformService {
  name: string;
  desc: string;
  replaces: string;
}

export const PLATFORM_SERVICES: PlatformService[] = [
  { name: "5D Deploy", desc: "GitOps-driven delivery pipeline", replaces: "CI/CD SaaS" },
  { name: "5D Observe", desc: "Unified monitoring, logs, traces", replaces: "Datadog, New Relic" },
  { name: "5D Vault", desc: "Secrets + dynamic credentials", replaces: "AWS Secrets Manager" },
  { name: "5D Data", desc: "Managed PostgreSQL with HA", replaces: "RDS, Cloud SQL" },
  { name: "5D Inference", desc: "Model runtime on your GPUs", replaces: "SageMaker, Vertex AI" },
  { name: "5D Store", desc: "S3-compatible object storage", replaces: "S3, GCS" },
  { name: "5D Sentinel", desc: "Security scanning + AI remediation", replaces: "Snyk, Wiz" },
  { name: "5D Edge", desc: "Ingress, TLS, DNS automation", replaces: "CloudFront" },
  { name: "5D Stream", desc: "Messaging + event streaming", replaces: "SQS, Kafka" },
  { name: "5D Node", desc: "Validator + RPC node ops", replaces: "Alchemy, QuickNode" },
  { name: "5D Mesh", desc: "Service mesh, mTLS, network policy", replaces: "Istio, Linkerd" },
  { name: "5D Auth", desc: "Identity, SSO, role-based access", replaces: "Auth0, Okta" },
  { name: "5D Registry", desc: "Container + artifact registry", replaces: "ECR, GCR" },
  { name: "5D Cache", desc: "Managed Redis / Valkey", replaces: "ElastiCache" },
  { name: "5D Queue", desc: "Background job processing", replaces: "SQS, Cloud Tasks" },
  { name: "5D Backup", desc: "Snapshots + disaster recovery", replaces: "AWS Backup" },
  { name: "5D Network", desc: "VPN, WireGuard, private net", replaces: "AWS VPC, Tailscale" },
  { name: "5D Search", desc: "Full-text + vector search", replaces: "Elasticsearch, Algolia" },
  { name: "5D GPU", desc: "GPU scheduling + fractional", replaces: "Lambda, CoreWeave" },
  { name: "5D Catalog", desc: "Service catalog + dev portal", replaces: "Backstage, Port" },
];

/* ─── Revenue streams (Slide 10) ─── */

export interface RevenueStream {
  name: string;
  type: string;
  desc: string;
}

export const REVENUE_STREAMS: RevenueStream[] = [
  { name: "CTO subscriptions", type: "Recurring", desc: "Monthly plans, Free to Enterprise. CodeRun-based." },
  { name: "Bare-metal rev-share", type: "Recurring", desc: "Infra margin on hardware we provision." },
  { name: "Proprietary trading", type: "Bootstrap", desc: "In-house capital engine. Funds experiments." },
  { name: "Advisory + implementation", type: "Services", desc: "Hands-on setup for early adopters." },
];

/* ─── Fund allocation (Slide 11) ─── */

export interface FundItem {
  label: string;
  value: string;
  percent: number;
}

export const FUND_ALLOCATION: FundItem[] = [
  { label: "Engineering (2 hires, loaded)", value: "$360-420K", percent: 100 },
  { label: "Founder salary (loaded)", value: "$120-140K", percent: 34 },
  { label: "Legal / accounting / 409A", value: "$30-40K", percent: 9 },
  { label: "AI model + inference", value: "$30-50K", percent: 9 },
  { label: "GTM + sales", value: "$20-30K", percent: 6 },
  { label: "Infrastructure (servers)", value: "$30-50K", percent: 9 },
  { label: "Buffer / contingency", value: "$20-30K", percent: 6 },
];

/* ─── Market rings (Slide 8) ─── */

export interface MarketRing {
  label: string;
  value: string;
  description: string;
}

export const MARKET_RINGS: MarketRing[] = [
  { label: "TAM", value: "$420B+", description: "Global cloud IaaS + PaaS" },
  { label: "SAM", value: "$40-80B", description: "AI-native dev teams + startups" },
  { label: "Beachhead", value: "$3-5B", description: "Teams replacing cloud with bare metal" },
];

/* ─── Traction metrics (Slide 7) ─── */

export interface TractionMetric {
  value: string;
  label: string;
  note: string;
}

export const TRACTION_METRICS: TractionMetric[] = [
  { value: "1", label: "Paying customer", note: "Sigma One - live in production" },
  { value: "$240K", label: "Pipeline", note: "ACV in active discussions" },
  { value: "17+", label: "Server deployments", note: "Across multiple regions" },
  { value: "22", label: "AI workers", note: "Shipping 24/7" },
  { value: "$0", label: "Outside capital", note: "Self-funded, zero dilution" },
];

/* ─── Competition quadrant (Slide 9) ─── */

export interface QuadrantCell {
  label: string;
  players: string[];
  highlight?: boolean;
}

export const COMPETITION_QUADRANT: QuadrantCell[] = [
  { label: "Cloud-managed + Human-built", players: ["AWS / GCP / Azure", "Heroku / Render", "Vercel / Netlify"] },
  { label: "Cloud-managed + AI-built", players: ["Replit / Bolt", "GitHub Copilot Workspace", "Vercel v0"] },
  { label: "Bare metal + Human-built", players: ["Coolify / CapRover", "Hetzner + Terraform", "Oxide Computer"] },
  { label: "Bare metal + AI-built", players: ["5D Labs"], highlight: true },
];

/* ─── Pipeline stages (Slide 5) ─── */

export const PIPELINE_STAGES = ["Spec", "Plan", "Build", "Test", "Ship"] as const;

/* ─── Funnel stages (Slide 10) ─── */

export interface FunnelStage {
  label: string;
  description: string;
}

export const FUNNEL_STAGES: FunnelStage[] = [
  { label: "Free tier", description: "Developers try a lightweight CTO at no cost." },
  { label: "Paid plans", description: "Teams upgrade for full AI workforce + bare-metal infrastructure." },
  { label: "Recurring revenue", description: "Monthly subscriptions, improving margins." },
];

/* ─── 12-slide deck ─── */

export const SLIDES: DeckSlide[] = [
  {
    id: "cover",
    label: "01 \u00b7 Cover",
    layout: "hero",
    headline: "Spec in. Software out.",
    subhead: "Pre-seed \u00b7 $750K \u00b7 Delaware C-Corp",
    body: "You describe what you want built. Our AI builds, tests, and ships it \u2014 hyperscale performance, bare-metal economics. Targeting a $420B+ cloud market where ~29% of spend is wasted.",
  },
  {
    id: "problem",
    label: "02 \u00b7 Problem",
    layout: "impact",
    headline: "AI changes faster than teams can ship.",
    bullets: [
      "Training compute for notable models doubles ~every 5 months \u2014 Stanford AI Index 2025.",
      "Surveyed orgs report ~29% of cloud spend wasted \u2014 Flexera State of the Cloud 2026.",
      "Stack sprawl \u2014 more vendors and glue, more meetings, not more velocity.",
    ],
  },
  {
    id: "founder",
    label: "03 \u00b7 Founder",
    headline: "Jonathon Fritz \u2014 20 years building infrastructure at scale.",
    bullets: [
      "Pocket \u2014 Head of Infrastructure. 13 engineers. 1B+ requests/day.",
      "Coinmiles \u2014 Promoted to CTO in 3 months.",
      "Blocknative \u2014 Real-time transaction monitoring at scale.",
      "5D Labs \u2014 Solo built: platform, first customer, $240K pipeline, 17+ server deployments.",
    ],
  },
  {
    id: "why-now",
    label: "04 \u00b7 Why now",
    headline: "The old startup model is structurally disadvantaged.",
    body: "One bet, 18 months of runway, pivot when you learn too late. Agentic coding + owned infrastructure changes the math.",
    bullets: [
      "Parallel ventures, not serial bets: agentic coding collapsed the cost of parallel execution \u2014 fail fast instead of pivoting near the end of your runway.",
      "Inference cost collapse: API pricing fell >280x in 18 months. Open-weight models run on your hardware, no vendor lock-in (Stanford AI Index 2025).",
      "Bare-metal validation: 37signals cut cloud from $3.2M -> ~$1.3M/yr on bare metal (The Register, 2024).",
      "Tokenized assets need infrastructure now: tokenized securities and on-chain instruments are going live \u2014 teams need owned infrastructure (SEC, EU MiCA 2024-26).",
    ],
  },
  {
    id: "solution",
    label: "05 \u00b7 Solution",
    headline: "One system: spec in, deployed software out.",
    body: "Describe what you want in plain English. Our AI workforce builds it, tests it, and ships it to bare-metal infrastructure you control.",
    bullets: [
      "Your servers \u2014 bare-metal servers we provision through hardware partners (Cherry, Hetzner, Latitude). You control the economics.",
      "Always current \u2014 AI tools change constantly. We absorb the updates, no context-switching for your team.",
      "Self-healing \u2014 production issues detected and fixed automatically.",
    ],
  },
  {
    id: "product",
    label: "06 \u00b7 Product",
    headline: "Agentic private cloud \u2014 20+ services on your hardware.",
    body: "Everything a team needs to build, ship, and run software \u2014 without renting it from AWS. Same capabilities. Your economics.",
    table: {
      headers: ["Service", "Description", "Replaces"],
      rows: PLATFORM_SERVICES.map((s) => [s.name, s.desc, s.replaces]),
    },
  },
  {
    id: "traction",
    label: "07 \u00b7 Traction",
    headline: "Revenue before fundraising.",
    stats: TRACTION_METRICS.map((m) => ({ value: m.value, label: `${m.label} \u2014 ${m.note}` })),
  },
  {
    id: "market",
    label: "08 \u00b7 Market",
    headline: "$420B+ spent on cloud every year.",
    body: "We start where the pain is highest: AI and crypto teams who already run their own servers and know how much they overpay for public cloud.",
    bullets: [
      "TAM: $420B+ \u2014 global cloud IaaS + PaaS.",
      "SAM: $40-80B \u2014 AI-native dev teams and startups.",
      "Beachhead: $3-5B \u2014 teams already replacing cloud with owned hardware.",
    ],
  },
  {
    id: "competition",
    label: "09 \u00b7 Competition",
    headline: "No one else combines both.",
    body: "AI code tools assume cloud. Infrastructure tools assume human engineers. We\u2019re the only player combining AI-native development with bare-metal economics.",
    bullets: [
      "Cloud-managed + Human-built: AWS, GCP, Azure, Heroku, Render.",
      "Cloud-managed + AI-built: Replit, Bolt, GitHub Copilot Workspace, Vercel v0.",
      "Bare metal + Human-built: Coolify, Hetzner + Terraform, Oxide Computer.",
      "Bare metal + AI-built: 5D Labs (unique position).",
    ],
  },
  {
    id: "business-gtm",
    label: "10 \u00b7 Business + GTM",
    headline: "Four revenue streams. One stack.",
    table: {
      headers: ["Stream", "Type", "Description"],
      rows: REVENUE_STREAMS.map((s) => [s.name, s.type, s.desc]),
    },
  },
  {
    id: "funds",
    label: "11 \u00b7 Use of funds",
    headline: "$750K. Two engineers. 18 months.",
    body: "The product works. The first customer is live. This capital goes to scaling \u2014 not discovery.\n\n1 senior backend/infra + 1 full-stack with AI systems experience. All costs loaded.",
    table: {
      headers: ["Line item", "Amount"],
      rows: FUND_ALLOCATION.map((f) => [f.label, f.value]),
    },
  },
  {
    id: "ask",
    label: "12 \u00b7 The ask",
    layout: "hero",
    headline: "$750K",
    subhead: "Product live. Customer paying. Pipeline in hand.",
    body: "Post-money SAFE. Cap aligned to AI infrastructure comps.\n3-5 customers at $5-8K/mo MRR = breakeven at month 15-18.\n\nLive demo available in any meeting.",
  },
];
