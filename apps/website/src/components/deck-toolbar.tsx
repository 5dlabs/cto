"use client";

import { useState, useCallback, useRef } from "react";
import { motion } from "framer-motion";
import {
  ChevronDown,
  FileText,
  Loader2,
  Presentation,
  Printer,
  UploadCloud,
} from "lucide-react";

import {
  loadGoogleIdentityScript,
  requestDriveFileAccessToken,
  uploadBlobToDrive,
} from "@/lib/google-drive-pitch";
import { createPitchDeckPdfBlob } from "@/lib/export-pdf";
import { createPitchDeckPptxBlob } from "@/lib/export-pptx";

/* ─── env (Next.js only inlines literal process.env.NEXT_PUBLIC_* access) ─── */

const GOOGLE_DRIVE_CLIENT_ID =
  process.env.NEXT_PUBLIC_GOOGLE_DRIVE_CLIENT_ID ||
  process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID ||
  process.env.NEXT_PUBLIC_GAPI_CLIENT_ID ||
  process.env.NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID ||
  "";

const GOOGLE_DRIVE_FOLDER_ID =
  process.env.NEXT_PUBLIC_GOOGLE_DRIVE_FOLDER_ID ||
  process.env.NEXT_PUBLIC_DRIVE_FOLDER_ID ||
  process.env.NEXT_PUBLIC_GOOGLE_DRIVE_PARENT_ID ||
  "";

type PdfDensity = "compact" | "readable";

/* ─── helpers ─── */

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

/* ─── toolbar component ─── */

const btnClass =
  "inline-flex cursor-pointer list-none items-center gap-1.5 rounded-lg px-3 py-2.5 text-sm font-medium text-foreground transition hover:bg-white/10 border border-border/50 bg-card/60 backdrop-blur-md [&::-webkit-details-marker]:hidden";

const dropdownClass =
  "absolute bottom-full right-0 z-50 mb-2 flex min-w-[13rem] flex-col gap-0.5 rounded-xl border border-border/90 bg-card/95 p-1.5 shadow-xl backdrop-blur-md";

const dropItemClass =
  "rounded-lg px-3 py-2 text-left text-sm text-foreground hover:bg-white/10";

export function DeckToolbar() {
  const [busy, setBusy] = useState<string | null>(null);
  const pdfRef = useRef<HTMLDetailsElement>(null);
  const driveRef = useRef<HTMLDetailsElement>(null);

  const onPrint = useCallback(() => window.print(), []);

  const onDownloadPdf = useCallback(async (density: PdfDensity) => {
    setBusy("pdf");
    try {
      const blob = await createPitchDeckPdfBlob(density);
      const suffix = density === "compact" ? "" : "-readable";
      downloadBlob(blob, `5D-Labs-Pitch-Deck${suffix}.pdf`);
      pdfRef.current?.removeAttribute("open");
    } catch (e) {
      console.error(e);
      window.alert("Could not build PDF. Try Print layout instead.");
    } finally {
      setBusy(null);
    }
  }, []);

  const onDownloadPptx = useCallback(async () => {
    setBusy("pptx");
    try {
      const blob = await createPitchDeckPptxBlob();
      downloadBlob(blob, "5D-Labs-Pitch-Deck.pptx");
    } catch (e) {
      console.error(e);
      window.alert("Could not build PowerPoint file. Try PDF export.");
    } finally {
      setBusy(null);
    }
  }, []);

  const onSaveToDrive = useCallback(
    async (kind: "pdf-compact" | "pdf-readable" | "pptx") => {
      if (!GOOGLE_DRIVE_CLIENT_ID) return;
      setBusy("gdrive");
      try {
        await loadGoogleIdentityScript();
        const token = await new Promise<string>((resolve, reject) => {
          requestDriveFileAccessToken(GOOGLE_DRIVE_CLIENT_ID, (t, err) => {
            if (t) resolve(t);
            else reject(new Error(err || "Google sign-in was cancelled."));
          });
        });

        let blob: Blob;
        let fileName: string;
        let mimeType: string;
        if (kind === "pptx") {
          blob = await createPitchDeckPptxBlob();
          fileName = "5D-Labs-Pitch-Deck.pptx";
          mimeType = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
        } else {
          const density = kind === "pdf-compact" ? "compact" : "readable";
          blob = await createPitchDeckPdfBlob(density);
          const suffix = density === "compact" ? "" : "-readable";
          fileName = `5D-Labs-Pitch-Deck${suffix}.pdf`;
          mimeType = "application/pdf";
        }

        const result = await uploadBlobToDrive(
          token,
          blob,
          fileName,
          mimeType,
          { folderId: GOOGLE_DRIVE_FOLDER_ID || undefined },
        );
        const link = result.webViewLink || `https://drive.google.com/file/d/${result.id}/view`;
        driveRef.current?.removeAttribute("open");

        const shouldOpen = window.confirm(
          `Saved to your Google Drive as "${result.name}".\n\nOpen the file in Drive now?`,
        );
        if (shouldOpen) {
          window.open(link, "_blank", "noopener,noreferrer");
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        window.alert(`Google Drive upload failed: ${msg}`);
      } finally {
        setBusy(null);
      }
    },
    [],
  );

  return (
    <>
      <div className="fixed bottom-0 left-0 right-0 z-50 flex justify-center pb-4 pointer-events-none print:hidden">
        <motion.div
          className="flex items-center gap-1.5 p-2 rounded-2xl border border-border/50 bg-background/80 backdrop-blur-xl shadow-2xl pointer-events-auto"
          initial={{ y: 40, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 1, duration: 0.6, ease: [0.25, 0.4, 0, 1] }}
        >
          {/* PDF dropdown */}
          <details ref={pdfRef} className="relative">
            <summary className={btnClass}>
              {busy === "pdf" ? (
                <Loader2 className="size-4 shrink-0 animate-spin" />
              ) : (
                <FileText className="size-4 shrink-0" />
              )}
              <span className="hidden sm:inline">PDF</span>
              <ChevronDown className="size-3.5 opacity-70" />
            </summary>
            <div className={dropdownClass}>
              <button
                type="button"
                disabled={busy === "pdf"}
                onClick={() => void onDownloadPdf("compact")}
                className={dropItemClass}
              >
                <span className="font-medium">Compact</span>
                <span className="block text-xs text-muted-foreground">
                  Dark theme · large type · email
                </span>
              </button>
              <button
                type="button"
                disabled={busy === "pdf"}
                onClick={() => void onDownloadPdf("readable")}
                className={dropItemClass}
              >
                <span className="font-medium">Readable</span>
                <span className="block text-xs text-muted-foreground">
                  Same look · extra-large type
                </span>
              </button>
            </div>
          </details>

          {/* Print */}
          <button type="button" onClick={onPrint} className={btnClass} title="Browser print">
            <Printer className="size-4 shrink-0" />
            <span className="hidden sm:inline">Print</span>
          </button>

          {/* PowerPoint download */}
          <button
            type="button"
            onClick={() => void onDownloadPptx()}
            disabled={busy === "pptx"}
            className={`${btnClass} ${busy === "pptx" ? "pointer-events-none opacity-60" : ""}`}
            title="Download .pptx"
          >
            {busy === "pptx" ? (
              <Loader2 className="size-4 shrink-0 animate-spin" />
            ) : (
              <Presentation className="size-4 shrink-0" />
            )}
            <span className="hidden sm:inline">PowerPoint</span>
          </button>

          {/* Save to Drive dropdown */}
          <details ref={driveRef} className="relative">
            <summary
              className={`${btnClass} ${!GOOGLE_DRIVE_CLIENT_ID || busy === "gdrive" ? "opacity-60" : ""}`}
            >
              {busy === "gdrive" ? (
                <Loader2 className="size-4 shrink-0 animate-spin" />
              ) : (
                <UploadCloud className="size-4 shrink-0" />
              )}
              <span className="hidden sm:inline">Save to Drive</span>
              <span className="sm:hidden">Drive</span>
              <ChevronDown className="size-3.5 opacity-70" />
            </summary>
            <div className={dropdownClass}>
              {!GOOGLE_DRIVE_CLIENT_ID ? (
                <p className="px-3 py-2 text-xs leading-relaxed text-muted-foreground">
                  Set{" "}
                  <code className="rounded bg-muted px-1 py-0.5 text-[0.65rem] text-foreground">
                    NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID
                  </code>{" "}
                  to enable Drive uploads.
                </p>
              ) : (
                <>
                  <button
                    type="button"
                    disabled={busy === "gdrive"}
                    onClick={() => void onSaveToDrive("pdf-compact")}
                    className={dropItemClass}
                  >
                    <span className="font-medium">PDF · Compact</span>
                    <span className="block text-xs text-muted-foreground">To your Drive</span>
                  </button>
                  <button
                    type="button"
                    disabled={busy === "gdrive"}
                    onClick={() => void onSaveToDrive("pdf-readable")}
                    className={dropItemClass}
                  >
                    <span className="font-medium">PDF · Readable</span>
                    <span className="block text-xs text-muted-foreground">To your Drive</span>
                  </button>
                  <button
                    type="button"
                    disabled={busy === "gdrive"}
                    onClick={() => void onSaveToDrive("pptx")}
                    className={dropItemClass}
                  >
                    <span className="font-medium">PowerPoint</span>
                    <span className="block text-xs text-muted-foreground">
                      Open in Slides from Drive
                    </span>
                  </button>
                </>
              )}
            </div>
          </details>
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
