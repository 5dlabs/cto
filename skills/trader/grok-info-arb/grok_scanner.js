/**
 * grok_scanner.js — X sentiment scanner via Grok Responses API
 *
 * Searches X in real-time for breaking news on configurable topics.
 * Uses the Grok Responses API (api.x.ai/v1/responses) with x_search tool
 * to get real-time X posts with author/engagement data.
 *
 * Exports: scanTopics(topics, opts) => [{ topic, posts: [...] }]
 */

import axios from "axios";

const GROK_API_URL = "https://api.x.ai/v1";
const GROK_MODEL = "grok-3";

// Default scan topics — geopolitical event markets
export const DEFAULT_TOPICS = [
  "Iran war escalation",
  "Iran ceasefire",
  "oil price surge",
  "Federal Reserve chair nomination",
  "US military Iran",
  "Strait of Hormuz",
  "Netanyahu",
  "crude oil WTI",
];

/**
 * Search X for a single topic using the Grok Responses API with x_search tool.
 *
 * The prompt is engineered to extract structured post data: author handle,
 * verified status, follower count (when available), engagement, and full text.
 *
 * @param {string} topic - Search topic
 * @param {Object} opts
 * @param {string} opts.apiKey - Grok API key
 * @param {number} opts.recencyMinutes - Only posts from the last N minutes (default 15)
 * @param {number} opts.maxResults - Max posts to return (default 10)
 * @returns {Object} { topic, posts: [...], rawResponse, scannedAt }
 */
export async function searchTopic(topic, opts = {}) {
  const {
    apiKey = process.env.GROK_API_KEY,
    recencyMinutes = 15,
    maxResults = 10,
  } = opts;

  if (!apiKey) {
    throw new Error(
      "GROK_API_KEY not set. Set env var or pass apiKey in opts."
    );
  }

  // Calculate date window for x_search tool
  const now = new Date();
  const from = new Date(now.getTime() - recencyMinutes * 60 * 1000);
  const toDate = now.toISOString().split("T")[0];
  const fromDate = from.toISOString().split("T")[0];

  const prompt = `Search X/Twitter for the most recent posts about "${topic}" from the last ${recencyMinutes} minutes.

For each post found, extract and return in this EXACT JSON format (return a JSON array):
[
  {
    "author": "@handle",
    "authorName": "Display Name",
    "verified": true/false,
    "text": "full post text",
    "likes": number,
    "retweets": number,
    "replies": number,
    "timestamp": "ISO timestamp or relative time",
    "url": "https://x.com/..."
  }
]

Focus on:
- Posts from journalists, officials, and verified accounts
- Breaking news, not old analysis
- High engagement posts (likes, retweets)
- Return up to ${maxResults} posts, most relevant first
- If no recent posts found, return an empty array: []

IMPORTANT: Return ONLY the JSON array, no markdown, no explanation.`;

  const body = {
    model: GROK_MODEL,
    input: [{ role: "user", content: prompt }],
    tools: [
      {
        type: "x_search",
        from_date: fromDate,
        to_date: toDate,
      },
    ],
  };

  try {
    const response = await axios.post(`${GROK_API_URL}/responses`, body, {
      headers: {
        Authorization: `Bearer ${apiKey}`,
        "Content-Type": "application/json",
      },
      timeout: 30000,
    });

    const data = response.data;
    const rawText = extractTextResponse(data);
    const posts = parsePostsFromResponse(rawText);

    return {
      topic,
      posts,
      rawResponse: rawText,
      scannedAt: now.toISOString(),
      postsFound: posts.length,
    };
  } catch (err) {
    const status = err.response?.status;
    const detail = err.response?.data
      ? JSON.stringify(err.response.data).substring(0, 200)
      : err.message;
    console.error(`[GrokScanner] Error scanning "${topic}": ${status} ${detail}`);
    return {
      topic,
      posts: [],
      error: `${status || "NETWORK"}: ${detail}`,
      scannedAt: now.toISOString(),
      postsFound: 0,
    };
  }
}

/**
 * Scan multiple topics in sequence (to avoid rate limiting).
 *
 * @param {string[]} topics - List of topics to scan
 * @param {Object} opts - Options passed through to searchTopic
 * @returns {Array} Array of { topic, posts, scannedAt, postsFound }
 */
export async function scanTopics(topics, opts = {}) {
  const results = [];
  const delayMs = opts.delayBetweenMs || 1000; // 1s between requests to be polite

  for (const topic of topics) {
    const result = await searchTopic(topic, opts);
    results.push(result);

    // Rate limit politeness
    if (topics.indexOf(topic) < topics.length - 1) {
      await sleep(delayMs);
    }
  }

  return results;
}

// ---------------------------------------------------------------------------
// Response parsing helpers
// ---------------------------------------------------------------------------

/**
 * Extract text from Grok Responses API output.
 * The response format has output[] -> content[] -> { type: "output_text", text }
 */
function extractTextResponse(data) {
  const output = data?.output;
  if (!output || !Array.isArray(output)) return "";

  const parts = [];
  for (const item of output) {
    const content = item?.content;
    if (!content || !Array.isArray(content)) continue;

    for (const entry of content) {
      if (entry.type === "output_text" && typeof entry.text === "string") {
        parts.push(entry.text);
      }
    }
  }

  return parts.join("\n\n");
}

/**
 * Parse structured post data from Grok's text response.
 * Grok returns JSON (sometimes wrapped in markdown code blocks).
 */
function parsePostsFromResponse(text) {
  if (!text || text.trim() === "") return [];

  // Strip markdown code fences if present
  let cleaned = text.trim();
  if (cleaned.startsWith("```")) {
    cleaned = cleaned.replace(/^```(?:json)?\n?/, "").replace(/\n?```$/, "");
  }

  try {
    const parsed = JSON.parse(cleaned);
    if (Array.isArray(parsed)) {
      return parsed.map(normalizePost).filter((p) => p.text);
    }
    // Sometimes Grok wraps in an object
    if (parsed.posts && Array.isArray(parsed.posts)) {
      return parsed.posts.map(normalizePost).filter((p) => p.text);
    }
    if (parsed.results && Array.isArray(parsed.results)) {
      return parsed.results.map(normalizePost).filter((p) => p.text);
    }
    return [];
  } catch {
    // Fallback: try to extract posts from unstructured text
    return extractPostsFromText(text);
  }
}

/**
 * Normalize a post object to a consistent shape.
 */
function normalizePost(raw) {
  return {
    author: raw.author || raw.handle || raw.username || "unknown",
    authorName: raw.authorName || raw.author_name || raw.name || "",
    verified: Boolean(raw.verified ?? raw.is_verified ?? false),
    text: raw.text || raw.content || raw.tweet || "",
    likes: parseInt(raw.likes || raw.like_count || 0, 10),
    retweets: parseInt(raw.retweets || raw.retweet_count || raw.reposts || 0, 10),
    replies: parseInt(raw.replies || raw.reply_count || 0, 10),
    timestamp: raw.timestamp || raw.created_at || raw.time || "",
    url: raw.url || raw.link || "",
  };
}

/**
 * Fallback extraction: pull whatever structured data we can from unstructured text.
 */
function extractPostsFromText(text) {
  const posts = [];
  // Look for @handle patterns with surrounding context
  const lines = text.split("\n").filter((l) => l.trim());
  let currentPost = null;

  for (const line of lines) {
    const handleMatch = line.match(/@(\w{1,15})/);
    if (handleMatch) {
      if (currentPost && currentPost.text) {
        posts.push(currentPost);
      }
      currentPost = {
        author: `@${handleMatch[1]}`,
        authorName: "",
        verified: false,
        text: line.replace(/@\w+/, "").trim(),
        likes: 0,
        retweets: 0,
        replies: 0,
        timestamp: "",
        url: "",
      };

      // Try to extract engagement numbers
      const likeMatch = line.match(/(\d[\d,]*)\s*likes?/i);
      if (likeMatch) currentPost.likes = parseInt(likeMatch[1].replace(",", ""), 10);

      const rtMatch = line.match(/(\d[\d,]*)\s*(?:retweets?|reposts?)/i);
      if (rtMatch) currentPost.retweets = parseInt(rtMatch[1].replace(",", ""), 10);
    } else if (currentPost) {
      currentPost.text += " " + line.trim();
    }
  }

  if (currentPost && currentPost.text) {
    posts.push(currentPost);
  }

  return posts;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ---------------------------------------------------------------------------
// Standalone test mode
// ---------------------------------------------------------------------------

if (
  process.argv[1] &&
  process.argv[1].includes("grok_scanner")
) {
  const { config } = await import("dotenv");
  config({ path: new URL("../.env", import.meta.url).pathname });

  const topics = process.argv.slice(2);
  const searchTopics = topics.length > 0 ? topics : DEFAULT_TOPICS.slice(0, 3);

  console.log(`[GrokScanner] Scanning ${searchTopics.length} topics...\n`);

  const results = await scanTopics(searchTopics, {
    recencyMinutes: 30, // Wider window for testing
    maxResults: 5,
  });

  for (const r of results) {
    console.log(`\n--- ${r.topic} (${r.postsFound} posts) ---`);
    if (r.error) {
      console.log(`  ERROR: ${r.error}`);
      continue;
    }
    for (const p of r.posts) {
      console.log(`  ${p.author} ${p.verified ? "[V]" : ""} | ${p.likes} likes`);
      console.log(`    ${p.text.substring(0, 120)}...`);
    }
  }

  process.exit(0);
}
