"use client";

import { useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";

import {
  loadGoogleIdentityScript,
  requestDriveFileAccessToken,
  uploadPptxBlobToDrive,
} from "@/lib/google-drive-pitch";

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

/** Prefer pitch-specific; fall back to names often already set on Cloudflare / .env.local */
function firstPublicEnv(...keys: string[]): string {
  for (const key of keys) {
    const v = process.env[key];
    if (typeof v === "string" && v.trim()) return v.trim();
  }
  return "";
}

const GOOGLE_DRIVE_CLIENT_ID = firstPublicEnv(
  "NEXT_PUBLIC_GOOGLE_DRIVE_CLIENT_ID",
  "NEXT_PUBLIC_GOOGLE_CLIENT_ID",
  "NEXT_PUBLIC_GAPI_CLIENT_ID",
  "NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID",
);

const GOOGLE_DRIVE_FOLDER_ID = firstPublicEnv(
  "NEXT_PUBLIC_GOOGLE_DRIVE_FOLDER_ID",
  "NEXT_PUBLIC_DRIVE_FOLDER_ID",
  "NEXT_PUBLIC_GOOGLE_DRIVE_PARENT_ID",
);

async function createPitchDeckPptxBlob(): Promise<Blob> {
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
      body: "You describe what you want built. Our AI builds, tests, and ships it — hyperscale performance, bare-metal economics. Targeting a $420B+ cloud market where ~29% of spend is wasted.",
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
      title: "Four forces just converged.",
      body: "Each existed before. Together they make AI-native infrastructure on owned hardware viable for the first time.",
      bullets: [
        "Inference cost collapse: API pricing fell >280× in 18 months — self-hosted AI is now economical (Stanford AI Index 2025).",
        "Open-weight models: Llama 3, Mistral, DeepSeek — run on your hardware, no vendor lock-in.",
        "Bare-metal validation: 37signals cut cloud from $3.2M → ~$1.3M/yr on bare metal (The Register, 2024).",
        "Agentic tooling maturity: coding agents went from demos to production-grade in 2025.",
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
        "[Live] CI/CD SaaS (any cloud) → 5D Deploy",
        "[Live] APM + observability → 5D Observe",
        "[Live] Secrets + KMS → 5D Vault",
        "[Next] Managed DB (RDS, Cloud SQL, Aurora…) → 5D Data",
        "[Next] Managed inference (Vertex, SageMaker…) → 5D Inference",
        "[Next] Managed workflows (Step Functions, Logic Apps…) → 5D Deploy",
        "[Planned] Object storage (S3, GCS, Blob…) → 5D Store",
        "[Planned] CDN + edge DNS → 5D Edge",
      ],
    },
    {
      label: "07 · Traction",
      title: "Revenue before fundraising.",
      bullets: [
        "1 paying customer — Sigma One, live in production.",
        "$240K pipeline — annual contract value in active discussions.",
        "17+ server deployments across multiple regions.",
        "22 AI workers — intake, code gen, testing, deploy, security, self-healing — shipping 24/7.",
        "$0 outside capital — self-funded via proprietary trading, zero dilution.",
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
      label: "09 · Competition",
      title: "No one else combines both.",
      body: "AI code tools assume cloud. Infrastructure tools assume human engineers. We’re the only player combining AI-native development with owned infrastructure economics.",
      bullets: [
        "Cloud-managed + Human-built: AWS, GCP, Azure, Heroku, Render.",
        "Cloud-managed + AI-built: Replit, Bolt, GitHub Copilot Workspace, Vercel v0.",
        "Self-hosted + Human-built: Coolify, Hetzner + Terraform, Oxide Computer.",
        "Self-hosted + AI-built: 5D Labs (unique position).",
      ],
    },
    {
      label: "10 · Business + GTM",
      title: "Recurring revenue. Margins that improve over time.",
      bullets: [
        "Free tier → developers try a lightweight version at no cost.",
        "Paid plans → full AI workforce and dedicated infrastructure.",
        "Recurring revenue → monthly subscriptions with expanding margins.",
        "Partner channel → MSPs and DevOps consultancies resell to their clients.",
      ],
    },
    {
      label: "11 · Use of funds",
      title: "$750K. Two engineers. 18 months.",
      body: "The product works. The first customer is live. This capital goes to scaling — not discovery.\n\n1 senior backend/infra + 1 full-stack with AI systems experience. All costs loaded (benefits, payroll tax).",
      bullets: [
        "Engineering (2 hires, loaded): $360–420K",
        "Founder salary (loaded): $120–140K",
        "Legal / accounting / 409A: $30–40K",
        "AI model + inference: $30–50K",
        "GTM + sales: $20–30K",
        "Infrastructure (servers): $30–50K",
        "Buffer / contingency: $20–30K",
      ],
    },
    {
      label: "12 · The ask",
      title: "$750K",
      subtitle: "Product live. Customer paying. Pipeline in hand.",
      body: "Post-money SAFE. Cap aligned to AI infrastructure comps.\n3–5 customers at $5–8K/mo MRR = breakeven at month 15–18.\n\nLive demo available in any meeting.",
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
      fontSize: s.title === "$750K" ? 44 : 28,
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

  const blob = await pptx.write({ outputType: "blob" });
  return blob as Blob;
}

function downloadBlob(blob: Blob, fileName: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;
  a.rel = "noopener";
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

export function DeckToolbar() {
  const [exporting, setExporting] = useState<string | null>(null);

  const handlePdf = useCallback(() => {
    window.print();
  }, []);

  const handlePptx = useCallback(async () => {
    setExporting("pptx");
    try {
      const blob = await createPitchDeckPptxBlob();
      downloadBlob(blob, "5D-Labs-Pitch-Deck.pptx");
    } finally {
      setExporting(null);
    }
  }, []);

  const handleGdrive = useCallback(async () => {
    if (!GOOGLE_DRIVE_CLIENT_ID) {
      const w = window.open(
        "https://drive.google.com/drive/my-drive",
        "_blank",
        "noopener,noreferrer",
      );
      if (w) {
        window.print();
      }
      return;
    }

    setExporting("gdrive");
    try {
      await loadGoogleIdentityScript();
      const blob = await createPitchDeckPptxBlob();
      await new Promise<void>((resolve, reject) => {
        requestDriveFileAccessToken(
          GOOGLE_DRIVE_CLIENT_ID,
          (token, err) => {
            void (async () => {
              try {
                if (!token) {
                  throw new Error(err || "Google sign-in was cancelled.");
                }
                const result = await uploadPptxBlobToDrive(token, blob, {
                  folderId: GOOGLE_DRIVE_FOLDER_ID || undefined,
                });
                const link =
                  result.webViewLink ||
                  `https://drive.google.com/file/d/${result.id}/view`;
                window.open(link, "_blank", "noopener,noreferrer");
                resolve();
              } catch (e) {
                reject(e);
              }
            })();
          },
        );
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      window.alert(`Google Drive upload failed: ${msg}`);
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

          <ToolbarButton
            onClick={handleGdrive}
            title={
              GOOGLE_DRIVE_CLIENT_ID
                ? "Upload pitch deck PowerPoint to your Google Drive"
                : "Open Google Drive and print / save as PDF (legacy)"
            }
          >
            <AnimatePresence mode="wait">
              {exporting === "gdrive" ? (
                <motion.div
                  key="gdrive-spin"
                  className="w-4 h-4 border-2 border-cyan/30 border-t-cyan rounded-full animate-spin"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                />
              ) : (
                <motion.svg
                  key="gdrive-icon"
                  className="w-4 h-4"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                >
                  <path d="M7.71 3.5L1.15 15l3.47 6h4.94L3.01 9.5 7.71 3.5zm1.43 0l6.56 11.5H9.15l-3.47-6L7.71 3.5h1.43zm7.72 0L22.85 15l-3.47 6h-4.94l6.55-11.5L14.86 3.5z" />
                </motion.svg>
              )}
            </AnimatePresence>
            Google Drive
          </ToolbarButton>
        </motion.div>
      </div>

      {/* Print-specific styles — preserve dark theme, force animated states */}
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
            padding-top: 1rem !important;
            padding-bottom: 1rem !important;
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

          /* Force all Framer Motion whileInView elements visible */
          [style*="opacity: 0"],
          [style*="opacity:0"] {
            opacity: 1 !important;
          }
          [style*="transform"] {
            transform: none !important;
          }

          /* Force bar chart widths via CSS custom property */
          .print\\:!w-\\[var\\(--print-w\\)\\],
          .print\\:\\!w-\\[var\\(--print-w\\)\\] {
            width: var(--print-w) !important;
          }

          /* Tighter cover to avoid wasted page */
          section:first-of-type {
            min-height: auto !important;
            padding-top: 2rem !important;
            padding-bottom: 2rem !important;
          }

          /* Tighter page margins */
          @page {
            margin: 0.5in 0.4in;
          }
        }
      `}</style>
    </>
  );
}
