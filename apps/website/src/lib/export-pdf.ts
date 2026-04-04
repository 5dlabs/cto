/**
 * Production PDF exporter for the 5D Labs pitch deck.
 * Dark theme, landscape 16:9, jsPDF + autotable.
 * Chart approximations for market rings, competition quadrant, fund bars, etc.
 */
import type { jsPDF } from "jspdf";
import type { DeckSlideStat, SlideTable } from "./deck-content";
import {
  BRAND, THEME, SLIDES,
  TRACTION_METRICS, MARKET_RINGS, COMPETITION_QUADRANT,
  FUND_ALLOCATION, PIPELINE_STAGES, FUNNEL_STAGES,
} from "./deck-content";

/* ─── Types ─── */

type RGB = [number, number, number];
type DocWithTable = jsPDF & { lastAutoTable?: { finalY: number } };
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AutoTableFn = (doc: jsPDF, opts: any) => void;

/* ─── Constants ─── */

const PG = { w: 297, h: 210 };
const SAFE_BOTTOM = 10;
const T = THEME.rgb;

/* ─── Unicode normalization — jsPDF chokes on smart quotes / dashes ─── */

function safe(s: string): string {
  return s
    .replace(/\u2192/g, "->")
    .replace(/\u2014/g, " - ")
    .replace(/\u2013/g, "-")
    .replace(/\u2019/g, "'")
    .replace(/\u2018/g, "'")
    .replace(/\u201c/g, '"')
    .replace(/\u201d/g, '"')
    .replace(/\u00b7/g, "*")
    .replace(/\u00a0/g, " ");
}

/* ─── Drawing helpers ─── */

function drawChrome(doc: jsPDF): void {
  doc.setFillColor(...T.bg);
  doc.rect(0, 0, PG.w, PG.h, "F");
  doc.setFillColor(...T.topBar);
  doc.rect(0, 0, PG.w, 1.4, "F");
}

function lh(fs: number, relaxed = false, display = false): number {
  if (display) return fs * 0.42;
  return fs * (relaxed ? 0.44 : 0.40);
}

function ensureSpace(doc: jsPDF, y: number, need: number, margin: number): number {
  if (y + need <= PG.h - margin - SAFE_BOTTOM) return y;
  doc.addPage();
  drawChrome(doc);
  return margin + 6;
}

function wrappedH(
  doc: jsPDF,
  text: string,
  maxW: number,
  fs: number,
  relaxed: boolean,
  style: "normal" | "bold" = "normal",
): number {
  doc.setFont("helvetica", style);
  doc.setFontSize(fs);
  const lines: string[] = doc.splitTextToSize(text, maxW);
  return lines.length * lh(fs, relaxed);
}

function writeLines(
  doc: jsPDF,
  text: string,
  x: number,
  y: number,
  maxW: number,
  fs: number,
  opts: { bold?: boolean; color: RGB; relaxed?: boolean; display?: boolean },
): number {
  doc.setFont("helvetica", opts.bold ? "bold" : "normal");
  doc.setFontSize(fs);
  doc.setTextColor(...opts.color);
  const step = lh(fs, opts.relaxed, opts.display);
  const lines: string[] = doc.splitTextToSize(text, maxW);
  let yy = y;
  for (const line of lines) {
    doc.text(line, x, yy);
    yy += step;
  }
  return yy;
}

function writeCalloutBox(
  doc: jsPDF,
  text: string,
  x: number,
  y: number,
  maxW: number,
  fs: number,
  margin: number,
): number {
  doc.setFont("helvetica", "italic");
  doc.setFontSize(fs);
  const step = lh(fs, true);
  const padX = 5.5;
  const padY = 5.5;
  const lines: string[] = doc.splitTextToSize(text, maxW - padX * 2);
  const boxH = lines.length * step + padY * 2;
  y = ensureSpace(doc, y, boxH + 4, margin);
  doc.setFillColor(...T.calloutBg);
  doc.setDrawColor(45, 212, 191);
  doc.setLineWidth(0.25);
  doc.roundedRect(x, y, maxW, boxH, 1.5, 1.5, "FD");
  doc.setTextColor(...T.calloutText);
  let yy = y + padY + fs * 0.32;
  for (const line of lines) {
    doc.text(line, x + padX, yy);
    yy += step;
  }
  doc.setFont("helvetica", "normal");
  return y + boxH + 5;
}

/* ─── Standard content renderers ─── */

function renderBullets(
  doc: jsPDF,
  bullets: string[],
  margin: number,
  y: number,
  maxW: number,
  fs: number,
): number {
  for (const b of bullets) {
    const block = safe(`\u2022  ${b}`);
    const h = wrappedH(doc, block, maxW, fs, true);
    y = ensureSpace(doc, y, h + 2, margin);
    y = writeLines(doc, block, margin, y, maxW, fs, { color: T.text, relaxed: true }) + 2;
  }
  return y + 2;
}

function renderStats(
  doc: jsPDF,
  stats: DeckSlideStat[],
  margin: number,
  y: number,
  maxW: number,
  fs: number,
): number {
  const block = safe(stats.map((st) => `${st.value} * ${st.label}`).join("\n"));
  const h = wrappedH(doc, block, maxW, fs + 2, true, "bold");
  y = ensureSpace(doc, y, h + 4, margin);
  y = writeLines(doc, block, margin, y, maxW, fs + 2, { bold: true, color: T.accent, relaxed: true }) + 2;
  return y;
}

function renderTable(
  doc: jsPDF,
  table: SlideTable,
  margin: number,
  y: number,
  maxW: number,
  tableFs: number,
  autoTable: AutoTableFn,
): number {
  const estH = 8 + table.rows.length * 6;
  y = ensureSpace(doc, y, Math.min(estH, 55), margin);

  /* Monkey-patch addPage so autotable overflow pages get dark chrome */
  const origAddPage = doc.addPage.bind(doc);
  doc.addPage = (...args: Parameters<typeof doc.addPage>) => {
    const result = origAddPage(...args);
    drawChrome(doc);
    return result;
  };

  autoTable(doc, {
    startY: y,
    head: [table.headers.map(safe)],
    body: table.rows.map((row) => row.map(safe)),
    margin: { left: margin, right: margin, top: margin + 6 },
    tableWidth: maxW,
    styles: {
      fontSize: tableFs,
      cellPadding: 2.8,
      textColor: T.text,
      fillColor: [12, 16, 28] as RGB,
      lineColor: [35, 45, 65] as RGB,
      lineWidth: 0.15,
    },
    headStyles: {
      fillColor: [18, 30, 45] as RGB,
      textColor: T.accent,
      fontStyle: "bold",
    },
    alternateRowStyles: { fillColor: [16, 20, 34] as RGB },
    theme: "striped",
  });

  /* Restore original addPage */
  doc.addPage = origAddPage;
  const dt = doc as DocWithTable;
  return (dt.lastAutoTable?.finalY ?? y + 40) + 4;
}

/* ─── Chart approximations ─── */

function drawTractionStats(doc: jsPDF, margin: number, y: number, maxW: number): number {
  const metrics = TRACTION_METRICS;
  const cols = metrics.length;
  const gap = 4;
  const boxW = (maxW - gap * (cols - 1)) / cols;
  const boxH = 40;

  y = ensureSpace(doc, y, boxH + 4, margin);

  for (let i = 0; i < cols; i++) {
    const m = metrics[i];
    const bx = margin + i * (boxW + gap);

    doc.setFillColor(...T.calloutBg);
    doc.setDrawColor(45, 212, 191);
    doc.setLineWidth(0.2);
    doc.roundedRect(bx, y, boxW, boxH, 1.5, 1.5, "FD");

    doc.setFont("helvetica", "bold");
    doc.setFontSize(22);
    doc.setTextColor(...T.accent);
    doc.text(safe(m.value), bx + boxW / 2, y + 14, { align: "center" });

    doc.setFont("helvetica", "bold");
    doc.setFontSize(11);
    doc.setTextColor(...T.text);
    doc.text(safe(m.label), bx + boxW / 2, y + 22, { align: "center" });

    doc.setFont("helvetica", "normal");
    doc.setFontSize(8);
    doc.setTextColor(...T.muted);
    const noteLines: string[] = doc.splitTextToSize(safe(m.note), boxW - 6);
    let ny = y + 28;
    for (const nl of noteLines) {
      doc.text(nl, bx + boxW / 2, ny, { align: "center" });
      ny += 3;
    }
  }

  return y + boxH + 5;
}

function drawMarketRings(doc: jsPDF, margin: number, y: number, maxW: number): number {
  const rings = MARKET_RINGS;
  const heights = [100, 64, 28];
  const widths = [maxW * 0.88, maxW * 0.60, maxW * 0.32];
  const fills: RGB[] = [[18, 28, 42], [20, 35, 50], [25, 45, 60]];
  const borders: RGB[] = [[50, 90, 130], [60, 130, 170], T.accent];
  const cx = margin + maxW / 2;

  y = ensureSpace(doc, y, heights[0] + 8, margin);
  const baseY = y;

  /* Draw all fills first (large to small) */
  for (let i = 0; i < rings.length; i++) {
    const w = widths[i];
    const h = heights[i];
    const rx = cx - w / 2;
    const ry = baseY + (heights[0] - h) / 2;

    doc.setFillColor(...fills[i]);
    doc.setDrawColor(...borders[i]);
    doc.setLineWidth(i === 2 ? 0.5 : 0.3);
    doc.roundedRect(rx, ry, w, h, 3, 3, "FD");
  }

  /* Draw text AFTER all fills so nothing is covered */
  for (let i = 0; i < rings.length; i++) {
    const h = heights[i];
    const ry = baseY + (heights[0] - h) / 2;

    /* Position text in the visible strip between this ring and the next inner one */
    let labelY: number;
    let descY: number;
    if (i < rings.length - 1) {
      const innerH = heights[i + 1];
      const innerRy = baseY + (heights[0] - innerH) / 2;
      const stripH = innerRy - ry;
      labelY = ry + stripH * 0.35;
      descY = ry + stripH * 0.65;
    } else {
      /* Innermost ring — center */
      labelY = ry + h * 0.38;
      descY = ry + h * 0.62;
    }

    const labelFs = i === 2 ? 16 : i === 1 ? 13 : 12;
    const descFs = i === 2 ? 11 : 9;

    doc.setFont("helvetica", "bold");
    doc.setFontSize(labelFs);
    doc.setTextColor(...T.accent);
    doc.text(safe(`${rings[i].label}: ${rings[i].value}`), cx, labelY, { align: "center" });

    doc.setFont("helvetica", "normal");
    doc.setFontSize(descFs);
    doc.setTextColor(...T.text);
    doc.text(safe(rings[i].description), cx, descY, { align: "center" });
  }

  return baseY + heights[0] + 6;
}

function drawCompetitionQuadrant(doc: jsPDF, margin: number, y: number, maxW: number): number {
  const cells = COMPETITION_QUADRANT;
  const cellW = maxW / 2;
  const cellH = 38;
  const totalH = cellH * 2;

  y = ensureSpace(doc, y, totalH + 4, margin);

  for (let i = 0; i < 4; i++) {
    const row = Math.floor(i / 2);
    const col = i % 2;
    const cx = margin + col * cellW;
    const cy = y + row * cellH;
    const cell = cells[i];
    const isHl = !!cell.highlight;

    const bgColor: RGB = isHl ? [15, 35, 50] : [14, 18, 30];
    const borderColor: RGB = isHl ? T.accent : [55, 65, 85];
    doc.setFillColor(...bgColor);
    doc.setDrawColor(...borderColor);
    doc.setLineWidth(isHl ? 0.5 : 0.15);
    doc.rect(cx, cy, cellW, cellH, "FD");

    const lblColor: RGB = isHl ? T.accent : T.muted;
    doc.setFont("helvetica", "bold");
    doc.setFontSize(9);
    doc.setTextColor(...lblColor);
    doc.text(safe(cell.label), cx + 5, cy + 7);

    const txtColor: RGB = isHl ? T.accent : T.text;
    doc.setFont("helvetica", isHl ? "bold" : "normal");
    doc.setFontSize(isHl ? 14 : 11);
    doc.setTextColor(...txtColor);
    let py = cy + 15;
    for (const p of cell.players) {
      doc.text(safe(p), cx + 5, py);
      py += isHl ? 7 : 5.5;
    }
  }

  return y + totalH + 4;
}

function drawFundBars(doc: jsPDF, margin: number, y: number, maxW: number): number {
  const items = FUND_ALLOCATION;
  const barH = 9;
  const gap = 4;
  const labelW = maxW * 0.42;
  const barAreaW = maxW * 0.40;
  const barX = margin + labelW;

  const totalH = items.length * (barH + gap);
  y = ensureSpace(doc, y, totalH + 4, margin);

  for (const item of items) {
    doc.setFont("helvetica", "normal");
    doc.setFontSize(11);
    doc.setTextColor(...T.text);
    doc.text(safe(item.label), margin, y + barH * 0.65);

    doc.setFillColor(20, 25, 40);
    doc.roundedRect(barX, y, barAreaW, barH, 1.5, 1.5, "F");

    const fillW = Math.max((item.percent / 100) * barAreaW, 2);
    doc.setFillColor(...T.accent);
    doc.roundedRect(barX, y, fillW, barH, 1.5, 1.5, "F");

    doc.setFont("helvetica", "bold");
    doc.setFontSize(11);
    doc.setTextColor(...T.accent);
    doc.text(safe(item.value), barX + barAreaW + 3, y + barH * 0.65);

    y += barH + gap;
  }

  return y + 4;
}

function drawPipelineFlow(doc: jsPDF, margin: number, y: number, maxW: number): number {
  const stages = PIPELINE_STAGES;
  const boxW = 40;
  const boxH = 18;
  const arrowW = 10;
  const totalW = stages.length * boxW + (stages.length - 1) * arrowW;
  const startX = margin + (maxW - totalW) / 2;

  y = ensureSpace(doc, y, boxH + 8, margin);

  for (let i = 0; i < stages.length; i++) {
    const bx = startX + i * (boxW + arrowW);
    const isEnd = i === 0 || i === stages.length - 1;

    const bgColor: RGB = isEnd ? [15, 40, 55] : [14, 18, 30];
    const borderColor: RGB = isEnd ? T.accent : [55, 65, 85];
    doc.setFillColor(...bgColor);
    doc.setDrawColor(...borderColor);
    doc.setLineWidth(isEnd ? 0.4 : 0.2);
    doc.roundedRect(bx, y, boxW, boxH, 2, 2, "FD");

    const txtColor: RGB = isEnd ? T.accent : T.text;
    doc.setFont("helvetica", "bold");
    doc.setFontSize(12);
    doc.setTextColor(...txtColor);
    doc.text(stages[i], bx + boxW / 2, y + boxH / 2 + 2, { align: "center" });

    if (i < stages.length - 1) {
      const ax = bx + boxW + 1;
      const ay = y + boxH / 2;
      doc.setDrawColor(...T.accent);
      doc.setLineWidth(0.3);
      doc.line(ax, ay, ax + arrowW - 3, ay);
      doc.setFillColor(...T.accent);
      doc.triangle(ax + arrowW - 3, ay - 1.5, ax + arrowW - 1, ay, ax + arrowW - 3, ay + 1.5, "F");
    }
  }

  return y + boxH + 6;
}

function drawCareerTimeline(doc: jsPDF, margin: number, y: number, maxW: number, fs: number): number {
  const milestones = [
    { company: "Pocket", detail: "Head of Infrastructure. 13 engineers. 1B+ requests/day.", current: false },
    { company: "Coinmiles", detail: "Promoted to CTO in 3 months.", current: false },
    { company: "Blocknative", detail: "Real-time transaction monitoring at scale.", current: false },
    { company: "5D Labs", detail: "Solo built: platform, first customer, $240K pipeline, 17+ server deployments.", current: true },
  ];

  const dotR = 3;
  const lineX = margin + dotR;
  const textX = margin + dotR * 2 + 6;
  const itemH = 20;
  const totalH = milestones.length * itemH;

  y = ensureSpace(doc, y, totalH + 4, margin);

  for (let i = 0; i < milestones.length; i++) {
    const m = milestones[i];
    const cy = y + i * itemH;

    if (i < milestones.length - 1) {
      doc.setDrawColor(55, 65, 85);
      doc.setLineWidth(0.3);
      doc.line(lineX, cy + dotR + 2, lineX, cy + itemH - 1);
    }

    const dotColor: RGB = m.current ? T.accent : [55, 65, 85];
    doc.setFillColor(...dotColor);
    doc.circle(lineX, cy + 1.5, dotR, "F");

    const nameColor: RGB = m.current ? T.accent : T.text;
    doc.setFont("helvetica", "bold");
    doc.setFontSize(fs);
    doc.setTextColor(...nameColor);
    doc.text(safe(m.company), textX, cy + 2);

    doc.setFont("helvetica", "normal");
    doc.setFontSize(fs - 2);
    doc.setTextColor(...T.muted);
    const detailLines: string[] = doc.splitTextToSize(safe(m.detail), maxW - textX + margin);
    let dy = cy + 9;
    for (const dl of detailLines) {
      doc.text(dl, textX, dy);
      dy += 4.5;
    }
  }

  return y + totalH + 4;
}

function drawFunnel(doc: jsPDF, margin: number, y: number, maxW: number): number {
  const stages = FUNNEL_STAGES;
  const stageH = 20;
  const gap = 3;
  const centerX = margin + maxW / 2;
  const fills: RGB[] = [[18, 28, 42], [23, 33, 48], [28, 38, 54]];
  const borders: RGB[] = [[55, 65, 85], [65, 80, 100], [80, 100, 120]];

  const totalH = stages.length * (stageH + gap);
  y = ensureSpace(doc, y, totalH + 4, margin);

  for (let i = 0; i < stages.length; i++) {
    const s = stages[i];
    const widthFrac = 1 - i * 0.25;
    const w = maxW * 0.7 * widthFrac;
    const bx = centerX - w / 2;
    const by = y + i * (stageH + gap);

    doc.setFillColor(...fills[i]);
    doc.setDrawColor(...borders[i]);
    doc.setLineWidth(0.2);
    doc.roundedRect(bx, by, w, stageH, 1.5, 1.5, "FD");

    doc.setFont("helvetica", "bold");
    doc.setFontSize(12);
    doc.setTextColor(...T.accent);
    doc.text(safe(s.label), centerX, by + 7, { align: "center" });

    doc.setFont("helvetica", "normal");
    doc.setFontSize(9);
    doc.setTextColor(...T.text);
    doc.text(safe(s.description), centerX, by + 14, { align: "center" });
  }

  return y + totalH + 4;
}

/* ─── Main export ─── */

export async function createPitchDeckPdfBlob(
  density: "compact" | "readable" = "compact",
): Promise<Blob> {
  const { jsPDF: JsPDF } = await import("jspdf");
  const { default: autoTable } = await import("jspdf-autotable");

  const margin = density === "compact" ? 9 : 12;
  const maxW = PG.w - 2 * margin;

  const sizes = density === "compact"
    ? { meta: 12, label: 13, headline: 34, sub: 20, body: 20, small: 14, table: 11 }
    : { meta: 13, label: 14, headline: 42, sub: 22, body: 22, small: 15, table: 12 };

  const doc = new JsPDF({ orientation: "landscape", unit: "mm", format: [PG.w, PG.h] });

  for (let si = 0; si < SLIDES.length; si++) {
    if (si > 0) doc.addPage();
    drawChrome(doc);

    const slide = SLIDES[si];

    /* Vertical offset — push lighter slides toward center */
    const vOffset = (() => {
      switch (slide.id) {
        case "cover": return PG.h * 0.35;
        case "ask": return PG.h * 0.30;
        case "problem": case "traction": return PG.h * 0.25;
        case "founder": case "market": case "competition": return PG.h * 0.10;
        case "solution": case "funds": return PG.h * 0.05;
        default: return margin + 1.5;
      }
    })();
    let y = vOffset;

    /* ── header — always at top regardless of offset ── */
    doc.setFont("helvetica", "normal");
    doc.setFontSize(sizes.meta);
    doc.setTextColor(...T.dim);
    doc.text(safe(BRAND.header), margin, margin + 1.5);
    doc.setTextColor(...T.muted);
    doc.text(`${si + 1} / ${SLIDES.length}`, PG.w - margin, margin + 1.5, { align: "right" });
    y = Math.max(y, margin + 6.5);

    /* ── label ── */
    y = writeLines(doc, safe(slide.label.toUpperCase()), margin, y, maxW, sizes.label, {
      color: T.muted,
    }) + 14;

    /* ── headline ── */
    const headlineFs = slide.id === "cover" ? 44 : slide.id === "ask" ? 48 : sizes.headline;
    y = writeLines(doc, safe(slide.headline), margin, y, maxW, headlineFs, {
      bold: true,
      color: T.accent,
      display: true,
    }) + 5;

    /* ── subhead ── */
    if (slide.subhead) {
      y = writeLines(doc, safe(slide.subhead), margin, y, maxW, sizes.sub, {
        color: T.muted,
        relaxed: true,
      }) + 3;
    }

    /* ── body ── */
    if (slide.body) {
      y = writeLines(doc, safe(slide.body), margin, y, maxW, sizes.body, {
        color: T.text,
        relaxed: true,
      }) + 4;
    }

    /* ── per-slide content ── */
    switch (slide.id) {
      case "founder":
        y = drawCareerTimeline(doc, margin, y, maxW, sizes.body);
        break;

      case "solution":
        y = drawPipelineFlow(doc, margin, y, maxW);
        if (slide.bullets) y = renderBullets(doc, slide.bullets, margin, y, maxW, sizes.body);
        break;

      case "traction":
        y = drawTractionStats(doc, margin, y, maxW);
        break;

      case "market":
        y = drawMarketRings(doc, margin, y, maxW);
        break;

      case "competition":
        y = drawCompetitionQuadrant(doc, margin, y, maxW);
        break;

      case "business-gtm":
        if (slide.table) y = renderTable(doc, slide.table, margin, y, maxW, sizes.table, autoTable as AutoTableFn);
        y = drawFunnel(doc, margin, y, maxW);
        break;

      case "funds":
        y = drawFundBars(doc, margin, y, maxW);
        break;

      default:
        if (slide.stats) y = renderStats(doc, slide.stats, margin, y, maxW, sizes.body);
        if (slide.bullets) y = renderBullets(doc, slide.bullets, margin, y, maxW, sizes.body);
        if (slide.table) y = renderTable(doc, slide.table, margin, y, maxW, sizes.table, autoTable as AutoTableFn);
        break;
    }

    /* ── callout ── */
    if (slide.callout) {
      y = writeCalloutBox(doc, safe(slide.callout), margin, y, maxW, sizes.body, margin);
    }

    /* ── footnote ── */
    if (slide.footnote) {
      writeLines(doc, safe(slide.footnote), margin, y, maxW, sizes.small, {
        color: T.footnote,
        relaxed: true,
      });
    }
  }

  return doc.output("blob");
}
