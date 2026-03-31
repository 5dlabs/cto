import {
  DECK_INFRA_SAVINGS_LABEL,
  DECK_REVENUE_STREAMS_COUNT,
} from "./cloud-vs-baremetal-analysis";

export type SlideTable = { headers: string[]; rows: string[][] };

export type DeckSlideStat = { value: string; label: string };

/** hero = full-bleed cover; impact = larger type for opening slides */
export type DeckSlideLayout = "default" | "hero" | "impact";

/** Optional primary action (e.g. schedule a call) \u2014 shown in web deck + noted in PDF/PPTX. */
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
  confidential: "Confidential \u2014 discussion only",
} as const;

/**
 * Single source of truth for web deck + PDF (print).
 * Copy kept short for live read + older investors; pair with verbal detail.
 * Strings are plain text everywhere (same output in the app, PDF, and PowerPoint).
 *
 * Headline track (read ONLY H1s and the whole pitch should land):
 * 1. We replace your cloud stack and most of your DevOps team.
 * 2. AI changes faster than your team can ship.
 * 3. The window is closing.
 * 4. One system: spec in, deployed software out.
 * 5. Write a spec. 22 agents build it. You approve what matters.
 * 6. Everything a startup needs to ship, except the idea.
 * 7. Pilot customer live. $240K in pipeline. Zero outside capital.
 * 8. $420B+ in cloud spend. We start where the pain is worst.
 * 9. Subscriptions, infrastructure margin, and trading revenue.
 * 10. Free desktop app > paid deployments > infrastructure rev-share.
 * 11. $750K for two engineers and 18 months of runway.
 * 12. Jonathon Fritz. 20 years of infra. Built this solo.
 * 13. $750K. Pipeline in hand. Product in pilot.
 */
export const slides: DeckSlide[] = [
  /* ------------------------------------------------------------------ */
  /* 1. COVER                                                           */
  /* ------------------------------------------------------------------ */
  {
    id: "cover",
    label: "Cover",
    layout: "hero",
    eyebrow: "Pre-seed \u00B7 Delaware C-Corp \u00B7 $750K",
    headline:
      "We replace your cloud stack and most of your DevOps team.",
    subhead: `5D Labs is building software that turns a product spec into deployed code on infrastructure that costs ${DECK_INFRA_SAVINGS_LABEL} less than AWS. Currently in pilot.`,
    stats: [
      { value: "1", label: "pilot customer" },
      { value: "$240K", label: "pipeline ACV" },
      {
        value: String(DECK_REVENUE_STREAMS_COUNT),
        label: "revenue streams",
      },
      {
        value: DECK_INFRA_SAVINGS_LABEL,
        label: "cheaper than cloud",
      },
    ],
    footnote:
      "*Savings range from current workloads vs public cloud list pricing.",
  },

  /* ------------------------------------------------------------------ */
  /* 2. PROBLEM  (P in PAS)                                             */
  /* ------------------------------------------------------------------ */
  {
    id: "problem",
    label: "Problem",
    layout: "impact",
    eyebrow: "The pain",
    headline:
      "AI changes faster than your team can ship.",
    subhead:
      "Your CTO evaluated 15 tools last year. Adopted 3. Wasted a quarter. The cloud bill went up anyway.",
    bullets: [
      "A new AI coding tool launches every week. Your engineering lead is choosing tools instead of shipping product.",
      "AWS advertises cheap compute, then charges 10x for storage, egress, and managed services. Most teams don\u2019t notice until the bill arrives.",
      "You hire more engineers to go faster. The coordination overhead eats the speed gain. Net output barely moves.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 3. WHY NOW / AGITATE  (A in PAS)                                   */
  /* ------------------------------------------------------------------ */
  {
    id: "why-now",
    label: "Why now",
    layout: "impact",
    eyebrow: "Urgency",
    headline: "The window is closing.",
    bullets: [
      "Model releases have gone from quarterly to weekly. The tool landscape doubles every six months.",
      "Teams that don\u2019t lock in one delivery system now will keep re-architecting until the money runs out.",
      "Bare-metal providers just hit the price point where you can match AWS capabilities at 60\u201380% less. That crossover happened in the last 18 months.",
      "First platform to own the full loop \u2014 plan, build, deploy, heal \u2014 on cheaper hardware wins the category.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 4. SOLUTION  (S in PAS)                                            */
  /* ------------------------------------------------------------------ */
  {
    id: "solution",
    label: "Solution",
    layout: "impact",
    eyebrow: "What we do",
    headline:
      "One system: spec in, deployed software out.",
    subhead:
      "CTO is the product. You write a spec. 22 AI agents plan, build, review, test, and deploy it on hardware you own.",
    bullets: [
      "Your code runs on dedicated servers, not AWS. Same capabilities. Fraction of the cost.",
      "When the AI ecosystem changes next week, we absorb the update. Your workflow stays the same.",
      "We also run an internal trading operation for bootstrap revenue. It is not a product. Investor capital is never at risk.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 5. HOW IT WORKS                                                    */
  /* ------------------------------------------------------------------ */
  {
    id: "loop",
    label: "How it works",
    headline:
      "Write a spec. 22 agents build it. You approve what matters.",
    bullets: [
      "Step 1: You write a product spec or describe what you need in plain language.",
      "Step 2: A PM agent breaks it into tasks and assigns specialist agents \u2014 backend, frontend, infrastructure, tests.",
      "Step 3: Every pull request gets automated code review. Humans approve critical decisions.",
      "Step 4: Deployment, monitoring, and incident response run through the same system. Issues get fixed before your team wakes up.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 6. PRODUCT                                                         */
  /* ------------------------------------------------------------------ */
  {
    id: "cto",
    label: "Product",
    headline:
      "Everything a startup needs to ship, except the idea.",
    subhead:
      "Planning, coding, review, security, testing, deployment, and monitoring. One subscription.",
    bullets: [
      "Multi-model routing: each task goes to the cheapest AI model that can handle it. Customers use credits across 8+ providers instead of locking into one.",
      "Self-healing production: automated monitoring detects issues and triggers fixes before anyone pages your oncall.",
      "20+ infrastructure services replace AWS equivalents on dedicated servers \u2014 database, storage, inference, CI/CD, secrets, edge.",
      "Open-source core (AGPL-3.0). Serious teams self-host. We sell the managed version and support.",
    ],
    table: {
      headers: ["5D Service", "Replaces", "Built on"],
      rows: [
        ["5D Data", "AWS RDS", "CloudNativePG"],
        ["5D Store", "S3", "SeaweedFS"],
        ["5D Inference", "SageMaker", "KubeAI"],
        ["5D Observe", "CloudWatch / Datadog", "Prometheus + Grafana"],
        ["5D Deploy", "CI/CD SaaS", "Argo CD"],
        ["5D Vault", "Secrets Manager", "OpenBao"],
        ["5D Edge", "CloudFront / Route53", "Cloudflare"],
      ],
    },
    footnote: "Full service catalog: 5dlabs.ai/cto",
  },

  /* ------------------------------------------------------------------ */
  /* 7. TRACTION                                                        */
  /* ------------------------------------------------------------------ */
  {
    id: "traction",
    label: "Traction",
    headline: "Pilot customer live. $240K in pipeline. Zero outside capital.",
    stats: [
      { value: "$240K", label: "pipeline ACV" },
      { value: "1", label: "pilot customer" },
      { value: "17+", label: "bare-metal deployments" },
      { value: "22", label: "agents built" },
    ],
    bullets: [
      "Sigma One \u2014 live pilot/partnership. CTO deployment running.",
      "Bloq (bloq.com) \u2014 in discussion for web3 infrastructure and application delivery. ~$240K ACV in pipeline.",
      "Infrastructure partners: servers.com, ID3.net, Latitude.",
      "In discussion: Cherry Servers, MiniMax (inbound \u2014 they approached us).",
      "One person built the platform, landed a pilot customer and $240K in pipeline, and deployed 17+ bare-metal sites before raising a dollar.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 8. MARKET                                                          */
  /* ------------------------------------------------------------------ */
  {
    id: "market",
    label: "Market",
    headline:
      "$420B+ in cloud spend. We start where the pain is worst.",
    stats: [
      { value: "$3\u20135B", label: "beachhead" },
      { value: "$40\u201380B", label: "SAM" },
      { value: "$420B+", label: "TAM" },
    ],
    bullets: [
      "Beachhead ($3\u20135B): crypto and AI teams already running dedicated servers. We have credibility and network here.",
      "SAM ($40\u201380B): any startup spending too much on cloud and hiring too many engineers to compensate.",
      "TAM ($420B+): 2025 global public-cloud IaaS + PaaS spend. Every dollar overspent on AWS is a dollar we can save.",
    ],
    footnote:
      "Source: Gartner public-cloud forecast, Nov 2024. PaaS $208.6B + IaaS $211.9B.",
  },

  /* ------------------------------------------------------------------ */
  /* 9. BUSINESS MODEL                                                  */
  /* ------------------------------------------------------------------ */
  {
    id: "business-model",
    label: "Model",
    headline:
      "Subscriptions, infrastructure margin, and trading revenue.",
    bullets: [
      "CTO subscriptions: the core product. Tiered by agents, users, and infrastructure scale.",
      "Infrastructure rev-share: we route customers to bare-metal partners and take a margin on hardware.",
      "In-house trading: bootstrap capital only. Funds experiments. Investor money is never at risk.",
      "Implementation work: bridge revenue while subscription base builds. De-prioritized as ARR scales.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 10. GO-TO-MARKET                                                   */
  /* ------------------------------------------------------------------ */
  {
    id: "gtm",
    label: "Go-to-market",
    headline:
      "Free desktop app \u2192 paid deployments \u2192 infrastructure rev-share.",
    bullets: [
      "CTO Lite on desktop: free, runs locally, limited agents. Low friction to try.",
      "Paid tiers: full agent fleet, multi-user, bare-metal deployment, production ops.",
      "Implementation-led sales land reference customers and shorten time to value.",
      "Infrastructure rev-share compounds behind every deployment as customers grow.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 11. USE OF FUNDS                                                   */
  /* ------------------------------------------------------------------ */
  {
    id: "use-of-funds",
    label: "Use of funds",
    headline:
      "$750K for two engineers and 18 months of runway.",
    table: {
      headers: ["Line", "USD", "Note"],
      rows: [
        ["2 engineers", "$300\u2013400K", "~$150\u2013200K each"],
        ["Founder salary", "$100\u2013120K", "Runway"],
        ["Market infra", "$20\u201340K", "Low-latency + data"],
        ["Lab server", "$16\u201320K", "Hardware"],
        ["AI model costs", "$30\u201350K", "R&D usage"],
        ["Buffer", "Rest", "Legal, ops"],
      ],
    },
    callout: "18 months to cash-flow positive.",
  },

  /* ------------------------------------------------------------------ */
  /* 12. FOUNDER                                                        */
  /* ------------------------------------------------------------------ */
  {
    id: "founder",
    label: "Founder",
    headline:
      "Jonathon Fritz. 20 years of infra. Built this solo.",
    subhead: "Victoria, BC.",
    bullets: [
      "Pocket \u2014 Head of Infrastructure. Managed 13 engineers. Systems handled 1B+ requests/day across 50+ network integrations.",
      "Coinmiles \u2014 hired as a senior engineer, promoted to CTO in 3 months.",
      "Blocknative \u2014 web3 infrastructure and real-time transaction monitoring.",
      "Built 5D Labs solo: working platform, live pilot customer, $240K pipeline, 17+ bare-metal deployments \u2014 before raising a dollar.",
      "Has worked ~18-hour days on this since May 2025. Nearly a year straight.",
      "Nontraditional path. High adversity tolerance. Takes the punches. Keeps shipping.",
    ],
  },

  /* ------------------------------------------------------------------ */
  /* 13. ASK                                                            */
  /* ------------------------------------------------------------------ */
  {
    id: "ask",
    label: "The ask",
    headline: "$750K. Pipeline in hand. Product in pilot.",
    bullets: [
      "Post-money SAFE. Cap aligned to AI infrastructure comps.",
      "Capital goes to speed, not discovery. We have a live pilot, $240K in pipeline, and a working product.",
      "18-month path to cash-flow positive. Two senior engineering hires are the bottleneck.",
    ],
    callout:
      "Live demo available in any meeting.",
    cta: {
      label: "Schedule a call",
      href: "https://cal.com/jonathon-fritz-2uhdqe/discovery",
    },
    footnote: "https://cal.com/jonathon-fritz-2uhdqe/discovery",
  },
];
