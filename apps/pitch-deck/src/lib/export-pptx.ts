import type { DeckSlide } from "./deck-content";
import { DECK_META } from "./deck-content";

/**
 * Build a 16:9 .pptx from the same slide model as the web deck (for PowerPoint + Google Slides import).
 */
export async function buildPitchDeckPptxBlob(slides: DeckSlide[]): Promise<Blob> {
  const pptxgen = (await import("pptxgenjs")).default;
  const pptx = new pptxgen();

  pptx.author = DECK_META.company;
  pptx.company = DECK_META.company;
  pptx.title = `${DECK_META.company} — Investor Deck`;
  pptx.subject = `${DECK_META.round} pitch`;
  pptx.layout = "LAYOUT_16x9";

  const margin = 0.45;
  const contentW = 9.1;
  const x = margin;

  for (const slide of slides) {
    const s = pptx.addSlide();
    let y = 0.35;

    s.addText(`${DECK_META.company} · ${DECK_META.round} · ${DECK_META.confidential}`, {
      x,
      y,
      w: contentW,
      h: 0.25,
      fontSize: 9,
      color: "666666",
    });
    y += 0.32;

    s.addText(slide.label, {
      x,
      y,
      w: contentW,
      h: 0.3,
      fontSize: 11,
      color: "888888",
    });
    y += 0.38;

    if (slide.eyebrow) {
      s.addText(slide.eyebrow, {
        x,
        y,
        w: contentW,
        h: 0.35,
        fontSize: 12,
        color: "0E7490",
      });
      /* ~2pt extra before headline vs previous step */
      y += 0.45;
    }

    const titleSize = slide.layout === "hero" ? 28 : slide.layout === "impact" ? 26 : 22;
    s.addText(slide.headline, {
      x,
      y,
      w: contentW,
      h: 1.1,
      fontSize: titleSize,
      bold: true,
      color: "111827",
      fontFace: "Arial",
    });
    y += slide.layout === "hero" ? 1.15 : 0.95;

    if (slide.subhead) {
      s.addText(slide.subhead, {
        x,
        y,
        w: contentW,
        h: 0.85,
        fontSize: 14,
        color: "374151",
        fontFace: "Arial",
      });
      y += 0.9;
    }

    if (slide.stats?.length) {
      const lines = slide.stats.map((st) => `${st.value} — ${st.label}`);
      s.addText(lines.join("\n"), {
        x,
        y,
        w: contentW,
        h: 0.35 + lines.length * 0.22,
        fontSize: 13,
        color: "111827",
        fontFace: "Arial",
      });
      y += 0.45 + lines.length * 0.22;
    }

    if (slide.bullets?.length) {
      const bulletBlocks = slide.bullets.map((b) => ({
        text: b,
        options: {
          bullet: true,
          fontSize: 13,
          color: "1F2937",
          fontFace: "Arial",
        },
      }));
      s.addText(bulletBlocks, {
        x,
        y,
        w: contentW,
        h: 0.35 + slide.bullets.length * 0.38,
      });
      y += 0.4 + slide.bullets.length * 0.38;
    }

    if (slide.table) {
      const tbl = slide.table;
      const headerFill = { color: "F3F4F6" };
      const tableRows = [
        tbl.headers.map((h) => ({
          text: h,
          options: { bold: true, fill: headerFill, color: "374151", fontSize: 11 },
        })),
        ...tbl.rows.map((row) =>
          row.map((cell) => ({
            text: cell,
            options: { fontSize: 11, color: "111827" },
          })),
        ),
      ];
      const colW = tbl.headers.map(() => contentW / tbl.headers.length);
      const rowH = 0.28;
      const tableH = rowH * tableRows.length;
      const h = Math.min(tableH, 2.8);
      s.addTable(tableRows, {
        x,
        y,
        w: contentW,
        h,
        colW,
        border: { type: "solid", color: "E5E7EB", pt: 0.5 },
        fontSize: 11,
      });
      y += h + 0.2;
    }

    if (slide.callout) {
      s.addText(slide.callout, {
        x,
        y,
        w: contentW,
        h: 0.9,
        fontSize: 13,
        italic: true,
        color: "0F766E",
        fill: { color: "ECFDF5" },
        fontFace: "Arial",
      });
      y += 0.95;
    }

    if (slide.cta) {
      s.addText(`${slide.cta.label}: ${slide.cta.href}`, {
        x,
        y,
        w: contentW,
        h: 0.35,
        fontSize: 12,
        color: "0E7490",
        bold: true,
        fontFace: "Arial",
      });
      y += 0.42;
    }

    if (slide.footnote) {
      s.addText(slide.footnote, {
        x,
        y,
        w: contentW,
        h: 1.2,
        fontSize: 9,
        color: "6B7280",
        fontFace: "Arial",
      });
    }
  }

  const out = (await pptx.write({ outputType: "blob" })) as Blob;
  return out;
}
