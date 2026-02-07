/**
 * Grok X Search - Shared Types
 */

/** Configuration for the Grok API client */
export interface GrokClientConfig {
  apiKey?: string;
  model?: string;
  apiUrl?: string;
}

/** Parameters for a raw X search */
export interface XSearchParams {
  query: string;
  minLikes?: number;
  maxResults?: number;
  fromDate?: string;
  toDate?: string;
  enableImageUnderstanding?: boolean;
  enableVideoUnderstanding?: boolean;
}

/** A single parsed result from X */
export interface XSearchResult {
  id: string;
  likes: number;
  content: string;
  url: string;
  author?: string;
  timestamp?: string;
}

/** Investor-specific result */
export interface InvestorResult {
  id: string;
  term: string;
  likes: number;
  content: string;
  url: string;
  timestamp: string;
}

/** Startup credit-specific result */
export interface StartupCreditResult {
  id: string;
  term: string;
  likes: number;
  content: string;
  url: string;
  timestamp: string;
}

/** Combined search results */
export interface SearchResults {
  investors: InvestorResult[];
  startupCredits: StartupCreditResult[];
}

/** Keyword category from keywords.json */
export interface KeywordCategory {
  description: string;
  keywords: string[];
}

/** Keywords.json schema */
export interface KeywordConfig {
  investors: KeywordCategory;
  startupCredits: KeywordCategory;
}

/** Cron search options */
export interface CronSearchOptions {
  categories?: string[];
  videoOnly?: boolean;
  dryRun?: boolean;
  minLikes?: number;
  allowedHandles?: string[];
  excludedHandles?: string[];
}
