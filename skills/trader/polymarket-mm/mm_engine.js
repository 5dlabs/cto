/**
 * mm_engine.js — Core market maker engine for Polymarket 5-min crypto markets
 *
 * Strategy: Place two-sided limit orders (YES bid + NO bid) on active contracts,
 * earn the bid-ask spread plus maker rebates from Polymarket's Jan 2026 fee model.
 *
 * Key dynamics:
 *   - Polymarket charges dynamic taker fees: fee_pct = 0.03 * (1 - (2p-1)^2)
 *     Peaks at 3% at 50c, tapers to 0% at extremes.
 *   - Taker fees are redistributed to MAKERS as USDC rebates.
 *   - Making (limit orders resting on book) pays ZERO fees.
 *   - The profitable edge: earn spread + rebates, avoid adverse selection via
 *     CEX-informed quote adjustments.
 *
 * Quote logic:
 *   1. Start with fair value = CLOB midpoint (or CEX-implied probability)
 *   2. Place YES bid at (fair - half_spread) and NO bid at ((1-fair) - half_spread)
 *      Equivalent: YES bid = fair - hs, YES ask (via NO bid) = fair + hs
 *   3. Lean quotes toward the likely winning side using CEX momentum:
 *      If CEX says UP, raise both quotes slightly (more likely to fill NO bid = sell YES)
 *   4. Skew quotes for inventory management:
 *      If holding too much YES, widen YES bid (less aggressive buying YES)
 *      and tighten NO bid (more aggressive selling YES)
 *   5. Cancel and replace when CEX momentum shifts materially
 *
 * Exports: createMMEngine()
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TRADE_LOG = path.join(__dirname, "logs", "mm-trades.jsonl");
const STATE_FILE = path.join(__dirname, "mm-state.json");

// -------------------------------------------------------------------------
// Default configuration
// -------------------------------------------------------------------------

const DEFAULT_CONFIG = {
  // Spread: total spread in probability points (e.g., 0.04 = 4c total, 2c each side)
  spread: 0.04,

  // Quote size: USD value per side per quote
  quoteSize: 1.0,

  // Max inventory: max USD worth of shares on one side before rebalancing
  maxInventory: 10.0,

  // Lean factor: how much to shift quotes based on CEX momentum (0-1)
  // 0 = ignore CEX, 1 = fully lean into CEX direction
  leanFactor: 0.5,

  // Max net directional exposure in USD
  maxPosition: 5.0,

  // Rebalance threshold: rebalance when inventory skew > this ratio (0-1)
  // skew = |yesShares - noShares| / (yesShares + noShares)
  rebalanceThreshold: 0.7,

  // Minimum spread: never quote tighter than this (avoids getting picked off)
  minSpread: 0.02,

  // Maximum spread: widen to this during high volatility
  maxSpread: 0.10,

  // Quote refresh: cancel/replace if fair value moves more than this
  requoteThresholdCents: 0.02,

  // Don't quote markets with < this many seconds remaining
  minRemainingSeconds: 45,

  // Skip markets too close to 50/50 (highest taker fees, most adverse selection)
  avoidMidrangeMin: 0.40,
  avoidMidrangeMax: 0.60,
  avoidMidrange: false, // Set true to skip 40-60c markets entirely

  // Momentum spike threshold: widen spread if 1m momentum exceeds this
  volatilityWidenPct: 0.5,
  volatilityWidenFactor: 1.5, // Multiply spread by this during spikes
};

/**
 * Create the market maker engine.
 *
 * @param {Object} opts
 * @param {Object} opts.clobClient      - CLOB client from clob_client.js
 * @param {Function} opts.getCexMomentum - (symbol, windowMs) => momentum object
 * @param {Function} opts.getCexPrice    - (symbol) => { price, ts, exchange }
 * @param {Function} opts.getContracts   - () => [contract] from polymarket_scanner
 * @param {boolean} opts.verbose
 * @param {Object} opts.config           - Override default config
 * @returns {Object}
 */
export function createMMEngine(opts = {}) {
  const {
    clobClient,
    getCexMomentum,
    getCexPrice,
    getContracts,
    verbose = false,
    config: userConfig = {},
  } = opts;

  const cfg = { ...DEFAULT_CONFIG, ...userConfig };

  // -----------------------------------------------------------------------
  // Inventory state
  // -----------------------------------------------------------------------

  // Per-contract inventory: conditionId -> { yesShares, noShares, yesCost, noCost }
  const inventory = new Map();

  // Active quotes: conditionId -> { yesBidOrderId, noBidOrderId, yesTokenId, noTokenId, lastFairValue, quotedAt }
  const activeQuotes = new Map();

  // Stats
  const stats = {
    cyclesRun: 0,
    quotesPlaced: 0,
    quotesCancelled: 0,
    fills: 0,
    totalYesBought: 0,
    totalNoBought: 0,
    totalYesSold: 0,
    totalNoBought: 0,
    grossProfit: 0,
    rebatesEarned: 0,
    startedAt: new Date().toISOString(),
  };

  // Load persisted state
  loadState();

  // -----------------------------------------------------------------------
  // Core loop: called every tick (2-5 seconds)
  // -----------------------------------------------------------------------

  /**
   * Run one market-making cycle.
   * Scan contracts, update quotes, simulate/check fills.
   *
   * @returns {{ quoted: number, cancelled: number, fills: Array, rebalanced: number }}
   */
  async function runCycle() {
    stats.cyclesRun++;
    const result = { quoted: 0, cancelled: 0, fills: [], rebalanced: 0 };

    const contracts = getContracts();
    if (!contracts || contracts.length === 0) {
      return result;
    }

    for (const contract of contracts) {
      try {
        const cycleResult = await processContract(contract);
        result.quoted += cycleResult.quoted;
        result.cancelled += cycleResult.cancelled;
        result.fills.push(...cycleResult.fills);
        if (cycleResult.rebalanced) result.rebalanced++;
      } catch (e) {
        if (verbose) console.error(`[MM] Error processing ${contract.asset}: ${e.message}`);
      }
    }

    // Expire quotes for contracts no longer active
    await expireStaleQuotes(contracts);

    // Persist state periodically (every 10 cycles)
    if (stats.cyclesRun % 10 === 0) {
      saveState();
    }

    return result;
  }

  /**
   * Process a single contract: check fills, rebalance, update quotes.
   */
  async function processContract(contract) {
    const { asset, cexSymbol, conditionId, clobTokenIds, remainingMs } = contract;
    const result = { quoted: 0, cancelled: 0, fills: [], rebalanced: false };

    // Validate contract
    if (!clobTokenIds || clobTokenIds.length < 1) return result;
    if (remainingMs && remainingMs < cfg.minRemainingSeconds * 1000) {
      // Too close to expiry — cancel all quotes and let positions resolve
      await cancelQuotesForContract(conditionId);
      return result;
    }

    const yesTokenId = clobTokenIds[0];
    const noTokenId = clobTokenIds.length > 1 ? clobTokenIds[1] : null;

    // Step 1: Check for fills on paper orders
    const midpoint = await clobClient.getMidpoint(yesTokenId);
    if (midpoint == null) return result;

    const yesFills = clobClient.simulateFills(yesTokenId, midpoint);
    const noFills = noTokenId ? clobClient.simulateFills(noTokenId, 1 - midpoint) : [];
    const allFills = [...yesFills, ...noFills];

    for (const fill of allFills) {
      processFill(conditionId, fill, asset);
      result.fills.push(fill);
    }

    // Step 2: Compute fair value and decide on quotes
    const fairValue = computeFairValue(contract, midpoint);
    if (fairValue == null) return result;

    // Step 3: Check if we should avoid mid-range (high adverse selection zone)
    if (cfg.avoidMidrange && fairValue > cfg.avoidMidrangeMin && fairValue < cfg.avoidMidrangeMax) {
      if (verbose) {
        console.log(`[MM] ${asset} fair=${fairValue.toFixed(3)} in midrange — skipping`);
      }
      await cancelQuotesForContract(conditionId);
      return result;
    }

    // Step 4: Compute inventory skew adjustment
    const inv = getInventory(conditionId);
    const skewAdj = computeSkewAdjustment(inv);

    // Step 5: Check if requote needed
    const existing = activeQuotes.get(conditionId);
    if (existing && Math.abs(fairValue - existing.lastFairValue) < cfg.requoteThresholdCents) {
      // Fair value hasn't moved enough — keep existing quotes
      return result;
    }

    // Step 6: Compute spread (may widen for volatility)
    const effectiveSpread = computeEffectiveSpread(cexSymbol);

    // Step 7: Compute quote prices
    const halfSpread = effectiveSpread / 2;

    // CEX lean: shift fair value toward the CEX-predicted direction
    const lean = computeCexLean(cexSymbol);

    // YES bid = fair + lean - halfSpread + skew
    // NO bid = (1 - fair - lean) - halfSpread - skew
    // (Buying NO at price P is equivalent to selling YES at price (1-P))
    let yesBidPrice = fairValue + lean - halfSpread + skewAdj;
    let noBidPrice = (1 - fairValue - lean) - halfSpread - skewAdj;

    // Clamp prices to valid range [0.01, 0.99]
    yesBidPrice = clamp(yesBidPrice, 0.01, 0.99);
    noBidPrice = clamp(noBidPrice, 0.01, 0.99);

    // Round to cent precision
    yesBidPrice = Math.round(yesBidPrice * 100) / 100;
    noBidPrice = Math.round(noBidPrice * 100) / 100;

    // Sanity: combined price must be < 1.00 (otherwise no profit possible)
    if (yesBidPrice + noBidPrice >= 1.0) {
      // Widen equally
      const overshoot = (yesBidPrice + noBidPrice - 0.98) / 2;
      yesBidPrice = Math.round((yesBidPrice - overshoot) * 100) / 100;
      noBidPrice = Math.round((noBidPrice - overshoot) * 100) / 100;
    }

    // Step 8: Check inventory limits before quoting
    const netExposure = computeNetExposure(inv);
    if (Math.abs(netExposure) >= cfg.maxPosition) {
      // At max position — only quote the side that reduces exposure
      if (netExposure > 0) {
        // Long YES — only quote NO bid (sell YES)
        yesBidPrice = 0;
      } else {
        // Short YES (long NO) — only quote YES bid
        noBidPrice = 0;
      }
    }

    // Step 9: Compute share quantities from USD quote size
    const yesShares = yesBidPrice > 0 ? cfg.quoteSize / yesBidPrice : 0;
    const noShares = noBidPrice > 0 ? cfg.quoteSize / noBidPrice : 0;

    // Step 10: Cancel existing quotes and place new ones
    if (existing) {
      await cancelQuotesForContract(conditionId);
      result.cancelled++;
    }

    let yesBidOrderId = null;
    let noBidOrderId = null;

    if (yesBidPrice > 0 && yesShares > 0) {
      const yesResult = await clobClient.placeOrder({
        tokenID: yesTokenId,
        price: yesBidPrice,
        size: yesShares,
        side: "BUY",
        feeRateBps: 0, // Maker pays zero
      });
      if (yesResult?.success) {
        yesBidOrderId = yesResult.orderId;
        stats.quotesPlaced++;
        result.quoted++;
      }
    }

    if (noBidPrice > 0 && noShares > 0 && noTokenId) {
      const noResult = await clobClient.placeOrder({
        tokenID: noTokenId,
        price: noBidPrice,
        size: noShares,
        side: "BUY",
        feeRateBps: 0, // Maker pays zero
      });
      if (noResult?.success) {
        noBidOrderId = noResult.orderId;
        stats.quotesPlaced++;
        result.quoted++;
      }
    }

    // Store active quote state
    activeQuotes.set(conditionId, {
      yesBidOrderId,
      noBidOrderId,
      yesTokenId,
      noTokenId,
      yesBidPrice,
      noBidPrice,
      lastFairValue: fairValue,
      quotedAt: Date.now(),
      asset,
      lean,
      skewAdj,
      effectiveSpread,
    });

    if (verbose && (yesBidOrderId || noBidOrderId)) {
      const invStr = `inv=[Y:${inv.yesShares.toFixed(1)} N:${inv.noShares.toFixed(1)}]`;
      console.log(
        `[MM] ${asset} fair=${fairValue.toFixed(3)} lean=${lean >= 0 ? "+" : ""}${(lean * 100).toFixed(1)}c ` +
          `spread=${(effectiveSpread * 100).toFixed(1)}c | ` +
          `YES bid=$${yesBidPrice.toFixed(2)} NO bid=$${noBidPrice.toFixed(2)} | ` +
          `${invStr}`
      );
    }

    // Step 11: Check if rebalance needed
    if (shouldRebalance(inv)) {
      const rebalanceResult = await performRebalance(conditionId, inv, contract);
      if (rebalanceResult) result.rebalanced = true;
    }

    return result;
  }

  // -----------------------------------------------------------------------
  // Fair value computation
  // -----------------------------------------------------------------------

  /**
   * Compute fair value of the YES outcome.
   * Combines CLOB midpoint with CEX-implied probability.
   *
   * @param {Object} contract
   * @param {number} clobMidpoint - Current CLOB midpoint (0-1)
   * @returns {number|null}
   */
  function computeFairValue(contract, clobMidpoint) {
    const { cexSymbol } = contract;

    // Start with CLOB midpoint as baseline
    let fair = clobMidpoint;

    // If we have CEX data, blend in a CEX-implied fair value
    // The idea: 5-min crypto market is "will price go up?"
    // CEX momentum gives us a signal about likely direction
    const m1 = getCexMomentum(cexSymbol, 60_000);
    if (m1 && m1.samples >= 2) {
      // CEX-implied probability shift: +0.3% momentum -> ~+8% probability
      // This is conservative vs the arb engine's model
      const cexShift = m1.pct * 0.08;
      const cexImplied = clamp(0.50 + cexShift, 0.15, 0.85);

      // Blend: 70% CLOB midpoint + 30% CEX-implied
      // We trust the market but lean toward CEX when it disagrees
      fair = clobMidpoint * 0.7 + cexImplied * 0.3;
    }

    return clamp(fair, 0.05, 0.95);
  }

  /**
   * Compute how much to lean quotes based on CEX momentum.
   * Positive lean = shift quotes up (expect YES more likely).
   *
   * @param {string} cexSymbol
   * @returns {number} Lean in probability points (e.g., +0.02 = 2c)
   */
  function computeCexLean(cexSymbol) {
    const m1 = getCexMomentum(cexSymbol, 60_000);
    if (!m1 || m1.samples < 2) return 0;

    // Each 0.1% CEX momentum translates to ~1c lean
    // Capped by leanFactor to control aggressiveness
    const rawLean = m1.pct * 0.10;
    const cappedLean = clamp(rawLean * cfg.leanFactor, -0.05, 0.05);

    return Math.round(cappedLean * 100) / 100;
  }

  // -----------------------------------------------------------------------
  // Spread computation
  // -----------------------------------------------------------------------

  /**
   * Compute effective spread, widening during high volatility.
   *
   * @param {string} cexSymbol
   * @returns {number} Spread in probability points
   */
  function computeEffectiveSpread(cexSymbol) {
    let spread = cfg.spread;

    // Widen spread during momentum spikes
    const m1 = getCexMomentum(cexSymbol, 60_000);
    if (m1 && Math.abs(m1.pct) >= cfg.volatilityWidenPct) {
      spread *= cfg.volatilityWidenFactor;
      if (verbose) {
        console.log(
          `[MM] Volatility spike ${m1.pct.toFixed(3)}% — widening spread to ${(spread * 100).toFixed(1)}c`
        );
      }
    }

    return clamp(spread, cfg.minSpread, cfg.maxSpread);
  }

  // -----------------------------------------------------------------------
  // Inventory management
  // -----------------------------------------------------------------------

  /**
   * Get inventory for a contract.
   */
  function getInventory(conditionId) {
    if (!inventory.has(conditionId)) {
      inventory.set(conditionId, {
        yesShares: 0,
        noShares: 0,
        yesCost: 0, // Total USD spent on YES
        noCost: 0,  // Total USD spent on NO
      });
    }
    return inventory.get(conditionId);
  }

  /**
   * Compute inventory skew adjustment.
   * If we're holding too much YES, make our YES bid less aggressive (lower)
   * and our NO bid more aggressive (higher) to rebalance.
   *
   * @param {Object} inv - { yesShares, noShares }
   * @returns {number} Adjustment in probability points (negative = shift away from YES)
   */
  function computeSkewAdjustment(inv) {
    const total = inv.yesShares + inv.noShares;
    if (total === 0) return 0;

    // Skew: +1 = all YES, -1 = all NO
    const skew = (inv.yesShares - inv.noShares) / total;

    // Each 10% skew shifts quotes by ~0.5c
    // Positive skew (too much YES) -> negative adjustment (less YES buying)
    const adjustment = -skew * 0.05;

    return clamp(adjustment, -0.03, 0.03);
  }

  /**
   * Compute net directional exposure in USD.
   * Positive = net long YES, negative = net long NO.
   */
  function computeNetExposure(inv) {
    return inv.yesCost - inv.noCost;
  }

  /**
   * Check if inventory needs rebalancing.
   */
  function shouldRebalance(inv) {
    const total = inv.yesShares + inv.noShares;
    if (total === 0) return false;

    const skew = Math.abs(inv.yesShares - inv.noShares) / total;
    return skew > cfg.rebalanceThreshold;
  }

  /**
   * Perform inventory rebalance: aggressively quote the overweight side
   * to shed exposure.
   */
  async function performRebalance(conditionId, inv, contract) {
    const skew = inv.yesShares - inv.noShares;
    const { asset } = contract;

    if (verbose) {
      console.log(
        `[MM:REBALANCE] ${asset} YES=${inv.yesShares.toFixed(1)} NO=${inv.noShares.toFixed(1)} ` +
          `skew=${skew > 0 ? "+" : ""}${skew.toFixed(1)} shares`
      );
    }

    // The normal quoting already skews via computeSkewAdjustment.
    // For heavy rebalance, we just log the event — the skew adjustment
    // will make our quotes more aggressive on the needed side.

    logEvent({
      type: "rebalance",
      conditionId,
      asset,
      yesShares: inv.yesShares,
      noShares: inv.noShares,
      skew,
      ts: new Date().toISOString(),
    });

    return true;
  }

  // -----------------------------------------------------------------------
  // Fill processing
  // -----------------------------------------------------------------------

  /**
   * Process a fill: update inventory, compute P&L, log.
   */
  function processFill(conditionId, fill, asset) {
    const inv = getInventory(conditionId);
    const price = parseFloat(fill.fillPrice || fill.price);
    const size = parseFloat(fill.filledSize || fill.size);
    const cost = price * size;

    // Determine which side was filled
    const isYesToken = activeQuotes.get(conditionId)?.yesTokenId === fill.tokenID;
    const isNoToken = activeQuotes.get(conditionId)?.noTokenId === fill.tokenID;

    if (fill.side === "BUY") {
      if (isYesToken) {
        inv.yesShares += size;
        inv.yesCost += cost;
      } else if (isNoToken) {
        inv.noShares += size;
        inv.noCost += cost;
      }
    } else if (fill.side === "SELL") {
      if (isYesToken) {
        inv.yesShares -= size;
        inv.yesCost -= cost;
      } else if (isNoToken) {
        inv.noShares -= size;
        inv.noCost -= cost;
      }
    }

    stats.fills++;
    if (isYesToken && fill.side === "BUY") stats.totalYesBought += cost;
    if (isNoToken && fill.side === "BUY") stats.totalNoBought += cost;

    // Estimate maker rebate: taker fee is redistributed to makers
    // Rebate = taker_fee_pct * fill_value (approximate)
    const feePct = 0.03 * (1 - Math.pow(2 * price - 1, 2));
    const estimatedRebate = feePct * cost;
    stats.rebatesEarned += estimatedRebate;

    // If we hold both YES and NO, they cancel out at expiry for $1
    // Gross profit when matched: (1 - yesBuyPrice - noBuyPrice) per matched pair
    const matchedPairs = Math.min(inv.yesShares, inv.noShares);
    if (matchedPairs > 0) {
      // We collected spread on matched pairs
      const avgYesCost = inv.yesCost / Math.max(inv.yesShares, 0.01);
      const avgNoCost = inv.noCost / Math.max(inv.noShares, 0.01);
      const spreadEarned = 1 - avgYesCost - avgNoCost;
      if (spreadEarned > 0) {
        stats.grossProfit += spreadEarned * matchedPairs;
      }
    }

    if (verbose) {
      const side = isYesToken ? "YES" : "NO";
      console.log(
        `[MM:FILL] ${asset} ${fill.side} ${size.toFixed(1)} ${side} @ $${price.toFixed(2)} ` +
          `($${cost.toFixed(2)}) | rebate ~$${estimatedRebate.toFixed(4)} | ` +
          `inv=[Y:${inv.yesShares.toFixed(1)} N:${inv.noShares.toFixed(1)}]`
      );
    }

    logEvent({
      type: "fill",
      conditionId,
      asset,
      tokenSide: isYesToken ? "YES" : "NO",
      orderSide: fill.side,
      price,
      size,
      cost,
      estimatedRebate,
      yesShares: inv.yesShares,
      noShares: inv.noShares,
      ts: new Date().toISOString(),
    });
  }

  // -----------------------------------------------------------------------
  // Quote management
  // -----------------------------------------------------------------------

  /**
   * Cancel all active quotes for a contract.
   */
  async function cancelQuotesForContract(conditionId) {
    const existing = activeQuotes.get(conditionId);
    if (!existing) return;

    if (existing.yesBidOrderId) {
      await clobClient.cancelOrder(existing.yesBidOrderId);
      stats.quotesCancelled++;
    }
    if (existing.noBidOrderId) {
      await clobClient.cancelOrder(existing.noBidOrderId);
      stats.quotesCancelled++;
    }

    activeQuotes.delete(conditionId);
  }

  /**
   * Expire quotes for contracts no longer in the active set.
   */
  async function expireStaleQuotes(activeContractsList) {
    const activeIds = new Set(activeContractsList.map((c) => c.conditionId));

    for (const [conditionId] of activeQuotes) {
      if (!activeIds.has(conditionId)) {
        if (verbose) console.log(`[MM] Expiring quotes for finished contract ${conditionId.substring(0, 12)}...`);
        await cancelQuotesForContract(conditionId);
      }
    }
  }

  /**
   * Emergency: cancel ALL open quotes.
   */
  async function cancelAllQuotes() {
    for (const [conditionId] of activeQuotes) {
      await cancelQuotesForContract(conditionId);
    }
    console.log(`[MM] All quotes cancelled`);
  }

  // -----------------------------------------------------------------------
  // P&L and reporting
  // -----------------------------------------------------------------------

  /**
   * Get current stats and P&L summary.
   */
  function getStats() {
    // Aggregate inventory across all contracts
    let totalYesShares = 0;
    let totalNoShares = 0;
    let totalYesCost = 0;
    let totalNoCost = 0;

    for (const [, inv] of inventory) {
      totalYesShares += inv.yesShares;
      totalNoShares += inv.noShares;
      totalYesCost += inv.yesCost;
      totalNoCost += inv.noCost;
    }

    const matchedPairs = Math.min(totalYesShares, totalNoShares);
    const avgYesCost = totalYesShares > 0 ? totalYesCost / totalYesShares : 0;
    const avgNoCost = totalNoShares > 0 ? totalNoCost / totalNoShares : 0;
    const spreadCaptured = matchedPairs > 0 ? (1 - avgYesCost - avgNoCost) * matchedPairs : 0;

    return {
      ...stats,
      inventory: {
        totalYesShares: round2(totalYesShares),
        totalNoShares: round2(totalNoShares),
        totalYesCost: round2(totalYesCost),
        totalNoCost: round2(totalNoCost),
        matchedPairs: round2(matchedPairs),
        avgYesCost: round4(avgYesCost),
        avgNoCost: round4(avgNoCost),
        netExposure: round2(totalYesCost - totalNoCost),
      },
      pnl: {
        spreadCaptured: round4(spreadCaptured),
        rebatesEarned: round4(stats.rebatesEarned),
        grossProfit: round4(stats.grossProfit),
        totalEstimated: round4(spreadCaptured + stats.rebatesEarned),
      },
      activeQuoteCount: activeQuotes.size,
      contractsTracked: inventory.size,
      config: { ...cfg },
    };
  }

  /**
   * Get active quotes summary for display.
   */
  function getActiveQuoteSummary() {
    const summary = [];
    for (const [conditionId, q] of activeQuotes) {
      summary.push({
        conditionId: conditionId.substring(0, 12),
        asset: q.asset,
        yesBid: q.yesBidPrice,
        noBid: q.noBidPrice,
        impliedAsk: q.noBidPrice > 0 ? round2(1 - q.noBidPrice) : null,
        fairValue: round4(q.lastFairValue),
        spread: round4(q.effectiveSpread),
        lean: round4(q.lean),
        age: Math.round((Date.now() - q.quotedAt) / 1000),
      });
    }
    return summary;
  }

  // -----------------------------------------------------------------------
  // Persistence
  // -----------------------------------------------------------------------

  function saveState() {
    try {
      const state = {
        inventory: Object.fromEntries(inventory),
        stats,
        savedAt: new Date().toISOString(),
      };
      fs.writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
    } catch (e) {
      if (verbose) console.error(`[MM] Failed to save state: ${e.message}`);
    }
  }

  function loadState() {
    try {
      if (fs.existsSync(STATE_FILE)) {
        const data = JSON.parse(fs.readFileSync(STATE_FILE, "utf8"));
        if (data.inventory) {
          for (const [k, v] of Object.entries(data.inventory)) {
            inventory.set(k, v);
          }
        }
        if (data.stats) {
          Object.assign(stats, data.stats);
        }
        if (verbose) console.log(`[MM] Loaded state from ${data.savedAt}`);
      }
    } catch (e) {
      if (verbose) console.error(`[MM] Failed to load state: ${e.message}`);
    }
  }

  // -----------------------------------------------------------------------
  // Logging
  // -----------------------------------------------------------------------

  function logEvent(event) {
    try {
      fs.appendFileSync(TRADE_LOG, JSON.stringify(event) + "\n");
    } catch (e) {
      // Silently fail — don't crash the engine for logging issues
    }
  }

  /**
   * Read trade/fill history from log.
   */
  function getHistory(limit = 100) {
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

  // -----------------------------------------------------------------------
  // Utilities
  // -----------------------------------------------------------------------

  function clamp(v, min, max) {
    return Math.max(min, Math.min(max, v));
  }
  function round2(n) {
    return Math.round(n * 100) / 100;
  }
  function round4(n) {
    return Math.round(n * 10000) / 10000;
  }

  // -----------------------------------------------------------------------
  // Public API
  // -----------------------------------------------------------------------

  return {
    runCycle,
    cancelAllQuotes,
    getStats,
    getActiveQuoteSummary,
    getHistory,
    getInventory,
    saveState,
    config: cfg,
  };
}
