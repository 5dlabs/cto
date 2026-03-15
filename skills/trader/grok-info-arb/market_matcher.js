/**
 * market_matcher.js — Match breaking news to active Polymarket event markets
 *
 * Fetches active event markets from Polymarket (via Gamma API) and uses
 * keyword matching + TF-IDF-style relevance scoring to match news posts
 * to tradeable markets.
 *
 * Exports:
 *   fetchActiveMarkets(opts) => [{ id, question, slug, ... }]
 *   matchNewsToMarkets(news, markets) => [{ market, news, relevance, sentiment }]
 *   refreshMarketCache(opts) => void
 */

import axios from "axios";

const GAMMA_API = "https://gamma-api.polymarket.com";
const SIMMER_API = "https://api.simmer.markets";

// Market cache to avoid re-fetching every scan cycle
let _marketCache = [];
let _marketCacheTs = 0;
const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes

// ---------------------------------------------------------------------------
// Keywords that map news topics to Polymarket market categories
// ---------------------------------------------------------------------------

const TOPIC_MARKET_KEYWORDS = {
  // Iran / Middle East
  iran: [
    "iran", "tehran", "khamenei", "irgc", "persian gulf", "strait of hormuz",
    "iran war", "iran attack", "iran strike", "iran ceasefire", "iran deal",
    "iran nuclear", "iran sanctions",
  ],
  israel: [
    "israel", "netanyahu", "idf", "gaza", "hamas", "hezbollah", "lebanon",
    "west bank", "tel aviv", "jerusalem",
  ],
  oil: [
    "oil", "crude", "wti", "brent", "opec", "petroleum", "barrel",
    "oil price", "oil surge", "oil spike", "energy crisis",
  ],
  fed: [
    "federal reserve", "fed chair", "fomc", "interest rate", "rate cut",
    "rate hike", "powell", "fed nomination", "monetary policy",
  ],
  election: [
    "election", "president", "trump", "biden", "harris", "desantis",
    "primary", "gop", "democrat", "republican", "electoral", "polling",
    "ballot", "swing state",
  ],
  military: [
    "military", "pentagon", "nato", "missile", "drone", "airstrike",
    "deployment", "troops", "carrier", "aircraft carrier", "centcom",
  ],
  geopolitical: [
    "china", "taiwan", "russia", "ukraine", "north korea", "kim jong",
    "sanctions", "embargo", "conflict", "escalation", "ceasefire",
    "peace deal", "treaty", "diplomacy",
  ],
  crypto: [
    "bitcoin", "ethereum", "crypto", "sec", "etf", "digital asset",
    "blockchain", "defi", "stablecoin", "usdc", "usdt",
  ],
};

// ---------------------------------------------------------------------------
// Market fetching
// ---------------------------------------------------------------------------

/**
 * Fetch active event markets from Polymarket via Gamma API.
 * Filters to event-type markets (not fast crypto 5-min markets).
 *
 * @param {Object} opts
 * @param {string} opts.simmerApiKey - Simmer API key (optional, for enriched data)
 * @param {boolean} opts.forceRefresh - Bypass cache
 * @param {string[]} opts.tags - Filter by market tags
 * @returns {Array} Active market objects
 */
export async function fetchActiveMarkets(opts = {}) {
  const { simmerApiKey, forceRefresh = false, tags } = opts;

  // Return cache if fresh
  if (!forceRefresh && _marketCache.length > 0 && Date.now() - _marketCacheTs < CACHE_TTL_MS) {
    return _marketCache;
  }

  let markets = [];

  // Try Simmer API first (enriched with AI consensus data)
  if (simmerApiKey) {
    markets = await fetchFromSimmer(simmerApiKey);
  }

  // Fallback to Gamma API
  if (markets.length === 0) {
    markets = await fetchFromGamma(tags);
  }

  // Filter to event markets (not 5-min crypto speed markets)
  markets = markets.filter((m) => {
    const q = (m.question || "").toLowerCase();
    // Exclude fast markets (5-min, 15-min crypto)
    if (q.includes("up or down") && (q.includes("5:") || q.includes("15:"))) return false;
    // Exclude low-volume markets
    if (m.volume !== undefined && m.volume < 1000) return false;
    return true;
  });

  _marketCache = markets;
  _marketCacheTs = Date.now();

  return markets;
}

/**
 * Fetch markets from Gamma API (public, no auth needed).
 */
async function fetchFromGamma(tags) {
  try {
    const params = {
      limit: 200,
      closed: false,
      order: "volume24hr",
      ascending: false,
    };
    if (tags && tags.length > 0) {
      params.tag = tags[0]; // Gamma only supports one tag
    }

    const resp = await axios.get(`${GAMMA_API}/markets`, {
      params,
      timeout: 15000,
      headers: { "User-Agent": "grok-info-arb/1.0" },
    });

    const raw = resp.data;
    if (!Array.isArray(raw)) return [];

    return raw.map((m) => ({
      id: m.conditionId || m.id,
      question: m.question || "",
      slug: m.slug || "",
      description: m.description || "",
      outcomes: m.outcomes || [],
      outcomePrices: parseOutcomePrices(m.outcomePrices),
      volume: parseFloat(m.volume || 0),
      volume24hr: parseFloat(m.volume24hr || 0),
      endDate: m.endDate,
      clobTokenIds: parseClobTokenIds(m.clobTokenIds),
      tags: m.tags || [],
      source: "gamma",
    }));
  } catch (err) {
    console.error("[MarketMatcher] Gamma API error:", err.message);
    return [];
  }
}

/**
 * Fetch markets from Simmer API (enriched with AI data).
 */
async function fetchFromSimmer(apiKey) {
  try {
    const resp = await axios.get(`${SIMMER_API}/api/sdk/markets`, {
      params: { status: "active", limit: 200 },
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "User-Agent": "grok-info-arb/1.0",
      },
      timeout: 15000,
    });

    const raw = resp.data?.markets || resp.data || [];
    if (!Array.isArray(raw)) return [];

    return raw.map((m) => ({
      id: m.id || m.condition_id,
      question: m.question || "",
      slug: m.slug || "",
      description: m.description || "",
      outcomes: m.outcomes || [],
      currentPrice: m.current_price || m.yes_price,
      aiConsensus: m.ai_consensus,
      volume: m.volume || 0,
      endDate: m.end_date || m.resolves_at,
      tags: m.tags || [],
      source: "simmer",
    }));
  } catch (err) {
    console.error("[MarketMatcher] Simmer API error:", err.message);
    return [];
  }
}

/**
 * Force-refresh the market cache.
 */
export async function refreshMarketCache(opts = {}) {
  _marketCache = [];
  _marketCacheTs = 0;
  return fetchActiveMarkets({ ...opts, forceRefresh: true });
}

// ---------------------------------------------------------------------------
// News-to-market matching
// ---------------------------------------------------------------------------

/**
 * Match an array of news scan results to active markets.
 *
 * @param {Array} scanResults - Output from scanTopics: [{ topic, posts }]
 * @param {Array} markets - Active markets from fetchActiveMarkets
 * @param {Object} opts
 * @param {number} opts.minRelevance - Minimum relevance score to include (default 0.3)
 * @returns {Array} [{ market, post, topic, relevance, sentiment, matchedKeywords }]
 */
export function matchNewsToMarkets(scanResults, markets, opts = {}) {
  const { minRelevance = 0.3 } = opts;
  const matches = [];

  for (const scan of scanResults) {
    if (!scan.posts || scan.posts.length === 0) continue;

    for (const post of scan.posts) {
      const postText = `${post.text} ${scan.topic}`.toLowerCase();

      for (const market of markets) {
        const { relevance, matchedKeywords } = computeRelevance(postText, market);

        if (relevance >= minRelevance) {
          const sentiment = analyzeSentiment(post.text, market.question);

          matches.push({
            market: {
              id: market.id,
              question: market.question,
              slug: market.slug,
              currentPrice: market.currentPrice || market.outcomePrices?.[0],
              aiConsensus: market.aiConsensus,
              volume: market.volume,
              clobTokenIds: market.clobTokenIds,
              source: market.source,
            },
            post: {
              author: post.author,
              verified: post.verified,
              text: post.text,
              likes: post.likes,
              retweets: post.retweets,
              url: post.url,
            },
            topic: scan.topic,
            relevance,
            matchedKeywords,
            sentiment,
            scannedAt: scan.scannedAt,
          });
        }
      }
    }
  }

  // Sort by relevance * engagement
  matches.sort((a, b) => {
    const scoreA = a.relevance * (1 + Math.log10(1 + (a.post.likes || 0)));
    const scoreB = b.relevance * (1 + Math.log10(1 + (b.post.likes || 0)));
    return scoreB - scoreA;
  });

  return matches;
}

// ---------------------------------------------------------------------------
// Relevance scoring
// ---------------------------------------------------------------------------

/**
 * Compute relevance between a post's text and a market.
 * Uses keyword overlap with TF-IDF-like weighting.
 */
function computeRelevance(postText, market) {
  const marketText = `${market.question} ${market.description || ""} ${market.slug || ""}`.toLowerCase();
  const matchedKeywords = [];
  let score = 0;

  // 1. Direct keyword overlap between post and market question
  const marketWords = new Set(marketText.split(/\s+/).filter((w) => w.length > 3));
  const postWords = new Set(postText.split(/\s+/).filter((w) => w.length > 3));

  let overlap = 0;
  for (const word of postWords) {
    if (marketWords.has(word)) {
      overlap++;
      matchedKeywords.push(word);
    }
  }

  if (marketWords.size > 0) {
    score += (overlap / Math.max(marketWords.size, 1)) * 0.4;
  }

  // 2. Topic-category keyword matching
  for (const [category, keywords] of Object.entries(TOPIC_MARKET_KEYWORDS)) {
    const postHits = keywords.filter((kw) => postText.includes(kw));
    const marketHits = keywords.filter((kw) => marketText.includes(kw));

    if (postHits.length > 0 && marketHits.length > 0) {
      // Both post and market share a topic category
      const categoryScore = Math.min(postHits.length, marketHits.length) / keywords.length;
      score += categoryScore * 0.4;
      matchedKeywords.push(...postHits.slice(0, 3));
    }
  }

  // 3. Named entity overlap (proper nouns, capitalized words)
  const postEntities = extractEntities(postText);
  const marketEntities = extractEntities(marketText);
  const entityOverlap = postEntities.filter((e) => marketEntities.includes(e));
  if (entityOverlap.length > 0) {
    score += Math.min(entityOverlap.length * 0.1, 0.2);
    matchedKeywords.push(...entityOverlap);
  }

  // Deduplicate keywords
  const uniqueKeywords = [...new Set(matchedKeywords)];

  return {
    relevance: Math.min(1.0, score),
    matchedKeywords: uniqueKeywords.slice(0, 10),
  };
}

/**
 * Extract likely named entities from text (crude but fast).
 */
function extractEntities(text) {
  // Find capitalized words/phrases (excluding common words)
  const stopWords = new Set([
    "the", "will", "has", "have", "been", "this", "that", "with",
    "from", "they", "their", "what", "when", "where", "which", "who",
    "how", "not", "are", "was", "were", "but", "and", "for", "yes", "no",
  ]);

  return text
    .split(/\s+/)
    .filter((w) => w.length > 2 && !stopWords.has(w))
    .map((w) => w.replace(/[^a-z0-9]/g, ""))
    .filter((w) => w.length > 2);
}

// ---------------------------------------------------------------------------
// Sentiment analysis (basic)
// ---------------------------------------------------------------------------

const BULLISH_WORDS = [
  "surge", "rise", "gain", "jump", "soar", "rally", "boost", "advance",
  "confirm", "approve", "pass", "succeed", "breakthrough", "deal",
  "agreement", "ceasefire", "peace", "launch", "announce", "win",
  "positive", "bullish", "strong", "above", "exceed", "beat",
];

const BEARISH_WORDS = [
  "crash", "drop", "fall", "plunge", "collapse", "decline", "sink",
  "reject", "fail", "block", "veto", "oppose", "cancel", "delay",
  "attack", "strike", "bomb", "war", "conflict", "escalation",
  "negative", "bearish", "weak", "below", "miss", "threat", "crisis",
];

/**
 * Analyze sentiment of a post relative to a market question.
 *
 * @param {string} postText
 * @param {string} marketQuestion
 * @returns {{ direction: "bullish"|"bearish"|"neutral", strength: number, side: "yes"|"no"|null }}
 */
function analyzeSentiment(postText, marketQuestion) {
  const text = (postText || "").toLowerCase();
  const question = (marketQuestion || "").toLowerCase();

  let bullScore = 0;
  let bearScore = 0;

  for (const word of BULLISH_WORDS) {
    if (text.includes(word)) bullScore++;
  }
  for (const word of BEARISH_WORDS) {
    if (text.includes(word)) bearScore++;
  }

  const total = bullScore + bearScore;
  if (total === 0) {
    return { direction: "neutral", strength: 0, side: null };
  }

  const direction = bullScore > bearScore ? "bullish" : bearScore > bullScore ? "bearish" : "neutral";
  const strength = Math.abs(bullScore - bearScore) / total;

  // Map sentiment to market side
  // For "positive" questions (Will X happen?), bullish news => YES
  // For "negative" questions (Will X crash?), bearish news => YES
  const isNegativeQuestion =
    question.includes("crash") ||
    question.includes("fall") ||
    question.includes("decline") ||
    question.includes("fail") ||
    question.includes("lose") ||
    question.includes("war") ||
    question.includes("attack") ||
    question.includes("conflict");

  let side = null;
  if (direction === "bullish") {
    side = isNegativeQuestion ? "no" : "yes";
  } else if (direction === "bearish") {
    side = isNegativeQuestion ? "yes" : "no";
  }

  return { direction, strength, side };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

function parseClobTokenIds(raw) {
  if (!raw) return [];
  if (typeof raw === "string") {
    try {
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }
  if (Array.isArray(raw)) return raw;
  return [];
}
