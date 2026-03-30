/**
 * Write compact + readable PDFs and PPTX to /tmp for layout inspection.
 *   cd apps/pitch-deck && npx tsx scripts/render-pdf-samples.ts
 */
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { buildPitchDeckPdfBlob } from "../src/lib/export-pdf";
import { buildPitchDeckPptxBlob } from "../src/lib/export-pptx";
import { slides } from "../src/lib/deck-content";

const outDir = process.env.PDF_OUT_DIR ?? "/tmp";

async function main() {
  for (const density of ["compact", "readable"] as const) {
    const blob = buildPitchDeckPdfBlob(slides, { density });
    const buf = Buffer.from(await blob.arrayBuffer());
    const name =
      density === "compact"
        ? "5dlabs-pitch-deck.pdf"
        : "5dlabs-pitch-deck-readable.pdf";
    const path = join(outDir, name);
    writeFileSync(path, buf);
    console.log(`Wrote ${path} (${buf.length} bytes)`);
  }

  const pptxBlob = await buildPitchDeckPptxBlob(slides);
  const pptxBuf = Buffer.from(await pptxBlob.arrayBuffer());
  const pptxPath = join(outDir, "5dlabs-pitch-deck.pptx");
  writeFileSync(pptxPath, pptxBuf);
  console.log(`Wrote ${pptxPath} (${pptxBuf.length} bytes)`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
