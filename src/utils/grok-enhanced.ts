#!/usr/bin/env bun
/**
 * Enhanced Grok X Search with Quality Filters
 */

import { execSync } from "child_process";
import fs from "fs/promises";
import path from "path";
import { CTO_KEYWORDS } from "./keywords";

const GROK_API_URL = "https://api.x.ai/v1";
const GROK_MODEL = "grok-4-1-fast-reasoning";

function getGrokApiKey(): string {
  try {
    return execSync('op item get "Grok X API Key" --vault Automation --fields xai_api_key --reveal', {
      encoding: "utf-8", timeout: 10000
    }).trim();
  } catch {
    throw new Error("Failed to get API key");
  }
}

interface SearchOptions {
  minLikes?: number;
  allowedHandles?: string[];
  excludedHandles?: string[];
  fromDate?: string;
  toDate?: string;
  model?: string;
}

function buildQuery(keywords: string[], options: SearchOptions = {}): string {
  const searchTerms = keywords.slice(0, 5).join(" OR ");
  let query = `(${searchTerms})`;
  
  if (options.minLikes) query += ` min_faves:${options.minLikes}`;
  if (options.allowedHandles?.length) {
    query += ` from:${options.allowedHandles.join(' OR from:')}`;
  }
  if (options.excludedHandles?.length) {
    query += options.excludedHandles.map(h => ` -from:${h}`).join('');
  }
  
  return `${query} lang:en`;
}

async function search(apiKey: string, query: string): Promise<any> {
  const response = await fetch(`${GROK_API_URL}/responses`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiKey}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      model: GROK_MODEL,
      input: [{ role: "user", content: query }],
      tools: [{ type: "x_search" }]
    })
  });
  
  if (!response.ok) throw new Error(`API error: ${response.status}`);
  return response.json();
}

async function main() {
  const apiKey = getGrokApiKey();
  const category = process.argv[2] || "codingAgents";
  const minLikes = parseInt(process.argv[3]) || 50;
  
  const keywords = CTO_KEYWORDS[category as keyof typeof CTO_KEYWORDS] || [];
  if (!keywords.length) {
    console.log(`Category "${category}" not found`);
    return;
  }
  
  const query = buildQuery(keywords, { minLikes });
  console.log(`Searching: ${category} (min ${minLikes} likes)`);
  console.log(`Query: ${query}\n`);
  
  const results = await search(apiKey, query);
  
  // Extract and display content
  for (const item of results.output || []) {
    if (item.content && typeof item.content === 'object') {
      for (const c of item.content) {
        if (c.type === 'output_text' && c.text) {
          console.log(c.text.substring(0, 2000));
        }
      }
    }
  }
}

main().catch(console.error);
