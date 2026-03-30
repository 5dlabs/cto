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
 * Strings are plain text everywhere (same output in the app, PDF, and PowerPoint).
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
      "Same stack across those lines—customers avoid hyperscaler egress bills and managed-service markup vs cloud-only AI.",
    footnote: `*Infra: deck shows ${DECK_INFRA_SAVINGS_LABEL} conservative vs hyperscale; internal migration models run ${INTERNAL_FULL_STACK_SAVINGS_RANGE_NOTES}. Example only — ~20TB/mo internet egress ≈ ~$${ILLUSTRATIVE_AWS_EGRESS_20TB_USD_PER_MONTH.toLocaleString("en-US")}/mo at ~$0.085/GB (AWS-style tier) vs $0 inside typical partner bundles (e.g. 20TB/mo included on Latitude Metal; https://www.latitude.sh/network/pricing ). Workload-specific — full math in cloud-vs-baremetal-analysis.ts.`,
  },
  {
    id: "problem",
    label: "Problem",
    layout: "impact",
    eyebrow: "Unit economics",
    headline: "The real hyperscale bill isn’t headline compute.",
    subhead:
      "The bill: egress, storage, margin — not list-price CPU. The org: sprawl + DevOps gap. Scarce: coordination as code commoditizes. CTO: bill + org on bare metal and automation.",
    bullets: [
      "Invoice line items: network + storage + managed-service markup vs bait-priced compute.",
      "People + sprawl: teams add headcount and tooling to move faster, but without shared rails the rework and coordination overhead grow with the team — more people does not mean more velocity.",
      "Pace and fatigue: models, CLIs, and vendors move at breakneck speed — dizzying for any engineering lead — and still no single operating model; teams re-choose instead of compound.",
      "Runway pattern: long build cycles and late pivots when cash is low — not early when learning is cheap; costs and stack are often locked in before the turn.",
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
    callout:
      "Product intake and PRDs live in CTO — agents and automation take it from there through build, review, and release to production.",
  },
  {
    id: "cto",
    label: "CTO",
    headline: "CTO — platform, not a chat skin",
    subhead:
      "Kubernetes-native; mostly Rust for latency. Same product on an enterprise cluster or kind on a workstation.",
    bullets: [
      "One operating model — agents, skills, and routing on shared rails; new models and providers plug in underneath so you are not re-inventing delivery every time the ecosystem shifts — full optionality, one spine.",
      "Multi-CLI routing — OpenClaw MCP harness (eight CLIs); picks provider + model for token and $ efficiency — works well when founders spread work across free credits on many accounts.",
      "Tool surface — MCP aggregator as the tool server; preset skills per agent with add / override anytime — methodology stays yours while tools and providers change.",
      "Operators & runtime — Kubernetes operators for backend services; runtime image packs languages and tooling; stack is mostly Rust where it matters for performance.",
      "Self-heal & review — Healer reacts to telemetry from the observability stack; Stitch does automated PR review and remediation; alerting to Discord, Slack, and webhooks.",
      "Coordination — NATS for agent-to-agent traffic across pods (fills a gap vs. stock Kubernetes); CRDs run headless jobs that report back to Morgan; ACP-style coordination across the fleet.",
      "Metal & GPU — automated bare-metal and GPU provisioning; we abstract provider contracts so customers don’t have to — or they keep a direct deal with the vendor.",
      "Research loop — ingest latest methodologies, patterns, and papers → PRD → features on the platform — so the product stays current without asking users to rip up how they work.",
      "Board room (debate, voice, chat with Morgan via ElevenLabs + LiveKit) ships in the main CTO app — not desktop-only. CTO Lite is the desktop-only freemium entry (local kind, limited agents). AR glasses roadmap: Even G2, Meta Ray-Ban Display, Rokid, Vuzix Z100 — 5dlabs.ai/cto/morgan.",
    ],
    footnote:
      "AGPL-3.0 — serious teams self-host. More detail: 5dlabs.ai/cto",
  },
  {
    id: "cto-roster",
    label: "Agent roster",
    headline: "22 specialist agents",
    subhead:
      "Preset skills and tools per role—extend or swap without retraining your team on a new methodology.",
    bullets: [
      "Control — Morgan (PM / coordination).",
      "Build — Rex, Grizz, Nova, Viper · Blaze, Tap, Spark (backend, web, mobile, desktop).",
      "Trust & ship — Cleo, Cipher, Tess, Stitch, Atlas, Bolt (quality, security, testing, review, merge, infra).",
      "Domain — Block, Vex, Angie, Glitch (chains, XR, agent systems, games).",
      "Business — Lex, Hype, Tally, Chase (legal, marketing, accounting, sales).",
    ],
    callout:
      "Avatars and detail on 5dlabs.ai/cto. Internal operators (e.g. Healer, Keeper, Pixel) also run self-heal, desktop, and orchestration under the hood.",
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
      "5D Git — self-hosted GitLab or Gitea: no per-seat GitHub fees; full pipeline on your metal.",
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
    id: "openclaw",
    label: "OpenClaw",
    headline: "OpenClaw — agent runtime on the same metal",
    subhead:
      "Orchestration and playbooks—not a separate product silo. Same fleet, same rails as CTO.",
    bullets: [
      "Coordinates specialist agents (PM, implementers, infra, security, …) with shared MCP + toolchains.",
      "Intake → tasks → Plays: one loop from PRD to merge with humans in the loop where it matters.",
      "Open source: GitOps-based OpenClaw platform + Helm to deploy and scale OpenClaw — the spine CTO runs on — so community and marketing funnel into full CTO + bare metal.",
      "Runs beside the 5D private-cloud services: agents operate the stack, stack hosts the workloads.",
    ],
    callout:
      "If the pitch names “Open Cloud,” read it as this: agents + services on your metal—OpenClaw is the control plane.",
  },
  {
    id: "intake",
    label: "Differentiation",
    headline: "Plan before code — keep your mental model",
    bullets: [
      "Structured planning workflow before implementation — explore options, use voice, listen async — so the team aligns before anyone ships the wrong thing.",
      "UI and scope clarified early with review + tracking tied together — less rework, less spend on design-only firefighting.",
      "Humans stay in the loop through listening and async review, not constant ticket-chasing.",
    ],
    callout: "ROI: fewer hires, lower infra bill, faster cycles.",
  },
  {
    id: "trading",
    label: "Trading",
    headline: "Trading engine — in-house capital & signal",
    subhead: "Bootstrap only — not a fund we market; investor capital is not trading principal.",
    bullets: [
      "Multi-chain stack in production: Solana, Base, Polygon, Near, Sui.",
      "Low-latency RPC / edge (Helius-class) without hedge-fund capex.",
      "Execution + risk workflows on the same infra discipline as CTO—observe, deploy, heal.",
      "Feeds experiments and on-chain signal; P&L stays inside the studio—not an external product line.",
    ],
    table: {
      headers: ["Layer", "What it covers"],
      rows: [
        ["Chains", "Solana · Base · Polygon · Near · Sui"],
        ["Edge", "Low-latency ingress + RPC path"],
        ["Capital", "Bootstrap only — studio balance sheet"],
        ["Positioning", "Not sold as a fund or third-party trading product"],
      ],
    },
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
      "Ship the open-source OpenClaw platform (GitOps + Helm to deploy and scale OpenClaw) so evals and installs funnel into the full CTO stack — not a vague “slice,” the same control plane we run.",
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
      "Animated avatar + voice — Q&A front door before a live conversation. Ground the host with KB URLs and deck links. No product intake or PRD path here — that runs in the CTO app, not this widget.",
    bullets: [
      "Morgan is the PM/coordination persona on CTO; this experience is voice + avatar Q&A only — not where specs or intake land.",
      "Investors use this for narrative questions; product intake and PRDs flow through CTO (same platform as the agent fleet), not through the browser widget.",
      "Live stack: OpenClaw + LiveKit + hosted avatar.",
    ],
    cta: {
      label: "Talk to Morgan",
      href: "https://5dlabs.ai/cto/morgan#talk",
    },
    footnote: "Avatar + voice: hosted provider · LiveKit · OpenClaw.",
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
