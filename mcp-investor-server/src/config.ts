/**
 * Load and manage keyword configuration
 */

import fs from 'fs/promises';
import path from 'path';
import type { KeywordConfig } from './types.js';

let configCache: KeywordConfig | null = null;

const DEFAULT_KEYWORDS_PATH = path.join(process.cwd(), 'keywords.json');

/**
 * Load keyword configuration from JSON file
 */
export async function loadKeywordsConfig(
  keywordsPath: string = DEFAULT_KEYWORDS_PATH
): Promise<KeywordConfig> {
  if (configCache) {
    return configCache;
  }

  try {
    const data = await fs.readFile(keywordsPath, 'utf-8');
    configCache = JSON.parse(data) as KeywordConfig;
    return configCache;
  } catch (error) {
    throw new Error(`Failed to load keywords from ${keywordsPath}: ${error}`);
  }
}

/**
 * Get default investor keywords (hardcoded fallback)
 */
export function getDefaultInvestorKeywords(): string[] {
  return [
    'early-stage investor',
    'angel investor',
    'seed fund',
    'pre-seed funding',
    'VC firm',
    'venture capital',
    'tech investor',
    'startup investor',
    'SaaS investor',
    'AI fund',
    'Canadian VC',
    'US VC',
    'global venture capital',
    'accelerator investor',
    'angel network',
    'Series A',
    'Series B',
    'growth equity',
    'micro VC',
    'splash fund',
    'founder collective',
    'first check fund',
    'founder led fund',
  ];
}

/**
 * Get default startup credit keywords (hardcoded fallback)
 */
export function getDefaultStartupCreditKeywords(): string[] {
  return [
    'startup credits',
    'startup program',
    'startup perk',
    'free tier startup',
    'startup discount',
    'cloud credits startup',
    'software for startups',
    'founder credits',
    'startup sandbox',
    'AWS Activate',
    'Google for Startups',
    'Microsoft for Startups',
    'Stripe Atlas',
    'Cloudflare Workers',
    'Vercel Startup',
    'GitHub Student',
    'OpenAI Startup Fund',
    'Twilio Startup',
    'MongoDB Atlas',
    'SendGrid Startup',
    'Algolia Startup',
    'DataDog Startup',
    'New Relic Startup',
    'Linear Startup',
    'Notion Startup',
  ];
}

/**
 * Get all investor keywords (from config or default)
 */
export async function getInvestorKeywords(): Promise<string[]> {
  try {
    const config = await loadKeywordsConfig();
    return config.investors.keywords;
  } catch {
    return getDefaultInvestorKeywords();
  }
}

/**
 * Get all startup credit keywords (from config or default)
 */
export async function getStartupCreditKeywords(): Promise<string[]> {
  try {
    const config = await loadKeywordsConfig();
    return config.startupCredits.keywords;
  } catch {
    return getDefaultStartupCreditKeywords();
  }
}

/**
 * Get combined keywords for broad search
 */
export async function getAllKeywords(): Promise<string[]> {
  const investors = await getInvestorKeywords();
  const credits = await getStartupCreditKeywords();
  return [...investors, ...credits];
}

/**
 * Clear config cache (useful for testing)
 */
export function clearConfigCache(): void {
  configCache = null;
}
