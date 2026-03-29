import type { DeckSlide } from "./deck-content";
import { DECK_META } from "./deck-content";

/** Normalize Unicode that standard PPTX fonts mangle */
function pptxText(s: string): string {
  return s
    .replace(/\u2192/g, " -> ")
    .replace(/\u2014/g, " - ")
    .replace(/\u2013/g, "-")
    .replace(/\u2248/g, "~")
    .replace(/\u00a0/g, " ");
}

/**
 * Dark-themed 16:9 .pptx matching the web deck aesthetic.
 * Colors aligned with the OKLCH cyberpunk theme: deep blue-black bg, cyan accents, light text.
 */
export async function buildPitchDeckPptxBlob(slides: DeckSlide[]): Promise<Blob> {
  const pptxgen = (await import("pptxgenjs")).default;
  const pptx = new pptxgen();

  pptx.author = DECK_META.company;
  pptx.company = DECK_META.company;
  pptx.title = `${DECK_META.company} — Investor Deck`;
  pptx.subject = `${DECK_META.round} pitch`;
  pptx.layout = "LAYOUT_16x9";

  /* ── Theme colors matching website OKLCH tokens ── */
  const C = {
    bg: "0A0C16",        // oklch(0.06 0.02 260) ≈ deep blue-black
    text: "F0F2F5",      // oklch(0.95 0 0) ≈ near-white
    muted: "7A8194",     // oklch(0.55 0 0) ≈ gray
    dim: "5A6478",       // dim meta text
    accent: "22D3EE",    // cyan-400
    accentDark: "0E7490", // cyan-700
    cardBg: "141822",    // oklch(0.10 0.015 260) ≈ card surface
    calloutBg: "0F2D2E", // teal-tinted dark
    calloutText: "CCFBF1", // teal-100
    calloutBorder: "2DD4BF", // teal-400
    tableBg: "0E1220",   // slightly lighter than bg
    tableHeaderBg: "162030", // header row
    tableBorder: "2A3550", // subtle border
    topBar: "06B6D4",    // cyan-500
  } as const;

  const margin = 0.5;
  const contentW = 9.0;
  const x = margin;

  const meta = pptxText(
    `${DECK_META.company} · ${DECK_META.round} · ${DECK_META.confidential}`,
  );

  for (let si = 0; si < slides.length; si++) {
    const slide = slides[si];
    const s = pptx.addSlide();

    /* Dark background */
    s.background = { color: C.bg };

    /* Cyan accent bar at top */
    s.addShape(pptx.ShapeType.rect, {
      x: 0,
      y: 0,
      w: 10,
      h: 0.04,
      fill: { color: C.topBar },
      line: { color: C.topBar, width: 0 },
    });

    let y = 0.35;

    /* ── Meta row: confidential header left, slide counter right ── */
    s.addText(meta, {
      x,
      y,
      w: contentW * 0.75,
      h: 0.25,
      fontSize: 8,
      color: C.dim,
      fontFace: "Arial",
    });
    s.addText(`${si + 1} / ${slides.length}`, {
      x: 10 - margin - 0.8,
      y,
      w: 0.8,
      h: 0.25,
      fontSize: 8,
      color: C.muted,
      fontFace: "Arial",
      align: "right",
    });
    y += 0.32;

    /* ── Label (section name) ── */
    s.addText(pptxText(slide.label.toUpperCase()), {
      x,
      y,
      w: contentW,
      h: 0.28,
      fontSize: 10,
      color: C.muted,
      fontFace: "Arial",
      charSpacing: 2.5,
    });
    y += 0.35;

    /* ── Eyebrow ── */
    if (slide.eyebrow) {
      s.addText(pptxText(slide.eyebrow), {
        x,
        y,
        w: contentW,
        h: 0.3,
        fontSize: 12,
        bold: true,
        color: C.accent,
        fontFace: "Arial",
        charSpacing: 1.5,
      });
      y += 0.4;
    }

    /* ── Headline ── */
    const titleSize =
      slide.layout === "hero" ? 30 : slide.layout === "impact" ? 28 : 24;
    s.addText(pptxText(slide.headline), {
      x,
      y,
      w: contentW,
      h: slide.layout === "hero" ? 1.0 : 0.85,
      fontSize: titleSize,
      bold: true,
      color: C.text,
      fontFace: "Arial",
      shrinkText: true,
    });
    y += slide.layout === "hero" ? 1.1 : 0.92;

    /* ── Subhead ── */
    if (slide.subhead) {
      s.addText(pptxText(slide.subhead), {
        x,
        y,
        w: contentW,
        h: 0.75,
        fontSize: 14,
        color: C.muted,
        fontFace: "Arial",
        shrinkText: true,
      });
      y += 0.8;
    }

    /* ── Stats ── */
    if (slide.stats?.length) {
      const statLines = slide.stats.map((st) =>
        pptxText(`${st.value}  ·  ${st.label}`),
      );
      s.addText(
        statLines.map((line) => ({
          text: line,
          options: {
            fontSize: 15,
            bold: true,
            color: C.accent,
            fontFace: "Arial",
            breakType: "break" as const,
          },
        })),
        {
          x,
          y,
          w: contentW,
          h: 0.32 + statLines.length * 0.28,
        },
      );
      y += 0.4 + statLines.length * 0.28;
    }

    /* ── Bullets ── */
    if (slide.bullets?.length) {
      const bulletBlocks = slide.bullets.map((b) => ({
        text: pptxText(b),
        options: {
          bullet: { type: "bullet" as const, code: "2022" },
          fontSize: 13,
          color: C.text,
          fontFace: "Arial",
          paraSpaceAfter: 6,
          breakType: "break" as const,
        },
      }));
      const bulletH = Math.min(0.32 + slide.bullets.length * 0.36, 3.5);
      s.addText(bulletBlocks, {
        x,
        y,
        w: contentW,
        h: bulletH,
        valign: "top",
        shrinkText: true,
      });
      y += bulletH + 0.1;
    }

    /* ── Table ── */
    if (slide.table) {
      const tbl = slide.table;
      const tableRows = [
        tbl.headers.map((h) => ({
          text: pptxText(h),
          options: {
            bold: true,
            fill: { color: C.tableHeaderBg },
            color: C.accent,
            fontSize: 11,
            fontFace: "Arial",
          },
        })),
        ...tbl.rows.map((row, ri) =>
          row.map((cell) => ({
            text: pptxText(cell),
            options: {
              fontSize: 11,
              color: C.text,
              fontFace: "Arial",
              fill: { color: ri % 2 === 0 ? C.bg : C.tableBg },
            },
          })),
        ),
      ];
      const colW = tbl.headers.map(() => contentW / tbl.headers.length);
      const rowH = 0.3;
      const tableH = Math.min(rowH * tableRows.length, 2.8);
      s.addTable(tableRows, {
        x,
        y,
        w: contentW,
        h: tableH,
        colW,
        border: { type: "solid", color: C.tableBorder, pt: 0.5 },
        fontSize: 11,
        autoPage: false,
      });
      y += tableH + 0.25;
    }

    /* ── Callout box ── */
    if (slide.callout) {
      const calloutH = slide.callout.length > 120 ? 0.95 : 0.65;
      s.addShape(pptx.ShapeType.roundRect, {
        x,
        y,
        w: contentW,
        h: calloutH,
        fill: { color: C.calloutBg },
        line: { color: C.calloutBorder, width: 0.75 },
        rectRadius: 0.08,
      });
      s.addText(pptxText(slide.callout), {
        x: x + 0.2,
        y: y + 0.08,
        w: contentW - 0.4,
        h: calloutH - 0.16,
        fontSize: 13,
        color: C.calloutText,
        fontFace: "Arial",
        valign: "middle",
        shrinkText: true,
      });
      y += calloutH + 0.2;
    }

    /* ── CTA link ── */
    if (slide.cta) {
      s.addText(
        [
          {
            text: pptxText(slide.cta.label),
            options: {
              bold: true,
              color: C.accent,
              fontSize: 13,
              fontFace: "Arial",
            },
          },
          {
            text: `  ->  ${slide.cta.href}`,
            options: {
              color: C.muted,
              fontSize: 11,
              fontFace: "Arial",
            },
          },
        ],
        {
          x,
          y,
          w: contentW,
          h: 0.35,
        },
      );
      y += 0.42;
    }

    /* ── Footnote ── */
    if (slide.footnote) {
      s.addText(pptxText(slide.footnote), {
        x,
        y,
        w: contentW,
        h: 1.0,
        fontSize: 8,
        color: C.dim,
        fontFace: "Arial",
        valign: "top",
        shrinkText: true,
      });
    }
  }

  const out = (await pptx.write({ outputType: "blob" })) as Blob;
  return out;
}
