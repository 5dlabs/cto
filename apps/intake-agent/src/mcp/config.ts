/**
 * MCP Server Configuration for intake-agent.
 * 
 * Configures external MCP servers for research mode:
 * - Firecrawl: Web scraping and research
 * - OctoCode: GitHub code search
 * - Context7: Library documentation lookup
 * - Tavily: AI-powered web search
 * - Perplexity: AI research assistant
 * - Exa: AI-powered web/code search
 * 
 * ## Tool Server Mode
 * 
 * When TOOLS_SERVER_URL is set, the agent connects to a centralized tool server
 * instead of spawning MCP servers locally via npx. This is the recommended
 * approach for production deployments:
 * 
 * ```
 * export TOOLS_SERVER_URL=http://cto-tools.cto.svc.cluster.local:3000/mcp
 * ```
 * 
 * The tool server aggregates multiple MCP tools and provides:
 * - Centralized credential management
 * - Better observability and logging
 * - Consistent tool availability across agents
 */

import type { Options } from '@anthropic-ai/claude-code';

/**
 * MCP server configuration type from Claude Agent SDK.
 */
type McpServerConfig = NonNullable<Options['mcpServers']>[string];

/**
 * Tool server URL (for centralized tool server mode).
 * When set, agents connect to this server instead of spawning local MCP processes.
 */
export const TOOLS_SERVER_URL = process.env['TOOLS_SERVER_URL'] || '';

/**
 * Check if tool server mode is enabled.
 */
export function isToolServerMode(): boolean {
  return TOOLS_SERVER_URL !== '';
}

/**
 * Get the tool server URL (returns empty if not configured).
 */
export function getToolServerUrl(): string {
  return TOOLS_SERVER_URL;
}

/**
 * Check if an environment variable is set and non-empty.
 */
function hasEnvVar(name: string): boolean {
  const value = process.env[name];
  return value !== undefined && value !== '';
}

/**
 * Get Firecrawl MCP server config if API key is available.
 * Firecrawl provides web scraping and crawling capabilities.
 */
function getFirecrawlConfig(): McpServerConfig | null {
  if (!hasEnvVar('FIRECRAWL_API_KEY')) {
    return null;
  }

  return {
    type: 'stdio',
    command: 'npx',
    args: ['-y', '@anthropic-ai/firecrawl-mcp'],
    env: {
      FIRECRAWL_API_KEY: process.env['FIRECRAWL_API_KEY'] ?? '',
    },
  };
}

/**
 * Get OctoCode MCP server config if available.
 * OctoCode provides semantic code search across GitHub.
 */
function getOctoCodeConfig(): McpServerConfig | null {
  // OctoCode uses GitHub token for API access
  if (!hasEnvVar('GITHUB_TOKEN') && !hasEnvVar('OCTOCODE_API_KEY')) {
    return null;
  }

  return {
    type: 'stdio',
    command: 'npx',
    args: ['-y', '@anthropic-ai/octocode-mcp'],
    env: {
      GITHUB_TOKEN: process.env['GITHUB_TOKEN'] ?? process.env['OCTOCODE_API_KEY'] ?? '',
    },
  };
}

/**
 * Get Context7 MCP server config if available.
 * Context7 provides library documentation lookup.
 */
function getContext7Config(): McpServerConfig | null {
  // Context7 may not require API key for basic usage
  return {
    type: 'stdio',
    command: 'npx',
    args: ['-y', '@anthropic-ai/context7-mcp'],
    env: {},
  };
}

/**
 * Get Tavily MCP server config for AI-powered web search.
 */
function getTavilyConfig(): McpServerConfig | null {
  if (!hasEnvVar('TAVILY_API_KEY')) {
    return null;
  }

  return {
    type: 'stdio',
    command: 'npx',
    args: ['-y', '@anthropic-ai/tavily-mcp'],
    env: {
      TAVILY_API_KEY: process.env['TAVILY_API_KEY'] ?? '',
    },
  };
}

/**
 * Get Perplexity MCP server config for AI-powered research.
 */
function getPerplexityConfig(): McpServerConfig | null {
  if (!hasEnvVar('PERPLEXITY_API_KEY')) {
    return null;
  }

  return {
    type: 'stdio',
    command: 'npx',
    args: ['-y', '@anthropic-ai/perplexity-mcp'],
    env: {
      PERPLEXITY_API_KEY: process.env['PERPLEXITY_API_KEY'] ?? '',
    },
  };
}

/**
 * Get Exa MCP server config.
 * Exa uses HTTP transport with auth in URL, no separate API key needed.
 */
function getExaConfig(): McpServerConfig | null {
  // Exa uses HTTP transport - always available
  // API key can be passed via URL or environment
  const exaUrl = process.env['EXA_MCP_URL'] || 'https://mcp.exa.ai/mcp';
  
  return {
    type: 'sse',
    url: exaUrl,
  };
}


/**
 * Build MCP server configuration for research mode.
 * Only includes servers that have required credentials configured.
 */
export function buildMcpConfig(options: {
  enableFirecrawl?: boolean;
  enableOctoCode?: boolean;
  enableContext7?: boolean;
  enableWebSearch?: boolean;
  enableTavily?: boolean;
  enablePerplexity?: boolean;
  enableExa?: boolean;
}): Record<string, McpServerConfig> {
  const config: Record<string, McpServerConfig> = {};

  // Add Firecrawl for web research
  if (options.enableFirecrawl !== false) {
    const firecrawl = getFirecrawlConfig();
    if (firecrawl) {
      config['firecrawl'] = firecrawl;
    }
  }

  // Add OctoCode for code search
  if (options.enableOctoCode !== false) {
    const octocode = getOctoCodeConfig();
    if (octocode) {
      config['octocode'] = octocode;
    }
  }

  // Add Context7 for library docs
  if (options.enableContext7 !== false) {
    const context7 = getContext7Config();
    if (context7) {
      config['context7'] = context7;
    }
  }

  // Add Tavily for web search (default web search provider)
  if (options.enableTavily !== false || options.enableWebSearch !== false) {
    const tavily = getTavilyConfig();
    if (tavily) {
      config['tavily'] = tavily;
    }
  }

  // Add Perplexity for AI-powered research
  if (options.enablePerplexity !== false) {
    const perplexity = getPerplexityConfig();
    if (perplexity) {
      config['perplexity'] = perplexity;
    }
  }

  // Add Exa for AI-powered web/code search
  if (options.enableExa !== false) {
    const exa = getExaConfig();
    if (exa) {
      config['exa'] = exa;
    }
  }

  return config;
}

/**
 * Get research-enabled MCP servers.
 * Returns a configuration suitable for research mode operations.
 */
export function getResearchMcpServers(): Record<string, McpServerConfig> {
  return buildMcpConfig({
    enableFirecrawl: true,
    enableOctoCode: true,
    enableContext7: true,
    enableTavily: true,
    enablePerplexity: true,
    enableExa: true,
  });
}

/**
 * Check if any research MCP servers are available.
 */
export function hasResearchCapability(): boolean {
  const servers = getResearchMcpServers();
  return Object.keys(servers).length > 0;
}

/**
 * List available MCP servers and their status.
 */
export function listAvailableMcpServers(): Array<{ name: string; available: boolean; reason?: string }> {
  return [
    {
      name: 'firecrawl',
      available: hasEnvVar('FIRECRAWL_API_KEY'),
      reason: hasEnvVar('FIRECRAWL_API_KEY') ? undefined : 'FIRECRAWL_API_KEY not set',
    },
    {
      name: 'octocode',
      available: hasEnvVar('GITHUB_TOKEN') || hasEnvVar('OCTOCODE_API_KEY'),
      reason: (hasEnvVar('GITHUB_TOKEN') || hasEnvVar('OCTOCODE_API_KEY')) 
        ? undefined 
        : 'GITHUB_TOKEN or OCTOCODE_API_KEY not set',
    },
    {
      name: 'context7',
      available: true, // Always available
      reason: undefined,
    },
    {
      name: 'tavily',
      available: hasEnvVar('TAVILY_API_KEY'),
      reason: hasEnvVar('TAVILY_API_KEY') ? undefined : 'TAVILY_API_KEY not set',
    },
    {
      name: 'perplexity',
      available: hasEnvVar('PERPLEXITY_API_KEY'),
      reason: hasEnvVar('PERPLEXITY_API_KEY') ? undefined : 'PERPLEXITY_API_KEY not set',
    },
    {
      name: 'exa',
      available: true, // Exa uses HTTP transport, always available
      reason: undefined,
    },
  ];
}
