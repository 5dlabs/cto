import { jsPDF } from "jspdf";
import autoTable from "jspdf-autotable";
import type { DeckSlide } from "./deck-content";
import { DECK_META } from "./deck-content";

/** jspdf-autotable attaches this after each `autoTable` call */
type DocWithTable = jsPDF & { lastAutoTable?: { finalY: number } };

export type PdfExportDensity = "compact" | "readable";

/** Align with deck globals: dark blue-black + cyan primary */
const THEME = {
  bg: [10, 12, 22] as [number, number, number],
  topBar: [6, 182, 212] as [number, number, number],
  text: [248, 250, 252] as [number, number, number],
  muted: [163, 163, 163] as [number, number, number],
  dim: [120, 130, 150] as [number, number, number],
  accent: [34, 211, 238] as [number, number, number],
  calloutBg: [15, 40, 45] as [number, number, number],
  calloutText: [204, 251, 241] as [number, number, number],
  footnote: [140, 150, 170] as [number, number, number],
};

const MM = {
  pageW: 297,
  pageH: 210,
};

/** Standard PDF fonts often mangle Unicode arrows/dashes — normalize for reliable output */
function pdfText(s: string): string {
  return s
    .replace(/\u2192/g, " -> ")
    .replace(/\u2014/g, " - ")
    .replace(/\u2013/g, "-") /* en dash */
    .replace(/\u2248/g, "~") /* approx equal */
    .replace(/\u00a0/g, " ");
}

/** Bottom inset — slide # moved to header, so we can use space closer to the fold */
const BOTTOM_SAFE_MM = 10;

function newDoc(): jsPDF {
  return new jsPDF({
    orientation: "landscape",
    unit: "mm",
    format: [MM.pageW, MM.pageH],
  });
}

function drawSlideChrome(doc: jsPDF): void {
  doc.setFillColor(...THEME.bg);
  doc.rect(0, 0, MM.pageW, MM.pageH, "F");
  doc.setFillColor(...THEME.topBar);
  doc.rect(0, 0, MM.pageW, 1.4, "F");
}

function ensureSpace(
  doc: jsPDF,
  y: number,
  needMm: number,
  margin: number,
): number {
  if (y + needMm <= MM.pageH - margin - BOTTOM_SAFE_MM) return y;
  doc.addPage();
  drawSlideChrome(doc);
  return margin + 6;
}

/**
 * Line height in mm.
 * - `displayHeadline`: large bold titles need looser leading or wrapped lines collide visually.
 * - `relaxed`: body / bullets.
 */
function lineHeightMm(
  fontSizePt: number,
  relaxed = false,
  displayHeadline = false,
): number {
  if (displayHeadline) {
    return fontSizePt * 0.42;
  }
  const factor = relaxed ? 0.52 : 0.44;
  return fontSizePt * factor;
}

/** Wrapped block height (mm) — sets font for accurate width measurement */
function wrappedBlockHeightMm(
  doc: jsPDF,
  text: string,
  maxW: number,
  fontSizePt: number,
  relaxed: boolean,
  style: "normal" | "bold" | "italic" = "normal",
): number {
  const font =
    style === "bold" ? "bold" : style === "italic" ? "italic" : "normal";
  doc.setFont("helvetica", font);
  doc.setFontSize(fontSizePt);
  const lines = doc.splitTextToSize(text, maxW);
  return lines.length * lineHeightMm(fontSizePt, relaxed, false);
}

function writeLines(
  doc: jsPDF,
  text: string,
  x: number,
  y: number,
  maxW: number,
  fontSizePt: number,
  opts: {
    bold?: boolean;
    color: [number, number, number];
    relaxed?: boolean;
    /** Large slide titles — use looser line height so wrapped lines do not overlap */
    displayHeadline?: boolean;
  },
): number {
  doc.setFont("helvetica", opts.bold ? "bold" : "normal");
  doc.setFontSize(fontSizePt);
  doc.setTextColor(...opts.color);
  const lh = lineHeightMm(
    fontSizePt,
    opts.relaxed ?? false,
    opts.displayHeadline ?? false,
  );
  const lines = doc.splitTextToSize(text, maxW);
  let yy = y;
  for (const line of lines) {
    doc.text(line, x, yy);
    yy += lh;
  }
  return yy;
}

function writeCalloutBox(
  doc: jsPDF,
  text: string,
  x: number,
  y: number,
  maxW: number,
  fontSizePt: number,
  margin: number,
): number {
  doc.setFont("helvetica", "italic");
  doc.setFontSize(fontSizePt);
  const lh = lineHeightMm(fontSizePt, true);
  const padX = 5.5;
  const padY = 5.5;
  const lines = doc.splitTextToSize(text, maxW - padX * 2);
  const boxH = lines.length * lh + padY * 2;
  const boxW = maxW;
  y = ensureSpace(doc, y, boxH + 4, margin);
  doc.setFillColor(...THEME.calloutBg);
  doc.setDrawColor(45, 212, 191);
  doc.setLineWidth(0.25);
  doc.roundedRect(x, y, boxW, boxH, 1.5, 1.5, "FD");
  doc.setTextColor(...THEME.calloutText);
  let yy = y + padY + fontSizePt * 0.32;
  for (const line of lines) {
    doc.text(line, x + padX, yy);
    yy += lh;
  }
  doc.setFont("helvetica", "normal");
  /* Extra air before footnote / next block */
  return y + boxH + 5;
}

/**
 * Vector PDF (jsPDF) — dark theme + large type to match deck vibe without rasterizing the DOM
 * (html2canvas looks “prettier” but blurs text and balloons file size).
 */
export function buildPitchDeckPdfBlob(
  slides: DeckSlide[],
  options?: { density?: PdfExportDensity },
): Blob {
  const density = options?.density ?? "compact";
  /** Margins: slightly more breathing room on readable */
  const margin = density === "compact" ? 9 : 12;
  const maxW = MM.pageW - 2 * margin;
  const headerMaxW = maxW * 0.72;

  /** pt — large display sizes to fill landscape A4 */
  const sizes =
    density === "compact"
      ? {
          meta: 13,
          label: 15,
          eyebrow: 17,
          headline: 34,
          sub: 21,
          body: 20,
          small: 14,
          table: 15,
        }
      : {
          meta: 14,
          label: 16,
          eyebrow: 18,
          headline: 34,
          sub: 21,
          body: 20,
          small: 14,
          table: 16,
        };

  /** Extra space (mm) after headline when bullets follow — both densities need clear separation */
  const headlineToBulletGapMm = density === "compact" ? 8 : 10;

  /** Vertical gaps after blocks (mm) — tuned to match web deck spacing */
  const gap = {
    afterMeta: 4,
    afterLabel: 3,
    /** Space after cyan eyebrow before big headline (e.g. "The shift" -> title); +~2pt vs label-only gap */
    afterEyebrow: 3.5,
    afterHeadline: 3,
    afterSub: 3,
    afterStats: 1.75,
    afterBullets: 1,
    afterTable: 4,
    afterCallout: 2,
    afterCta: 1.5,
    bulletBetween: 0.85,
  };

  const doc = newDoc();
  drawSlideChrome(doc);

  const header = pdfText(
    `${DECK_META.company} · ${DECK_META.round} · ${DECK_META.confidential}`,
  );

  for (let si = 0; si < slides.length; si++) {
    const slide = slides[si];
    if (si > 0) {
      doc.addPage();
      drawSlideChrome(doc);
    }

    /** Per-slide type scale — cover is dense; shrink slightly so footnote stays on one page */
    const slideSizes = { ...sizes };
    if (slide.id === "cover") {
      if (density === "compact") {
        Object.assign(slideSizes, {
          label: 14,
          eyebrow: 15,
          headline: 30,
          sub: 19,
          body: 18,
          small: 11,
        });
      } else {
        Object.assign(slideSizes, {
          label: 14,
          eyebrow: 16,
          headline: 30,
          sub: 19,
          body: 18,
          small: 11,
        });
      }
    }

    let y = margin + 1.5;

    /* Header row: meta left (wrapped), slide counter top-right — avoids footnote overlap at bottom */
    doc.setFont("helvetica", "normal");
    doc.setFontSize(slideSizes.meta);
    doc.setTextColor(...THEME.dim);
    const headerLines = doc.splitTextToSize(header, headerMaxW);
    const metaLh = lineHeightMm(slideSizes.meta);
    let hy = y;
    for (let hi = 0; hi < headerLines.length; hi++) {
      doc.text(headerLines[hi], margin, hy);
      if (hi === 0) {
        doc.setTextColor(...THEME.muted);
        doc.text(`${si + 1} / ${slides.length}`, MM.pageW - margin, hy, {
          align: "right",
        });
        doc.setTextColor(...THEME.dim);
      }
      hy += metaLh;
    }
    y = hy + gap.afterMeta;

    y =
      writeLines(doc, pdfText(slide.label.toUpperCase()), margin, y, maxW, slideSizes.label, {
        color: THEME.muted,
      }) +
      gap.afterLabel +
      /* Slides with no eyebrow: small extra gap so label doesn't crowd the title */
      (!slide.eyebrow ? 2 : 0);

    if (slide.eyebrow) {
      y =
        writeLines(doc, pdfText(slide.eyebrow), margin, y, maxW, slideSizes.eyebrow, {
          bold: true,
          color: THEME.accent,
        }) + gap.afterEyebrow;
    }

    const gapAfterHeadline =
      slide.bullets?.length && !slide.subhead
        ? headlineToBulletGapMm
        : gap.afterHeadline;
    y =
      writeLines(doc, pdfText(slide.headline), margin, y, maxW, slideSizes.headline, {
        bold: true,
        color: THEME.text,
        displayHeadline: true,
      }) + gapAfterHeadline;

    if (slide.subhead) {
      const gapAfterSub =
        slide.id === "cover" && slide.stats?.length ? 0.85 : gap.afterSub;
      y =
        writeLines(doc, pdfText(slide.subhead), margin, y, maxW, slideSizes.sub, {
          color: THEME.text,
          relaxed: true,
        }) + gapAfterSub;
    }

    if (slide.stats?.length) {
      /* Middle dot reads cleanly in PDF; avoids wide em-dash spacing from pdfText() */
      const statBlock = pdfText(
        slide.stats.map((st) => `${st.value} · ${st.label}`).join("\n"),
      );
      doc.setFont("helvetica", "normal");
      doc.setFontSize(slideSizes.body + 2);
      const statH = wrappedBlockHeightMm(
        doc,
        statBlock,
        maxW,
        slideSizes.body + 2,
        true,
        "bold",
      );
      y = ensureSpace(doc, y, statH + 4, margin);
      y =
        writeLines(doc, statBlock, margin, y, maxW, slideSizes.body + 2, {
          bold: true,
          color: THEME.accent,
          relaxed: true,
        }) +
        (slide.id === "cover" && slide.callout
          ? gap.afterStats + 2.75
          : gap.afterStats);
    }

    if (slide.bullets?.length) {
      doc.setFont("helvetica", "normal");
      for (const b of slide.bullets) {
        const block = pdfText(`•  ${b}`);
        doc.setFontSize(slideSizes.body);
        const h = wrappedBlockHeightMm(doc, block, maxW, slideSizes.body, true);
        y = ensureSpace(doc, y, h + gap.bulletBetween + 2, margin);
        y =
          writeLines(doc, block, margin, y, maxW, slideSizes.body, {
            color: THEME.text,
            relaxed: true,
          }) + gap.bulletBetween;
      }
      y += gap.afterBullets;
    }

    if (slide.table) {
      const estTableH = 8 + slide.table.rows.length * 6;
      y = ensureSpace(doc, y, Math.min(estTableH, 55), margin);
      autoTable(doc, {
        startY: y,
        head: [slide.table.headers.map((h) => pdfText(h))],
        body: slide.table.rows.map((row) => row.map((cell) => pdfText(cell))),
        margin: { left: margin, right: margin },
        styles: {
          fontSize: slideSizes.table,
          cellPadding: 2.8,
          textColor: THEME.text,
          fillColor: THEME.bg,
          lineColor: [55, 65, 85],
          lineWidth: 0.15,
        },
        headStyles: {
          fillColor: [22, 35, 48],
          textColor: THEME.accent,
          fontStyle: "bold",
        },
        alternateRowStyles: { fillColor: [14, 18, 30] },
        theme: "grid",
      });
      const dt = doc as DocWithTable;
      y = (dt.lastAutoTable?.finalY ?? y + 40) + gap.afterTable;
    }

    if (slide.callout) {
      y = writeCalloutBox(doc, pdfText(slide.callout), margin, y, maxW, slideSizes.body, margin);
    }

    if (slide.cta) {
      const ctaText = pdfText(`${slide.cta.label} → ${slide.cta.href}`);
      doc.setFont("helvetica", "bold");
      const ctaH = wrappedBlockHeightMm(
        doc,
        ctaText,
        maxW,
        slideSizes.body,
        false,
        "bold",
      );
      y = ensureSpace(doc, y, ctaH + 2, margin);
      y =
        writeLines(doc, ctaText, margin, y, maxW, slideSizes.body, {
          bold: true,
          color: THEME.accent,
        }) + gap.afterCta;
    }

    if (slide.footnote) {
      const foot = pdfText(slide.footnote);
      const maxY = MM.pageH - margin - BOTTOM_SAFE_MM;
      let fnFs = slideSizes.small;
      let fnH = wrappedBlockHeightMm(doc, foot, maxW, fnFs, true);
      while (y + fnH > maxY && fnFs > 7.5) {
        fnFs -= 0.5;
        fnH = wrappedBlockHeightMm(doc, foot, maxW, fnFs, true);
      }
      y = ensureSpace(doc, y, fnH + 3, margin);
      y =
        writeLines(doc, foot, margin, y, maxW, fnFs, {
          color: THEME.footnote,
          relaxed: true,
        }) + 1;
    }
  }

  return doc.output("blob");
}
