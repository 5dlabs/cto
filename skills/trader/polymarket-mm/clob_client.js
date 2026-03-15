/**
 * clob_client.js — Polymarket CLOB API client for market making
 *
 * Handles order placement, cancellation, and orderbook fetching against the
 * Polymarket Central Limit Order Book (CLOB).
 *
 * Authentication: API key + HMAC-SHA256 signature using secret + passphrase.
 * See: https://docs.polymarket.com/
 *
 * In paper mode, all mutating calls are stubbed — they log what would be sent
 * and return simulated responses. Read-only calls (orderbook, midpoint) always
 * hit the real API.
 *
 * Exports: createClobClient()
 */

import axios from "axios";
import crypto from "crypto";

const CLOB_BASE = "https://clob.polymarket.com";

/**
 * Create a CLOB client for Polymarket.
 *
 * @param {Object} opts
 * @param {string} opts.apiKey       - POLYMARKET_API_KEY
 * @param {string} opts.secret       - POLYMARKET_SECRET (for HMAC signing)
 * @param {string} opts.passphrase   - POLYMARKET_PASSPHRASE
 * @param {boolean} opts.paperMode   - If true, stub all mutating calls (default: true)
 * @param {boolean} opts.verbose     - Log requests
 * @returns {Object}
 */
export function createClobClient(opts = {}) {
  const {
    apiKey = process.env.POLYMARKET_API_KEY || "",
    secret = process.env.POLYMARKET_SECRET || "",
    passphrase = process.env.POLYMARKET_PASSPHRASE || "",
    paperMode = true,
    verbose = false,
  } = opts;

  const hasCredentials = apiKey && secret && passphrase;

  // Track open orders for paper mode simulation
  const paperOrders = new Map(); // orderId -> order
  let paperOrderCounter = 0;

  // -----------------------------------------------------------------------
  // Auth helpers
  // -----------------------------------------------------------------------

  /**
   * Generate HMAC-SHA256 signature for a CLOB request.
   *
   * Polymarket CLOB signing:
   *   timestamp = Unix epoch seconds
   *   message   = timestamp + method + requestPath + body
   *   signature = HMAC-SHA256(base64decode(secret), message)
   *
   * Headers sent:
   *   POLY-API-KEY, POLY-SIGNATURE, POLY-TIMESTAMP, POLY-PASSPHRASE
   */
  function signRequest(method, requestPath, body = "") {
    const timestamp = Math.floor(Date.now() / 1000).toString();
    const bodyStr = typeof body === "string" ? body : JSON.stringify(body);
    const message = timestamp + method.toUpperCase() + requestPath + (bodyStr || "");
    const keyBuffer = Buffer.from(secret, "base64");
    const signature = crypto
      .createHmac("sha256", keyBuffer)
      .update(message)
      .digest("base64");

    return {
      "POLY-API-KEY": apiKey,
      "POLY-SIGNATURE": signature,
      "POLY-TIMESTAMP": timestamp,
      "POLY-PASSPHRASE": passphrase,
    };
  }

  /**
   * Build full headers for an authenticated request.
   */
  function authHeaders(method, path, body) {
    const sig = signRequest(method, path, body);
    return {
      ...sig,
      "Content-Type": "application/json",
      "User-Agent": "polymarket-mm/1.0",
    };
  }

  // -----------------------------------------------------------------------
  // Read-only endpoints (always hit real API)
  // -----------------------------------------------------------------------

  /**
   * Fetch the full orderbook for a token.
   *
   * @param {string} tokenId - CLOB token ID (YES or NO token)
   * @returns {{ bids: Array<{price: string, size: string}>, asks: Array, market: string, asset_id: string } | null}
   */
  async function getOrderbook(tokenId) {
    try {
      const resp = await axios.get(`${CLOB_BASE}/book`, {
        params: { token_id: tokenId },
        timeout: 5000,
        headers: { "User-Agent": "polymarket-mm/1.0" },
      });
      return resp.data;
    } catch (e) {
      if (verbose) console.error(`[CLOB] getOrderbook error: ${e.message}`);
      return null;
    }
  }

  /**
   * Fetch the midpoint price for a token.
   *
   * @param {string} tokenId
   * @returns {number|null} Midpoint price (0-1)
   */
  async function getMidpoint(tokenId) {
    try {
      const resp = await axios.get(`${CLOB_BASE}/midpoint`, {
        params: { token_id: tokenId },
        timeout: 5000,
        headers: { "User-Agent": "polymarket-mm/1.0" },
      });
      const mid = parseFloat(resp.data?.mid);
      return isNaN(mid) ? null : mid;
    } catch (e) {
      if (verbose) console.error(`[CLOB] getMidpoint error: ${e.message}`);
      return null;
    }
  }

  /**
   * Parse an orderbook into structured best-bid/ask and depth.
   *
   * @param {Object} book - Raw orderbook from getOrderbook
   * @returns {{ bestBid, bestAsk, midpoint, spreadAbs, spreadPct, bidDepth5, askDepth5, bids, asks } | null}
   */
  function parseOrderbook(book) {
    if (!book) return null;
    const bids = (book.bids || []).map((b) => ({
      price: parseFloat(b.price),
      size: parseFloat(b.size),
    }));
    const asks = (book.asks || []).map((a) => ({
      price: parseFloat(a.price),
      size: parseFloat(a.size),
    }));

    if (!bids.length || !asks.length) return null;

    // Bids sorted descending, asks sorted ascending
    bids.sort((a, b) => b.price - a.price);
    asks.sort((a, b) => a.price - b.price);

    const bestBid = bids[0].price;
    const bestAsk = asks[0].price;
    const midpoint = (bestBid + bestAsk) / 2;
    const spreadAbs = bestAsk - bestBid;
    const spreadPct = midpoint > 0 ? (spreadAbs / midpoint) * 100 : 0;

    // Top-5 depth in USD
    const bidDepth5 = bids.slice(0, 5).reduce((s, b) => s + b.price * b.size, 0);
    const askDepth5 = asks.slice(0, 5).reduce((s, a) => s + a.price * a.size, 0);

    return {
      bestBid,
      bestAsk,
      midpoint,
      spreadAbs: round4(spreadAbs),
      spreadPct: round4(spreadPct),
      bidDepth5: round2(bidDepth5),
      askDepth5: round2(askDepth5),
      bids,
      asks,
    };
  }

  // -----------------------------------------------------------------------
  // Mutating endpoints (stubbed in paper mode)
  // -----------------------------------------------------------------------

  /**
   * Place a limit order on the CLOB.
   *
   * @param {Object} order
   * @param {string} order.tokenID     - CLOB token ID
   * @param {number} order.price       - Limit price (0-1, in cents precision)
   * @param {number} order.size        - Number of shares (contracts)
   * @param {"BUY"|"SELL"} order.side  - BUY or SELL
   * @param {number} order.feeRateBps  - Fee rate in bps (0 for maker)
   * @param {number} [order.expiration] - Unix timestamp expiration (optional)
   * @returns {{ success: boolean, orderId: string, paper: boolean } | null}
   */
  async function placeOrder(order) {
    const { tokenID, price, size, side, feeRateBps = 0, expiration } = order;

    // Round price to nearest cent (Polymarket uses 2 decimal precision)
    const roundedPrice = Math.round(price * 100) / 100;

    const orderPayload = {
      tokenID,
      price: roundedPrice.toFixed(2),
      size: size.toFixed(2),
      side,
      feeRateBps,
      nonce: Date.now().toString(),
      expiration: expiration || 0,
    };

    if (paperMode) {
      paperOrderCounter++;
      const orderId = `paper-${paperOrderCounter}-${Date.now()}`;
      const paperOrder = {
        ...orderPayload,
        orderId,
        status: "OPEN",
        filledSize: 0,
        createdAt: Date.now(),
      };
      paperOrders.set(orderId, paperOrder);

      if (verbose) {
        console.log(
          `[CLOB:PAPER] placeOrder ${side} ${size.toFixed(2)} @ $${roundedPrice.toFixed(2)} ` +
            `token=${tokenID.substring(0, 12)}... -> ${orderId}`
        );
      }

      return { success: true, orderId, paper: true, order: paperOrder };
    }

    // Live mode
    if (!hasCredentials) {
      console.error("[CLOB] Cannot place live order — missing API credentials");
      return null;
    }

    const path = "/order";
    const body = JSON.stringify(orderPayload);

    try {
      const resp = await axios.post(`${CLOB_BASE}${path}`, body, {
        headers: authHeaders("POST", path, body),
        timeout: 5000,
      });

      const data = resp.data;
      if (verbose) {
        console.log(
          `[CLOB:LIVE] placeOrder ${side} ${size.toFixed(2)} @ $${roundedPrice.toFixed(2)} -> ${data?.orderID || "?"}`
        );
      }

      return {
        success: !!data?.orderID,
        orderId: data?.orderID,
        paper: false,
        order: data,
      };
    } catch (e) {
      console.error(`[CLOB] placeOrder error: ${e.response?.data?.error || e.message}`);
      return null;
    }
  }

  /**
   * Cancel an open order.
   *
   * @param {string} orderId
   * @returns {{ success: boolean, paper: boolean }}
   */
  async function cancelOrder(orderId) {
    if (paperMode) {
      const existed = paperOrders.has(orderId);
      if (existed) {
        paperOrders.get(orderId).status = "CANCELLED";
        paperOrders.delete(orderId);
      }

      if (verbose) {
        console.log(`[CLOB:PAPER] cancelOrder ${orderId} -> ${existed ? "cancelled" : "not found"}`);
      }

      return { success: existed, paper: true };
    }

    // Live mode
    if (!hasCredentials) {
      console.error("[CLOB] Cannot cancel — missing API credentials");
      return { success: false, paper: false };
    }

    const path = `/order/${orderId}`;
    try {
      await axios.delete(`${CLOB_BASE}${path}`, {
        headers: authHeaders("DELETE", path),
        timeout: 5000,
      });

      if (verbose) console.log(`[CLOB:LIVE] cancelOrder ${orderId} -> cancelled`);
      return { success: true, paper: false };
    } catch (e) {
      console.error(`[CLOB] cancelOrder error: ${e.response?.data?.error || e.message}`);
      return { success: false, paper: false };
    }
  }

  /**
   * Cancel all open orders for a token.
   *
   * @param {string} tokenId - Cancel all orders for this token
   * @returns {{ cancelled: number, paper: boolean }}
   */
  async function cancelAllOrders(tokenId) {
    if (paperMode) {
      let cancelled = 0;
      for (const [id, order] of paperOrders) {
        if (order.tokenID === tokenId) {
          order.status = "CANCELLED";
          paperOrders.delete(id);
          cancelled++;
        }
      }

      if (verbose) console.log(`[CLOB:PAPER] cancelAll token=${tokenId.substring(0, 12)}... -> ${cancelled} cancelled`);
      return { cancelled, paper: true };
    }

    // Live mode — cancel individually (CLOB API may support batch cancel)
    if (!hasCredentials) {
      return { cancelled: 0, paper: false };
    }

    const path = "/orders";
    try {
      const resp = await axios.delete(`${CLOB_BASE}${path}`, {
        data: JSON.stringify({ asset_id: tokenId }),
        headers: authHeaders("DELETE", path, JSON.stringify({ asset_id: tokenId })),
        timeout: 5000,
      });

      const cancelled = resp.data?.cancelled || 0;
      if (verbose) console.log(`[CLOB:LIVE] cancelAll token=${tokenId.substring(0, 12)}... -> ${cancelled} cancelled`);
      return { cancelled, paper: false };
    } catch (e) {
      console.error(`[CLOB] cancelAll error: ${e.response?.data?.error || e.message}`);
      return { cancelled: 0, paper: false };
    }
  }

  // -----------------------------------------------------------------------
  // Paper mode simulation
  // -----------------------------------------------------------------------

  /**
   * Simulate fills on open paper orders based on the current midpoint.
   * If midpoint crosses our bid price, the bid is "filled".
   *
   * @param {string} tokenId   - Token to check
   * @param {number} midpoint  - Current CLOB midpoint
   * @returns {Array} Filled orders
   */
  function simulateFills(tokenId, midpoint) {
    if (!paperMode) return [];

    const fills = [];
    for (const [id, order] of paperOrders) {
      if (order.tokenID !== tokenId || order.status !== "OPEN") continue;

      const price = parseFloat(order.price);

      // BUY fills when midpoint drops to or below our bid
      if (order.side === "BUY" && midpoint <= price) {
        order.status = "FILLED";
        order.filledSize = parseFloat(order.size);
        order.filledAt = Date.now();
        order.fillPrice = price;
        paperOrders.delete(id);
        fills.push({ ...order });
      }

      // SELL fills when midpoint rises to or above our ask
      if (order.side === "SELL" && midpoint >= price) {
        order.status = "FILLED";
        order.filledSize = parseFloat(order.size);
        order.filledAt = Date.now();
        order.fillPrice = price;
        paperOrders.delete(id);
        fills.push({ ...order });
      }
    }

    return fills;
  }

  /**
   * Get all open paper orders.
   */
  function getOpenPaperOrders() {
    return Array.from(paperOrders.values()).filter((o) => o.status === "OPEN");
  }

  /**
   * Get open paper orders for a specific token.
   */
  function getOpenOrdersForToken(tokenId) {
    return Array.from(paperOrders.values()).filter(
      (o) => o.tokenID === tokenId && o.status === "OPEN"
    );
  }

  // -----------------------------------------------------------------------
  // Utility
  // -----------------------------------------------------------------------

  function round2(n) {
    return Math.round(n * 100) / 100;
  }
  function round4(n) {
    return Math.round(n * 10000) / 10000;
  }

  function isLive() {
    return !paperMode && hasCredentials;
  }

  return {
    // Read-only (always real)
    getOrderbook,
    getMidpoint,
    parseOrderbook,

    // Mutating (stubbed in paper mode)
    placeOrder,
    cancelOrder,
    cancelAllOrders,

    // Paper mode
    simulateFills,
    getOpenPaperOrders,
    getOpenOrdersForToken,

    // Info
    isLive,
    paperMode,
  };
}
