#!/usr/bin/env bun
/**
 * Grok X Search Utility
 * 
 * Research workflow:
 * 1. Read research documents from docs/research/
 * 2. Extract keywords/topics
 * 3. Search X using Grok API
 * 4. Save results
 */

import { $ } from "bun";
import { execSync } from "child_process";
import fs from "fs/promises";
import path from "path";

const DOCS_DIR = "./docs/research";
const RESULTS_DIR = "./docs/grok-results";

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

// Grok API endpoint
const GROK_API_URL = "https://api.x.ai/v1";

// Extract topics/tags from a markdown document
function extractKeywords(content: string): string[] {
  const keywords = new Set<string>();
  
  // Extract topics section
  const topicsMatch = content.match(/^topics:\s*$(.*?)(?=\n\w|\n---|\Z)/gms);
  if (topicsMatch) {
    topicsMatch.forEach(block => {
      const matches = block.matchAll(/^\s+-\s+(\S+)/gm);
      matches.forEach(m => keywords.add(m[1]));
    });
  }
  
  // Extract tags section
  const tagsMatch = content.match(/^tags:\s*$(.*?)(?=\n\w|\n---|\Z)/gms);
  if (tagsMatch) {
    tagsMatch.forEach(block => {
      const matches = block.matchAll(/^\s+-\s+(\S+)/gm);
      matches.forEach(m => keywords.add(m[1]));
    });
  }
  
  // Extract # headers (document titles)
  const headerMatches = content.matchAll(/^#+\s+(.+)/gm);
  headerMatches.forEach(m => {
    const words = m[1].toLowerCase().split(/\s+/);
    words.forEach(w => {
      if (w.length > 3 && !w.match(/^[0-9]/)) {
        keywords.add(w.replace(/[^a-z0-9-]/g, ""));
      }
    });
  });
  
  return Array.from(keywords).slice(0, 50);
}

// Get all research documents
async function getResearchDocuments(): Promise<string[]> {
  const files: string[] = [];
  
  async function scanDir(dir: string) {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isFile() && entry.name.endsWith(".md")) {
        files.push(fullPath);
      } else if (entry.isDirectory()) {
        await scanDir(fullPath);
      }
    }
  }
  
  await scanDir(DOCS_DIR);
  return files;
}

// Search X using Grok API
async function searchWithGrok(query: string, apiKey: string): Promise<any> {
  const response = await fetch(`${GROK_API_URL}/x-search`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiKey}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      query,
      max_results: 20,
      enable_image_understanding: true,
      enable_video_understanding: true
    })
  });

  if (!response.ok) {
    throw new Error(`Grok API error: ${response.status} ${response.statusText}`);
  }

  return response.json();
}

// Main search function
async function researchFromKeywords(keywords: string[]): Promise<void> {
  console.log(`\n🔍 Starting Grok X Search Research`);
  console.log(`📁 Reading from: ${DOCS_DIR}`);
  console.log(`🎯 Keywords found: ${keywords.length}\n`);
  
  const apiKey = getGrokApiKey();
  
  // Create results directory
  try {
    await fs.mkdir(RESULTS_DIR, { recursive: true });
  } catch { /* directory exists */ }
  
  // Search for each keyword category
  const categories = {
    "agent-orchestration": keywords.filter(k => k.includes("agent") || k.includes("orchestration")),
    "ai-coding": keywords.filter(k => k.includes("code") || k.includes("coding") || k.includes("generation")),
    "memory-systems": keywords.filter(k => k.includes("memory") || k.includes("context")),
    "mcp": keywords.filter(k => k.includes("mcp")),
    "workflow-automation": keywords.filter(k => k.includes("workflow") || k.includes("automation")),
    "tools-capabilities": keywords.filter(k => k.includes("tool") || k.includes("skill") || k.includes("capability"))
  };
  
  for (const [category, categoryKeywords] of Object.entries(categories)) {
    if (categoryKeywords.length === 0) continue;
    
    console.log(`\n📊 Category: ${category}`);
    console.log(`   Keywords: ${categoryKeywords.slice(0, 5).join(", ")}`);
    
    // Search with combined keywords
    const query = `(${categoryKeywords.slice(0, 3).join(" OR ")}) lang:en has:images`;
    
    try {
      const results = await searchWithGrok(query, apiKey);
      
      const outputFile = `${RESULTS_DIR}/${category}-${Date.now()}.json`;
      await fs.writeFile(outputFile, JSON.stringify(results, null, 2));
      console.log(`   ✅ Saved ${results.data?.length || 0} results to ${outputFile}`);
    } catch (error) {
      console.log(`   ❌ Error: ${error}`);
    }
  }
  
  console.log("\n✨ Research complete!");
}

// CLI entry point
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes("--help") || args.includes("-h")) {
    printHelp();
    process.exit(0);
  }
  
  if (args.includes("--extract")) {
    // Extract keywords from documents only
    const docs = await getResearchDocuments();
    const allKeywords = new Set<string>();
    
    for (const doc of docs) {
      const content = await fs.readFile(doc, "utf-8");
      const keywords = extractKeywords(content);
      keywords.forEach(k => allKeywords.add(k));
    }
    
    console.log("\n📋 Extracted Keywords:");
    console.log(Array.from(allKeywords).sort().join("\n"));
    return;
  }
  
  if (args.includes("--search") && args[1]) {
    // Search specific query
    const apiKey = getGrokApiKey();
    const results = await searchWithGrok(args[1], apiKey);
    console.log(JSON.stringify(results, null, 2));
    return;
  }
  
  // Default: extract keywords and search all categories
  const docs = await getResearchDocuments();
  const allKeywords: string[] = [];
  
  for (const doc of docs) {
    const content = await fs.readFile(doc, "utf-8");
    const keywords = extractKeywords(content);
    allKeywords.push(...keywords);
  }
  
  await researchFromKeywords(allKeywords);
}

function printHelp() {
  console.log(`
🛠️ Grok X Search Utility

Usage:
  bun run grok-search.ts [options]

Options:
  --extract       Extract keywords from research documents only
  --search <query>  Search X with specific query
  --help, -h      Show this help message

Examples:
  bun run grok-search.ts --extract
  bun run grok-search.ts --search "AI agents 2024"

Environment:
  Requires Grok API key in 1Password vault "Automation"
  Item name: "Grok X API Key"
  Field: xai_api_key
`);
}

main().catch(console.error);
