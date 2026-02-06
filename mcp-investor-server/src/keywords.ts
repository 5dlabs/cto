import fs from 'fs/promises';
import path from 'path';

export interface KeywordCategory {
  description: string;
  keywords: string[];
}

export interface KeywordConfig {
  investors: KeywordCategory;
  startupCredits: KeywordCategory;
}

let keywordsCache: KeywordConfig | null = null;

export async function loadKeywords(keywordsPath?: string): Promise<KeywordConfig> {
  if (keywordsCache) return keywordsCache;

  const defaultPath = path.join(process.cwd(), 'keywords.json');
  const filePath = keywordsPath || defaultPath;

  const data = await fs.readFile(filePath, 'utf-8');
  keywordsCache = JSON.parse(data) as KeywordConfig;

  return keywordsCache;
}

export function getInvestorKeywords(keywords: KeywordConfig): string[] {
  return keywords.investors.keywords;
}

export function getStartupCreditKeywords(keywords: KeywordConfig): string[] {
  return keywords.startupCredits.keywords;
}

export function getAllInvestorKeywords(): string[] {
  return [
    "early-stage investor",
    "angel investor",
    "seed fund",
    "pre-seed funding",
    "VC firm",
    "venture capital",
    "tech investor",
    "startup investor",
    "SaaS investor",
    "AI fund",
    "Canadian VC",
    "US VC",
    "global venture capital",
    "accelerator investor",
    "angel network",
    "Series A",
    "Series B",
    "growth equity",
    "micro VC",
    "splash fund"
  ];
}

export function getAllStartupCreditKeywords(): string[] {
  return [
    "startup credits",
    "startup program",
    "startup perk",
    "free tier startup",
    "startup discount",
    "cloud credits startup",
    "software for startups",
    "founder credits",
    "startup sandbox",
    "AWS Activate",
    "Google for Startups",
    "Microsoft for Startups",
    "Stripe Atlas",
    "Cloudflare Workers",
    "Vercel Startup",
    "GitHub Student",
    "OpenAI Startup Fund"
  ];
}
