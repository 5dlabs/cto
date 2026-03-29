import {
  DECK_INFRA_SAVINGS_LABEL,
  DECK_REVENUE_STREAMS_COUNT,
  ILLUSTRATIVE_AWS_EGRESS_20TB_USD_PER_MONTH,
  INTERNAL_FULL_STACK_SAVINGS_RANGE_NOTES,
} from "./cloud-vs-baremetal-analysis";

export type SlideTable = { headers: string[]; rows: string[][] };

export type DeckSlideStat = { value: string; label: string };

/** hero = full-bleed cover; impact = larger type for opening slides */
export type DeckSlideLayout = "default" | "hero" | "impact";

/** Optional primary action (e.g. Talk to Morgan) — shown in web deck + noted in PDF/PPTX. */
export type DeckSlideCta = { label: string; href: string };

export type DeckSlide = {
  id: string;
  label: string;
  eyebrow?: string;
  headline: string;
  subhead?: string;
  bullets?: string[];
  stats?: DeckSlideStat[];
  callout?: string;
  table?: SlideTable;
  footnote?: string;
  layout?: DeckSlideLayout;
  cta?: DeckSlideCta;
};

export const DECK_META = {
  company: "5D Labs",
  round: "Pre-seed",
  confidential: "Confidential — discussion only",
} as const;

/**
 * Single source of truth for web deck + PDF (print).
 * Copy kept short for live read + older investors; pair with verbal detail.
 */
export const slides: DeckSlide[] = [
  {
    id: "cover",
    label: "Cover",
    layout: "hero",
    eyebrow: "Pre-seed · Delaware C-Corp · $750K",
    headline: "We built the stack that helps us build companies.",
    subhead:
      "AI-native venture studio. CTO ships on bare metal today. Capital engine + ventures turn that stack into outcomes.",
    stats: [
      { value: "2", label: "paying customers" },
      { value: "$240K", label: "ACV" },
      {
        value: String(DECK_REVENUE_STREAMS_COUNT),
        label: "revenue streams",
      },
      {
        value: DECK_INFRA_SAVINGS_LABEL,
        label: "infra savings vs cloud*",
      },
    ],
    callout:
      "CTO implementations · bare-metal rev-share · trading · advisory — same stack. Customers cut egress + managed-service overhead vs cloud-only AI stacks.",
    footnote: `*Infra: deck shows ${DECK_INFRA_SAVINGS_LABEL} conservative vs hyperscale; internal migration models run ${INTERNAL_FULL_STACK_SAVINGS_RANGE_NOTES}. Example only — ~20TB/mo internet egress ≈ ~$${ILLUSTRATIVE_AWS_EGRESS_20TB_USD_PER_MONTH.toLocaleString("en-US")}/mo at ~$0.085/GB (AWS-style tier) vs $0 inside typical partner bundles (e.g. 20TB/mo included on Latitude Metal; https://www.latitude.sh/network/pricing ). Workload-specific — full math in cloud-vs-baremetal-analysis.ts.`,
  },
  {
    id: "problem",
    label: "Problem",
    layout: "impact",
    eyebrow: "The shift",
    headline: "When code is cheap, coordination wins.",
    subhead:
      "“Code is a commodity” means implementation keeps getting automated — more software builds itself, marginal cost of another feature drops. The scarce part is aim, orchestration, and learning fast enough.",
    bullets: [
      "Execution still needs direction, discipline, and infra that stays up.",
      "Tool sprawl + cloud tax + scarce DevOps/design — cost explodes without a system.",
      "Model mix: frontier models for planning and architecture; local + competitive open models (incl. Chinese stacks) for iteration loops — front-load thinking, iterate cheaply without giving up quality.",
      "Winners run decide → ship → learn in one loop with real signal.",
    ],
  },
  {
    id: "solution",
    label: "One machine",
    layout: "impact",
    eyebrow: "Thesis",
    headline: "One machine: build, fund, ship.",
    subhead: "CTO, trading, ventures — one OS, not three random bets.",
    bullets: [
      "CTO — agents, multi-CLI, bare metal K8s, self-healing delivery.",
      "Trading — in-house capital + on-chain signal (not a product we sell).",
      "Ventures + OpenClaw — same playbooks; one fleet.",
    ],
    callout: "“Too wide?” One loop. Cut a leg and it breaks.",
  },
  {
    id: "origin",
    label: "Origin",
    headline: "Built it for ourselves — then it became the product",
    bullets: [
      "Wanted a Solana trading stack; one person, pre-reliable models.",
      "Built CTO to ship anyway — didn’t wait for “good enough” AI.",
      "8 months on CTO; back to trading now that infra is real.",
    ],
    callout: "Scratch-the-itch infra → prove in prod → sell the wedge.",
  },
  {
    id: "loop",
    label: "How it works",
    headline: "Decide → deploy → learn → compound",
    bullets: [
      "Decide — one thesis, wedge, success signals.",
      "Deploy — agents on shared rails (CTO + OpenClaw).",
      "Learn — runtime + market feed the roadmap.",
      "Compound — reuse prompts, workflows, infra.",
    ],
  },
  {
    id: "cto",
    label: "CTO",
    headline: "CTO — build engine + first commercial wedge",
    bullets: [
      "SDLC + ops (Morgan: sales, marketing, accounting).",
      "Self-healing delivery · multi-CLI (Cursor, Claude, Codex, Factory, …).",
      "Bare metal — sovereignty, predictable cost, no lock-in. “And” not “or.”",
    ],
    footnote: "AGPL-3.0 — serious teams self-host.",
  },
  {
    id: "private-cloud",
    label: "Private cloud",
    headline: "Agentic private cloud — AWS parity, your metal",
    subhead:
      "20+ productized services — Plan, Code, Git, Edge, Data, Deploy, Observe, … — not wrappers around ChatGPT.",
    bullets: [
      "5D Plan — PRD → structured deliberation before build.",
      "5D Code — multi-CLI harness (Cursor, Claude, Codex, …) with intelligent routing.",
      "5D Git — self-hosted GitLab or Gitea: no per-seat GitHub tax; full pipeline on your metal.",
      "5D Edge — Cloudflare tunnels + Cloudflare edge (we use CF end-to-end for secure ingress and connectivity).",
    ],
    table: {
      headers: ["5D", "≈ Replaces", "Stack"],
      rows: [
        ["5D Data", "RDS", "CloudNativePG"],
        ["5D Store", "Object", "SeaweedFS"],
        ["5D Inference", "SageMaker", "KubeAI"],
        ["5D Observe", "CW / DD", "Prom / Grafana / Loki"],
        ["5D Deploy", "CI/CD SaaS", "Argo CD"],
        ["5D Vault", "Secrets", "OpenBao + ESO"],
        ["5D Edge", "Edge / DNS / ZT", "Cloudflare + ingress + certs"],
      ],
    },
    callout:
      "Hyperscaler-shaped capabilities on hardware you own — run and healed by agents. Full catalog: cto/services on 5dlabs.ai.",
  },
  {
    id: "intake",
    label: "Differentiation",
    headline: "Plan before code — keep your mental model",
    bullets: [
      "Lobster: optimist vs. pessimist, voice — listen anywhere.",
      "Stitch + Linear — pick UI before build; less designer headcount.",
      "Humans stay in the loop via listening, not micromanagement.",
    ],
    callout: "ROI: fewer hires, lower infra bill, faster cycles.",
  },
  {
    id: "trading",
    label: "Trading",
    headline: "In-house capital engine (bootstrap only)",
    bullets: [
      "Solana, Base, Polygon, Near, Sui — production stack.",
      "Low-latency edge (Helius-class) without hedge-fund capex.",
      "Funds experiments + signal — not an external fund product.",
    ],
    footnote: "Investor $ is not trading principal.",
  },
  {
    id: "traction",
    label: "Traction",
    headline: "Traction & partnerships",
    bullets: [
      "Sigma One — full CTO + ops; self-hosted reference.",
      "Bloq (bloq.com) — ~$20K/mo engagement (~$240K ACV). Web3 infra & applications partner.",
      "Partnerships — servers.com · ID3.net · Latitude (metal / network).",
      "In discussion — Cherry Servers (not closed yet). MiniMax — approached us to partner.",
      "Stack: 17+ bare-metal sites · 4 chains · 22 specialist agents.",
      "Founder velocity: 10.6k GH/yr · Pocket-era infra: 1B+ req/day peak, 50+ networks.",
    ],
  },
  {
    id: "market",
    label: "Market",
    headline: "Beachhead → TAM",
    bullets: [
      "Start: crypto-native teams + founder credibility.",
      "Expand: any startup burning cloud + delivery headcount.",
      "Moat: bare metal + full automation vs. cloud-only agents.",
    ],
    footnote: "Cover footnote + cloud-vs-baremetal-analysis.ts — per-customer appendix TBD.",
  },
  {
    id: "business-model",
    label: "Model",
    headline: "Four revenue streams · one stack",
    bullets: [
      "CTO subscriptions + implementations (near-term $).",
      "Bare-metal rev-share (partners we route customers to).",
      "In-house trading P&L (bootstrap capital only).",
      "Advisory / consulting engagements.",
    ],
  },
  {
    id: "gtm",
    label: "Go-to-market",
    headline: "Open core → freemium desktop → paid tiers",
    subhead:
      "Distribution that points everything back to CTO — desktop is the primary product surface.",
    bullets: [
      "Ship an open-source OpenClaw slice (sub-components of CTO) so community and marketing funnel to the full CTO stack.",
      "Freemium CTO Lite on desktop: local kind cluster, no bare-metal path, limited agent set — enough to get hooked.",
      "Feature flags and upgrade hooks create FOMO; paid tiers (still being defined) unlock full fleet, metal, and agent depth.",
      "Long-term: subscriptions on top of the desktop app as the main commercial surface.",
    ],
  },
  {
    id: "use-of-funds",
    label: "Use of funds",
    headline: "$750K — team, infra, runway",
    table: {
      headers: ["Line", "USD", "Note"],
      rows: [
        ["2 engineers", "$300–400K", "~$150–200K each"],
        ["Founder", "$100–120K", "Runway salary"],
        ["Trading edge", "$20–40K", "Low-latency"],
        ["Lab server", "$16–20K", "Hardware"],
        ["Models", "$30–50K", "R&D; credits help"],
        ["Buffer", "Rest", "Legal, ops"],
      ],
    },
    callout: "18 months: path to cash-flow positive.",
  },
  {
    id: "morgan",
    label: "Morgan",
    headline: "Talk to Morgan before we meet",
    subhead:
      "Animated avatar + voice — your Q&A front door. Give Morgan deck and product context; investors can explore on their own before the live conversation.",
    bullets: [
      "Same Morgan that runs intake and coordinates agents on the CTO platform.",
      "Optional: paste context (links, PRD snippets) so answers stay on-narrative.",
      "Live stack: OpenClaw + LiveKit + LemonSlice (avatar) — pricing via Lemon Squeezy in progress.",
    ],
    cta: {
      label: "Talk to Morgan",
      href: "https://5dlabs.ai/cto/morgan#talk",
    },
    footnote:
      "Avatar + voice: LemonSlice · LiveKit · OpenClaw. Commerce: Lemon Squeezy (pricing in progress).",
  },
  {
    id: "founder",
    label: "Founder",
    headline: "Jonathon Fritz",
    bullets: [
      "20+ yrs ops · Victoria, BC.",
      "Pocket — Head of Infra, 1B+ req/day, 50+ networks (management scope at scale).",
      "Blocknative · Coinmiles SE→CTO in 3 mo.",
    ],
  },
  {
    id: "ask",
    label: "The ask",
    headline: "$750K post-money SAFE",
    bullets: [
      "Hires, founder salary, edge, lab, models.",
      "Cap aligned to AI infra comps — terms in conversation.",
      "Bias: profitability optionality; M&A > IPO as default.",
    ],
    callout:
      "Live demo in meeting · Export PDF, PowerPoint, or Google Slides from the deck chrome.",
    footnote: "https://cal.com/jonathon-fritz-2uhdqe/discovery",
  },
];
