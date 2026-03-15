#!/usr/bin/env node
/**
 * Polymarket Market Maker — 5-min Crypto Fast Markets
 *
 * Earns spread + maker rebates by providing two-sided liquidity on Polymarket's
 * 5-minute BTC/ETH/SOL binary outcome markets.
 *
 * Since Polymarket's Jan 2026 dynamic taker fee model, fees peak at ~3% at 50%
 * odds and taper to 0% at extremes. These fees are redistributed to MAKERS as
 * USDC rebates. This bot is a maker — it places limit orders resting on the book,
 * never crosses the spread.
 *
 * Architecture:
 *   CEX Feed (Binance/Coinbase) -> real-time BTC/ETH/SOL prices + momentum
 *   Polymarket Scanner          -> discover active 5-min contracts
 *   CLOB Client                 -> place/cancel limit orders (paper or live)
 *   MM Engine                   -> two-sided quoting with inventory management
 *
 * Usage:
 *   node index.js                     # Paper mode (default)
 *   node index.js --live              # Real orders via CLOB API
 *   node index.js --verbose           # Detailed logging
 *   node index.js --stats             # Show trade/fill history
 *   node index.js --spread 0.06       # Override spread (6c)
 *   node index.js --size 2.00         # Override quote size ($2 per side)
 *   node index.js --max-inventory 20  # Override max inventory ($20 per side)
 *
 * Env (from /Users/jonathon/5dlabs/cto/skills/trader/.env):
 *   POLYMARKET_API_KEY      — CLOB API key (required for --live)
 *   POLYMARKET_SECRET       — CLOB HMAC secret (required for --live)
 *   POLYMARKET_PASSPHRASE   — CLOB passphrase (required for --live)
 *   SIMMER_API_KEY          — Simmer SDK key for contract discovery (optional)
 *
 * Logs: logs/mm-trades.jsonl
 */

import { config as dotenvConfig } from "dotenv";
import path from "path";
import { fileURLToPath } from "url";
import { createCexFeed } from "../temporal-arb/cex_feed.js";
import { createPolymarketScanner } from "../temporal-arb/polymarket_scanner.js";
import { createClobClient } from "./clob_client.js";
import { createMMEngine } from "./mm_engine.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Load env: parent trader dir first, then local overrides
dotenvConfig({ path: path.join(__dirname, "..", ".env") });
dotenvConfig({ path: path.join(__dirname, ".env") });

// -------------------------------------------------------------------------
// CLI argument parsing
// -------------------------------------------------------------------------

const args = process.argv.slice(2);
const LIVE_MODE = args.includes("--live");
const VERBOSE = args.includes("--verbose") || args.includes("-v");
const STATS_ONLY = args.includes("--stats");
const QUIET = args.includes("--quiet") || args.includes("-q");

function getArg(flag) {
  const idx = args.indexOf(flag);
  if (idx === -1 || idx + 1 >= args.length) return null;
  return args[idx + 1];
}

// Config overrides from CLI
const cliSpread = getArg("--spread");
const cliSize = getArg("--size");
const cliMaxInventory = getArg("--max-inventory");
const cliMaxPosition = getArg("--max-position");
const cliLeanFactor = getArg("--lean-factor");
const cliInterval = getArg("--interval");

const TICK_INTERVAL_MS = parseInt(cliInterval || process.env.MM_TICK_INTERVAL || "3000", 10);
const SCAN_INTERVAL_CYCLES = 10; // Re-scan Polymarket every N ticks

// -------------------------------------------------------------------------
// Stats display
// -------------------------------------------------------------------------

async function showStats() {
  const fs = await import("fs");
  const logPath = path.join(__dirname, "logs", "mm-trades.jsonl");

  try {
    if (!fs.existsSync(logPath)) {
      console.log("No trade history found.");
      return;
    }

    const lines = fs.readFileSync(logPath, "utf8").trim().split("\n");
    const events = lines
      .map((l) => {
        try { return JSON.parse(l); } catch { return null; }
      })
      .filter(Boolean);

    const fills = events.filter((e) => e.type === "fill");
    const rebalances = events.filter((e) => e.type === "rebalance");

    console.log("=".repeat(60));
    console.log("  POLYMARKET MM — Trade History");
    console.log("=".repeat(60));
    console.log(`  Total events: ${events.length}`);
    console.log(`  Fills: ${fills.length}`);
    console.log(`  Rebalances: ${rebalances.length}`);

    if (fills.length > 0) {
      const totalCost = fills.reduce((s, f) => s + (f.cost || 0), 0);
      const totalRebate = fills.reduce((s, f) => s + (f.estimatedRebate || 0), 0);
      console.log(`\n  Total fill volume: $${totalCost.toFixed(2)}`);
      console.log(`  Estimated rebates: $${totalRebate.toFixed(4)}`);

      console.log("\n  Recent fills:");
      for (const f of fills.slice(-15)) {
        console.log(
          `    ${f.ts} | ${f.asset} ${f.orderSide} ${f.tokenSide} ` +
            `${(f.size || 0).toFixed(1)} @ $${(f.price || 0).toFixed(2)} ` +
            `($${(f.cost || 0).toFixed(2)}) rebate=$${(f.estimatedRebate || 0).toFixed(4)}`
        );
      }
    }
  } catch (e) {
    console.error("Failed to read stats:", e.message);
  }
}

// -------------------------------------------------------------------------
// Main
// -------------------------------------------------------------------------

async function main() {
  console.log("=".repeat(60));
  console.log("  POLYMARKET MARKET MAKER — 5-min Crypto Fast Markets");
  console.log("=".repeat(60));
  console.log();

  if (STATS_ONLY) {
    await showStats();
    return;
  }

  // Build config from defaults + CLI overrides + env
  const mmConfig = {};
  if (cliSpread) mmConfig.spread = parseFloat(cliSpread);
  if (cliSize) mmConfig.quoteSize = parseFloat(cliSize);
  if (cliMaxInventory) mmConfig.maxInventory = parseFloat(cliMaxInventory);
  if (cliMaxPosition) mmConfig.maxPosition = parseFloat(cliMaxPosition);
  if (cliLeanFactor) mmConfig.leanFactor = parseFloat(cliLeanFactor);

  // Env overrides
  if (process.env.MM_SPREAD) mmConfig.spread = parseFloat(process.env.MM_SPREAD);
  if (process.env.MM_QUOTE_SIZE) mmConfig.quoteSize = parseFloat(process.env.MM_QUOTE_SIZE);
  if (process.env.MM_MAX_INVENTORY) mmConfig.maxInventory = parseFloat(process.env.MM_MAX_INVENTORY);
  if (process.env.MM_MAX_POSITION) mmConfig.maxPosition = parseFloat(process.env.MM_MAX_POSITION);
  if (process.env.MM_LEAN_FACTOR) mmConfig.leanFactor = parseFloat(process.env.MM_LEAN_FACTOR);

  const effectiveConfig = {
    spread: mmConfig.spread || 0.04,
    quoteSize: mmConfig.quoteSize || 1.0,
    maxInventory: mmConfig.maxInventory || 10.0,
    leanFactor: mmConfig.leanFactor || 0.5,
    maxPosition: mmConfig.maxPosition || 5.0,
    rebalanceThreshold: 0.7,
    ...mmConfig,
  };

  console.log(`  MODE: ${LIVE_MODE ? "LIVE (real CLOB orders)" : "PAPER (simulated fills)"}`);
  console.log(`  Tick interval: ${TICK_INTERVAL_MS / 1000}s`);
  console.log(`  Spread: ${(effectiveConfig.spread * 100).toFixed(1)}c each side`);
  console.log(`  Quote size: $${effectiveConfig.quoteSize.toFixed(2)} per side`);
  console.log(`  Max inventory: $${effectiveConfig.maxInventory.toFixed(2)} per side`);
  console.log(`  Max position: $${effectiveConfig.maxPosition.toFixed(2)} net`);
  console.log(`  Lean factor: ${effectiveConfig.leanFactor}`);
  console.log(`  Assets: BTC, ETH, SOL`);
  console.log();

  // Validate live mode credentials
  if (LIVE_MODE) {
    const hasKey = process.env.POLYMARKET_API_KEY;
    const hasSecret = process.env.POLYMARKET_SECRET;
    const hasPass = process.env.POLYMARKET_PASSPHRASE;

    if (!hasKey || !hasSecret || !hasPass) {
      console.error("ERROR: Live mode requires POLYMARKET_API_KEY, POLYMARKET_SECRET, and POLYMARKET_PASSPHRASE");
      console.error("Set these in /Users/jonathon/5dlabs/cto/skills/trader/.env");
      process.exit(1);
    }
    console.log("  CLOB credentials loaded");
  }

  // -----------------------------------------------------------------------
  // 1. Start CEX feed
  // -----------------------------------------------------------------------

  console.log("[1/4] Starting CEX price feed...");
  const cexFeed = createCexFeed({
    symbols: ["BTC/USDT", "ETH/USDT", "SOL/USDT"],
    verbose: VERBOSE,
    spikeThresholdPct: 0.3,
    onMomentumSpike: (spike) => {
      if (!QUIET) {
        console.log(
          `  [CEX SPIKE] ${spike.symbol} ${spike.direction} ${spike.pct.toFixed(3)}% ` +
            `($${spike.priceThen.toFixed(2)} -> $${spike.priceNow.toFixed(2)})`
        );
      }
    },
  });

  await cexFeed.start();

  // Warm up — need at least 2 ticks for momentum calculation
  console.log("[CEX] Warming up (8s)...");
  await sleep(8000);

  // -----------------------------------------------------------------------
  // 2. Create Polymarket scanner
  // -----------------------------------------------------------------------

  console.log("[2/4] Initializing Polymarket scanner...");
  const scanner = createPolymarketScanner({
    assets: ["BTC", "ETH", "SOL"],
    window: "5m",
    verbose: VERBOSE,
    simmerApiKey: process.env.SIMMER_API_KEY,
  });

  const initialContracts = await scanner.scan();
  console.log(`[Scanner] Found ${initialContracts.length} active contract(s)`);

  // -----------------------------------------------------------------------
  // 3. Create CLOB client
  // -----------------------------------------------------------------------

  console.log("[3/4] Creating CLOB client...");
  const clobClient = createClobClient({
    paperMode: !LIVE_MODE,
    verbose: VERBOSE,
  });

  console.log(`[CLOB] Mode: ${clobClient.paperMode ? "PAPER" : "LIVE"}`);

  // -----------------------------------------------------------------------
  // 4. Create MM engine
  // -----------------------------------------------------------------------

  console.log("[4/4] Starting market maker engine...\n");
  const engine = createMMEngine({
    clobClient,
    getCexMomentum: (symbol, windowMs) => cexFeed.getMomentum(symbol, windowMs),
    getCexPrice: (symbol) => cexFeed.getCurrentPrice(symbol),
    getContracts: () => scanner.getActiveContracts(),
    verbose: VERBOSE,
    config: effectiveConfig,
  });

  console.log("=".repeat(60));
  console.log("  Market maker running. Press Ctrl+C to stop.");
  console.log("=".repeat(60));
  console.log();

  // -----------------------------------------------------------------------
  // Main tick loop
  // -----------------------------------------------------------------------

  let tickCount = 0;

  async function runOneTick() {
    tickCount++;
    const tickStart = Date.now();

    try {
      // Re-scan Polymarket periodically (contracts rotate every 5 min)
      if (tickCount % SCAN_INTERVAL_CYCLES === 1 || scanner.getActiveContracts().length === 0) {
        if (!QUIET) console.log(`\n[Tick ${tickCount}] Scanning for active contracts...`);
        await scanner.scan();
        const contracts = scanner.getActiveContracts();
        if (!QUIET && contracts.length > 0) {
          for (const c of contracts) {
            const remaining = c.remainingMs ? `${Math.round(c.remainingMs / 1000)}s` : "?";
            const price = c.liveYesPrice != null ? `$${c.liveYesPrice.toFixed(3)}` : "N/A";
            console.log(`  ${c.asset} | YES: ${price} | ${remaining} remaining`);
          }
        }
      }

      // Run MM engine
      const result = await engine.runCycle();

      // Log significant events
      if (!QUIET) {
        if (result.fills.length > 0) {
          console.log(`  [FILLS] ${result.fills.length} fill(s) this tick`);
        }
        if (result.quoted > 0 && VERBOSE) {
          console.log(`  [QUOTES] ${result.quoted} placed, ${result.cancelled} cancelled`);
        }
      }

      // Periodic status display (every ~30 seconds)
      if (!QUIET && tickCount % 10 === 0) {
        printStatus(engine, cexFeed);
      }

      // Performance tracking
      const tickMs = Date.now() - tickStart;
      if (VERBOSE && tickMs > 1000) {
        console.log(`  [PERF] Tick ${tickCount} took ${tickMs}ms (target: ${TICK_INTERVAL_MS}ms)`);
      }
    } catch (e) {
      console.error(`[Tick ${tickCount}] Error: ${e.message}`);
      if (VERBOSE) console.error(e.stack);
    }
  }

  // First tick immediately
  await runOneTick();

  // Then run on interval
  const intervalId = setInterval(runOneTick, TICK_INTERVAL_MS);

  // -----------------------------------------------------------------------
  // Graceful shutdown
  // -----------------------------------------------------------------------

  async function shutdown() {
    console.log("\n\nShutting down market maker...");

    // Cancel all open quotes
    try {
      await engine.cancelAllQuotes();
    } catch (e) {
      console.error("Error cancelling quotes:", e.message);
    }

    // Save state
    engine.saveState();

    // Stop feeds
    clearInterval(intervalId);
    cexFeed.stop();

    // Print final report
    printFinalReport(engine);

    process.exit(0);
  }

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

// -------------------------------------------------------------------------
// Display helpers
// -------------------------------------------------------------------------

function printStatus(engine, cexFeed) {
  const stats = engine.getStats();
  const quotes = engine.getActiveQuoteSummary();
  const momentum = cexFeed.getAllMomentum();

  console.log("\n--- MM Status ---");
  console.log(
    `  Cycles: ${stats.cyclesRun} | Fills: ${stats.fills} | ` +
      `Quotes active: ${stats.activeQuoteCount} | ` +
      `Est. rebates: $${stats.pnl.rebatesEarned.toFixed(4)}`
  );

  // Inventory summary
  const inv = stats.inventory;
  console.log(
    `  Inventory: YES=${inv.totalYesShares} ($${inv.totalYesCost.toFixed(2)}) ` +
      `NO=${inv.totalNoShares} ($${inv.totalNoCost.toFixed(2)}) | ` +
      `Net: $${inv.netExposure.toFixed(2)} | Matched: ${inv.matchedPairs}`
  );

  // P&L
  console.log(
    `  P&L: spread=$${stats.pnl.spreadCaptured.toFixed(4)} ` +
      `rebates=$${stats.pnl.rebatesEarned.toFixed(4)} ` +
      `total=$${stats.pnl.totalEstimated.toFixed(4)}`
  );

  // Active quotes
  if (quotes.length > 0) {
    for (const q of quotes) {
      const askStr = q.impliedAsk ? `ask=$${q.impliedAsk.toFixed(2)}` : "no ask";
      console.log(
        `  ${q.asset}: bid=$${q.yesBid?.toFixed(2) || "---"} ${askStr} ` +
          `(fair=$${q.fairValue.toFixed(3)} spread=${(q.spread * 100).toFixed(1)}c ` +
          `lean=${q.lean >= 0 ? "+" : ""}${(q.lean * 100).toFixed(1)}c age=${q.age}s)`
      );
    }
  }

  // CEX prices
  for (const [sym, data] of Object.entries(momentum)) {
    const cur = data.current;
    const m1 = data["1m"];
    if (cur) {
      console.log(
        `  ${sym}: $${cur.price.toFixed(2)} (1m: ${m1?.pct?.toFixed(3) || "N/A"}%)`
      );
    }
  }
  console.log();
}

function printFinalReport(engine) {
  const stats = engine.getStats();

  console.log("\n" + "=".repeat(60));
  console.log("  MARKET MAKER — Final Report");
  console.log("=".repeat(60));
  console.log(`  Runtime: ${stats.startedAt} to ${new Date().toISOString()}`);
  console.log(`  Total cycles:      ${stats.cyclesRun}`);
  console.log(`  Quotes placed:     ${stats.quotesPlaced}`);
  console.log(`  Quotes cancelled:  ${stats.quotesCancelled}`);
  console.log(`  Fills:             ${stats.fills}`);
  console.log();

  // Inventory
  const inv = stats.inventory;
  console.log("  Inventory:");
  console.log(`    YES shares: ${inv.totalYesShares} (cost: $${inv.totalYesCost.toFixed(2)}, avg: $${inv.avgYesCost.toFixed(4)})`);
  console.log(`    NO shares:  ${inv.totalNoShares} (cost: $${inv.totalNoCost.toFixed(2)}, avg: $${inv.avgNoCost.toFixed(4)})`);
  console.log(`    Matched:    ${inv.matchedPairs}`);
  console.log(`    Net exposure: $${inv.netExposure.toFixed(2)}`);
  console.log();

  // P&L
  console.log("  P&L Estimate:");
  console.log(`    Spread captured: $${stats.pnl.spreadCaptured.toFixed(4)}`);
  console.log(`    Maker rebates:   $${stats.pnl.rebatesEarned.toFixed(4)}`);
  console.log(`    Total estimated: $${stats.pnl.totalEstimated.toFixed(4)}`);
  console.log();

  // Recent fills
  const history = engine.getHistory(20);
  const fills = history.filter((e) => e.type === "fill");
  if (fills.length > 0) {
    console.log(`  Recent fills (last ${fills.length}):`);
    for (const f of fills.slice(-10)) {
      console.log(
        `    ${f.ts} | ${f.asset} ${f.orderSide} ${f.tokenSide} ` +
          `${(f.size || 0).toFixed(1)} @ $${(f.price || 0).toFixed(2)} ` +
          `rebate=$${(f.estimatedRebate || 0).toFixed(4)}`
      );
    }
  }

  console.log();
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// -------------------------------------------------------------------------
// Entry point
// -------------------------------------------------------------------------

main().catch((e) => {
  console.error("Fatal error:", e);
  process.exit(1);
});
