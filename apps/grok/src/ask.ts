#!/usr/bin/env bun
/**
 * Grok Ask CLI - Open-ended X search via Grok
 *
 * Usage:
 *   bun run src/ask.ts "what are people saying about Claude Code?"
 *   bun run src/ask.ts --days 3 "latest AI agent frameworks"
 *   bun run src/ask.ts              # interactive mode
 */

import * as readline from 'readline';
import { queryX, getDateRange } from './lib/index.js';

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);

function showHelp(): void {
  console.log(`
  grok ask - Open-ended X search via Grok

  Usage:
    bun run src/ask.ts <query>          Single query
    bun run src/ask.ts                  Interactive mode
    bun run src/ask.ts -h | --help      Show this help

  Options:
    -d, --days <n>    Search last N days (default: no date filter)
    -h, --help        Show help
  `.trim());
  process.exit(0);
}

let days: number | undefined;
const positional: string[] = [];

for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  if (arg === '-h' || arg === '--help') {
    showHelp();
  } else if (arg === '-d' || arg === '--days') {
    days = parseInt(args[++i], 10);
    if (isNaN(days) || days < 1) {
      console.error('Error: --days requires a positive integer');
      process.exit(1);
    }
  } else {
    positional.push(arg);
  }
}

// ---------------------------------------------------------------------------
// Query execution
// ---------------------------------------------------------------------------

async function ask(query: string): Promise<void> {
  const dateOpts = days ? getDateRange(days) : undefined;

  try {
    const response = await queryX(
      query,
      dateOpts ? { fromDate: dateOpts.fromDate, toDate: dateOpts.toDate } : undefined,
    );
    console.log();
    console.log(response);
    console.log();
  } catch (err) {
    console.error('Error:', err instanceof Error ? err.message : err);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

if (positional.length > 0) {
  // Single-shot mode
  await ask(positional.join(' '));
} else {
  // Interactive mode
  console.log('grok ask - interactive mode (type "exit" or Ctrl+C to quit)');
  console.log();

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  const prompt = (): void => {
    rl.question('ask> ', async (input) => {
      const trimmed = input.trim();
      if (!trimmed || trimmed === 'exit' || trimmed === 'quit') {
        rl.close();
        return;
      }
      await ask(trimmed);
      prompt();
    });
  };

  prompt();
}
