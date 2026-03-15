/**
 * credibility_ranker.js — Source credibility scoring for X accounts
 *
 * Scores each X post/account on a 0-100 credibility scale based on:
 * - Verified badge status
 * - Engagement metrics (likes, retweets)
 * - Known journalist/official accounts
 * - Historical prediction accuracy (tracked in credibility_db.json)
 *
 * Exports:
 *   scorePost(post) => { credibility: 0-100, factors: {...} }
 *   scorePosts(posts) => sorted array with credibility scores
 *   updateAccuracy(author, prediction, outcome) => void
 *   getAccountHistory(author) => { accuracy, predictions, ... }
 */

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DB_PATH = join(__dirname, "credibility_db.json");

// ---------------------------------------------------------------------------
// Known high-credibility sources (curated list)
// ---------------------------------------------------------------------------

const KNOWN_JOURNALISTS = new Set([
  // War correspondents & geopolitical
  "@baborek",
  "@idaborobova",
  "@jackdetsch",
  "@likibird",
  "@ragikibar",
  "@jonathanbrunson",
  "@richardengel",
  "@eaborba",
  "@naborrowman",
  "@clarissaward",
  "@matthewchayes",
  // Breaking news / wire services
  "@reuters",
  "@ap",
  "@afp",
  "@breakingnews",
  "@baborunews",
  "@baborin",
  // Financial / markets
  "@zaborherogh",
  "@livesquawk",
  "@dabordelivenews",
  "@faborxhunter",
  "@newsquawk",
  "@walterbloomberg",
  "@deitaone",
  "@firstsquawk",
  // Government / officials
  "@potus",
  "@dod_ousd_pac",
  "@centcom",
  "@iaborangovmil",
  "@statedept",
  "@whitehouse",
  "@secdef",
  "@pentagondept",
  // Iran-specific
  "@iraborannewsupdate",
  "@intaborelnews",
  "@baboraknews",
]);

const KNOWN_OFFICIALS = new Set([
  "@potus",
  "@secdef",
  "@statedept",
  "@whitehouse",
  "@centcom",
  "@pentagondept",
  "@dod_ousd_pac",
]);

// ---------------------------------------------------------------------------
// Credibility scoring
// ---------------------------------------------------------------------------

/**
 * Score a single post for credibility.
 *
 * @param {Object} post - Post object from grok_scanner
 * @param {string} post.author - @handle
 * @param {boolean} post.verified - Has verified badge
 * @param {number} post.likes - Like count
 * @param {number} post.retweets - Retweet count
 * @param {string} post.text - Post text
 * @returns {{ credibility: number, factors: Object, tier: string }}
 */
export function scorePost(post) {
  const factors = {};
  let score = 0;

  const author = (post.author || "").toLowerCase();

  // 1. Verified badge (+15 points)
  if (post.verified) {
    factors.verified = 15;
    score += 15;
  }

  // 2. Known journalist (+25 points)
  if (KNOWN_JOURNALISTS.has(author)) {
    factors.knownJournalist = 25;
    score += 25;
  }

  // 3. Known official (+30 points)
  if (KNOWN_OFFICIALS.has(author)) {
    factors.knownOfficial = 30;
    score += 30;
  }

  // 4. Engagement score (log scale, max 20 points)
  const engagement = (post.likes || 0) + (post.retweets || 0) * 2;
  if (engagement > 0) {
    const engagementScore = Math.min(20, Math.round(Math.log10(engagement + 1) * 7));
    factors.engagement = engagementScore;
    score += engagementScore;
  }

  // 5. Retweet/like ratio — high retweet ratio suggests breaking news (+5)
  if (post.likes > 0 && post.retweets > 0) {
    const rtRatio = post.retweets / post.likes;
    if (rtRatio > 0.5) {
      factors.highRetweetRatio = 5;
      score += 5;
    }
  }

  // 6. Historical accuracy from DB (+/- up to 15 points)
  const history = getAccountHistory(author);
  if (history && history.totalPredictions >= 3) {
    const accuracyBonus = Math.round((history.accuracy - 0.5) * 30);
    factors.historicalAccuracy = accuracyBonus;
    factors.historicalAccuracyPct = Math.round(history.accuracy * 100);
    factors.totalPredictions = history.totalPredictions;
    score += accuracyBonus;
  }

  // 7. Text quality signals
  const textLower = (post.text || "").toLowerCase();

  // Cites sources (+5)
  if (
    textLower.includes("according to") ||
    textLower.includes("sources say") ||
    textLower.includes("per ") ||
    textLower.includes("confirmed by")
  ) {
    factors.citesSource = 5;
    score += 5;
  }

  // Breaking news indicator (+5)
  if (
    textLower.includes("breaking") ||
    textLower.includes("just in") ||
    textLower.includes("developing")
  ) {
    factors.breakingNews = 5;
    score += 5;
  }

  // Hedging language (-5)
  if (
    textLower.includes("rumor") ||
    textLower.includes("unconfirmed") ||
    textLower.includes("allegedly") ||
    textLower.includes("i think") ||
    textLower.includes("imo")
  ) {
    factors.hedgingLanguage = -5;
    score -= 5;
  }

  // Clamp to 0-100
  const credibility = Math.max(0, Math.min(100, score));

  // Tier classification
  let tier;
  if (credibility >= 70) tier = "high";
  else if (credibility >= 40) tier = "medium";
  else if (credibility >= 20) tier = "low";
  else tier = "noise";

  return { credibility, factors, tier, author };
}

/**
 * Score and sort an array of posts by credibility (highest first).
 *
 * @param {Array} posts - Array of post objects
 * @returns {Array} Posts enriched with { _credibility, _factors, _tier }
 */
export function scorePosts(posts) {
  return posts
    .map((post) => {
      const { credibility, factors, tier } = scorePost(post);
      return {
        ...post,
        _credibility: credibility,
        _factors: factors,
        _tier: tier,
      };
    })
    .sort((a, b) => b._credibility - a._credibility);
}

// ---------------------------------------------------------------------------
// Prediction tracking — tracks how accurate each account's calls are
// ---------------------------------------------------------------------------

/**
 * Load the credibility database.
 * @returns {Object} { accounts: { "@handle": { correct, incorrect, predictions: [...] } } }
 */
function loadDb() {
  if (!existsSync(DB_PATH)) {
    return { accounts: {}, updatedAt: new Date().toISOString() };
  }
  try {
    return JSON.parse(readFileSync(DB_PATH, "utf-8"));
  } catch {
    return { accounts: {}, updatedAt: new Date().toISOString() };
  }
}

/**
 * Save the credibility database.
 */
function saveDb(db) {
  db.updatedAt = new Date().toISOString();
  const dir = dirname(DB_PATH);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  writeFileSync(DB_PATH, JSON.stringify(db, null, 2));
}

/**
 * Get historical accuracy for an account.
 *
 * @param {string} author - @handle
 * @returns {{ accuracy: number, totalPredictions: number, correct: number, incorrect: number } | null}
 */
export function getAccountHistory(author) {
  const db = loadDb();
  const handle = (author || "").toLowerCase();
  const account = db.accounts?.[handle];
  if (!account) return null;

  const total = (account.correct || 0) + (account.incorrect || 0);
  if (total === 0) return null;

  return {
    accuracy: account.correct / total,
    totalPredictions: total,
    correct: account.correct || 0,
    incorrect: account.incorrect || 0,
  };
}

/**
 * Record a prediction outcome for an account.
 * Call this when a Polymarket market resolves — update the accounts
 * that made predictions about the outcome.
 *
 * @param {string} author - @handle
 * @param {string} prediction - What the account predicted (brief text)
 * @param {boolean} wasCorrect - Whether the prediction was correct
 * @param {Object} meta - Optional metadata (marketId, resolvedAt, etc.)
 */
export function updateAccuracy(author, prediction, wasCorrect, meta = {}) {
  const db = loadDb();
  const handle = (author || "").toLowerCase();

  if (!db.accounts[handle]) {
    db.accounts[handle] = {
      correct: 0,
      incorrect: 0,
      predictions: [],
    };
  }

  const account = db.accounts[handle];
  if (wasCorrect) {
    account.correct++;
  } else {
    account.incorrect++;
  }

  // Keep last 50 predictions
  account.predictions.unshift({
    prediction,
    wasCorrect,
    recordedAt: new Date().toISOString(),
    ...meta,
  });
  account.predictions = account.predictions.slice(0, 50);

  saveDb(db);
}

/**
 * Get the full credibility database summary.
 * @returns {Object} { totalAccounts, topAccounts: [...] }
 */
export function getDbSummary() {
  const db = loadDb();
  const accounts = Object.entries(db.accounts || {}).map(([handle, data]) => {
    const total = (data.correct || 0) + (data.incorrect || 0);
    return {
      handle,
      accuracy: total > 0 ? data.correct / total : 0,
      totalPredictions: total,
      correct: data.correct || 0,
      incorrect: data.incorrect || 0,
    };
  });

  // Sort by accuracy (min 3 predictions), then by count
  accounts.sort((a, b) => {
    if (a.totalPredictions >= 3 && b.totalPredictions < 3) return -1;
    if (b.totalPredictions >= 3 && a.totalPredictions < 3) return 1;
    return b.accuracy - a.accuracy || b.totalPredictions - a.totalPredictions;
  });

  return {
    totalAccounts: accounts.length,
    topAccounts: accounts.slice(0, 20),
    updatedAt: db.updatedAt,
  };
}
