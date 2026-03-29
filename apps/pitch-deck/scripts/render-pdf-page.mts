/**
 * Render a single PDF page to PNG for layout checks.
 * Usage: PDF=/tmp/5dlabs-pitch-deck.pdf PAGE=5 npx tsx scripts/render-pdf-page.mts
 */
import { execSync } from "node:child_process";

const pdf = process.env.PDF ?? "/tmp/5dlabs-pitch-deck.pdf";
const page = process.env.PAGE ?? "5";
const out = process.env.OUT ?? `/tmp/pdf-page-${page}.png`;

execSync(
  `pdftoppm -png -f ${page} -l ${page} -r 200 "${pdf}" /tmp/pdf-check`,
  { stdio: "inherit" },
);
execSync(`mv /tmp/pdf-check-${String(page).padStart(2, "0")}.png "${out}"`, {
  stdio: "inherit",
});
console.log(out);
