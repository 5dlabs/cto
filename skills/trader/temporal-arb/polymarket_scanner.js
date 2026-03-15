/**
 * polymarket_scanner.js — Find and track active 5-min crypto contracts on Polymarket
 *
 * Uses Gamma API for market discovery + Polymarket CLOB for live prices.
 * Mirrors the discovery logic from polymarket-fast-loop skill.
 *
 * Exports: createPolymarketScanner()
 */

import axios from "axios";

const GAMMA_API = "https://gamma-api.polymarket.com";
const CLOB_API = "https://clob.polymarket.com";

// Asset search patterns (must match Polymarket question text)
const ASSET_PATTERNS = {
  BTC: ["bitcoin up or down"],
  ETH: ["ethereum up or down"],
  SOL: ["solana up or down"],
};

// Map asset to the symbol we use in CEX feed
const ASSET_TO_CEX_SYMBOL = {
  BTC: "BTC/USDT",
  ETH: "ETH/USDT",
  SOL: "SOL/USDT",
};

/**
 * Create a Polymarket scanner for active 5-min crypto contracts.
 *
 * @param {Object} opts
 * @param {string[]} opts.assets - Assets to scan for, default ["BTC", "ETH", "SOL"]
 * @param {string} opts.window - Market window, "5m" or "15m"
 * @param {boolean} opts.verbose
 * @param {string} opts.simmerApiKey - Simmer API key for SDK-based discovery (optional)
 * @returns {Object}
 */
export function createPolymarketScanner(opts = {}) {
  const {
    assets = ["BTC", "ETH", "SOL"],
    window = "5m",
    verbose = false,
    simmerApiKey = null,
  } = opts;

  // Cache of active contracts: conditionId -> contract info
  let activeContracts = [];
  let lastScanTs = 0;

  /**
   * Discover active fast markets via Gamma API.
   * @returns {Array} Array of contract objects
   */
  async function discoverViaGamma() {
    try {
      const resp = await axios.get(`${GAMMA_API}/markets`, {
        params: {
          limit: 100,
          closed: false,
          tag: "crypto",
          order: "endDate",
          ascending: true,
        },
        timeout: 10000,
        headers: { "User-Agent": "temporal-arb/1.0" },
      });

      const markets = resp.data;
      if (!Array.isArray(markets)) return [];

      const found = [];
      for (const m of markets) {
        const question = (m.question || "").toLowerCase();
        const slug = m.slug || "";

        // Check if this is a fast market for one of our assets
        let matchedAsset = null;
        for (const asset of assets) {
          const patterns = ASSET_PATTERNS[asset] || [];
          if (patterns.some((p) => question.includes(p))) {
            matchedAsset = asset;
            break;
          }
        }
        if (!matchedAsset) continue;

        // Check window match (e.g. "-5m-" in slug)
        if (!slug.includes(`-${window}-`)) continue;

        // Skip closed markets
        if (m.closed) continue;

        // Parse CLOB token IDs
        let clobTokenIds = [];
        const raw = m.clobTokenIds;
        if (typeof raw === "string") {
          try {
            clobTokenIds = JSON.parse(raw);
          } catch {
            clobTokenIds = [];
          }
        } else if (Array.isArray(raw)) {
          clobTokenIds = raw;
        }

        // Parse end time from question
        const endTime = parseEndTime(m.question || "");

        found.push({
          asset: matchedAsset,
          cexSymbol: ASSET_TO_CEX_SYMBOL[matchedAsset],
          question: m.question || "",
          slug,
          conditionId: m.conditionId || "",
          clobTokenIds,
          endTime,
          outcomes: m.outcomes || [],
          outcomePrices: parseOutcomePrices(m.outcomePrices),
          feeRateBps: parseInt(m.feeRateBps || m.fee_rate_bps || 0, 10),
          source: "gamma",
        });
      }

      return found;
    } catch (e) {
      console.error("[Scanner] Gamma API error:", e.message);
      return [];
    }
  }

  /**
   * Discover fast markets via Simmer API (if key available).
   */
  async function discoverViaSimmer() {
    if (!simmerApiKey) return [];

    try {
      const results = [];
      for (const asset of assets) {
        const resp = await axios.get("https://api.simmer.markets/api/sdk/fast-markets", {
          params: { asset, window, limit: 20 },
          headers: {
            Authorization: `Bearer ${simmerApiKey}`,
            "User-Agent": "temporal-arb/1.0",
          },
          timeout: 10000,
        });

        const markets = resp.data?.markets || resp.data || [];
        for (const m of markets) {
          const clobTokenIds = [];
          if (m.polymarket_token_id) clobTokenIds.push(m.polymarket_token_id);
          if (m.polymarket_no_token_id) clobTokenIds.push(m.polymarket_no_token_id);

          results.push({
            asset,
            cexSymbol: ASSET_TO_CEX_SYMBOL[asset],
            question: m.question || "",
            marketId: m.id, // Already imported in Simmer
            conditionId: m.condition_id || "",
            clobTokenIds,
            endTime: m.resolves_at ? new Date(m.resolves_at.replace(" ", "T").replace(/Z$/, "+00:00")) : null,
            isLiveNow: m.is_live_now,
            spreadCents: m.spread_cents,
            liquidityTier: m.liquidity_tier,
            externalPriceYes: m.external_price_yes,
            feeRateBps: m.fee_rate_bps || 0,
            source: "simmer",
          });
        }
      }
      return results;
    } catch (e) {
      if (verbose) console.error("[Scanner] Simmer API error:", e.message);
      return [];
    }
  }

  /**
   * Fetch live YES midpoint price from Polymarket CLOB.
   * @param {string} tokenId - YES token ID
   * @returns {number|null} Price between 0-1
   */
  async function fetchLivePrice(tokenId) {
    try {
      const resp = await axios.get(`${CLOB_API}/midpoint`, {
        params: { token_id: tokenId },
        timeout: 5000,
      });
      return parseFloat(resp.data?.mid);
    } catch {
      return null;
    }
  }

  /**
   * Fetch orderbook for spread/depth analysis.
   * @param {string} tokenId - YES token ID
   * @returns {{ bestBid, bestAsk, spreadPct, bidDepth, askDepth } | null}
   */
  async function fetchOrderbook(tokenId) {
    try {
      const resp = await axios.get(`${CLOB_API}/book`, {
        params: { token_id: tokenId },
        timeout: 5000,
      });

      const bids = resp.data?.bids || [];
      const asks = resp.data?.asks || [];
      if (!bids.length || !asks.length) return null;

      const bestBid = parseFloat(bids[0].price);
      const bestAsk = parseFloat(asks[0].price);
      const mid = (bestBid + bestAsk) / 2;
      const spreadPct = mid > 0 ? ((bestAsk - bestBid) / mid) * 100 : 0;

      // Top 5 level depth in USD
      const bidDepth = bids.slice(0, 5).reduce((sum, b) => sum + parseFloat(b.size || 0) * parseFloat(b.price || 0), 0);
      const askDepth = asks.slice(0, 5).reduce((sum, a) => sum + parseFloat(a.size || 0) * parseFloat(a.price || 0), 0);

      return { bestBid, bestAsk, spreadPct, bidDepth, askDepth };
    } catch {
      return null;
    }
  }

  /**
   * Scan for all active contracts and enrich with live prices.
   * @returns {Array} Enriched contract list
   */
  async function scan() {
    // Try Simmer first (pre-imported, more reliable), fall back to Gamma
    let contracts = await discoverViaSimmer();
    if (!contracts.length) {
      contracts = await discoverViaGamma();
    }

    if (verbose) {
      console.log(`[Scanner] Found ${contracts.length} raw contracts`);
    }

    // Filter to live contracts with enough time remaining
    const now = new Date();
    const minTimeMs = 30_000; // At least 30 seconds remaining

    const live = contracts.filter((c) => {
      if (c.isLiveNow === false) return false;
      if (c.endTime) {
        const remaining = c.endTime.getTime() - now.getTime();
        if (remaining < minTimeMs) return false;
        if (remaining > 600_000) return false; // > 10 min = not started yet
        c.remainingMs = remaining;
      }
      return true;
    });

    // Enrich with live CLOB prices
    for (const c of live) {
      if (c.clobTokenIds && c.clobTokenIds.length > 0) {
        const yesToken = c.clobTokenIds[0];
        c.liveYesPrice = await fetchLivePrice(yesToken);
        // Only fetch orderbook for top candidates (rate limit aware)
        if (live.indexOf(c) < 6) {
          c.orderbook = await fetchOrderbook(yesToken);
        }
      }
    }

    activeContracts = live;
    lastScanTs = Date.now();

    if (verbose) {
      console.log(`[Scanner] ${live.length} live contracts after filtering`);
      for (const c of live) {
        const remaining = c.remainingMs ? `${Math.round(c.remainingMs / 1000)}s` : "?";
        const price = c.liveYesPrice != null ? `$${c.liveYesPrice.toFixed(3)}` : "N/A";
        console.log(`  ${c.asset} | ${c.question.substring(0, 50)}... | ${remaining} left | YES: ${price}`);
      }
    }

    return live;
  }

  /**
   * Get currently cached active contracts.
   */
  function getActiveContracts() {
    return activeContracts;
  }

  /**
   * Get price for a specific contract.
   * @param {string} conditionId
   * @returns {{ yesPrice: number, noPrice: number } | null}
   */
  function getContractPrice(conditionId) {
    const c = activeContracts.find((x) => x.conditionId === conditionId);
    if (!c || c.liveYesPrice == null) return null;
    return {
      yesPrice: c.liveYesPrice,
      noPrice: 1 - c.liveYesPrice,
    };
  }

  return {
    scan,
    getActiveContracts,
    getContractPrice,
    fetchLivePrice,
    fetchOrderbook,
  };
}

// --- Helpers ---

/**
 * Parse end time from Polymarket question text.
 * e.g. "Bitcoin Up or Down - March 15, 5:30AM-5:35AM ET"
 */
function parseEndTime(question) {
  const pattern = /(\w+ \d+),.*?-\s*(\d{1,2}:\d{2}(?:AM|PM))\s*ET/;
  const match = question.match(pattern);
  if (!match) return null;

  try {
    const [, datePart, timePart] = match;
    const year = new Date().getFullYear();
    const dateStr = `${datePart} ${year} ${timePart}`;

    // Parse as ET (Eastern Time)
    // Simple approach: parse, then adjust for ET offset
    const months = {
      January: 0, February: 1, March: 2, April: 3, May: 4, June: 5,
      July: 6, August: 7, September: 8, October: 9, November: 10, December: 11,
    };

    const parts = datePart.match(/(\w+)\s+(\d+)/);
    if (!parts) return null;

    const month = months[parts[1]];
    const day = parseInt(parts[2], 10);
    if (month === undefined) return null;

    // Parse time
    const timeMatch = timePart.match(/(\d{1,2}):(\d{2})(AM|PM)/);
    if (!timeMatch) return null;

    let hours = parseInt(timeMatch[1], 10);
    const minutes = parseInt(timeMatch[2], 10);
    const ampm = timeMatch[3];

    if (ampm === "PM" && hours !== 12) hours += 12;
    if (ampm === "AM" && hours === 12) hours = 0;

    // Create date in ET (UTC-5 or UTC-4 for DST)
    // For simplicity, assume EDT (UTC-4) during March-November, EST (UTC-5) otherwise
    const isDST = month >= 2 && month <= 10; // Rough DST approximation
    const utcOffset = isDST ? 4 : 5;

    const dt = new Date(Date.UTC(year, month, day, hours + utcOffset, minutes, 0));
    return dt;
  } catch {
    return null;
  }
}

/**
 * Parse outcome prices from Gamma API response.
 */
function parseOutcomePrices(raw) {
  if (!raw) return [];
  if (typeof raw === "string") {
    try {
      return JSON.parse(raw).map(Number);
    } catch {
      return [];
    }
  }
  if (Array.isArray(raw)) return raw.map(Number);
  return [];
}

// --- Standalone mode ---
if (process.argv[1] && process.argv[1].endsWith("polymarket_scanner.js")) {
  const { config } = await import("dotenv");
  config({ path: new URL("../.env", import.meta.url).pathname });

  const scanner = createPolymarketScanner({
    verbose: true,
    simmerApiKey: process.env.SIMMER_API_KEY,
  });

  console.log("Scanning for active 5-min crypto contracts...\n");
  const contracts = await scanner.scan();

  if (!contracts.length) {
    console.log("\nNo active contracts found. Markets may be between windows or outside trading hours.");
  } else {
    console.log(`\nFound ${contracts.length} active contract(s):`);
    for (const c of contracts) {
      console.log(`\n  Asset: ${c.asset}`);
      console.log(`  Question: ${c.question}`);
      console.log(`  YES price: ${c.liveYesPrice != null ? "$" + c.liveYesPrice.toFixed(3) : "N/A"}`);
      console.log(`  Remaining: ${c.remainingMs ? Math.round(c.remainingMs / 1000) + "s" : "?"}`);
      console.log(`  Fee: ${c.feeRateBps}bps`);
      if (c.orderbook) {
        console.log(`  Spread: ${c.orderbook.spreadPct.toFixed(2)}% (bid $${c.orderbook.bestBid.toFixed(3)} / ask $${c.orderbook.bestAsk.toFixed(3)})`);
        console.log(`  Depth: $${c.orderbook.bidDepth.toFixed(0)} bid / $${c.orderbook.askDepth.toFixed(0)} ask`);
      }
    }
  }

  process.exit(0);
}
