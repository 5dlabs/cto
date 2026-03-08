/**
 * Grok X Search - Shared Library
 *
 * Re-exports everything from the unified library so consumers can import from
 * a single entry point: `import { searchX, CTO_KEYWORDS } from './lib/index.js'`
 */

// Client
export {
  searchX,
  queryX,
  searchInvestors,
  searchStartupCredits,
  buildSearchQuery,
  getDateRange,
} from './client.js';

// Configuration
export {
  resolveApiKey,
  resolveConfig,
  GROK_API_URL,
  GROK_MODEL,
} from './config.js';

// Keywords
export {
  loadKeywordsConfig,
  clearConfigCache,
  getDefaultInvestorKeywords,
  getDefaultStartupCreditKeywords,
  getInvestorKeywords,
  getStartupCreditKeywords,
  getAllKeywords,
  CTO_KEYWORDS,
  ALL_CTO_KEYWORDS,
  getCtoKeywords,
  getCtoCategoryNames,
} from './keywords.js';

// Types
export type {
  GrokClientConfig,
  XSearchParams,
  XSearchResult,
  InvestorResult,
  StartupCreditResult,
  SearchResults,
  KeywordCategory,
  KeywordConfig,
  CronSearchOptions,
} from './types.js';
