/**
 * Grok X Search - Unified API Client
 *
 * Provides two search modes:
 *  1. Responses API (`/responses`) – structured search with x_search tool, date ranges, media understanding
 *  2. X-Search API (`/x-search`) – simpler keyword search
 */

import { resolveConfig } from './config.js';
import type {
  GrokClientConfig,
  XSearchParams,
  XSearchResult,
  InvestorResult,
  StartupCreditResult,
} from './types.js';

// ---------------------------------------------------------------------------
// Core search via Responses API
// ---------------------------------------------------------------------------

/**
 * Search X using the Grok Responses API (`/responses`).
 *
 * This is the primary search method. It supports date ranges, image/video
 * understanding, and handle filtering.
 */
export async function searchX(
  params: XSearchParams,
  config?: GrokClientConfig,
): Promise<XSearchResult[]> {
  const resolved = resolveConfig(config);

  const body: Record<string, unknown> = {
    model: resolved.model,
    input: [
      {
        role: 'user',
        content: `Find top X posts about "${params.query}". Return top ${params.maxResults ?? 10} posts with: ID, likes, brief summary, and author if available.`,
      },
    ],
    tools: [
      {
        type: 'x_search',
        ...(params.fromDate ? { from_date: params.fromDate } : {}),
        ...(params.toDate ? { to_date: params.toDate } : {}),
        enable_image_understanding: params.enableImageUnderstanding ?? true,
        enable_video_understanding: params.enableVideoUnderstanding ?? true,
      },
    ],
  };

  const response = await fetch(`${resolved.apiUrl}/responses`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${resolved.apiKey}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Grok API error: ${response.status} - ${error}`);
  }

  const data: Record<string, unknown> = await response.json() as Record<string, unknown>;
  return parseGrokResponse(data);
}

// ---------------------------------------------------------------------------
// Open-ended query via Responses API
// ---------------------------------------------------------------------------

/**
 * Send a free-form query to Grok with X search enabled.
 *
 * Unlike `searchX`, this returns Grok's natural language response rather than
 * parsing structured tweet data. Useful for open-ended questions like
 * "what are people saying about Claude Code?" or "summarize the latest drama
 * around OpenAI".
 */
export async function queryX(
  query: string,
  options?: {
    fromDate?: string;
    toDate?: string;
    enableImageUnderstanding?: boolean;
    enableVideoUnderstanding?: boolean;
  },
  config?: GrokClientConfig,
): Promise<string> {
  const resolved = resolveConfig(config);

  const body: Record<string, unknown> = {
    model: resolved.model,
    input: [{ role: 'user', content: query }],
    tools: [
      {
        type: 'x_search',
        ...(options?.fromDate ? { from_date: options.fromDate } : {}),
        ...(options?.toDate ? { to_date: options.toDate } : {}),
        enable_image_understanding: options?.enableImageUnderstanding ?? true,
        enable_video_understanding: options?.enableVideoUnderstanding ?? true,
      },
    ],
  };

  const response = await fetch(`${resolved.apiUrl}/responses`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${resolved.apiKey}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Grok API error: ${response.status} - ${error}`);
  }

  const data = (await response.json()) as Record<string, unknown>;
  return extractTextResponse(data);
}

/**
 * Extract the full text response from a Grok API response.
 */
function extractTextResponse(data: Record<string, unknown>): string {
  const output = data.output as Array<Record<string, unknown>> | undefined;
  if (!output) return '(no response)';

  const parts: string[] = [];
  for (const item of output) {
    const content = item.content as Array<Record<string, unknown>> | undefined;
    if (!content || !Array.isArray(content)) continue;

    for (const entry of content) {
      if (entry.type === 'output_text' && typeof entry.text === 'string') {
        parts.push(entry.text);
      }
    }
  }

  return parts.join('\n\n') || '(no response)';
}

// ---------------------------------------------------------------------------
// Convenience wrappers for domain-specific searches
// ---------------------------------------------------------------------------

/**
 * Search for investor-related content on X.
 */
export async function searchInvestors(
  queries: string[],
  config?: GrokClientConfig,
  minLikes = 10,
): Promise<InvestorResult[]> {
  const results: InvestorResult[] = [];
  const timestamp = new Date().toISOString();

  for (const query of queries.slice(0, 5)) {
    try {
      const xResults = await searchX({ query, minLikes, maxResults: 10 }, config);

      for (const r of xResults) {
        results.push({
          id: `inv_${r.id}`,
          term: query,
          likes: r.likes,
          content: r.content,
          url: r.url,
          timestamp,
        });
      }
    } catch (error) {
      console.error(`Failed to search investors for "${query}":`, error);
    }
  }

  return results.sort((a, b) => b.likes - a.likes);
}

/**
 * Search for startup credit-related content on X.
 */
export async function searchStartupCredits(
  queries: string[],
  config?: GrokClientConfig,
  minLikes = 5,
): Promise<StartupCreditResult[]> {
  const results: StartupCreditResult[] = [];
  const timestamp = new Date().toISOString();

  for (const query of queries.slice(0, 5)) {
    try {
      const xResults = await searchX({ query, minLikes, maxResults: 10 }, config);

      for (const r of xResults) {
        results.push({
          id: `cred_${r.id}`,
          term: query,
          likes: r.likes,
          content: r.content,
          url: r.url,
          timestamp,
        });
      }
    } catch (error) {
      console.error(`Failed to search startup credits for "${query}":`, error);
    }
  }

  return results.sort((a, b) => b.likes - a.likes);
}

// ---------------------------------------------------------------------------
// Cron-style search helpers
// ---------------------------------------------------------------------------

/**
 * Build a structured search query from keywords with optional filters.
 */
export function buildSearchQuery(
  keywords: string[],
  options: {
    minLikes?: number;
    allowedHandles?: string[];
    excludedHandles?: string[];
  } = {},
): string {
  const searchTerms = keywords.slice(0, 5).join(' OR ');
  let query = `(${searchTerms})`;

  if (options.minLikes) {
    query += ` min_faves:${options.minLikes}`;
  }
  if (options.allowedHandles?.length) {
    query += ` from:${options.allowedHandles.join(' OR from:')}`;
  }
  if (options.excludedHandles?.length) {
    query += options.excludedHandles.map((h) => ` -from:${h}`).join('');
  }

  return `${query} lang:en`;
}

/**
 * Get a date range covering the last N days (defaults to 5).
 */
export function getDateRange(days = 5): { fromDate: string; toDate: string } {
  const now = new Date();
  const from = new Date(now.getTime() - days * 24 * 60 * 60 * 1000);

  return {
    fromDate: from.toISOString().split('T')[0],
    toDate: now.toISOString().split('T')[0],
  };
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

function parseGrokResponse(data: Record<string, unknown>): XSearchResult[] {
  const results: XSearchResult[] = [];
  const output = data.output as Array<Record<string, unknown>> | undefined;

  if (!output) return results;

  for (const item of output) {
    const content = item.content as Array<Record<string, unknown>> | undefined;
    if (!content || !Array.isArray(content)) continue;

    for (const entry of content) {
      if (entry.type !== 'output_text' || typeof entry.text !== 'string') continue;

      const text = entry.text;
      const idMatch = text.match(/ID[:\s]*(\d+)/i);
      const likeMatch = text.match(/(\d+)[\s-]*likes?/i);
      const authorMatch = text.match(/@[\w]+/g);

      if (idMatch) {
        results.push({
          id: idMatch[1],
          likes: parseInt(likeMatch?.[1] ?? '0', 10),
          content: text.substring(0, 500),
          url: `https://x.com/i/web/status/${idMatch[1]}`,
          author: authorMatch?.[0],
        });
      }
    }
  }

  return results;
}
