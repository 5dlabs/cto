#!/usr/bin/env node
/**
 * Grok X Search Information Arbitrage Engine
 *
 * Pipeline: Scan X -> Rank Credibility -> Match to Markets -> Generate Signals -> Execute/Log
 *
 * Usage:
 *   node index.js                # Run scan loop (paper mode, every 60s)
 *   node index.js --live         # Real trades via Simmer
 *   node index.js --once         # Single scan then exit
 *   node index.js --interval 120 # Custom interval (seconds)
 *   node index.js --topics "Iran,Fed"  # Override topics
 *   node index.js --verbose      # Verbose logging
 */

// --- Env loading MUST happen before any app imports ---
// In ESM, static `import` statements are hoisted and execute before module-level
// code, so dotenv must be loaded eagerly here, with app modules loaded via
// dynamic import() below to guarantee process.env is populated first.

import { config as loadEnv } from "dotenv";
import { resolve, dirname, join } from "path";
import { fileURLToPath } from "url";
import { appendFileSync, writeFileSync, existsSync, mkdirSync } from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load .env from trader root BEFORE any app modules read process.env.
loadEnv({ path: resolve(__dirname, "../.env") });

// Now dynamically import app modules — env vars are guaranteed available.
const { scanTopics, DEFAULT_TOPICS } = await import("./grok_scanner.js");
const { scorePosts } = await import("./credibility_ranker.js");
const { fetchActiveMarkets, matchNewsToMarkets, refreshMarketCache } = await import("./market_matcher.js");
const { generateSignals, executeSignal, getDailyStats } = await import("./signal_engine.js");

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const ARGS = parseArgs(process.argv.slice(2));

const CONFIG = {
  paperMode: !ARGS.live,
  interval: (ARGS.interval || 60) * 1000,
  once: ARGS.once || false,
  verbose: ARGS.verbose || false,
  topics: ARGS.topics
    ? ARGS.topics.split(",").map((t) => t.trim())
    : DEFAULT_TOPICS,

  // API keys
  grokApiKey: process.env.GROK_API_KEY,
  simmerApiKey: process.env.SIMMER_API_KEY,

  // Scanner settings
  recencyMinutes: parseInt(process.env.GROK_RECENCY_MINUTES || "60", 10),
  maxPostsPerTopic: parseInt(process.env.GROK_MAX_POSTS || "10", 10),
  timeoutMs: parseInt(process.env.GROK_TIMEOUT_MS || "120000", 10),

  // Signal thresholds
  minCredibility: parseInt(process.env.INFO_ARB_MIN_CREDIBILITY || "30", 10),
  minRelevance: parseFloat(process.env.INFO_ARB_MIN_RELEVANCE || "0.35"),
  minConfidence: parseFloat(process.env.INFO_ARB_MIN_CONFIDENCE || "0.25"),
  maxPositionUsd: parseFloat(process.env.INFO_ARB_MAX_USD || "25"),
  bankroll: parseFloat(process.env.INFO_ARB_BANKROLL || "100"),
};

// Log directories
const LOGS_DIR = join(__dirname, "logs");
const SIGNAL_LOG = join(LOGS_DIR, "info-arb-signals.jsonl");
const TRADE_LOG = join(LOGS_DIR, "info-arb-trades.jsonl");
const SCAN_LOG = join(LOGS_DIR, "info-arb-scans.jsonl");
const STATUS_FILE = join(__dirname, "status.json");

if (!existsSync(LOGS_DIR)) {
  mkdirSync(LOGS_DIR, { recursive: true });
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

function log(msg) {
  const ts = new Date().toISOString();
  console.log(`[${ts}] ${msg}`);
}

function logVerbose(msg) {
  if (CONFIG.verbose) log(msg);
}

function logJsonl(filepath, data) {
  try {
    appendFileSync(filepath, JSON.stringify(data) + "\n");
  } catch (err) {
    console.error(`Failed to write log ${filepath}: ${err.message}`);
  }
}

function writeHeartbeat(extra = {}) {
  try {
    writeFileSync(STATUS_FILE, JSON.stringify({
      updatedAt: new Date().toISOString(),
      mode: CONFIG.paperMode ? "paper" : "live",
      ...extra,
    }));
  } catch (err) {
    console.error(`Failed to write heartbeat: ${err.message}`);
  }
}

// ---------------------------------------------------------------------------
// Signal pipeline (steps 1-4): scan, rank, match, generate signals
// ---------------------------------------------------------------------------

async function runSignalPipeline() {
  const cycleStart = Date.now();
  const cycleId = `cycle_${cycleStart}`;

  log(`=== Scan Cycle ${cycleId} ===`);
  log(`Mode: ${CONFIG.paperMode ? "PAPER" : "LIVE"} | Topics: ${CONFIG.topics.length} | Recency: ${CONFIG.recencyMinutes}m`);

  // Step 1: Scan X via Grok
  log("Step 1/4: Scanning X for breaking news...");
  const scanResults = await scanTopics(CONFIG.topics, {
    apiKey: CONFIG.grokApiKey,
    recencyMinutes: CONFIG.recencyMinutes,
    maxResults: CONFIG.maxPostsPerTopic,
    timeoutMs: CONFIG.timeoutMs,
    delayBetweenMs: 2000, // Be polite to Grok API (grok-4 is slow)
  });

  let totalPosts = 0;
  for (const scan of scanResults) {
    totalPosts += scan.posts.length;
    if (scan.error) {
      log(`  [WARN] ${scan.topic}: ${scan.error}`);
    } else {
      logVerbose(`  ${scan.topic}: ${scan.posts.length} posts`);
    }
  }
  log(`  Found ${totalPosts} posts across ${CONFIG.topics.length} topics`);

  if (totalPosts === 0) {
    log("  No posts found. Skipping remaining steps.");
    writeHeartbeat({ cycleId, postsFound: 0, signals: 0, trades: 0 });
    return { cycleId, cycleStart, totalPosts, markets: [], matches: [], signals: [], scanResults };
  }

  // Step 2: Rank credibility
  log("Step 2/4: Ranking source credibility...");
  for (const scan of scanResults) {
    if (scan.posts.length > 0) {
      scan.posts = scorePosts(scan.posts);
      const highCred = scan.posts.filter((p) => p._tier === "high" || p._tier === "medium");
      logVerbose(`  ${scan.topic}: ${highCred.length}/${scan.posts.length} credible posts`);
    }
  }

  // Step 3: Fetch active markets and match news
  log("Step 3/4: Matching news to Polymarket markets...");
  const markets = await fetchActiveMarkets({
    simmerApiKey: CONFIG.simmerApiKey,
  });
  log(`  ${markets.length} active event markets loaded`);

  const matches = matchNewsToMarkets(scanResults, markets, {
    minRelevance: CONFIG.minRelevance,
  });
  log(`  ${matches.length} news-market matches found`);

  if (matches.length === 0) {
    log("  No relevant matches. Skipping signal generation.");
    writeHeartbeat({ cycleId, postsFound: totalPosts, markets: markets.length, signals: 0, trades: 0 });
    return { cycleId, cycleStart, totalPosts, markets, matches: [], signals: [], scanResults };
  }

  // Log top matches
  for (const m of matches.slice(0, 5)) {
    logVerbose(
      `  Match: "${m.post.author}" -> "${m.market.question?.substring(0, 50)}..." ` +
        `(relevance: ${(m.relevance * 100).toFixed(0)}%, sentiment: ${m.sentiment.direction})`
    );
  }

  // Step 4: Generate signals
  log("Step 4/4: Generating trading signals...");
  const signals = generateSignals(matches, {
    paperMode: CONFIG.paperMode,
    minCredibility: CONFIG.minCredibility,
    minRelevance: CONFIG.minRelevance,
    minConfidence: CONFIG.minConfidence,
    maxPositionUsd: CONFIG.maxPositionUsd,
    bankroll: CONFIG.bankroll,
  });
  log(`  ${signals.length} signals generated`);

  // Log all signals
  for (const signal of signals) {
    log(
      `  SIGNAL: ${signal.side.toUpperCase()} $${signal.amount} on "${signal.market.question?.substring(0, 50)}..." ` +
        `(confidence: ${(signal.confidence * 100).toFixed(1)}%)`
    );
    log(`    Source: ${signal.sourcePost?.author} | ${signal.reason}`);

    logJsonl(SIGNAL_LOG, {
      cycleId,
      ...signal,
      market: { id: signal.market.id, question: signal.market.question },
    });
  }

  writeHeartbeat({ cycleId, postsFound: totalPosts, markets: markets.length, matches: matches.length, signals: signals.length, trades: 0 });
  return { cycleId, cycleStart, totalPosts, markets, matches, signals, scanResults };
}

// ---------------------------------------------------------------------------
// Execute signals (step 5): trade each approved signal
// ---------------------------------------------------------------------------

async function executeSignals(signals, cycleId) {
  log(`Executing ${CONFIG.paperMode ? "paper trades" : "live trades"} for ${signals.length} signals...`);
  let executed = 0;
  let totalAmount = 0;

  for (const signal of signals) {
    const result = await executeSignal(signal, {
      simmerApiKey: CONFIG.simmerApiKey,
      paperMode: CONFIG.paperMode,
    });

    if (result.success) {
      executed++;
      totalAmount += signal.amount;
      log(
        `  ${result.paper ? "[PAPER]" : "[LIVE]"} ` +
          `${signal.side.toUpperCase()} $${signal.amount} -> ` +
          `${result.sharesBought} shares @ $${result.avgPrice?.toFixed(3) || "?"} ` +
          `on "${signal.market.question?.substring(0, 40)}..."`
      );
    } else {
      log(`  [FAIL] ${signal.market.id}: ${result.error}`);
    }

    // Log trade result
    logJsonl(TRADE_LOG, {
      cycleId,
      ...result,
      signal: {
        confidence: signal.confidence,
        reason: signal.reason,
        topic: signal.topic,
      },
    });
  }

  return { executed, totalAmount };
}

// ---------------------------------------------------------------------------
// Full pipeline (scan + execute) — used in loop mode
// ---------------------------------------------------------------------------

async function runPipeline() {
  const { cycleId, cycleStart, totalPosts, markets, matches, signals, scanResults } = await runSignalPipeline();

  if (signals.length === 0) {
    logJsonl(SCAN_LOG, {
      cycleId,
      postsFound: totalPosts,
      marketsLoaded: markets.length,
      matchesFound: matches.length,
      signalsGenerated: 0,
      tradesExecuted: 0,
      durationMs: Date.now() - cycleStart,
    });
    return;
  }

  const { executed, totalAmount } = await executeSignals(signals, cycleId);

  // Cycle summary
  const durationMs = Date.now() - cycleStart;
  const stats = getDailyStats();

  log(`\n=== Cycle Complete (${(durationMs / 1000).toFixed(1)}s) ===`);
  log(`  Posts scanned: ${totalPosts}`);
  log(`  Markets checked: ${markets.length}`);
  log(`  Matches found: ${matches.length}`);
  log(`  Signals generated: ${signals.length}`);
  log(`  Trades ${CONFIG.paperMode ? "simulated" : "executed"}: ${executed}`);
  log(`  Total amount: $${totalAmount.toFixed(2)}`);
  log(`  Daily stats: ${stats.tradesExecuted} trades, $${stats.totalExposure.toFixed(2)} exposure`);

  logJsonl(SCAN_LOG, {
    cycleId,
    postsFound: totalPosts,
    marketsLoaded: markets.length,
    matchesFound: matches.length,
    signalsGenerated: signals.length,
    tradesExecuted: executed,
    totalAmountUsd: totalAmount,
    durationMs,
    dailyStats: stats,
    paperMode: CONFIG.paperMode,
  });

  writeHeartbeat({ cycleId, postsFound: totalPosts, markets: markets.length, matches: matches.length, signals: signals.length, trades: executed });
}

// ---------------------------------------------------------------------------
// Scan loop
// ---------------------------------------------------------------------------

async function main() {
  console.log(`
  +-----------------------------------------+
  |  Grok X Search Information Arbitrage Engine  |
  |  Event-driven Polymarket trading             |
  +-----------------------------------------+
  `);

  log(`Mode: ${CONFIG.paperMode ? "PAPER (use --live for real trades)" : "LIVE TRADING"}`);
  log(`Topics: ${CONFIG.topics.join(", ")}`);
  log(`Interval: ${CONFIG.interval / 1000}s`);
  log(`Grok API: ${CONFIG.grokApiKey ? "configured" : "MISSING"}`);
  log(`Simmer API: ${CONFIG.simmerApiKey ? "configured" : "MISSING"}`);
  log(`Signal log: ${SIGNAL_LOG}`);
  log(`Trade log: ${TRADE_LOG}`);
  console.log();

  if (!CONFIG.grokApiKey) {
    log("WARNING: GROK_API_KEY not set — Grok X search disabled.");
    if (process.env.GEMINI_API_KEY) {
      log("Gemini fallback is available — running in Gemini-only mode.");
    } else {
      console.error("ERROR: Neither GROK_API_KEY nor GEMINI_API_KEY is set. No search provider available.");
      process.exit(1);
    }
  }

  // Agent integration: signals-only mode (outputs JSON to stdout for LLM evaluation)
  if (ARGS.signalsOnly) {
    const { signals } = await runSignalPipeline();
    // Output clean JSON to stdout for agent consumption
    console.log(JSON.stringify(signals));
    process.exit(0);
  }

  // Agent integration: execute a single pre-approved signal
  if (ARGS.executeSignal) {
    const signal = JSON.parse(ARGS.executeSignal);
    const result = await executeSignal(signal, {
      simmerApiKey: CONFIG.simmerApiKey,
      paperMode: CONFIG.paperMode,
    });
    console.log(JSON.stringify(result));
    process.exit(0);
  }

  if (CONFIG.once) {
    await runPipeline();
    process.exit(0);
  }

  // Main scan loop
  while (true) {
    try {
      await runPipeline();
    } catch (err) {
      log(`[ERROR] Pipeline failed: ${err.message}`);
      console.error(err.stack);
    }

    log(`Sleeping ${CONFIG.interval / 1000}s until next scan...`);
    await sleep(CONFIG.interval);

    // Refresh market cache every 5 cycles
    if (Date.now() % (CONFIG.interval * 5) < CONFIG.interval) {
      logVerbose("Refreshing market cache...");
      await refreshMarketCache({ simmerApiKey: CONFIG.simmerApiKey });
    }
  }
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs(args) {
  const result = {};
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--live") result.live = true;
    else if (arg === "--once") result.once = true;
    else if (arg === "--verbose" || arg === "-v") result.verbose = true;
    else if (arg === "--interval" && args[i + 1]) result.interval = parseInt(args[++i], 10);
    else if (arg === "--topics" && args[i + 1]) result.topics = args[++i];
    else if (arg === "--recency" && args[i + 1]) result.recencyMinutes = parseInt(args[++i], 10);
    else if (arg === "--signals-only") result.signalsOnly = true;
    else if (arg === "--execute-signal" && args[i + 1]) result.executeSignal = args[++i];
    else if (arg === "--help" || arg === "-h") {
      console.log(`
  Grok X Search Information Arbitrage Engine

  Usage:
    node index.js              Run scan loop in paper mode
    node index.js --live       Run with live Simmer execution
    node index.js --once       Single scan then exit
    node index.js --verbose    Verbose logging

  Options:
    --live                     Execute real trades via Simmer API
    --once                     Run once then exit
    --interval <sec>           Scan interval in seconds (default: 60)
    --topics "Iran,Fed,..."    Override scan topics
    --recency <min>            Post recency window in minutes (default: 15)
    --signals-only             Run pipeline and output signals as JSON to stdout, then exit
    --execute-signal <json>    Execute a single pre-approved signal (JSON string)
    --verbose, -v              Verbose logging
    --help, -h                 Show this help

  Environment variables:
    GROK_API_KEY               Grok API key (required)
    SIMMER_API_KEY             Simmer API key (for execution)
    INFO_ARB_MIN_CREDIBILITY   Min credibility score (default: 30)
    INFO_ARB_MIN_RELEVANCE     Min market relevance (default: 0.35)
    INFO_ARB_MIN_CONFIDENCE    Min signal confidence (default: 0.25)
    INFO_ARB_MAX_USD           Max position size USD (default: 25)
    INFO_ARB_BANKROLL          Total bankroll for Kelly sizing (default: 100)
      `);
      process.exit(0);
    }
  }
  return result;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
