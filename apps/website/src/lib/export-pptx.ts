/**
 * Production PPTX exporter for the 5D Labs pitch deck.
 * Dark theme, 16:9, shape-based chart approximations.
 */
import {
  BRAND, THEME, SLIDES,
  TRACTION_METRICS, MARKET_RINGS, COMPETITION_QUADRANT,
  FUND_ALLOCATION, PIPELINE_STAGES, FUNNEL_STAGES,
} from "./deck-content";

/* ─── Constants ─── */

const M = 0.45; // margin (inches)
const W = 9.1;  // content width
const H = THEME.hex;

/* ─── Main export ─── */

export async function createPitchDeckPptxBlob(): Promise<Blob> {
  const { default: PptxGenJS } = await import("pptxgenjs");
  const pptx = new PptxGenJS();

  pptx.layout = "LAYOUT_16x9";
  pptx.author = BRAND.company;
  pptx.title = `${BRAND.company} \u2014 Pitch Deck`;

  /* ── helper: slide chrome ── */

  function addChrome(
    slide: ReturnType<typeof pptx.addSlide>,
    si: number,
  ) {
    slide.background = { color: H.bg };

    // Cyan accent line at top
    slide.addShape("rect", {
      x: 0, y: 0, w: 10, h: 0.04,
      fill: { color: H.cyan },
    });

    // Footer
    slide.addText(`${BRAND.company} \u00b7 Pitch Deck`, {
      x: M, y: 5.0, w: 5,
      fontSize: 8, color: H.muted, fontFace: "Arial",
    });
    slide.addText(`${si + 1} / ${SLIDES.length}`, {
      x: 5, y: 5.0, w: 4.55,
      fontSize: 8, color: H.muted, fontFace: "Arial",
      align: "right",
    });
  }

  /* ── chart: traction stat boxes ── */

  function addTractionStats(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const cols = TRACTION_METRICS.length;
    const gap = 0.1;
    const boxW = (W - gap * (cols - 1)) / cols;
    const boxH = 1.1;

    for (let i = 0; i < cols; i++) {
      const m = TRACTION_METRICS[i];
      const bx = M + i * (boxW + gap);

      slide.addShape("roundRect", {
        x: bx, y, w: boxW, h: boxH,
        fill: { color: H.calloutBg },
        line: { color: H.cyan, width: 0.5 },
        rectRadius: 0.05,
      });

      slide.addText(m.value, {
        x: bx, y: y + 0.08, w: boxW, h: 0.4,
        fontSize: 18, bold: true, color: H.cyan, fontFace: "Arial",
        align: "center", valign: "middle",
      });

      slide.addText(m.label, {
        x: bx, y: y + 0.48, w: boxW, h: 0.22,
        fontSize: 9, bold: true, color: H.white, fontFace: "Arial",
        align: "center",
      });

      slide.addText(m.note, {
        x: bx + 0.08, y: y + 0.72, w: boxW - 0.16, h: 0.3,
        fontSize: 7, color: H.muted, fontFace: "Arial",
        align: "center",
      });
    }

    return y + boxH + 0.2;
  }

  /* ── chart: market concentric rings ── */

  function addMarketRings(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const widths = [W * 0.8, W * 0.55, W * 0.3];
    const heights = [1.8, 1.3, 0.8];
    const fills = ["121C2A", "142332", "193C4A"];
    const borders = ["284060", "3278A0", H.cyan];
    const cx = M + W / 2;

    for (let i = 0; i < MARKET_RINGS.length; i++) {
      const r = MARKET_RINGS[i];
      const rx = cx - widths[i] / 2;
      const ry = y + (heights[0] - heights[i]) / 2;

      slide.addShape("roundRect", {
        x: rx, y: ry, w: widths[i], h: heights[i],
        fill: { color: fills[i] },
        line: { color: borders[i], width: 0.5 },
        rectRadius: 0.05,
      });

      slide.addText(`${r.label}: ${r.value}`, {
        x: rx, y: ry + heights[i] * 0.15, w: widths[i], h: 0.3,
        fontSize: i === 2 ? 12 : 10, bold: true, color: H.cyan, fontFace: "Arial",
        align: "center",
      });

      slide.addText(r.description, {
        x: rx, y: ry + heights[i] * 0.5, w: widths[i], h: 0.25,
        fontSize: 8, color: H.white, fontFace: "Arial",
        align: "center",
      });
    }

    return y + heights[0] + 0.3;
  }

  /* ── chart: competition 2x2 quadrant ── */

  function addCompetitionQuadrant(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const cellW = W / 2;
    const cellH = 1.1;

    for (let i = 0; i < 4; i++) {
      const row = Math.floor(i / 2);
      const col = i % 2;
      const cell = COMPETITION_QUADRANT[i];
      const isHl = !!cell.highlight;
      const cx = M + col * cellW;
      const cy = y + row * cellH;

      slide.addShape("rect", {
        x: cx, y: cy, w: cellW, h: cellH,
        fill: { color: isHl ? "0F2332" : H.altRowBg },
        line: { color: isHl ? H.cyan : H.border, width: isHl ? 1 : 0.5 },
      });

      slide.addText(cell.label, {
        x: cx + 0.1, y: cy + 0.04, w: cellW - 0.2, h: 0.22,
        fontSize: 7, bold: true, color: isHl ? H.cyan : H.muted, fontFace: "Arial",
      });

      slide.addText(cell.players.join("\n"), {
        x: cx + 0.1, y: cy + 0.28, w: cellW - 0.2, h: 0.75,
        fontSize: isHl ? 13 : 9, bold: isHl, color: isHl ? H.cyan : H.white, fontFace: "Arial",
      });
    }

    return y + cellH * 2 + 0.2;
  }

  /* ── chart: fund allocation bars ── */

  function addFundBars(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const barH = 0.28;
    const gap = 0.12;
    const labelW = W * 0.42;
    const barAreaW = W * 0.40;
    const barX = M + labelW;
    let cy = y;

    for (const item of FUND_ALLOCATION) {
      slide.addText(item.label, {
        x: M, y: cy, w: labelW, h: barH,
        fontSize: 9, color: H.white, fontFace: "Arial", valign: "middle",
      });

      slide.addShape("roundRect", {
        x: barX, y: cy, w: barAreaW, h: barH,
        fill: { color: "141928" },
        rectRadius: 0.03,
      });

      const fillW = Math.max((item.percent / 100) * barAreaW, 0.1);
      slide.addShape("roundRect", {
        x: barX, y: cy, w: fillW, h: barH,
        fill: { color: H.cyan },
        rectRadius: 0.03,
      });

      slide.addText(item.value, {
        x: barX + barAreaW + 0.1, y: cy, w: W - labelW - barAreaW - 0.1, h: barH,
        fontSize: 9, bold: true, color: H.cyan, fontFace: "Arial", valign: "middle",
      });

      cy += barH + gap;
    }

    return cy + 0.1;
  }

  /* ── chart: pipeline flow ── */

  function addPipelineFlow(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const boxW = 1.3;
    const boxH = 0.55;
    const arrowGap = 0.4;
    const totalW = PIPELINE_STAGES.length * boxW + (PIPELINE_STAGES.length - 1) * arrowGap;
    const startX = M + (W - totalW) / 2;

    for (let i = 0; i < PIPELINE_STAGES.length; i++) {
      const bx = startX + i * (boxW + arrowGap);
      const isEnd = i === 0 || i === PIPELINE_STAGES.length - 1;

      slide.addShape("roundRect", {
        x: bx, y, w: boxW, h: boxH,
        fill: { color: isEnd ? "0F2838" : H.altRowBg },
        line: { color: isEnd ? H.cyan : H.border, width: isEnd ? 0.75 : 0.5 },
        rectRadius: 0.05,
      });

      slide.addText(PIPELINE_STAGES[i], {
        x: bx, y, w: boxW, h: boxH,
        fontSize: 11, bold: true, color: isEnd ? H.cyan : H.white, fontFace: "Arial",
        align: "center", valign: "middle",
      });

      if (i < PIPELINE_STAGES.length - 1) {
        slide.addText("\u2192", {
          x: bx + boxW, y, w: arrowGap, h: boxH,
          fontSize: 16, color: H.cyan, fontFace: "Arial",
          align: "center", valign: "middle",
        });
      }
    }

    return y + boxH + 0.3;
  }

  /* ── chart: career timeline ── */

  function addCareerTimeline(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const milestones = [
      { co: "Pocket", detail: "Head of Infrastructure. 13 engineers. 1B+ requests/day.", cur: false },
      { co: "Coinmiles", detail: "Promoted to CTO in 3 months.", cur: false },
      { co: "Blocknative", detail: "Real-time transaction monitoring at scale.", cur: false },
      { co: "5D Labs", detail: "Solo built: platform, customer, pipeline, 17+ servers.", cur: true },
    ];

    const itemH = 0.55;
    const dotSize = 0.1;
    const textX = M + 0.3;

    for (let i = 0; i < milestones.length; i++) {
      const m = milestones[i];
      const cy = y + i * itemH;

      // Dot
      slide.addShape("ellipse", {
        x: M + 0.05, y: cy + itemH * 0.3, w: dotSize, h: dotSize,
        fill: { color: m.cur ? H.cyan : H.border },
      });

      // Connecting line
      if (i < milestones.length - 1) {
        slide.addShape("rect", {
          x: M + 0.09, y: cy + itemH * 0.45 + dotSize / 2, w: 0.02, h: itemH * 0.5,
          fill: { color: H.border },
        });
      }

      // Text
      slide.addText([
        { text: m.co, options: { bold: true, fontSize: 12, color: m.cur ? H.cyan : H.white } },
        { text: ` \u2014 ${m.detail}`, options: { fontSize: 11, color: H.muted } },
      ], {
        x: textX, y: cy, w: W - textX + M, h: itemH,
        fontFace: "Arial", valign: "middle",
      });
    }

    return y + milestones.length * itemH + 0.2;
  }

  /* ── chart: funnel ── */

  function addFunnel(slide: ReturnType<typeof pptx.addSlide>, y: number): number {
    const stageH = 0.5;
    const gap = 0.08;
    const centerX = M + W / 2;
    const fills = ["121C2A", "172230", "1C2838"];
    const borders = ["374155", "415065", "4A5E78"];

    for (let i = 0; i < FUNNEL_STAGES.length; i++) {
      const s = FUNNEL_STAGES[i];
      const widthFrac = 1 - i * 0.25;
      const w = W * 0.65 * widthFrac;
      const bx = centerX - w / 2;
      const by = y + i * (stageH + gap);

      slide.addShape("roundRect", {
        x: bx, y: by, w, h: stageH,
        fill: { color: fills[i] },
        line: { color: borders[i], width: 0.5 },
        rectRadius: 0.03,
      });

      slide.addText(s.label, {
        x: bx, y: by, w, h: stageH * 0.5,
        fontSize: 10, bold: true, color: H.cyan, fontFace: "Arial",
        align: "center", valign: "bottom",
      });

      slide.addText(s.description, {
        x: bx, y: by + stageH * 0.45, w, h: stageH * 0.5,
        fontSize: 7, color: H.white, fontFace: "Arial",
        align: "center", valign: "top",
      });
    }

    return y + FUNNEL_STAGES.length * (stageH + gap) + 0.1;
  }

  /* ── helper: add bullets ── */

  function addBullets(
    slide: ReturnType<typeof pptx.addSlide>,
    bullets: string[],
    y: number,
  ): number {
    slide.addText(
      bullets.map((b) => ({
        text: b,
        options: {
          fontSize: 13,
          color: H.white,
          fontFace: "Arial" as const,
          bullet: { type: "bullet" as const, color: H.cyan },
          paraSpaceBefore: 6,
        },
      })),
      { x: M, y, w: W, h: Math.max(0.35 + bullets.length * 0.38, 1) },
    );
    return y + 0.4 + bullets.length * 0.38;
  }

  /* ── helper: add table ── */

  function addTable(
    slide: ReturnType<typeof pptx.addSlide>,
    headers: string[],
    rows: string[][],
    y: number,
  ): number {
    const colW = headers.map(() => W / headers.length);
    const headerRow = headers.map((h) => ({
      text: h,
      options: { bold: true, fill: { color: H.headerBg }, color: H.cyan, fontSize: 11 },
    }));
    const bodyRows = rows.map((row) =>
      row.map((cell) => ({
        text: cell,
        options: { fontSize: 11, color: H.white },
      })),
    );
    const rowH = 0.28;
    const tableH = Math.min(rowH * (1 + rows.length), 3.5);

    slide.addTable([headerRow, ...bodyRows], {
      x: M, y, w: W, h: tableH, colW,
      border: { type: "solid", color: H.border, pt: 0.5 },
      fontSize: 11,
      fill: { color: H.altRowBg },
      autoPage: false,
    });

    return y + tableH + 0.2;
  }

  /* ──────── build slides ──────── */

  for (let si = 0; si < SLIDES.length; si++) {
    const s = SLIDES[si];
    const slide = pptx.addSlide();
    addChrome(slide, si);

    let y = 0.3;

    /* label */
    slide.addText(s.label.toUpperCase(), {
      x: M, y, w: W,
      fontSize: 10, color: H.muted, fontFace: "Arial", charSpacing: 3,
    });
    y += 0.35;

    /* headline */
    const headlineFs = s.id === "ask" ? 44 : s.layout === "hero" ? 28 : s.layout === "impact" ? 26 : 22;
    slide.addText(s.headline, {
      x: M, y, w: W, h: s.id === "ask" ? 1.4 : 1.1,
      fontSize: headlineFs, bold: true, color: H.cyan, fontFace: "Arial",
    });
    y += s.id === "ask" ? 1.5 : s.layout === "hero" ? 1.15 : 0.95;

    /* subhead */
    if (s.subhead) {
      slide.addText(s.subhead, {
        x: M, y, w: W, h: 0.6,
        fontSize: 14, color: H.muted, fontFace: "Arial",
      });
      y += 0.65;
    }

    /* body */
    if (s.body) {
      slide.addText(s.body, {
        x: M, y, w: W, h: 1.0,
        fontSize: 14, color: H.white, fontFace: "Arial",
        lineSpacingMultiple: 1.3,
      });
      y += 0.85;
    }

    /* per-slide content */
    switch (s.id) {
      case "founder":
        y = addCareerTimeline(slide, y);
        break;

      case "solution":
        y = addPipelineFlow(slide, y);
        if (s.bullets) y = addBullets(slide, s.bullets, y);
        break;

      case "traction":
        y = addTractionStats(slide, y);
        break;

      case "market":
        y = addMarketRings(slide, y);
        break;

      case "competition":
        y = addCompetitionQuadrant(slide, y);
        break;

      case "business-gtm":
        if (s.table) y = addTable(slide, s.table.headers, s.table.rows, y);
        y = addFunnel(slide, y);
        break;

      case "funds":
        y = addFundBars(slide, y);
        break;

      case "product":
        if (s.table) y = addTable(slide, s.table.headers, s.table.rows, y);
        break;

      default:
        if (s.stats) {
          const statBlock = s.stats.map((st) => `${st.value}  ${st.label}`).join("\n");
          slide.addText(statBlock, {
            x: M, y, w: W, h: 0.35 + s.stats.length * 0.22,
            fontSize: 13, bold: true, color: H.cyan, fontFace: "Arial",
          });
          y += 0.45 + s.stats.length * 0.22;
        }
        if (s.bullets) y = addBullets(slide, s.bullets, y);
        if (s.table) y = addTable(slide, s.table.headers, s.table.rows, y);
        break;
    }

    /* callout */
    if (s.callout) {
      slide.addText(s.callout, {
        x: M, y, w: W, h: 0.9,
        fontSize: 13, italic: true, color: H.calloutText,
        fill: { color: H.calloutBg }, fontFace: "Arial",
      });
      y += 0.95;
    }

    /* footnote */
    if (s.footnote) {
      slide.addText(s.footnote, {
        x: M, y, w: W, h: 1.0,
        fontSize: 9, color: H.dim, fontFace: "Arial",
      });
    }
  }

  const blob = await pptx.write({ outputType: "blob" });
  return blob as Blob;
}
