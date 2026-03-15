/**
 * signal_engine.js — Trading signal generator for information arbitrage
 *
 * Combines credibility scores, news sentiment, and market relevance
 * to generate BUY/SELL signals with Kelly-criterion position sizing.
 *
 * Exports:
 *   generateSignals(matches, opts) => [{ market, side, amount, confidence, ... }]
 *   executeSignal(signal, opts) => { success, tradeId, ... }
 *   paperTrade(signal) => { logged trade details }
 */

import axios from "axios";
import { scorePost } from "./credibility_ranker.js";

const SIMMER_API = "https://api.simmer.markets";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const DEFAULT_CONFIG = {
  // Minimum thresholds
  minCredibility: 30,      // Minimum credibility score (0-100) to act on
  minRelevance: 0.35,      // Minimum market relevance score
  minSentimentStrength: 0.2, // Minimum sentiment strength
  minConfidence: 0.25,     // Minimum composite confidence to generate signal

  // Position sizing
  maxPositionUsd: 25,      // Maximum position size per trade
  minPositionUsd: 2,       // Minimum position size (below this, skip)
  kellyFraction: 0.25,     // Fraction of Kelly to use (quarter-Kelly for safety)
  bankroll: 100,           // Total bankroll for Kelly sizing

  // Risk limits
  maxDailyTrades: 10,      // Maximum trades per day
  maxDailyExposure: 100,   // Maximum total exposure per day
  maxPerMarket: 50,        // Maximum total position per market

  // Execution
  paperMode: true,         // Paper mode by default
};

// Track daily stats
let _dailyStats = {
  date: new Date().toISOString().split("T")[0],
  tradesExecuted: 0,
  totalExposure: 0,
};

function resetDailyStatsIfNeeded() {
  const today = new Date().toISOString().split("T")[0];
  if (_dailyStats.date !== today) {
    _dailyStats = { date: today, tradesExecuted: 0, totalExposure: 0 };
  }
}

// ---------------------------------------------------------------------------
// Signal generation
// ---------------------------------------------------------------------------

/**
 * Generate trading signals from news-market matches.
 *
 * Each match is scored on three dimensions:
 * 1. Source credibility (0-100 from credibility_ranker)
 * 2. Market relevance (0-1 from market_matcher)
 * 3. Sentiment strength (0-1 from market_matcher)
 *
 * These combine into a composite confidence score which determines:
 * - Whether to trade (above threshold)
 * - Which side (sentiment direction)
 * - How much (Kelly criterion)
 *
 * @param {Array} matches - Output from matchNewsToMarkets
 * @param {Object} opts - Override default config
 * @returns {Array} [{ market, side, amount, confidence, reason, ... }]
 */
export function generateSignals(matches, opts = {}) {
  const config = { ...DEFAULT_CONFIG, ...opts };
  resetDailyStatsIfNeeded();

  const signals = [];
  const seenMarkets = new Set(); // Deduplicate: one signal per market per cycle

  for (const match of matches) {
    const marketId = match.market?.id;
    if (!marketId || seenMarkets.has(marketId)) continue;

    // 1. Score source credibility
    const credScore = scorePost(match.post);
    if (credScore.credibility < config.minCredibility) continue;

    // 2. Check relevance threshold
    if (match.relevance < config.minRelevance) continue;

    // 3. Check sentiment
    if (!match.sentiment?.side) continue;
    if (match.sentiment.strength < config.minSentimentStrength) continue;

    // 4. Compute composite confidence
    const confidence = computeConfidence(
      credScore.credibility,
      match.relevance,
      match.sentiment.strength,
      match.post
    );

    if (confidence < config.minConfidence) continue;

    // 5. Kelly sizing
    const marketPrice = match.market.currentPrice || 0.5;
    const amount = kellySize(
      confidence,
      marketPrice,
      match.sentiment.side,
      config
    );

    if (amount < config.minPositionUsd) continue;

    // 6. Daily risk limits
    if (_dailyStats.tradesExecuted >= config.maxDailyTrades) {
      continue;
    }
    if (_dailyStats.totalExposure + amount > config.maxDailyExposure) {
      continue;
    }

    seenMarkets.add(marketId);

    signals.push({
      // Trade parameters
      market: match.market,
      side: match.sentiment.side,
      amount: Math.round(amount * 100) / 100,
      confidence: Math.round(confidence * 1000) / 1000,

      // Context
      reason: buildReason(match, credScore, confidence),
      topic: match.topic,
      credibility: credScore,
      sentiment: match.sentiment,
      relevance: match.relevance,
      matchedKeywords: match.matchedKeywords,

      // Source post
      sourcePost: {
        author: match.post.author,
        verified: match.post.verified,
        text: match.post.text.substring(0, 300),
        likes: match.post.likes,
        url: match.post.url,
      },

      // Metadata
      generatedAt: new Date().toISOString(),
      paperMode: config.paperMode,
    });
  }

  // Sort by confidence descending
  signals.sort((a, b) => b.confidence - a.confidence);

  return signals;
}

// ---------------------------------------------------------------------------
// Confidence computation
// ---------------------------------------------------------------------------

/**
 * Compute composite confidence from multiple dimensions.
 *
 * Weights:
 * - Credibility: 40% (source quality is most important for info arb)
 * - Relevance: 30% (how well does the news match the market)
 * - Sentiment: 20% (how clear is the directional signal)
 * - Engagement: 10% (viral posts = market-moving)
 */
function computeConfidence(credibility, relevance, sentimentStrength, post) {
  const credNorm = credibility / 100; // 0-1
  const engagementNorm = Math.min(
    1,
    Math.log10(1 + (post.likes || 0) + (post.retweets || 0) * 2) / 5
  );

  const confidence =
    credNorm * 0.4 +
    relevance * 0.3 +
    sentimentStrength * 0.2 +
    engagementNorm * 0.1;

  return Math.min(0.95, confidence); // Cap at 95%
}

// ---------------------------------------------------------------------------
// Kelly criterion position sizing
// ---------------------------------------------------------------------------

/**
 * Calculate position size using fractional Kelly criterion.
 *
 * Kelly formula: f* = (p * b - q) / b
 * Where:
 *   p = probability of winning (our confidence)
 *   q = 1 - p
 *   b = odds (payout ratio)
 *
 * For Polymarket: buying YES at price P means odds = (1/P - 1)
 *
 * @param {number} confidence - Our confidence in the outcome (0-1)
 * @param {number} marketPrice - Current YES price (0-1)
 * @param {string} side - "yes" or "no"
 * @param {Object} config - Configuration with kellyFraction, bankroll, etc.
 * @returns {number} Position size in USD
 */
function kellySize(confidence, marketPrice, side, config) {
  // Our estimated probability
  const p = confidence;
  const q = 1 - p;

  // Price we'd be buying at
  const price = side === "yes" ? marketPrice : 1 - marketPrice;

  // Odds: if we buy at price P, payout is 1/P, so profit ratio is (1/P - 1)
  if (price <= 0 || price >= 1) return 0;
  const b = (1 / price) - 1;

  // Kelly fraction
  const kelly = (p * b - q) / b;

  // If Kelly is negative, don't bet
  if (kelly <= 0) return 0;

  // Apply fractional Kelly (quarter-Kelly by default)
  const adjustedKelly = kelly * config.kellyFraction;

  // Size in USD
  const size = adjustedKelly * config.bankroll;

  // Clamp to max position
  return Math.min(size, config.maxPositionUsd);
}

// ---------------------------------------------------------------------------
// Signal execution
// ---------------------------------------------------------------------------

/**
 * Execute a signal via Simmer API.
 *
 * @param {Object} signal - Signal from generateSignals
 * @param {Object} opts
 * @param {string} opts.simmerApiKey - Simmer API key
 * @param {boolean} opts.paperMode - Paper mode (default true)
 * @returns {Object} Trade result
 */
export async function executeSignal(signal, opts = {}) {
  const { simmerApiKey, paperMode = true } = opts;

  if (paperMode) {
    return paperTrade(signal);
  }

  if (!simmerApiKey) {
    return { success: false, error: "SIMMER_API_KEY not set" };
  }

  resetDailyStatsIfNeeded();

  try {
    const resp = await axios.post(
      `${SIMMER_API}/api/sdk/trade`,
      {
        market_id: signal.market.id,
        side: signal.side,
        amount: signal.amount,
        source: "sdk:grok-info-arb",
      },
      {
        headers: {
          Authorization: `Bearer ${simmerApiKey}`,
          "Content-Type": "application/json",
          "User-Agent": "grok-info-arb/1.0",
        },
        timeout: 30000,
      }
    );

    const result = resp.data;

    // Update daily stats
    _dailyStats.tradesExecuted++;
    _dailyStats.totalExposure += signal.amount;

    return {
      success: result.success !== false,
      tradeId: result.trade_id || result.id,
      sharesBought: result.shares_bought || result.shares,
      avgPrice: result.avg_price,
      marketId: signal.market.id,
      side: signal.side,
      amount: signal.amount,
      confidence: signal.confidence,
      executedAt: new Date().toISOString(),
    };
  } catch (err) {
    const detail = err.response?.data
      ? JSON.stringify(err.response.data).substring(0, 200)
      : err.message;
    return {
      success: false,
      error: detail,
      marketId: signal.market.id,
      side: signal.side,
      amount: signal.amount,
    };
  }
}

/**
 * Paper trade — simulate execution and return what would have happened.
 */
export function paperTrade(signal) {
  const price =
    signal.side === "yes"
      ? signal.market.currentPrice || 0.5
      : 1 - (signal.market.currentPrice || 0.5);

  const shares = price > 0 ? signal.amount / price : 0;

  return {
    success: true,
    paper: true,
    tradeId: `paper_${Date.now()}_${Math.random().toString(36).substring(2, 8)}`,
    sharesBought: Math.round(shares * 100) / 100,
    avgPrice: price,
    marketId: signal.market.id,
    side: signal.side,
    amount: signal.amount,
    confidence: signal.confidence,
    question: signal.market.question,
    executedAt: new Date().toISOString(),
    reason: signal.reason,
    sourceAuthor: signal.sourcePost?.author,
  };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Build a human-readable reason string for the signal.
 */
function buildReason(match, credScore, confidence) {
  const parts = [];

  // Source info
  const author = match.post.author || "unknown";
  parts.push(`${author}${match.post.verified ? " [verified]" : ""}`);

  // Credibility tier
  parts.push(`credibility: ${credScore.tier} (${credScore.credibility}/100)`);

  // Sentiment
  parts.push(`sentiment: ${match.sentiment.direction} (${Math.round(match.sentiment.strength * 100)}%)`);

  // Relevance
  parts.push(`relevance: ${Math.round(match.relevance * 100)}%`);

  // Keywords
  if (match.matchedKeywords?.length > 0) {
    parts.push(`keywords: ${match.matchedKeywords.slice(0, 5).join(", ")}`);
  }

  return parts.join(" | ");
}

/**
 * Get current daily stats.
 */
export function getDailyStats() {
  resetDailyStatsIfNeeded();
  return { ..._dailyStats };
}
