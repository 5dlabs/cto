/**
 * cex_feed.js — Real-time CEX price feed via ccxt websockets
 *
 * Connects to Binance and Coinbase for BTC/USDT, ETH/USDT, SOL/USDT.
 * Tracks price changes over configurable windows (1m, 5m).
 * Detects momentum spikes for temporal arb signals.
 *
 * Exports: createCexFeed(), which returns { getCurrentPrice, getMomentum, getSpread, stop }
 */

import ccxt from "ccxt";

// Sliding window of price snapshots per symbol
// Each entry: { ts: number, price: number, volume: number }
const MAX_HISTORY_MS = 10 * 60 * 1000; // Keep 10 minutes of ticks

/**
 * Create a CEX price feed that tracks real-time prices from Binance and Coinbase.
 *
 * @param {Object} opts
 * @param {string[]} opts.symbols - Symbols to track, e.g. ["BTC/USDT", "ETH/USDT", "SOL/USDT"]
 * @param {boolean} opts.verbose - Log price updates
 * @param {Function} opts.onMomentumSpike - Callback when momentum exceeds threshold
 * @param {number} opts.spikeThresholdPct - Momentum % to trigger callback (default 0.3)
 * @returns {Object} { getCurrentPrice, getMomentum, getSpread, stop, isReady }
 */
export function createCexFeed(opts = {}) {
  const {
    symbols = ["BTC/USDT", "ETH/USDT", "SOL/USDT"],
    verbose = false,
    onMomentumSpike = null,
    spikeThresholdPct = 0.3,
  } = opts;

  // Price history: symbol -> [{ ts, price, volume, exchange }]
  const history = {};
  // Latest prices per exchange: "binance:BTC/USDT" -> { price, ts }
  const latestPrices = {};
  // Track readiness
  let binanceReady = false;
  let coinbaseReady = false;

  for (const sym of symbols) {
    history[sym] = [];
  }

  // --- Binance REST polling (ccxt pro websockets require paid features, REST klines are simpler) ---
  let binance = null;
  let coinbase = null;
  let pollInterval = null;
  let stopped = false;

  async function initExchanges() {
    binance = new ccxt.binance({ enableRateLimit: true });
    coinbase = new ccxt.coinbase({ enableRateLimit: true });

    // Load markets
    try {
      await binance.loadMarkets();
      binanceReady = true;
      if (verbose) console.log("[CEX] Binance markets loaded");
    } catch (e) {
      console.error("[CEX] Binance init failed:", e.message);
    }

    try {
      await coinbase.loadMarkets();
      coinbaseReady = true;
      if (verbose) console.log("[CEX] Coinbase markets loaded");
    } catch (e) {
      console.error("[CEX] Coinbase init failed:", e.message);
    }
  }

  async function fetchPrices() {
    if (stopped) return;
    const now = Date.now();

    for (const sym of symbols) {
      // Binance
      if (binanceReady) {
        try {
          const ticker = await binance.fetchTicker(sym);
          if (ticker && ticker.last) {
            const entry = { ts: now, price: ticker.last, volume: ticker.baseVolume || 0, exchange: "binance" };
            history[sym].push(entry);
            latestPrices[`binance:${sym}`] = { price: ticker.last, ts: now };
            if (verbose) {
              console.log(`[CEX] Binance ${sym}: $${ticker.last.toFixed(2)}`);
            }
          }
        } catch (e) {
          if (verbose) console.error(`[CEX] Binance ${sym} fetch error:`, e.message);
        }
      }

      // Coinbase — use different symbol format
      if (coinbaseReady) {
        try {
          // Coinbase uses BTC/USD not BTC/USDT
          const cbSym = sym.replace("/USDT", "/USD");
          if (coinbase.markets[cbSym]) {
            const ticker = await coinbase.fetchTicker(cbSym);
            if (ticker && ticker.last) {
              const entry = { ts: now, price: ticker.last, volume: ticker.baseVolume || 0, exchange: "coinbase" };
              history[sym].push(entry);
              latestPrices[`coinbase:${sym}`] = { price: ticker.last, ts: now };
            }
          }
        } catch (e) {
          if (verbose) console.error(`[CEX] Coinbase ${sym} fetch error:`, e.message);
        }
      }

      // Prune old entries
      const cutoff = now - MAX_HISTORY_MS;
      history[sym] = history[sym].filter((e) => e.ts > cutoff);
    }

    // Check for momentum spikes
    if (onMomentumSpike) {
      for (const sym of symbols) {
        const m60 = getMomentum(sym, 60_000); // 1-min momentum
        if (m60 && Math.abs(m60.pct) >= spikeThresholdPct) {
          onMomentumSpike({ symbol: sym, ...m60 });
        }
      }
    }
  }

  /**
   * Get the current best price for a symbol (average of exchanges or single).
   * @param {string} symbol - e.g. "BTC/USDT"
   * @returns {{ price: number, ts: number, exchange: string } | null}
   */
  function getCurrentPrice(symbol) {
    const binKey = `binance:${symbol}`;
    const cbKey = `coinbase:${symbol}`;
    const bp = latestPrices[binKey];
    const cp = latestPrices[cbKey];

    if (bp && cp) {
      // Return the most recent price
      return bp.ts >= cp.ts
        ? { price: bp.price, ts: bp.ts, exchange: "binance" }
        : { price: cp.price, ts: cp.ts, exchange: "coinbase" };
    }
    if (bp) return { price: bp.price, ts: bp.ts, exchange: "binance" };
    if (cp) return { price: cp.price, ts: cp.ts, exchange: "coinbase" };
    return null;
  }

  /**
   * Get momentum over a time window.
   * @param {string} symbol - e.g. "BTC/USDT"
   * @param {number} windowMs - Window in milliseconds (e.g. 60000 for 1 min)
   * @returns {{ pct: number, direction: "up"|"down", priceNow: number, priceThen: number, samples: number } | null}
   */
  function getMomentum(symbol, windowMs = 60_000) {
    const entries = history[symbol];
    if (!entries || entries.length < 2) return null;

    const now = Date.now();
    const cutoff = now - windowMs;

    // Find the oldest entry within the window
    const inWindow = entries.filter((e) => e.ts >= cutoff);
    if (inWindow.length < 2) return null;

    const oldest = inWindow[0];
    const newest = inWindow[inWindow.length - 1];

    const pct = ((newest.price - oldest.price) / oldest.price) * 100;
    const direction = pct >= 0 ? "up" : "down";

    return {
      pct: Math.round(pct * 10000) / 10000, // 4 decimal places
      direction,
      priceNow: newest.price,
      priceThen: oldest.price,
      samples: inWindow.length,
      windowMs,
    };
  }

  /**
   * Get cross-exchange spread for a symbol.
   * @param {string} symbol
   * @returns {{ spreadPct: number, binancePrice: number, coinbasePrice: number } | null}
   */
  function getSpread(symbol) {
    const bp = latestPrices[`binance:${symbol}`];
    const cp = latestPrices[`coinbase:${symbol}`];
    if (!bp || !cp) return null;

    const mid = (bp.price + cp.price) / 2;
    const spreadPct = ((bp.price - cp.price) / mid) * 100;

    return {
      spreadPct: Math.round(spreadPct * 10000) / 10000,
      binancePrice: bp.price,
      coinbasePrice: cp.price,
    };
  }

  /**
   * Start the feed — polls every 2 seconds for low-latency price tracking.
   */
  async function start() {
    await initExchanges();
    // Initial fetch
    await fetchPrices();
    // Poll every 2 seconds
    pollInterval = setInterval(fetchPrices, 2000);
    console.log(`[CEX] Feed started — tracking ${symbols.join(", ")} every 2s`);
  }

  function stop() {
    stopped = true;
    if (pollInterval) clearInterval(pollInterval);
    console.log("[CEX] Feed stopped");
  }

  function isReady() {
    return binanceReady || coinbaseReady;
  }

  /**
   * Get all momentum data for display/logging.
   */
  function getAllMomentum() {
    const result = {};
    for (const sym of symbols) {
      result[sym] = {
        "1m": getMomentum(sym, 60_000),
        "5m": getMomentum(sym, 300_000),
        current: getCurrentPrice(sym),
        spread: getSpread(sym),
      };
    }
    return result;
  }

  return {
    start,
    stop,
    isReady,
    getCurrentPrice,
    getMomentum,
    getSpread,
    getAllMomentum,
    history,
  };
}

// --- Standalone mode: run directly to test the feed ---
if (process.argv[1] && process.argv[1].endsWith("cex_feed.js")) {
  const feed = createCexFeed({
    verbose: true,
    onMomentumSpike: (spike) => {
      console.log(
        `\n[SPIKE] ${spike.symbol} ${spike.direction} ${spike.pct.toFixed(3)}% in ${spike.windowMs / 1000}s ` +
          `($${spike.priceThen.toFixed(2)} -> $${spike.priceNow.toFixed(2)})`
      );
    },
    spikeThresholdPct: 0.1, // Lower threshold for testing
  });

  await feed.start();

  // Print momentum summary every 10 seconds
  setInterval(() => {
    console.log("\n--- Momentum Summary ---");
    const all = feed.getAllMomentum();
    for (const [sym, data] of Object.entries(all)) {
      const cur = data.current;
      const m1 = data["1m"];
      const m5 = data["5m"];
      const spread = data.spread;
      console.log(
        `${sym}: $${cur?.price?.toFixed(2) || "N/A"} | ` +
          `1m: ${m1?.pct?.toFixed(3) || "N/A"}% | ` +
          `5m: ${m5?.pct?.toFixed(3) || "N/A"}% | ` +
          `spread: ${spread?.spreadPct?.toFixed(4) || "N/A"}%`
      );
    }
  }, 10000);

  process.on("SIGINT", () => {
    feed.stop();
    process.exit(0);
  });
}
