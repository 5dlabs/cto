/**
 * Grok X API Client
 */

import type { InvestorResult, StartupCreditResult } from './types.js';

const GROK_API_URL = process.env.GROK_API_URL || 'https://api.x.ai/v1';
const GROK_MODEL = process.env.GROK_MODEL || 'grok-4-1-fast-reasoning';

export interface GrokConfig {
  apiKey?: string;
  model?: string;
  apiUrl?: string;
}

export interface XSearchParams {
  query: string;
  minLikes?: number;
  maxResults?: number;
}

export interface XSearchResult {
  id: string;
  likes: number;
  content: string;
  url: string;
  author?: string;
  timestamp?: string;
}

/**
 * Search X using Grok API
 */
export async function searchX(params: XSearchParams, config: GrokConfig = {}): Promise<XSearchResult[]> {
  const apiKey = config.apiKey || process.env.GROK_API_KEY;
  
  if (!apiKey) {
    throw new Error('GROK_API_KEY not set. Set GROK_API_KEY env var or pass apiKey to config.');
  }

  const model = config.model || GROK_MODEL;
  const apiUrl = config.apiUrl || GROK_API_URL;

  try {
    const response = await fetch(`${apiUrl}/responses`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model,
        input: [{
          role: 'user',
          content: `Find top X posts about "${params.query}". Return top ${params.maxResults || 10} posts with: ID, likes, brief summary, and author if available.`
        }],
        tools: [{ type: 'x_search' }]
      })
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Grok API error: ${response.status} - ${error}`);
    }

    const data = await response.json();
    return parseGrokResponse(data, params.query);
  } catch (error) {
    console.error(`X search failed for "${params.query}":`, error);
    throw error;
  }
}

/**
 * Parse Grok API response into structured results
 */
function parseGrokResponse(data: any, query: string): XSearchResult[] {
  const results: XSearchResult[] = [];

  if (!data.output) return results;

  for (const output of data.output) {
    if (output.content && Array.isArray(output.content)) {
      for (const item of output.content) {
        if (item.type === 'output_text' && item.text) {
          const text = item.text;
          const idMatch = text.match(/ID[:\s]*(\d+)/i);
          const likeMatch = text.match(/(\d+)[\s-]*likes?/i);
          const authorMatch = text.match(/@[\w]+/g);

          if (idMatch) {
            results.push({
              id: idMatch[1],
              likes: parseInt(likeMatch?.[1] || '0'),
              content: text.substring(0, 500),
              url: `https://x.com/i/web/status/${idMatch[1]}`,
              author: authorMatch?.[0],
            });
          }
        }
      }
    }
  }

  return results;
}

/**
 * Search for investor-related content
 */
export async function searchInvestors(
  queries: string[],
  config: GrokConfig = {},
  minLikes: number = 10
): Promise<InvestorResult[]> {
  const results: InvestorResult[] = [];
  const timestamp = new Date().toISOString();

  for (const query of queries.slice(0, 5)) { // Limit concurrent searches
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
 * Search for startup credit-related content
 */
export async function searchStartupCredits(
  queries: string[],
  config: GrokConfig = {},
  minLikes: number = 5
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
