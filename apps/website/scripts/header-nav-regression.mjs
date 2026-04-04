import assert from "node:assert/strict";
import { chromium } from "playwright";

const baseUrl = process.env.BASE_URL ?? "http://localhost:3000";
const timeoutMs = Number(process.env.TEST_TIMEOUT_MS ?? 25000);

const cases = [
  { from: "/cto/pricing", linkName: "Bare Metal", expectedPath: "/cto", expectedHash: "#infrastructure" },
  { from: "/cto/services", linkName: "Bare Metal", expectedPath: "/cto", expectedHash: "#infrastructure" },
  { from: "/cto/pricing", linkName: "Agents", expectedPath: "/cto", expectedHash: "#agents" },
  { from: "/cto/services", linkName: "Agents", expectedPath: "/cto", expectedHash: "#agents" },
];

function withBase(path) {
  return `${baseUrl.replace(/\/+$/, "")}${path}`;
}

function normalizePath(pathname) {
  return pathname.replace(/\/+$/, "") || "/";
}

async function waitForSection(page, sectionId) {
  await page.waitForFunction(
    (id) => {
      const el = document.getElementById(id);
      if (!el) return false;
      const rect = el.getBoundingClientRect();
      return rect.top < window.innerHeight * 0.7;
    },
    sectionId,
    { timeout: timeoutMs }
  );
}

async function run() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  page.setDefaultTimeout(timeoutMs);

  try {
    for (const c of cases) {
      try {
        await page.goto(withBase(c.from), { waitUntil: "domcontentloaded" });

        const links = page.getByRole("link", { name: c.linkName, exact: true });
        const count = await links.count();
        let clicked = false;
        for (let i = 0; i < count; i += 1) {
          const candidate = links.nth(i);
          if (await candidate.isVisible()) {
            await candidate.click();
            clicked = true;
            break;
          }
        }
        assert.ok(clicked, `No visible "${c.linkName}" link found on ${c.from}`);

        await page.waitForFunction(
          ({ path, hash }) => {
            const normalized = window.location.pathname.replace(/\/+$/, "") || "/";
            const expectedNormalized = path.replace(/\/+$/, "") || "/";
            return normalized === expectedNormalized && window.location.hash === hash;
          },
          { path: c.expectedPath, hash: c.expectedHash },
          { timeout: timeoutMs }
        );

        const sectionId = c.expectedHash.slice(1);
        await waitForSection(page, sectionId);

        const currentPath = normalizePath(await page.evaluate(() => window.location.pathname));
        const currentHash = await page.evaluate(() => window.location.hash);
        assert.equal(
          currentPath,
          normalizePath(c.expectedPath),
          `Expected path ${c.expectedPath} from ${c.from} on ${c.linkName}`
        );
        assert.equal(currentHash, c.expectedHash, `Expected hash ${c.expectedHash} from ${c.from} on ${c.linkName}`);
      } catch (error) {
        const url = page.url();
        throw new Error(
          `Case failed: from=${c.from}, link=${c.linkName}, expected=${c.expectedPath}${c.expectedHash}, actual=${url}\n${String(error)}`
        );
      }
    }

    console.log(`Header navigation regression passed against ${baseUrl}`);
  } finally {
    await browser.close();
  }
}

run().catch((error) => {
  console.error("Header navigation regression failed:");
  console.error(error);
  process.exitCode = 1;
});
