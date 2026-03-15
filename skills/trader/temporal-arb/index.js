#!/usr/bin/env node
/**
 * Temporal Arbitrage Strategy — Polymarket 5-min Crypto Markets
 *
 * Exploits the latency between CEX price movements and Polymarket CLOB odds updates.
 * When BTC spikes on Binance but the "BTC Up" contract is still cheap, buy it.
 *
 * A bot turned $313 into $438K using this exact approach.
 *
 * Architecture:
 *   CEX Feed (Binance/Coinbase) → real-time price + momentum
 *   Polymarket Scanner → active 5-min contracts + live CLOB prices
 *   Arb Engine → detect dislocations, size via Kelly, execute/paper trade
 *
 * Usage:
 *   node index.js              # Paper mode (default)
 *   node index.js --live       # Real trades via Simmer SDK
 *   node index.js --verbose    # Detailed logging
 *   node index.js --stats      # Show trade history and stats
 *
 * Requires: SIMMER_API_KEY in .env (or parent .env)
 */

import { config as dotenvConfig } from "dotenv";
import path from "path";
import { fileURLToPath } from "url";
import { createCexFeed } from "./cex_feed.js";
import { createPolymarketScanner } from "./polymarket_scanner.js";
import { createArbEngine } from "./arb_engine.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Load .env from parent trader directory, then local
dotenvConfig({ path: path.join(__dirname, "..", ".env") });
dotenvConfig({ path: path.join(__dirname, ".env") });

// --- CLI Args ---
const args = process.argv.slice(2);
const LIVE_MODE = args.includes("--live");
const VERBOSE = args.includes("--verbose") || args.includes("-v");
const STATS_ONLY = args.includes("--stats");
const QUIET = args.includes("--quiet") || args.includes("-q");
const SCAN_INTERVAL_MS = parseInt(getArg("--interval") || "5000", 10); // Default 5s

function getArg(flag) {
  const idx = args.indexOf(flag);
  if (idx === -1 || idx + 1 >= args.length) return null;
  return args[idx + 1];
}

// --- Simmer SDK integration for live trading ---
let simmerClient = null;

async function initSimmerClient() {
  if (!LIVE_MODE) return;

  const apiKey = process.env.SIMMER_API_KEY;
  if (!apiKey) {
    console.error("[Error] SIMMER_API_KEY not set. Required for live trading.");
    console.error("  Get your key from: simmer.markets/dashboard -> SDK tab");
    process.exit(1);
  }

  // Dynamic import of simmer-sdk (Python SDK — we use HTTP API instead)
  // For JS, we use the Simmer REST API directly via axios
  console.log("[Simmer] Live mode — trades will execute via Simmer API");
  simmerClient = { apiKey, live: true };
}

async function executeTradeViaSimmer(contract, side, amount) {
  const { default: axios } = await import("axios");

  // If contract has a Simmer market_id, trade directly
  // Otherwise, import first then trade
  let marketId = contract.marketId;

  if (!marketId && contract.slug) {
    // Import the market to Simmer
    try {
      const importResp = await axios.post(
        "https://api.simmer.markets/api/sdk/import",
        { url: `https://polymarket.com/event/${contract.slug}` },
        {
          headers: {
            Authorization: `Bearer ${simmerClient.apiKey}`,
            "Content-Type": "application/json",
          },
          timeout: 15000,
        }
      );
      marketId = importResp.data?.market_id;
      if (!marketId) {
        return { success: false, error: "Import returned no market_id" };
      }
    } catch (e) {
      return { success: false, error: `Import failed: ${e.message}` };
    }
  }

  if (!marketId) {
    return { success: false, error: "No market_id and no slug to import" };
  }

  // Execute trade
  try {
    const tradeResp = await axios.post(
      "https://api.simmer.markets/api/sdk/trade",
      {
        market_id: marketId,
        side,
        amount,
        source: "sdk:temporal-arb",
        skill_slug: "temporal-arb",
      },
      {
        headers: {
          Authorization: `Bearer ${simmerClient.apiKey}`,
          "Content-Type": "application/json",
        },
        timeout: 15000,
      }
    );

    const data = tradeResp.data;
    return {
      success: data?.success || data?.trade_id != null,
      tradeId: data?.trade_id,
      sharesBought: data?.shares_bought,
      simulated: data?.simulated,
      error: data?.error,
    };
  } catch (e) {
    const errData = e.response?.data;
    return {
      success: false,
      error: errData?.detail || errData?.error || e.message,
    };
  }
}

// --- Main ---

async function main() {
  console.log("=".repeat(60));
  console.log("  TEMPORAL ARBITRAGE — Polymarket 5-min Crypto Markets");
  console.log("=".repeat(60));
  console.log();

  if (LIVE_MODE) {
    console.log("  MODE: LIVE (real trades via Simmer)");
    await initSimmerClient();
  } else {
    console.log("  MODE: PAPER (simulated trades, logged to temporal-arb-trades.jsonl)");
  }

  console.log(`  Scan interval: ${SCAN_INTERVAL_MS / 1000}s`);
  console.log(`  Assets: BTC, ETH, SOL`);
  console.log(`  CEX sources: Binance, Coinbase`);
  console.log();

  // Show stats and exit
  if (STATS_ONLY) {
    showStats();
    return;
  }

  // 1. Start CEX feed
  console.log("[1/3] Starting CEX price feed...");
  const cexFeed = createCexFeed({
    symbols: ["BTC/USDT", "ETH/USDT", "SOL/USDT"],
    verbose: VERBOSE,
    spikeThresholdPct: 0.2,
    onMomentumSpike: (spike) => {
      if (!QUIET) {
        console.log(
          `\n  [SPIKE] ${spike.symbol} ${spike.direction} ${spike.pct.toFixed(3)}% ` +
            `($${spike.priceThen.toFixed(2)} -> $${spike.priceNow.toFixed(2)})`
        );
      }
    },
  });

  await cexFeed.start();

  // Wait for initial price data (need at least 2 ticks for momentum)
  console.log("[CEX] Waiting for initial price data (10s warmup)...");
  await sleep(10000);

  // 2. Create Polymarket scanner
  console.log("[2/3] Initializing Polymarket scanner...");
  const scanner = createPolymarketScanner({
    assets: ["BTC", "ETH", "SOL"],
    window: "5m",
    verbose: VERBOSE,
    simmerApiKey: process.env.SIMMER_API_KEY,
  });

  // Initial scan
  const initialContracts = await scanner.scan();
  console.log(`[Scanner] Found ${initialContracts.length} active contracts`);

  // 3. Create arb engine
  console.log("[3/3] Starting arb engine...\n");
  const engine = createArbEngine({
    getCexMomentum: (symbol, windowMs) => cexFeed.getMomentum(symbol, windowMs),
    getCexPrice: (symbol) => cexFeed.getCurrentPrice(symbol),
    getContracts: () => scanner.getActiveContracts(),
    executeTrade: LIVE_MODE ? executeTradeViaSimmer : null,
    paperMode: !LIVE_MODE,
    verbose: VERBOSE,
    config: {
      momentumThresholdPct: parseFloat(process.env.TEMPORAL_ARB_MOMENTUM_THRESHOLD || "0.3"),
      maxPositionUsd: parseFloat(process.env.TEMPORAL_ARB_MAX_POSITION || "5.0"),
      dailyBudgetUsd: parseFloat(process.env.TEMPORAL_ARB_DAILY_BUDGET || "25.0"),
      kellyCap: parseFloat(process.env.TEMPORAL_ARB_KELLY_CAP || "0.25"),
    },
  });

  console.log("--- Temporal Arb Engine Running ---");
  console.log("Press Ctrl+C to stop\n");

  // Main loop
  let cycleCount = 0;

  async function runOneCycle() {
    cycleCount++;
    const cycleStart = Date.now();

    try {
      // Re-scan Polymarket every 30 seconds (contracts change every 5 min)
      if (cycleCount % 6 === 1 || scanner.getActiveContracts().length === 0) {
        if (!QUIET) console.log(`\n[Cycle ${cycleCount}] Scanning Polymarket for active contracts...`);
        await scanner.scan();
      }

      const contracts = scanner.getActiveContracts();
      if (contracts.length === 0) {
        if (!QUIET && cycleCount % 12 === 0) {
          console.log(`[Cycle ${cycleCount}] No active contracts — waiting for next market window`);
        }
        return;
      }

      // Run arb engine
      const { signals, trades } = await engine.runCycle();

      if (signals.length > 0 && !QUIET) {
        console.log(`\n[Cycle ${cycleCount}] ${signals.length} signal(s) found:`);
        for (const s of signals) {
          console.log(
            `  ${s.asset} ${s.direction.toUpperCase()} | ` +
              `momentum: ${s.momentum1m.toFixed(3)}% | ` +
              `edge: ${(s.edge * 100).toFixed(1)}c | ` +
              `YES: $${s.marketPrice.toFixed(3)} vs fair: $${s.fairPrice.toFixed(3)} | ` +
              `${s.conviction}`
          );
        }
      }

      if (trades.length > 0) {
        console.log(`  Trades: ${trades.length}`);
      }

      // Periodic status (every 60 seconds)
      if (!QUIET && cycleCount % 12 === 0) {
        printStatus(engine, cexFeed, contracts);
      }
    } catch (e) {
      console.error(`[Cycle ${cycleCount}] Error:`, e.message);
      if (VERBOSE) console.error(e.stack);
    }
  }

  // Run first cycle immediately
  await runOneCycle();

  // Then run on interval
  const intervalId = setInterval(runOneCycle, SCAN_INTERVAL_MS);

  // Graceful shutdown
  process.on("SIGINT", () => {
    console.log("\n\nShutting down...");
    clearInterval(intervalId);
    cexFeed.stop();
    printFinalReport(engine);
    process.exit(0);
  });

  process.on("SIGTERM", () => {
    clearInterval(intervalId);
    cexFeed.stop();
    process.exit(0);
  });
}

// --- Display helpers ---

function printStatus(engine, cexFeed, contracts) {
  const stats = engine.getStats();
  const momentum = cexFeed.getAllMomentum();

  console.log("\n--- Status ---");
  console.log(`  Cycles: ${stats.cyclesRun} | Signals: ${stats.signalsFound} | Paper trades: ${stats.tradesPaper} | Live trades: ${stats.tradesExecuted}`);
  console.log(`  Daily spend: $${stats.dailySpend.spent.toFixed(2)}/$${stats.config.dailyBudgetUsd}`);
  console.log(`  Active contracts: ${contracts.length}`);

  for (const [sym, data] of Object.entries(momentum)) {
    const cur = data.current;
    const m1 = data["1m"];
    const m5 = data["5m"];
    if (cur) {
      console.log(
        `  ${sym}: $${cur.price.toFixed(2)} | ` +
          `1m: ${m1?.pct?.toFixed(3) || "N/A"}% | ` +
          `5m: ${m5?.pct?.toFixed(3) || "N/A"}%`
      );
    }
  }
  console.log();
}

function printFinalReport(engine) {
  const stats = engine.getStats();
  const history = engine.getTradeHistory(100);

  console.log("\n" + "=".repeat(60));
  console.log("  FINAL REPORT");
  console.log("=".repeat(60));
  console.log(`  Total cycles:     ${stats.cyclesRun}`);
  console.log(`  Signals found:    ${stats.signalsFound}`);
  console.log(`  Paper trades:     ${stats.tradesPaper}`);
  console.log(`  Live trades:      ${stats.tradesExecuted}`);
  console.log(`  Total spent:      $${stats.totalSpent.toFixed(2)}`);

  if (history.length > 0) {
    console.log(`\n  Recent trades (last ${history.length}):`);
    for (const t of history.slice(-10)) {
      const time = new Date(t.timestamp).toLocaleTimeString();
      console.log(
        `    ${time} | ${t.asset} ${t.side.toUpperCase()} $${t.amount.toFixed(2)} | ` +
          `edge: ${(t.edge * 100).toFixed(1)}c | ${t.paper ? "PAPER" : "LIVE"}`
      );
    }
  }
  console.log();
}

function showStats() {
  // Create a temporary engine just to read history
  const engine = createArbEngine({
    getCexMomentum: () => null,
    getCexPrice: () => null,
    getContracts: () => [],
    paperMode: true,
  });

  const history = engine.getTradeHistory(200);
  const stats = engine.getStats();

  console.log("Trade History:");
  console.log("=".repeat(60));

  if (history.length === 0) {
    console.log("  No trades recorded yet.");
    return;
  }

  // Group by day
  const byDay = {};
  for (const t of history) {
    const day = t.timestamp?.split("T")[0] || "unknown";
    if (!byDay[day]) byDay[day] = [];
    byDay[day].push(t);
  }

  for (const [day, trades] of Object.entries(byDay)) {
    const totalSpent = trades.reduce((sum, t) => sum + (t.amount || 0), 0);
    const paper = trades.filter((t) => t.paper).length;
    const live = trades.length - paper;

    console.log(`\n  ${day}: ${trades.length} trades (${paper} paper, ${live} live) — $${totalSpent.toFixed(2)} total`);

    for (const t of trades.slice(-5)) {
      const time = new Date(t.timestamp).toLocaleTimeString();
      console.log(
        `    ${time} | ${t.asset} ${t.side?.toUpperCase()} $${(t.amount || 0).toFixed(2)} | ` +
          `edge: ${((t.edge || 0) * 100).toFixed(1)}c | mom: ${(t.momentum1m || 0).toFixed(3)}% | ` +
          `${t.paper ? "PAPER" : "LIVE"}`
      );
    }
  }

  console.log(`\n  Total trades: ${history.length}`);
  console.log(`  Total volume: $${history.reduce((s, t) => s + (t.amount || 0), 0).toFixed(2)}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// --- Entry point ---
main().catch((e) => {
  console.error("Fatal error:", e);
  process.exit(1);
});
