#!/usr/bin/env bun
/**
 * Grok X Search Cron Job
 *
 * Runs periodically to search for CTO-relevant posts on X.
 * Uses the shared lib for API calls, keywords, and configuration.
 *
 * Usage:
 *   bun run src/cron.ts                       # Run all categories
 *   bun run src/cron.ts -c mcp                # Single category
 *   bun run src/cron.ts --list                # List categories
 *   bun run src/cron.ts --dry-run             # Preview without API calls
 */

import fs from 'fs/promises';
import {
  searchX,
  buildSearchQuery,
  getDateRange,
  CTO_KEYWORDS,
  getCtoKeywords,
  getCtoCategoryNames,
} from './lib/index.js';

const RESULTS_DIR = './docs/research/grok-results';
const STATE_FILE = './docs/research/grok-results/.last-run.json';

// ---------------------------------------------------------------------------
// Category lookup (supports camelCase and kebab-case)
// ---------------------------------------------------------------------------

function getKeywordsForCategory(category: string): string[] | undefined {
  const keywords = getCtoKeywords(category);
  if (keywords.length > 0) return keywords;

  // Try converting kebab-case to camelCase
  const camelVersion = category.replace(/-([a-z])/g, (_, letter: string) =>
    letter.toUpperCase(),
  );
  const camelKeywords = getCtoKeywords(camelVersion);
  if (camelKeywords.length > 0) return camelKeywords;

  return undefined;
}

// ---------------------------------------------------------------------------
// Save results and state
// ---------------------------------------------------------------------------

async function saveResults(
  category: string,
  results: Record<string, unknown>,
): Promise<void> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const outputFile = `${RESULTS_DIR}/${category}/${timestamp}.json`;

  await fs.mkdir(`${RESULTS_DIR}/${category}`, { recursive: true });
  await fs.writeFile(outputFile, JSON.stringify(results, null, 2));

  const count =
    Array.isArray(results) ? results.length : (results as Record<string, unknown>).data
      ? ((results as Record<string, unknown>).data as unknown[]).length
      : 0;

  console.log(`  Saved ${count} results to ${outputFile}`);
}

interface LastRunState {
  lastRun: string;
  categories: Record<string, string>;
}

async function updateLastRun(category: string): Promise<void> {
  const state: LastRunState = {
    lastRun: new Date().toISOString(),
    categories: {},
  };

  try {
    const existing = await fs.readFile(STATE_FILE, 'utf-8');
    const parsed = JSON.parse(existing) as LastRunState;
    Object.assign(state, parsed);
  } catch {
    /* fresh run */
  }

  state.categories[category] = new Date().toISOString();
  await fs.writeFile(STATE_FILE, JSON.stringify(state, null, 2));
}

// ---------------------------------------------------------------------------
// Main search
// ---------------------------------------------------------------------------

interface RunSearchOptions {
  categories?: string[];
  videoOnly?: boolean;
  dryRun?: boolean;
}

async function runSearch(options: RunSearchOptions = {}): Promise<void> {
  console.log('\nGrok X Search - CTO Research');
  console.log('Date range: last 5 days');
  console.log(`Video understanding: ${options.videoOnly ? 'disabled' : 'enabled'}\n`);

  const dateRange = getDateRange(5);
  const categoriesToSearch =
    options.categories ?? getCtoCategoryNames();

  for (const category of categoriesToSearch) {
    const keywords = getKeywordsForCategory(category);
    if (!keywords) {
      console.log(`Unknown category: ${category}`);
      continue;
    }

    console.log(`\nCategory: ${category}`);
    console.log(`  Keywords: ${keywords.slice(0, 3).join(', ')}...`);

    if (options.dryRun) {
      console.log(
        `  [DRY RUN] Would search for: ${keywords.slice(0, 5).join(' OR ')}`,
      );
      continue;
    }

    const query = buildSearchQuery(keywords);

    try {
      const results = await searchX({
        query,
        fromDate: dateRange.fromDate,
        toDate: dateRange.toDate,
        enableVideoUnderstanding: !options.videoOnly,
        enableImageUnderstanding: true,
      });

      await saveResults(category, results as unknown as Record<string, unknown>);
      await updateLastRun(category);
    } catch (error) {
      console.log(`  Error: ${String(error)}`);
    }
  }

  console.log('\nSearch complete!');
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function printHelp(): void {
  console.log(`
Grok X Search Cron Job for CTO Research

Usage:
  bun run src/cron.ts [options]

Options:
  --category, -c <name>  Run only specific category
  --list, -l             List all keyword categories
  --video-only           Search only video content
  --dry-run, -n          Show what would be searched without executing
  --help, -h             Show this help

Examples:
  bun run src/cron.ts                       # Run all categories
  bun run src/cron.ts -c mcp                # MCP only
  bun run src/cron.ts --video-only          # Video content only
  bun run src/cron.ts -l                    # List categories

Cron setup (run every 6 hours):
  0 */6 * * * cd /path/to/grok && bun run src/cron.ts

Environment:
  Set GROK_API_KEY env var or have 1Password CLI configured.
`);
}

function printCategories(): void {
  console.log('\nCTO Research Keyword Categories:\n');

  for (const category of getCtoCategoryNames()) {
    const keywords = getCtoKeywords(category);
    console.log(`  ${category}: (${keywords.length} keywords)`);
    console.log(`    ${keywords.slice(0, 5).join(', ')}...`);
    console.log('');
  }
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);

  if (args.includes('--help') || args.includes('-h')) {
    printHelp();
    return;
  }

  if (args.includes('--list') || args.includes('-l')) {
    printCategories();
    return;
  }

  if (args.includes('--dry-run') || args.includes('-n')) {
    await runSearch({ dryRun: true });
    return;
  }

  if (args.includes('--video-only')) {
    await runSearch({ videoOnly: true });
    return;
  }

  if (args.includes('--category') || args.includes('-c')) {
    const categoryIndex =
      args.indexOf('--category') + 1 || args.indexOf('-c') + 1;
    const category = args[categoryIndex];
    await runSearch({ categories: [category] });
    return;
  }

  // Default: run all categories
  await runSearch();
}

main().catch(console.error);
