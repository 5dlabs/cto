"use client";

import { useCallback, useRef, useState } from "react";
import {
  ChevronDown,
  FileText,
  Loader2,
  Presentation,
  Printer,
  UploadCloud,
} from "lucide-react";
import { slides } from "@/lib/deck-content";
import { buildPitchDeckPdfBlob, type PdfExportDensity } from "@/lib/export-pdf";
import { buildPitchDeckPptxBlob } from "@/lib/export-pptx";
import {
  getGoogleOAuthClientId,
  requestDriveAccessToken,
  uploadBlobToDrive,
} from "@/lib/google-drive-upload";
import { cn } from "@/lib/utils";

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.rel = "noopener";
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function driveFileUrl(id: string, webViewLink?: string): string {
  if (webViewLink) return webViewLink;
  return `https://drive.google.com/file/d/${id}/view`;
}

export function DeckExportControls() {
  const [pptxBusy, setPptxBusy] = useState(false);
  const [pdfBusy, setPdfBusy] = useState(false);
  const [driveBusy, setDriveBusy] = useState(false);
  const pdfDetailsRef = useRef<HTMLDetailsElement>(null);
  const driveDetailsRef = useRef<HTMLDetailsElement>(null);
  const clientId = getGoogleOAuthClientId();

  const onPrintLayout = useCallback(() => {
    window.print();
  }, []);

  const onDownloadPdf = useCallback(async (density: PdfExportDensity) => {
    setPdfBusy(true);
    try {
      const blob = buildPitchDeckPdfBlob(slides, { density });
      const suffix = density === "compact" ? "" : "-readable";
      downloadBlob(blob, `5dlabs-pitch-deck${suffix}.pdf`);
      pdfDetailsRef.current?.removeAttribute("open");
    } catch (e) {
      console.error(e);
      alert("Could not build PDF. Try Print layout instead.");
    } finally {
      setPdfBusy(false);
    }
  }, []);

  const onDownloadPptx = useCallback(async () => {
    setPptxBusy(true);
    try {
      const blob = await buildPitchDeckPptxBlob(slides);
      downloadBlob(blob, "5dlabs-pitch-deck.pptx");
    } catch (e) {
      console.error(e);
      alert("Could not build PowerPoint file. Try again or use PDF export.");
    } finally {
      setPptxBusy(false);
    }
  }, []);

  const onSaveToDrive = useCallback(
    async (kind: "pdf-compact" | "pdf-readable" | "pptx") => {
      if (!clientId) return;
      setDriveBusy(true);
      try {
        const accessToken = await requestDriveAccessToken(clientId);
        let blob: Blob;
        let filename: string;
        let mimeType: string;
        if (kind === "pdf-compact") {
          blob = buildPitchDeckPdfBlob(slides, { density: "compact" });
          filename = "5dlabs-pitch-deck.pdf";
          mimeType = "application/pdf";
        } else if (kind === "pdf-readable") {
          blob = buildPitchDeckPdfBlob(slides, { density: "readable" });
          filename = "5dlabs-pitch-deck-readable.pdf";
          mimeType = "application/pdf";
        } else {
          blob = await buildPitchDeckPptxBlob(slides);
          filename = "5dlabs-pitch-deck.pptx";
          mimeType =
            "application/vnd.openxmlformats-officedocument.presentationml.presentation";
        }
        const result = await uploadBlobToDrive(blob, filename, mimeType, accessToken);
        const url = driveFileUrl(result.id, result.webViewLink);
        driveDetailsRef.current?.removeAttribute("open");
        const open = window.confirm(
          `Saved to your Google Drive as “${result.name}”.\n\nOpen the file in Drive now?`,
        );
        if (open) {
          window.open(url, "_blank", "noopener,noreferrer");
        }
      } catch (e) {
        console.error(e);
        const msg = e instanceof Error ? e.message : "Save to Drive failed.";
        alert(msg);
      } finally {
        setDriveBusy(false);
      }
    },
    [clientId],
  );

  return (
    <div className="flex flex-wrap items-center justify-end gap-1.5 sm:gap-2">
      <details
        ref={pdfDetailsRef}
        className="group relative"
      >
        <summary
          className={cn(
            "glass-badge inline-flex cursor-pointer list-none items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs font-medium text-foreground transition hover:bg-white/10 sm:gap-1.5 sm:px-3 sm:text-sm",
            "[&::-webkit-details-marker]:hidden",
          )}
        >
          {pdfBusy ? (
            <Loader2 className="size-4 shrink-0 animate-spin" aria-hidden />
          ) : (
            <FileText className="size-4 shrink-0" aria-hidden />
          )}
          <span className="hidden sm:inline">PDF</span>
          <ChevronDown className="size-3.5 opacity-70" aria-hidden />
        </summary>
        <div
          className={cn(
            "absolute bottom-full right-0 z-50 mb-2 flex min-w-[12.5rem] flex-col gap-0.5 rounded-xl border border-border/90 bg-card/95 p-1.5 shadow-xl backdrop-blur-md",
          )}
        >
          <button
            type="button"
            disabled={pdfBusy}
            onClick={() => void onDownloadPdf("compact")}
            className="rounded-lg px-3 py-2 text-left text-xs text-foreground hover:bg-white/10 sm:text-sm"
          >
            <span className="font-medium">Compact</span>
            <span className="block text-[0.7rem] text-muted-foreground sm:text-xs">
              Dark theme · large type · email
            </span>
          </button>
          <button
            type="button"
            disabled={pdfBusy}
            onClick={() => void onDownloadPdf("readable")}
            className="rounded-lg px-3 py-2 text-left text-xs text-foreground hover:bg-white/10 sm:text-sm"
          >
            <span className="font-medium">Readable</span>
            <span className="block text-[0.7rem] text-muted-foreground sm:text-xs">
              Same look · extra-large type
            </span>
          </button>
        </div>
      </details>

      <button
        type="button"
        onClick={onPrintLayout}
        className={cn(
          "glass-badge inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium text-foreground transition hover:bg-white/10 sm:px-3 sm:text-sm",
        )}
        title="Browser print — Save as PDF to match on-screen colors & layout"
      >
        <Printer className="size-4 shrink-0" aria-hidden />
        <span className="hidden sm:inline">Print</span>
      </button>

      <button
        type="button"
        onClick={onDownloadPptx}
        disabled={pptxBusy}
        className={cn(
          "glass-badge inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium text-foreground transition hover:bg-white/10 sm:px-3 sm:text-sm",
          pptxBusy && "pointer-events-none opacity-60",
        )}
        title="Download .pptx for PowerPoint"
      >
        {pptxBusy ? (
          <Loader2 className="size-4 shrink-0 animate-spin" aria-hidden />
        ) : (
          <Presentation className="size-4 shrink-0" aria-hidden />
        )}
        <span className="hidden sm:inline">PowerPoint</span>
      </button>

      <details
        ref={driveDetailsRef}
        className="group relative"
      >
        <summary
          className={cn(
            "glass-badge inline-flex cursor-pointer list-none items-center gap-1 rounded-lg px-2.5 py-1.5 text-xs font-medium text-foreground transition hover:bg-white/10 sm:gap-1.5 sm:px-3 sm:text-sm",
            "[&::-webkit-details-marker]:hidden",
            (!clientId || driveBusy) && "opacity-60",
          )}
          aria-disabled={!clientId}
        >
          {driveBusy ? (
            <Loader2 className="size-4 shrink-0 animate-spin" aria-hidden />
          ) : (
            <UploadCloud className="size-4 shrink-0" aria-hidden />
          )}
          <span className="hidden sm:inline">Save to Drive</span>
          <span className="sm:hidden">Drive</span>
          <ChevronDown className="size-3.5 opacity-70" aria-hidden />
        </summary>
        <div
          className={cn(
            "absolute bottom-full right-0 z-50 mb-2 flex w-[min(100vw-2rem,17rem)] flex-col gap-0.5 rounded-xl border border-border/90 bg-card/95 p-1.5 shadow-xl backdrop-blur-md",
          )}
        >
          {!clientId ? (
            <p className="px-3 py-2 text-xs leading-relaxed text-muted-foreground">
              Set{" "}
              <code className="rounded bg-muted px-1 py-0.5 text-[0.65rem] text-foreground">
                NEXT_PUBLIC_GOOGLE_OAUTH_CLIENT_ID
              </code>{" "}
              and enable the Drive API (scope{" "}
              <code className="rounded bg-muted px-1 py-0.5 text-[0.65rem]">drive.file</code>
              ).
            </p>
          ) : (
            <>
              <button
                type="button"
                disabled={driveBusy}
                onClick={() => void onSaveToDrive("pdf-compact")}
                className="rounded-lg px-3 py-2 text-left text-xs text-foreground hover:bg-white/10 sm:text-sm"
              >
                <span className="font-medium">PDF · Compact</span>
                <span className="block text-[0.7rem] text-muted-foreground sm:text-xs">
                  To your Drive
                </span>
              </button>
              <button
                type="button"
                disabled={driveBusy}
                onClick={() => void onSaveToDrive("pdf-readable")}
                className="rounded-lg px-3 py-2 text-left text-xs text-foreground hover:bg-white/10 sm:text-sm"
              >
                <span className="font-medium">PDF · Readable</span>
                <span className="block text-[0.7rem] text-muted-foreground sm:text-xs">
                  To your Drive
                </span>
              </button>
              <button
                type="button"
                disabled={driveBusy}
                onClick={() => void onSaveToDrive("pptx")}
                className="rounded-lg px-3 py-2 text-left text-xs text-foreground hover:bg-white/10 sm:text-sm"
              >
                <span className="font-medium">PowerPoint</span>
                <span className="block text-[0.7rem] text-muted-foreground sm:text-xs">
                  Open in Slides from Drive
                </span>
              </button>
            </>
          )}
        </div>
      </details>
    </div>
  );
}
