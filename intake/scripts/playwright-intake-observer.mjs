#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import readline from "node:readline/promises";

const DEFAULT_DISCORD_URL = process.env.INTAKE_DISCORD_URL || "https://discord.com/channels/@me";
const DEFAULT_LINEAR_URL = process.env.INTAKE_LINEAR_URL || "https://linear.app";
const DEFAULT_OUTPUT_ROOT = process.env.INTAKE_PLAYWRIGHT_OUTPUT_DIR || "output/playwright/intake-observer";
const DEFAULT_PROFILE_DIR = process.env.INTAKE_PLAYWRIGHT_PROFILE_DIR || path.join(DEFAULT_OUTPUT_ROOT, "profile");

function printUsage() {
  console.log(`Playwright intake observer

Usage:
  node intake/scripts/playwright-intake-observer.mjs [options]

Options:
  --discord-url <url>      Discord channel or app URL
  --linear-url <url>       Linear project, issue, or app URL
  --output-dir <path>      Root artifact directory
  --profile-dir <path>     Persistent Chromium profile directory
  --poll-seconds <n>       Seconds between captures (default: 0)
  --max-captures <n>       Capture count when polling (default: 1)
  --headless               Run headless instead of headed
  --wait-for-login         Pause before first capture so you can log in manually
  --help                   Show this help

Environment:
  INTAKE_DISCORD_URL
  INTAKE_LINEAR_URL
  INTAKE_PLAYWRIGHT_OUTPUT_DIR
  INTAKE_PLAYWRIGHT_PROFILE_DIR
`);
}

function parseNumber(value, flagName) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error(`${flagName} must be a non-negative integer`);
  }
  return parsed;
}

function parseArgs(argv) {
  const args = {
    discordUrl: DEFAULT_DISCORD_URL,
    linearUrl: DEFAULT_LINEAR_URL,
    outputDir: DEFAULT_OUTPUT_ROOT,
    profileDir: DEFAULT_PROFILE_DIR,
    pollSeconds: 0,
    maxCaptures: 1,
    headless: false,
    waitForLogin: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    switch (arg) {
      case "--discord-url":
        args.discordUrl = argv[++index];
        break;
      case "--linear-url":
        args.linearUrl = argv[++index];
        break;
      case "--output-dir":
        args.outputDir = argv[++index];
        break;
      case "--profile-dir":
        args.profileDir = argv[++index];
        break;
      case "--poll-seconds":
        args.pollSeconds = parseNumber(argv[++index], "--poll-seconds");
        break;
      case "--max-captures":
        args.maxCaptures = parseNumber(argv[++index], "--max-captures");
        break;
      case "--headless":
        args.headless = true;
        break;
      case "--wait-for-login":
        args.waitForLogin = true;
        break;
      case "--help":
        printUsage();
        process.exit(0);
        break;
      default:
        if (arg?.startsWith("--")) {
          throw new Error(`Unknown flag: ${arg}`);
        }
        throw new Error(`Unexpected argument: ${arg}`);
    }

    if (typeof args.discordUrl !== "string" || typeof args.linearUrl !== "string") {
      throw new Error(`Missing value for ${arg}`);
    }
  }

  args.outputDir = path.resolve(args.outputDir);
  args.profileDir = path.resolve(args.profileDir);
  return args;
}

function timestampForDir(date = new Date()) {
  return date.toISOString().replace(/[:.]/g, "-");
}

async function ensureDir(dirPath) {
  await fs.mkdir(dirPath, { recursive: true });
}

async function navigateAndSettle(page, targetUrl) {
  await page.goto(targetUrl, { waitUntil: "domcontentloaded", timeout: 60_000 });
  await page.waitForTimeout(2_000);
}

async function findOrCreatePage(context, label, targetUrl) {
  const host = new URL(targetUrl).host;
  const existingPage = context.pages().find((page) => {
    try {
      return new URL(page.url()).host === host;
    } catch {
      return false;
    }
  });

  const page = existingPage || (await context.newPage());
  await page.bringToFront();

  try {
    if (page.url() !== targetUrl) {
      console.log(`Navigating ${label} tab to ${targetUrl}`);
      await navigateAndSettle(page, targetUrl);
    } else {
      await page.waitForLoadState("domcontentloaded", { timeout: 10_000 }).catch(() => {});
      await page.waitForTimeout(1_000);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.warn(`Navigation warning for ${label}: ${message}`);
  }

  return page;
}

async function collectPageSummary(page) {
  return page.evaluate(() => {
    const textLines = (document.body?.innerText || "")
      .split(/\n+/)
      .map((value) => value.trim())
      .filter(Boolean)
      .slice(0, 60);

    const headings = Array.from(
      document.querySelectorAll("h1, h2, h3, [role='heading']")
    )
      .map((element) => element.textContent?.trim())
      .filter(Boolean)
      .slice(0, 20);

    return {
      title: document.title,
      url: window.location.href,
      headings,
      textSample: textLines,
      capturedAt: new Date().toISOString(),
    };
  });
}

async function capturePage(page, label, captureDir) {
  await page.bringToFront();
  await page.waitForTimeout(1_000);

  const summary = await collectPageSummary(page);
  const screenshotPath = path.join(captureDir, `${label}.png`);
  const summaryPath = path.join(captureDir, `${label}.json`);

  await page.screenshot({
    path: screenshotPath,
    fullPage: false,
  });
  await fs.writeFile(summaryPath, `${JSON.stringify(summary, null, 2)}\n`, "utf8");

  return {
    label,
    screenshotPath,
    summaryPath,
    title: summary.title,
    url: summary.url,
  };
}

async function waitForEnter(promptText) {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    await rl.question(`${promptText}\n`);
  } finally {
    rl.close();
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  await ensureDir(args.outputDir);
  await ensureDir(args.profileDir);

  let chromium;
  try {
    ({ chromium } = await import("playwright"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Playwright is not installed in this checkout (${message}). Run 'npm install' from the repo root, then 'npx playwright install chromium'.`
    );
  }

  const runDir = path.join(args.outputDir, timestampForDir());
  await ensureDir(runDir);

  console.log(`Artifacts: ${runDir}`);
  console.log(`Profile:   ${args.profileDir}`);

  const context = await chromium.launchPersistentContext(args.profileDir, {
    headless: args.headless,
    viewport: { width: 1440, height: 1000 },
    args: ["--disable-dev-shm-usage"],
  });

  const discordPage = await findOrCreatePage(context, "discord", args.discordUrl);
  const linearPage = await findOrCreatePage(context, "linear", args.linearUrl);

  if (args.waitForLogin) {
    await waitForEnter(
      "Log into Discord and Linear in the opened browser if needed, then press Enter to start capturing."
    );
  }

  const captureCount = args.pollSeconds > 0 ? Math.max(1, args.maxCaptures) : 1;
  const manifest = {
    startedAt: new Date().toISOString(),
    args: {
      discordUrl: args.discordUrl,
      linearUrl: args.linearUrl,
      outputDir: args.outputDir,
      profileDir: args.profileDir,
      pollSeconds: args.pollSeconds,
      maxCaptures: captureCount,
      headless: args.headless,
      waitForLogin: args.waitForLogin,
    },
    captures: [],
  };

  for (let captureIndex = 1; captureIndex <= captureCount; captureIndex += 1) {
    const captureDir = path.join(runDir, `capture-${String(captureIndex).padStart(3, "0")}`);
    await ensureDir(captureDir);

    console.log(`Capturing iteration ${captureIndex}/${captureCount}`);

    const discordArtifact = await capturePage(discordPage, "discord", captureDir);
    const linearArtifact = await capturePage(linearPage, "linear", captureDir);

    manifest.captures.push({
      index: captureIndex,
      capturedAt: new Date().toISOString(),
      artifacts: [discordArtifact, linearArtifact],
    });

    if (captureIndex < captureCount) {
      await new Promise((resolve) => {
        setTimeout(resolve, args.pollSeconds * 1_000);
      });
    }
  }

  manifest.finishedAt = new Date().toISOString();
  const manifestPath = path.join(runDir, "manifest.json");
  await fs.writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  console.log(`Wrote manifest: ${manifestPath}`);

  await context.close();
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack || error.message : String(error);
  console.error(message);
  process.exit(1);
});
