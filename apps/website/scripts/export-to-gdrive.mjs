#!/usr/bin/env node
/**
 * Generate a PDF of the pitch deck and prepare it for Google Drive.
 *
 * Usage:
 *   node scripts/export-to-gdrive.mjs            # generate + open Drive
 *   node scripts/export-to-gdrive.mjs --pdf-only  # generate only
 */

import fs from "node:fs";
import path from "node:path";
import { execSync } from "node:child_process";

const PDF_PATH = path.resolve(
  import.meta.dirname,
  "../exports/5D-Labs-Pitch-Deck.pdf"
);
const PITCH_URL = "https://pitch.5dlabs.ai/";

async function generatePdf() {
  console.log("📄 Generating PDF from", PITCH_URL, "...");
  const { chromium } = await import("playwright");
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto(PITCH_URL, { waitUntil: "networkidle" });
  await page.waitForTimeout(3000);

  await page.evaluate(() => {
    document.querySelectorAll("[style]").forEach((el) => {
      const s = /** @type {HTMLElement} */ (el).style;
      if (s.opacity === "0" || parseFloat(s.opacity) < 0.1) s.opacity = "1";
      if (s.transform && s.transform !== "none") s.transform = "none";
    });
  });

  fs.mkdirSync(path.dirname(PDF_PATH), { recursive: true });
  await page.pdf({
    path: PDF_PATH,
    format: "A4",
    printBackground: true,
    margin: { top: "0.4in", bottom: "0.4in", left: "0.4in", right: "0.4in" },
  });
  await browser.close();
  console.log("✅ PDF saved to", PDF_PATH);
}

async function main() {
  const pdfOnly = process.argv.includes("--pdf-only");

  await generatePdf();

  if (!pdfOnly) {
    console.log("🌐 Opening Google Drive...");
    execSync("open https://drive.google.com/drive/my-drive");
    console.log("📂 Revealing PDF in Finder...");
    execSync(`open -R "${PDF_PATH}"`);
    console.log(
      "\n💡 Drag the PDF from Finder into the Google Drive browser tab to upload."
    );
  }

  console.log("\n🎉 Done!");
}

main().catch((err) => {
  console.error("❌", err.message || err);
  process.exit(1);
});
