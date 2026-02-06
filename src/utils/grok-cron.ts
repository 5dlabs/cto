#!/usr/bin/env bun
/**
 * Grok X Search Cron Job
 * 
 * Runs periodically to search for CTO-relevant posts on X
 * Searches last 5 days with video/image understanding enabled
 */

import { $ } from "bun";
import { execSync } from "child_process";
import fs from "fs/promises";
import path from "path";
import { CTO_KEYWORDS } from "./keywords";

const DOCS_DIR = "./docs/research";
const RESULTS_DIR = "./docs/grok-results";
const STATE_FILE = "./docs/grok-results/.last-run.json";

// Grok API endpoint (Responses API format)
const GROK_API_URL = "https://api.x.ai/v1";
const GROK_MODEL = "grok-4-1-fast-reasoning";

// Get Grok API key from 1Password
function getGrokApiKey(): string {
  try {
    const result = execSync('op item get "Grok X API Key" --vault Automation --fields xai_api_key --reveal', {
      encoding: "utf-8",
      timeout: 10000
    });
    return result.trim();
  } catch {
    throw new Error("Failed to retrieve Grok API key from 1Password");
  }
}

// Convert camelCase to kebab-case for API compatibility
function toKebabCase(str: string): string {
  return str
    .replace(/([a-z])([A-Z])/g, "$1-$2")  // lowercase followed by uppercase
    .replace(/([A-Z])([A-Z][a-z])/g, "$1-$2") // consecutive caps
    .toLowerCase();
}

// Get keywords from either kebab-case or camelCase category name
function getKeywordsForCategory(category: string): string[] | undefined {
  // Try exact match first
  if (CTO_KEYWORDS[category as keyof typeof CTO_KEYWORDS]) {
    return CTO_KEYWORDS[category as keyof typeof CTO_KEYWORDS];
  }
  // Try converting kebab to camel
  const camelVersion = category.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
  if (CTO_KEYWORDS[camelVersion as keyof typeof CTO_KEYWORDS]) {
    return CTO_KEYWORDS[camelVersion as keyof typeof CTO_KEYWORDS];
  }
  return undefined;
}

// Build search query from keywords
function buildQuery(category: string, keywords: string[], options: {
  minLikes?: number;
  allowedHandles?: string[];
  excludedHandles?: string[];
} = {}): string {
  const searchTerms = keywords.slice(0, 5).join(" OR ");
  
  let query = `(${searchTerms})`;
  
  // Engagement filter
  if (options.minLikes) {
    query += ` min_faves:${options.minLikes}`;
  }
  
  // Handle filters
  if (options.allowedHandles?.length) {
    query += ` from:${options.allowedHandles.join(' OR from:')}`;
  }
  
  if (options.excludedHandles?.length) {
    query += ` -from:${options.excludedHandles.join(' -from:')}`;
  }
  
  query += ` lang:en`;
  
  return query;
}

// Enhanced search with quality filters
async function searchWithGrokEnhanced(
  query: string,
  apiKey: string,
  options: {
    enableImageUnderstanding?: boolean;
    enableVideoUnderstanding?: boolean;
    fromDate?: string;
    toDate?: string;
    model?: string;
  } = {}
): Promise<any> {
  const response = await fetch(`${GROK_API_URL}/responses`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiKey}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      model: options.model || GROK_MODEL,
      input: [
        {
          role: "user",
          content: query
        }
      ],
      tools: [
        {
          type: "x_search",
          allowed_x_handles: undefined,
          excluded_x_handles: undefined,
          from_date: options.fromDate,
          to_date: options.toDate,
          enable_image_understanding: options.enableImageUnderstanding ?? true,
          enable_video_understanding: options.enableVideoUnderstanding ?? true
        }
      ]
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Grok API error: ${response.status} ${response.statusText} - ${error}`);
  }

  return response.json();
}

// Get date range (last 5 days)
function getDateRange(): { from_date: string; to_date: string } {
  const now = new Date();
  const toDate = now.toISOString();
  
  const fromDate = new Date(now.getTime() - 5 * 24 * 60 * 60 * 1000);
  const fromDateStr = fromDate.toISOString();
  
  return {
    from_date: fromDateStr.split('T')[0], // YYYY-MM-DD
    to_date: toDate.split('T')[0]
  };
}

// Search with Grok API (Responses API format)
async function searchWithGrok(
  query: string,
  apiKey: string,
  options: {
    enableImageUnderstanding?: boolean;
    enableVideoUnderstanding?: boolean;
    fromDate?: string;
    toDate?: string;
  } = {}
): Promise<any> {
  const response = await fetch(`${GROK_API_URL}/responses`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiKey}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      model: GROK_MODEL,
      input: [
        {
          role: "user",
          content: query
        }
      ],
      tools: [
        {
          type: "x_search",
          allowed_x_handles: undefined,
          excluded_x_handles: undefined,
          from_date: options.fromDate,
          to_date: options.toDate,
          enable_image_understanding: options.enableImageUnderstanding ?? true,
          enable_video_understanding: options.enableVideoUnderstanding ?? true
        }
      ]
    })
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Grok API error: ${response.status} ${response.statusText} - ${error}`);
  }

  return response.json();
}

// Save results and state
async function saveResults(category: string, results: any): Promise<void> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const outputFile = `${RESULTS_DIR}/${category}/${timestamp}.json`;
  
  await fs.mkdir(`${RESULTS_DIR}/${category}`, { recursive: true });
  await fs.writeFile(outputFile, JSON.stringify(results, null, 2));
  
  console.log(`✅ Saved ${results.data?.length || 0} results to ${outputFile}`);
}

// Update last run state
async function updateLastRun(category: string): Promise<void> {
  const state = {
    lastRun: new Date().toISOString(),
    categories: {}
  };
  
  try {
    const existing = await fs.readFile(STATE_FILE, "utf-8");
    const parsed = JSON.parse(existing);
    Object.assign(state, parsed);
  } catch { /* fresh run */ }
  
  state.categories[category] = new Date().toISOString();
  await fs.writeFile(STATE_FILE, JSON.stringify(state, null, 2));
}

// Main search function
async function runSearch(options: {
  categories?: string[];
  videoOnly?: boolean;
  dryRun?: boolean;
} = {}): Promise<void> {
  console.log(`\n🔍 Grok X Search - CTO Research`);
  console.log(`📅 Date range: last 5 days`);
  console.log(`📹 Video understanding: ${!options.videoOnly ? "enabled" : "disabled"}`);
  console.log("");
  
  const apiKey = getGrokApiKey();
  const dateRange = getDateRange();
  
  const categoriesToSearch = options.categories || Object.keys(CTO_KEYWORDS);
  
  for (const category of categoriesToSearch) {
    const keywords = getKeywordsForCategory(category);
    if (!keywords) {
      console.log(`⚠️ Unknown category: ${category}`);
      continue;
    }
    
    console.log(`\n📊 Category: ${category}`);
    console.log(`   Keywords: ${keywords.slice(0, 3).join(", ")}...`);
    
    if (options.dryRun) {
      console.log(`   [DRY RUN] Would search for: ${keywords.slice(0, 5).join(" OR ")}`);
      continue;
    }
    
    const query = buildQuery(category, keywords);
    
    try {
      const results = await searchWithGrok(query, apiKey, {
        fromDate: dateRange.from_date,
        toDate: dateRange.to_date,
        enableVideoUnderstanding: !options.videoOnly,
        enableImageUnderstanding: true
      });
      
      await saveResults(category, results);
      await updateLastRun(category);
    } catch (error) {
      console.log(`   ❌ Error: ${error}`);
    }
  }
  
  console.log("\n✨ Search complete!");
}

// CLI entry point
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes("--help") || args.includes("-h")) {
    printHelp();
    process.exit(0);
  }
  
  if (args.includes("--list")) {
    printCategories();
    process.exit(0);
  }
  
  if (args.includes("--dry-run") || args.includes("-n")) {
    await runSearch({ dryRun: true });
    return;
  }
  
  if (args.includes("--video-only")) {
    await runSearch({ videoOnly: true });
    return;
  }
  
  if (args.includes("--category") || args.includes("-c")) {
    const categoryIndex = args.indexOf("--category") + 1 || args.indexOf("-c") + 1;
    const category = args[categoryIndex];
    await runSearch({ categories: [category] });
    return;
  }
  
  // Default: run all categories
  await runSearch();
}

function printHelp() {
  console.log(`
🛠️ Grok X Search Cron Job for CTO Research

Usage:
  bun run grok-cron.ts [options]

Options:
  --category, -c <name>  Run only specific category
  --list, -l             List all keyword categories
  --video-only           Search only video content
  --dry-run, -n          Show what would be searched without executing
  --help, -h             Show this help

Categories:
  agent-development, claude-sdk, openai-sdk, mcp, llm-infrastructure,
  vector-databases, ai-observability, ai-security, coding-agents,
  developer-tools, mcp-servers, kubernetes, cloud-infra,
  docker-sandboxes, rust-ai, go-ai, tsnode-ai, memory-context,
  evaluation-testing, api-design, prompt-engineering, multi-model,
  autonomous-agents, deepseek-kimi, mistral-ai, gemini-google,
  open-source-models, blockchain-solana, claude-code-patterns, openclaw

Examples:
  bun run grok-cron.ts                       # Run all categories
  bun run grok-cron.ts -c mcp                # MCP only
  bun run grok-cron.ts --video-only         # Video content only
  bun run grok-cron.ts -l                   # List categories

Cron setup (run every 6 hours):
  0 */6 * * * cd /path/to/agents/research && bun run src/utils/grok-cron.ts

Environment:
  Requires Grok API key in 1Password vault "Automation"
`);
}

function printCategories() {
  console.log("\n📋 CTO Research Keyword Categories:\n");
  
  // Manual mapping for clean display names
  const displayNames: Record<string, string> = {
    agentPlatforms: "agent-platforms",
    claudeAnthropic: "claude-anthropic",
    openAIGPT: "openai-gpt",
    mcp: "mcp",
    codingAgents: "coding-agents",
    skillsCapabilities: "skills-capabilities",
    memoryContext: "memory-context",
    deepSeekKimi: "deepseek-kimi",
    genAIMultimodal: "genai-multimodal",
    deepResearch: "deep-research",
    workflowAutomation: "workflow-automation",
    observabilitySafety: "observability-safety",
    aiIDEs: "ai-ides",
    geminiGoogle: "gemini-google",
    factoryAI: "factory-ai",
    openClaw: "openclaw",
    solanaBlockchain: "solana-blockchain",
    aiLabsModels: "ai-labs-models",
    investorsFunding: "investors-funding",
    cloudInfrastructure: "cloud-infrastructure",
    kubernetesDevops: "kubernetes-devops",
    securityOptimization: "security-optimization",
    uiFrontend: "ui-frontend",
    sandboxesTesting: "sandboxes-testing",
    codingTools: "coding-tools",
    agentSwarms: "agent-swarms",
    programmingLanguages: "programming-languages",
    frameworksLibraries: "frameworks-libraries",
    searchScraping: "search-scraping",
    desktopVercel: "desktop-vercel"
  };
  
  for (const [category, keywords] of Object.entries(CTO_KEYWORDS)) {
    const displayName = displayNames[category] || category;
    console.log(`🎯 ${displayName}:`);
    console.log(`   ${keywords.slice(0, 5).join(", ")}...`);
    console.log("");
  }
}

main().catch(console.error);
