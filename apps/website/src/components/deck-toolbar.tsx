"use client";

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";

function ToolbarButton({
  onClick,
  children,
  title,
}: {
  onClick: () => void;
  children: React.ReactNode;
  title: string;
}) {
  return (
    <motion.button
      onClick={onClick}
      title={title}
      className="flex items-center gap-2 px-4 py-2.5 rounded-lg border border-border/50 bg-card/60 backdrop-blur-md text-sm font-medium hover:border-cyan/30 hover:bg-cyan/5 transition-colors"
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
    >
      {children}
    </motion.button>
  );
}

async function generatePptx() {
  const { default: PptxGenJS } = await import("pptxgenjs");
  const pptx = new PptxGenJS();
  pptx.layout = "LAYOUT_16x9";
  pptx.author = "5D Labs";
  pptx.title = "5D Labs — Pitch Deck";

  const bg = { color: "0A0E1A" };
  const cyan = "22D3EE";
  const white = "F0F0F0";
  const muted = "8A8A8A";

  const slides: {
    title: string;
    subtitle?: string;
    body?: string;
    bullets?: string[];
    label?: string;
  }[] = [
    {
      label: "01 · Cover",
      title: "Spec in. Software out.",
      subtitle: "Pre-seed · $750K · Delaware C-Corp",
      body: "You describe what you want built. Our AI builds, tests, and ships it — on owned hardware that can materially undercut hyperscale public-cloud TCO for steady workloads (public exits like 37signals show ~59% lower annual infra in comparable cases).",
    },
    {
      label: "02 · Problem",
      title: "AI changes faster than teams can ship.",
      bullets: [
        "Training compute for notable models doubles ~every 5 months — Stanford AI Index 2025.",
        "Surveyed orgs report ~29% of cloud spend wasted — Flexera State of the Cloud 2026.",
        "Stack sprawl — more vendors and glue, more meetings, not more velocity.",
      ],
    },
    {
      label: "03 · Founder",
      title: "Jonathon Fritz — 20 years building infrastructure at scale.",
      bullets: [
        "Pocket — Head of Infrastructure. 13 engineers. 1B+ requests/day.",
        "Coinmiles — Promoted to CTO in 3 months.",
        "Blocknative — Real-time transaction monitoring at scale.",
        "5D Labs — Solo built: platform, first customer, $240K pipeline, 17+ server deployments.",
      ],
    },
    {
      label: "04 · Why now",
      title: "The window is closing.",
      body: "Stanford AI Index 2025: 149 foundation models in 2023 (2× vs 2022); training compute doubles ~every 5 months; GPT-3.5-class inference $/M tokens fell >280× in ~18 months. Flexera 2026: ~29% of cloud spend wasted. 37signals publicly cut cloud from $3.2M → ~$1.3M/yr on bare metal (The Register, 2024).",
      bullets: [
        "2023: 149 foundation models — more than double vs 2022 (Stanford AI Index 2025).",
        "2024–25: Training compute doubles ~every 5 months for notable models (Stanford AI Index 2025).",
        "2026+: ~29% cloud spend wasted — survey respondents (Flexera State of the Cloud 2026).",
      ],
    },
    {
      label: "05 · Solution",
      title: "One system: spec in, deployed software out.",
      body: "Describe what you want in plain English. Our AI workforce builds it, tests it, and ships it to servers you control.",
      bullets: [
        "Your servers — dedicated hardware you own, not rented cloud.",
        "Always current — when AI tools change, we handle the update.",
        "Self-healing — production issues detected and fixed automatically.",
      ],
    },
    {
      label: "06 · Product",
      title: "Replaces the managed cloud services that eat your margin.",
      body: "Across AWS, GCP, Azure, and the rest — databases, storage, inference, observability, deploy, secrets, edge, and more. Not a fixed count; the stack grows as we replace more of the bill.",
      bullets: [
        "Managed DB (RDS, Cloud SQL, Aurora…) → 5D Data",
        "Object storage (S3, GCS, Blob…) → 5D Store",
        "Managed inference (Vertex, SageMaker…) → 5D Inference",
        "APM + observability → 5D Observe",
        "CI/CD SaaS (any cloud) → 5D Deploy",
        "Secrets + KMS → 5D Vault",
        "CDN + edge DNS → 5D Edge",
        "Managed workflows (Step Functions, Logic Apps…) → 5D Deploy",
      ],
    },
    {
      label: "07 · Traction",
      title: "Revenue before fundraising.",
      bullets: [
        "1 paying customer — Sigma One, live in production.",
        "$240K pipeline — annual contract value in active discussions.",
        "17+ server deployments across multiple regions.",
        "22 AI workers — specialized, coordinated roles.",
        "$0 outside capital to date — entirely self-funded.",
      ],
    },
    {
      label: "08 · Market",
      title: "$420B+ spent on cloud every year.",
      body: "We start where the pain is highest: AI and crypto teams who already run their own servers and know how much they overpay for public cloud.",
      bullets: [
        "TAM: $420B+ — global cloud IaaS + PaaS.",
        "SAM: $40–80B — AI-native dev teams and startups.",
        "Beachhead: $3–5B — teams already replacing cloud with owned hardware.",
      ],
    },
    {
      label: "09 · Business + GTM",
      title: "Recurring revenue. Margins that improve over time.",
      bullets: [
        "Free tier → developers try a lightweight version at no cost.",
        "Paid plans → full AI workforce and dedicated infrastructure.",
        "Recurring revenue → monthly subscriptions with expanding margins.",
        "Trading engine → funds operations internally. Investor capital never at risk.",
      ],
    },
    {
      label: "10 · Use of funds",
      title: "$750K. Two engineers. 18 months.",
      body: "The product works. The first customer is live. This capital goes to scaling — not figuring out what to build.",
      bullets: [
        "Engineering (2 hires): $300–400K",
        "Founder salary: $100–120K",
        "AI model costs: $30–50K",
        "Market infra: $20–40K",
        "Lab server: $16–20K",
      ],
    },
    {
      label: "11 · The ask",
      title: "$750K",
      subtitle: "Product live. Customer paying. Pipeline in hand.",
      body: "Post-money SAFE. Cap aligned to AI infrastructure comps.\n\n18-month path to cash-flow positive.\nLive demo available in any meeting.",
    },
  ];

  for (const s of slides) {
    const slide = pptx.addSlide();
    slide.background = bg;

    if (s.label) {
      slide.addText(s.label, {
        x: 0.5,
        y: 0.3,
        w: 9,
        fontSize: 10,
        color: muted,
        fontFace: "Arial",
        charSpacing: 3,
      });
    }

    slide.addText(s.title, {
      x: 0.5,
      y: s.subtitle ? 1.2 : 0.8,
      w: 9,
      fontSize: s.label === "11 · The ask" ? 44 : 28,
      color: cyan,
      fontFace: "Arial",
      bold: true,
    });

    if (s.subtitle) {
      slide.addText(s.subtitle, {
        x: 0.5,
        y: 0.7,
        w: 9,
        fontSize: 14,
        color: muted,
        fontFace: "Arial",
      });
    }

    let yPos = s.subtitle ? 2.2 : 1.8;

    if (s.body) {
      slide.addText(s.body, {
        x: 0.5,
        y: yPos,
        w: 9,
        fontSize: 14,
        color: white,
        fontFace: "Arial",
        lineSpacingMultiple: 1.3,
      });
      yPos += 1.0;
    }

    if (s.bullets) {
      slide.addText(
        s.bullets.map((b) => ({
          text: b,
          options: {
            fontSize: 13,
            color: white,
            fontFace: "Arial" as const,
            bullet: { type: "bullet" as const, color: cyan },
            paraSpaceBefore: 6,
          },
        })),
        { x: 0.5, y: yPos, w: 9, h: 3 }
      );
    }

    slide.addText("5D Labs · Pitch Deck", {
      x: 0.5,
      y: 5.0,
      w: 5,
      fontSize: 8,
      color: muted,
      fontFace: "Arial",
    });
  }

  await pptx.writeFile({ fileName: "5D-Labs-Pitch-Deck.pptx" });
}

export function DeckToolbar() {
  const [exporting, setExporting] = useState<string | null>(null);

  const handlePdf = useCallback(() => {
    window.print();
  }, []);

  const handlePptx = useCallback(async () => {
    setExporting("pptx");
    try {
      await generatePptx();
    } finally {
      setExporting(null);
    }
  }, []);

  return (
    <>
      <div className="fixed bottom-0 left-0 right-0 z-50 flex justify-center pb-4 pointer-events-none print:hidden">
        <motion.div
          className="flex items-center gap-2 p-2 rounded-2xl border border-border/50 bg-background/80 backdrop-blur-xl shadow-2xl pointer-events-auto"
          initial={{ y: 40, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 1, duration: 0.6, ease: [0.25, 0.4, 0, 1] }}
        >
          <ToolbarButton onClick={handlePdf} title="Save as PDF or print">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z" />
            </svg>
            PDF / Print
          </ToolbarButton>

          <ToolbarButton onClick={handlePptx} title="Download PowerPoint">
            <AnimatePresence mode="wait">
              {exporting === "pptx" ? (
                <motion.div
                  key="spinner"
                  className="w-4 h-4 border-2 border-cyan/30 border-t-cyan rounded-full animate-spin"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                />
              ) : (
                <motion.svg
                  key="icon"
                  className="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </motion.svg>
              )}
            </AnimatePresence>
            PowerPoint
          </ToolbarButton>
        </motion.div>
      </div>

      {/* Print-specific styles — preserve dark theme and colors */}
      <style jsx global>{`
        @media print {
          -webkit-print-color-adjust: exact !important;
          print-color-adjust: exact !important;
          color-adjust: exact !important;

          html, body {
            -webkit-print-color-adjust: exact !important;
            print-color-adjust: exact !important;
            background: #0a0e1a !important;
            color: #f0f0f0 !important;
          }

          .fixed:not(.print-keep) {
            display: none !important;
          }
          header, footer, nav {
            display: none !important;
          }

          .circuit-bg, .noise-overlay {
            display: none !important;
          }

          section {
            page-break-inside: avoid;
            break-inside: avoid;
            min-height: auto !important;
            padding-top: 1.5rem !important;
            padding-bottom: 1.5rem !important;
          }

          .backdrop-blur-sm, .backdrop-blur-md, .backdrop-blur-xl {
            backdrop-filter: none !important;
          }

          * {
            animation: none !important;
            animation-duration: 0s !important;
            transition: none !important;
            transition-duration: 0s !important;
          }

          [style*="opacity: 0"] {
            opacity: 1 !important;
          }
          [style*="transform"] {
            transform: none !important;
          }
        }
      `}</style>
    </>
  );
}
