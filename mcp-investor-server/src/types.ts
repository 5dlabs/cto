/**
 * MCP Investor Research Server - Core Types
 */

export interface KeywordCategory {
  description: string;
  keywords: string[];
}

export interface KeywordConfig {
  investors: KeywordCategory;
  startupCredits: KeywordCategory;
}

export interface InvestorResult {
  id: string;
  term: string;
  likes: number;
  content: string;
  url: string;
  timestamp: string;
}

export interface StartupCreditResult {
  id: string;
  term: string;
  likes: number;
  content: string;
  url: string;
  timestamp: string;
}

export interface SearchResults {
  investors: InvestorResult[];
  startupCredits: StartupCreditResult[];
}

export interface MCPEnv {
  GROK_API_KEY?: string;
  XAI_API_KEY?: string;
}
