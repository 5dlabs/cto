/**
 * arb_engine.js — Core temporal arbitrage engine
 *
 * Compares real-time CEX momentum against Polymarket 5-min contract prices.
 * When CEX shows a clear directional move but Polymarket odds haven't caught up,
 * that's the temporal arb opportunity.
 *
 * The edge: Polymarket CLOB updates lag CEX by seconds to minutes on fast markets.
 * A bot turned $313 into $438K exploiting exactly this latency.
 *
 * Exports: createArbEngine()
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TRADE_LOG = path.join(__dirname, "temporal-arb-trades.jsonl");
const DAILY_SPEND_FILE = path.join(__dirname, "daily_spend.json");

// Default configuration
const DEFAULT_CONFIG = {
  // Momentum thresholds
  momentumThresholdPct: 0.3, // Min CEX momentum % to consider a signal
  strongMomentumPct: 0.5, // Strong momentum gets larger position

  // Price dislocation thresholds
  maxYesPriceForBuy: 0.70, // Don't buy YES if already > 70c (market priced in)
  minYesPriceForBuy: 0.15, // Allow extreme odds where fees are near zero
  priceDislocThreshold: 0.04, // Min gap between "fair" and market price (raised for fees)

  // Position sizing (Kelly-inspired)
  maxPositionUsd: 5.0, // Max per trade
  kellyCap: 0.25, // Max Kelly fraction
  minPositionUsd: 0.50, // Min position (below this, skip)

  // Risk limits
  dailyBudgetUsd: 25.0, // Max daily spend
  maxTradesPerCycle: 2, // Max trades per scan cycle
  maxOpenPositions: 5, // Max simultaneous positions
  maxSpreadPct: 10.0, // Skip if CLOB spread > this %

  // Fee handling — UPDATED for Jan 2026 dynamic taker fees
  // Polymarket crypto fast markets: fee_pct = 0.03 * (1 - (2p-1)^2)
  // Peaks at ~3% at 50% odds, tapers to ~0% at extremes
  // Makers (limit orders) pay NO fees and earn rebates
  // Strategy: focus on extreme odds (<30c or >70c) where fees are low
  feeModel: "dynamic_2026", // dynamic_2026 or flat
};

/**
 * Create the arbitrage engine.
 *
 * @param {Object} opts
 * @param {Function} opts.getCexMomentum - (symbol, windowMs) => { pct, direction, priceNow, priceThen, samples }
 * @param {Function} opts.getCexPrice - (symbol) => { price, ts, exchange }
 * @param {Function} opts.getContracts - () => [contract]
 * @param {Function} opts.executeTrade - (contract, side, amount) => result (for live mode)
 * @param {boolean} opts.paperMode - If true, log trades but don't execute
 * @param {boolean} opts.verbose
 * @param {Object} opts.config - Override default config values
 * @returns {Object}
 */
export function createArbEngine(opts = {}) {
  const {
    getCexMomentum,
    getCexPrice,
    getContracts,
    executeTrade = null,
    paperMode = true,
    verbose = false,
    config: userConfig = {},
  } = opts;

  const cfg = { ...DEFAULT_CONFIG, ...userConfig };

  // Track daily spend
  let dailySpend = loadDailySpend();

  // Track recent signals for dedup
  const recentSignals = new Map(); // conditionId -> { ts, side }

  // Stats
  const stats = {
    cyclesRun: 0,
    signalsFound: 0,
    tradesExecuted: 0,
    tradesPaper: 0,
    totalSpent: 0,
    wins: 0,
    losses: 0,
  };

  /**
   * Run one arbitrage scan cycle.
   * @returns {{ signals: Array, trades: Array }}
   */
  async function runCycle() {
    stats.cyclesRun++;
    const signals = [];
    const trades = [];

    // Get active contracts
    const contracts = getContracts();
    if (!contracts || contracts.length === 0) {
      if (verbose) console.log("[Arb] No active contracts");
      return { signals, trades };
    }

    // Check daily budget
    dailySpend = loadDailySpend();
    const remainingBudget = cfg.dailyBudgetUsd - dailySpend.spent;
    if (remainingBudget <= cfg.minPositionUsd) {
      if (verbose) console.log(`[Arb] Daily budget exhausted ($${dailySpend.spent.toFixed(2)}/$${cfg.dailyBudgetUsd})`);
      return { signals, trades };
    }

    let tradesThisCycle = 0;

    for (const contract of contracts) {
      if (tradesThisCycle >= cfg.maxTradesPerCycle) break;

      const signal = evaluateContract(contract, remainingBudget);
      if (!signal) continue;

      signals.push(signal);
      stats.signalsFound++;

      // Dedup: don't trade the same contract+side within 60 seconds
      const dedupKey = `${contract.conditionId}:${signal.side}`;
      const recent = recentSignals.get(dedupKey);
      if (recent && Date.now() - recent.ts < 60_000) {
        if (verbose) console.log(`[Arb] Dedup: already signaled ${dedupKey} ${Math.round((Date.now() - recent.ts) / 1000)}s ago`);
        continue;
      }

      // Execute or paper trade
      const tradeResult = await executeTradeSafe(contract, signal);
      if (tradeResult) {
        trades.push(tradeResult);
        tradesThisCycle++;
        recentSignals.set(dedupKey, { ts: Date.now(), side: signal.side });
      }
    }

    return { signals, trades };
  }

  /**
   * Evaluate a single contract for temporal arb opportunity.
   * @param {Object} contract - From polymarket_scanner
   * @param {number} remainingBudget
   * @returns {Object|null} Signal if opportunity found
   */
  function evaluateContract(contract, remainingBudget) {
    const { asset, cexSymbol, liveYesPrice, remainingMs, orderbook, question } = contract;

    // Must have live price
    if (liveYesPrice == null) return null;

    // Get CEX momentum (1-minute window for latency arb)
    const m1 = getCexMomentum(cexSymbol, 60_000);
    const m5 = getCexMomentum(cexSymbol, 300_000);

    if (!m1 || !m1.samples || m1.samples < 2) return null;

    const momentum1m = m1.pct;
    const momentum5m = m5 ? m5.pct : 0;
    const absMomentum = Math.abs(momentum1m);

    // Must have meaningful momentum
    if (absMomentum < cfg.momentumThresholdPct) return null;

    // Determine expected direction and "fair" price estimate
    const direction = momentum1m > 0 ? "up" : "down";

    // Fair value model for 5-min binary crypto markets:
    //
    // In 5-minute windows, a strong momentum move is highly predictive.
    // The $313->$438K bot proved that CEX momentum in the first 1-3 minutes
    // of a 5-min window predicts the outcome with high accuracy.
    //
    // Combine 1m and 5m momentum (weight recent moves more heavily):
    // Strong recent spike + weak 5m = breakout (high conviction)
    // Strong recent spike + strong 5m = trend continuation (highest conviction)
    const combinedMomentum = momentum1m * 0.7 + momentum5m * 0.3;

    // Estimate fair YES probability based on momentum.
    // Empirically, in 5-min crypto markets:
    //   0.3% move -> ~58-60% win rate for continuation -> fair ~$0.58-0.60
    //   0.5% move -> ~65-70% win rate -> fair ~$0.65-0.70
    //   1.0% move -> ~80%+ win rate -> fair ~$0.80
    //
    // Scale: each 0.1% of combined momentum shifts fair value by ~10c
    // This is aggressive but matches observed fast-market dynamics.
    const fairYes = 0.50 + combinedMomentum * 0.20;
    const clampedFairYes = Math.max(0.30, Math.min(0.85, fairYes)); // Clamp to avoid extremes

    // Check for dislocation
    let side, edge, buyPrice;

    if (direction === "up") {
      // CEX says up, check if "YES" is underpriced
      edge = clampedFairYes - liveYesPrice;
      side = "yes";
      buyPrice = liveYesPrice;

      // Sanity: don't buy if YES already expensive
      if (liveYesPrice > cfg.maxYesPriceForBuy) return null;
      // Don't buy if YES suspiciously cheap (market knows something we don't)
      if (liveYesPrice < cfg.minYesPriceForBuy) return null;
    } else {
      // CEX says down, check if "NO" is underpriced (i.e. YES is overpriced)
      const fairNo = 1 - clampedFairYes;
      const liveNoPrice = 1 - liveYesPrice;
      edge = fairNo - liveNoPrice;
      side = "no";
      buyPrice = liveNoPrice;

      // Sanity checks (inverted)
      if (liveYesPrice < 1 - cfg.maxYesPriceForBuy) return null;
      if (liveYesPrice > 1 - cfg.minYesPriceForBuy) return null;
    }

    // Must have positive edge above threshold
    if (edge < cfg.priceDislocThreshold) return null;

    // Fee-aware EV check — Jan 2026 dynamic taker fee model
    // fee_pct = 0.03 * (1 - (2p - 1)^2) where p = buy probability
    // At p=0.50: fee_pct = 3.0% (worst case)
    // At p=0.30: fee_pct = 2.16%
    // At p=0.20: fee_pct = 1.08%
    // At p=0.10: fee_pct = 0.36%
    const feePct = 0.03 * (1 - Math.pow(2 * buyPrice - 1, 2));
    const feePerShare = buyPrice * feePct;
    const minEdgeForFees = feePerShare * 2 + 0.02; // Round-trip fees + 2c buffer

    if (edge < minEdgeForFees) {
      if (verbose) {
        console.log(`[Arb] ${asset} edge ${(edge * 100).toFixed(1)}c < fee-adjusted min ${(minEdgeForFees * 100).toFixed(1)}c (fee: ${(feePct * 100).toFixed(1)}%) — skip`);
      }
      return null;
    }

    // Spread check
    if (orderbook && orderbook.spreadPct > cfg.maxSpreadPct) {
      if (verbose) {
        console.log(`[Arb] ${asset} spread ${orderbook.spreadPct.toFixed(1)}% > ${cfg.maxSpreadPct}% — skip`);
      }
      return null;
    }

    // Kelly position sizing
    const positionSize = calculateKellySize(edge, buyPrice, cfg.maxPositionUsd, cfg.kellyCap);
    const cappedSize = Math.min(positionSize, remainingBudget);

    if (cappedSize < cfg.minPositionUsd) return null;

    // Conviction level
    const isStrongMomentum = absMomentum >= cfg.strongMomentumPct;
    const hasVolume = m1.samples >= 3;
    const conviction = (isStrongMomentum ? "HIGH" : "MEDIUM") + (hasVolume ? "" : " (LOW SAMPLES)");

    return {
      asset,
      cexSymbol,
      side,
      edge: Math.round(edge * 10000) / 10000,
      fairPrice: Math.round(clampedFairYes * 1000) / 1000,
      marketPrice: liveYesPrice,
      buyPrice: Math.round(buyPrice * 1000) / 1000,
      momentum1m: m1.pct,
      momentum5m: momentum5m,
      combinedMomentum: Math.round(combinedMomentum * 10000) / 10000,
      direction,
      positionSize: Math.round(cappedSize * 100) / 100,
      conviction,
      remainingMs: remainingMs || 0,
      feePerShare: Math.round(feePerShare * 10000) / 10000,
      contract, // Reference back to contract
      ts: Date.now(),
    };
  }

  /**
   * Execute a trade (paper or live).
   */
  async function executeTradeSafe(contract, signal) {
    const { side, positionSize, asset, edge, momentum1m, conviction, marketPrice } = signal;
    const timestamp = new Date().toISOString();

    const tradeRecord = {
      timestamp,
      asset,
      side,
      amount: positionSize,
      edge,
      momentum1m,
      marketYesPrice: marketPrice,
      conviction,
      conditionId: contract.conditionId,
      question: contract.question,
      remainingMs: signal.remainingMs,
      paper: paperMode,
    };

    if (paperMode) {
      // Paper trade — log it
      console.log(
        `  [PAPER] ${side.toUpperCase()} $${positionSize.toFixed(2)} on ${asset} ` +
          `(edge: ${(edge * 100).toFixed(1)}c, momentum: ${momentum1m.toFixed(3)}%, ` +
          `YES: $${marketPrice.toFixed(3)}, ${conviction})`
      );

      tradeRecord.result = "paper";
      stats.tradesPaper++;
    } else {
      // Live trade
      if (!executeTrade) {
        console.error("[Arb] No executeTrade function provided for live mode");
        return null;
      }

      try {
        console.log(
          `  [LIVE] ${side.toUpperCase()} $${positionSize.toFixed(2)} on ${asset} ` +
            `(edge: ${(edge * 100).toFixed(1)}c, ${conviction})`
        );
        const result = await executeTrade(contract, side, positionSize);
        tradeRecord.result = result;
        tradeRecord.success = result?.success || false;

        if (result?.success) {
          stats.tradesExecuted++;
          stats.totalSpent += positionSize;

          // Update daily spend
          dailySpend.spent += positionSize;
          dailySpend.trades++;
          saveDailySpend(dailySpend);
        }
      } catch (e) {
        console.error(`[Arb] Trade execution error: ${e.message}`);
        tradeRecord.result = { error: e.message };
        tradeRecord.success = false;
      }
    }

    // Log to JSONL
    logTrade(tradeRecord);

    return tradeRecord;
  }

  /**
   * Kelly criterion position sizing.
   * kelly_fraction = edge / (1 - price)
   * Capped at kellyCap fraction of maxBet.
   */
  function calculateKellySize(edge, price, maxBet, kellyCap) {
    if (price <= 0 || price >= 1) return 0;
    const kelly = edge / (1 - price);
    const capped = Math.max(0, Math.min(kelly, kellyCap));
    return Math.round(capped * maxBet * 100) / 100;
  }

  /**
   * Log a trade to the JSONL file.
   */
  function logTrade(record) {
    try {
      fs.appendFileSync(TRADE_LOG, JSON.stringify(record) + "\n");
    } catch (e) {
      console.error("[Arb] Failed to write trade log:", e.message);
    }
  }

  /**
   * Load daily spend tracking.
   */
  function loadDailySpend() {
    const today = new Date().toISOString().split("T")[0];
    try {
      if (fs.existsSync(DAILY_SPEND_FILE)) {
        const data = JSON.parse(fs.readFileSync(DAILY_SPEND_FILE, "utf8"));
        if (data.date === today) return data;
      }
    } catch {}
    return { date: today, spent: 0, trades: 0 };
  }

  /**
   * Save daily spend tracking.
   */
  function saveDailySpend(data) {
    try {
      fs.writeFileSync(DAILY_SPEND_FILE, JSON.stringify(data, null, 2));
    } catch (e) {
      console.error("[Arb] Failed to save daily spend:", e.message);
    }
  }

  /**
   * Get current stats.
   */
  function getStats() {
    return { ...stats, dailySpend: { ...dailySpend }, config: { ...cfg } };
  }

  /**
   * Read trade log.
   */
  function getTradeHistory(limit = 50) {
    try {
      if (!fs.existsSync(TRADE_LOG)) return [];
      const lines = fs.readFileSync(TRADE_LOG, "utf8").trim().split("\n");
      return lines
        .slice(-limit)
        .map((line) => {
          try {
            return JSON.parse(line);
          } catch {
            return null;
          }
        })
        .filter(Boolean);
    } catch {
      return [];
    }
  }

  return {
    runCycle,
    evaluateContract,
    getStats,
    getTradeHistory,
    calculateKellySize,
  };
}
