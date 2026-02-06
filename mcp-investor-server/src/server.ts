/**
 * MCP Investor Research Server
 * 
 * Provides tools for researching early-stage investors and startup credits via X/Grok.
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { z } from 'zod';
import {
  loadKeywordsConfig,
  getInvestorKeywords,
  getStartupCreditKeywords,
  getAllKeywords,
  clearConfigCache,
} from './config.js';
import { searchInvestors, searchStartupCredits, type GrokConfig } from './grok-client.js';

// Create MCP server
const server = new Server(
  { name: 'mcp-investor-research', version: '0.1.0' },
  { capabilities: { tools: {} } }
);

// Schema definitions
const SearchInvestorSchema = z.object({
  query: z.string().optional().describe('Specific investor or theme to search for'),
  minLikes: z.number().optional().default(10).describe('Minimum tweet likes to include'),
  limit: z.number().optional().default(20).describe('Maximum number of results'),
});

const SearchStartupCreditSchema = z.object({
  query: z.string().optional().describe('Specific program or credit type to search for'),
  minLikes: z.number().optional().default(5).describe('Minimum tweet likes to include'),
  limit: z.number().optional().default(20).describe('Maximum number of results'),
});

const AddKeywordSchema = z.object({
  keyword: z.string().min(2).describe('The new keyword to add'),
  category: z.enum(['investors', 'startupCredits']).describe('Which category to add to'),
});

const RemoveKeywordSchema = z.object({
  keyword: z.string().min(2).describe('The keyword to remove'),
  category: z.enum(['investors', 'startupCredits']).describe('Which category to remove from'),
});

const LoadCustomSchema = z.object({
  filePath: z.string().describe('Path to custom keywords JSON file'),
});

/**
 * Get Grok configuration from environment
 */
function getGrokConfig(): GrokConfig {
  return {
    apiKey: process.env.GROK_API_KEY,
    model: process.env.GROK_MODEL,
    apiUrl: process.env.GROK_API_URL,
  };
}

// Tool definitions
const tools = {
  search_investors: {
    name: 'search_investors',
    description: 'Search for early-stage investors, angels, VCs, seed funds, and accelerators on X',
    inputSchema: {
      type: 'object',
      properties: {
        query: { type: 'string', description: 'Specific investor or theme to search for' },
        minLikes: { type: 'number', description: 'Minimum tweet likes to include', default: 10 },
        limit: { type: 'number', description: 'Maximum number of results', default: 20 },
      },
    },
  },
  search_startup_credits: {
    name: 'search_startup_credits',
    description: 'Search for startup credits, perks, discounts, and free tier programs on X',
    inputSchema: {
      type: 'object',
      properties: {
        query: { type: 'string', description: 'Specific program or credit type to search for' },
        minLikes: { type: 'number', description: 'Minimum tweet likes to include', default: 5 },
        limit: { type: 'number', description: 'Maximum number of results', default: 20 },
      },
    },
  },
  search_all: {
    name: 'search_all',
    description: 'Search for both investors and startup credits in a single request',
    inputSchema: {
      type: 'object',
      properties: {
        query: { type: 'string', description: 'Search term (optional)' },
        minLikes: { type: 'number', description: 'Minimum likes', default: 5 },
        limit: { type: 'number', description: 'Results per category', default: 10 },
      },
    },
  },
  list_investor_keywords: {
    name: 'list_investor_keywords',
    description: 'List all investor-related search keywords',
    inputSchema: { type: 'object', properties: {} },
  },
  list_startup_credit_keywords: {
    name: 'list_startup_credit_keywords',
    description: 'List all startup credit and perk search keywords',
    inputSchema: { type: 'object', properties: {} },
  },
  get_keywords_config: {
    name: 'get_keywords_config',
    description: 'Get the complete keywords configuration from keywords.json',
    inputSchema: { type: 'object', properties: {} },
  },
  add_keyword: {
    name: 'add_keyword',
    description: 'Add a new keyword to the keywords.json configuration file',
    inputSchema: {
      type: 'object',
      properties: {
        keyword: { type: 'string', description: 'The new keyword to add' },
        category: { type: 'string', enum: ['investors', 'startupCredits'], description: 'Which category to add to' },
      },
      required: ['keyword', 'category'],
    },
  },
  remove_keyword: {
    name: 'remove_keyword',
    description: 'Remove a keyword from the keywords.json configuration file',
    inputSchema: {
      type: 'object',
      properties: {
        keyword: { type: 'string', description: 'The keyword to remove' },
        category: { type: 'string', enum: ['investors', 'startupCredits'], description: 'Which category to remove from' },
      },
      required: ['keyword', 'category'],
    },
  },
  load_custom_keywords: {
    name: 'load_custom_keywords',
    description: 'Load custom keywords from a JSON file path',
    inputSchema: {
      type: 'object',
      properties: {
        filePath: { type: 'string', description: 'Path to custom keywords JSON file' },
      },
      required: ['filePath'],
    },
  },
  health_check: {
    name: 'health_check',
    description: 'Check if the server is running and configured correctly',
    inputSchema: { type: 'object', properties: {} },
  },
};

// Handle tool list requests
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return { tools: Object.values(tools) };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case 'search_investors': {
        const { query, minLikes, limit } = SearchInvestorSchema.parse(args);
        const keywords = await getInvestorKeywords();
        const searchTerms = query ? [query, ...keywords] : keywords;
        const results = await searchInvestors(searchTerms.slice(0, limit), getGrokConfig(), minLikes);
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, count: results.length, results: results.slice(0, limit), searched: searchTerms.slice(0, limit).length }, null, 2),
          }],
        };
      }

      case 'search_startup_credits': {
        const { query, minLikes, limit } = SearchStartupCreditSchema.parse(args);
        const keywords = await getStartupCreditKeywords();
        const searchTerms = query ? [query, ...keywords] : keywords;
        const results = await searchStartupCredits(searchTerms.slice(0, limit), getGrokConfig(), minLikes);
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, count: results.length, results: results.slice(0, limit), searched: searchTerms.slice(0, limit).length }, null, 2),
          }],
        };
      }

      case 'search_all': {
        const parsed = z.object({
          query: z.string().optional(),
          minLikes: z.number().optional().default(5),
          limit: z.number().optional().default(10),
        }).parse(args);

        const [investorResults, creditResults] = await Promise.all([
          searchInvestors(await getInvestorKeywords(), getGrokConfig(), parsed.minLikes),
          searchStartupCredits(await getStartupCreditKeywords(), getGrokConfig(), parsed.minLikes),
        ]);
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({
              success: true,
              investors: { count: investorResults.length, results: investorResults.slice(0, parsed.limit) },
              startupCredits: { count: creditResults.length, results: creditResults.slice(0, parsed.limit) },
            }, null, 2),
          }],
        };
      }

      case 'list_investor_keywords': {
        const keywords = await getInvestorKeywords();
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, count: keywords.length, keywords }, null, 2),
          }],
        };
      }

      case 'list_startup_credit_keywords': {
        const keywords = await getStartupCreditKeywords();
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, count: keywords.length, keywords }, null, 2),
          }],
        };
      }

      case 'get_keywords_config': {
        const config = await loadKeywordsConfig();
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, config }, null, 2),
          }],
        };
      }

      case 'add_keyword': {
        const { keyword, category } = AddKeywordSchema.parse(args);
        const config = await loadKeywordsConfig();
        const normalizedKeyword = keyword.toLowerCase().trim();
        const targetKeywords = category === 'investors' ? config.investors.keywords : config.startupCredits.keywords;
        
        if (targetKeywords.includes(normalizedKeyword)) {
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({ success: false, message: `Keyword "${normalizedKeyword}" already exists in ${category}` }, null, 2),
            }],
          };
        }
        
        targetKeywords.push(normalizedKeyword);
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, message: `Keyword "${normalizedKeyword}" would be added to ${category}`, keyword: normalizedKeyword, category, totalKeywords: targetKeywords.length }, null, 2),
          }],
        };
      }

      case 'remove_keyword': {
        const { keyword, category } = RemoveKeywordSchema.parse(args);
        const config = await loadKeywordsConfig();
        const normalizedKeyword = keyword.toLowerCase().trim();
        const targetKeywords = category === 'investors' ? config.investors.keywords : config.startupCredits.keywords;
        const index = targetKeywords.indexOf(normalizedKeyword);
        
        if (index === -1) {
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({ success: false, message: `Keyword "${normalizedKeyword}" not found in ${category}` }, null, 2),
            }],
          };
        }
        
        targetKeywords.splice(index, 1);
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, message: `Keyword "${normalizedKeyword}" would be removed from ${category}`, keyword: normalizedKeyword, category, totalKeywords: targetKeywords.length }, null, 2),
          }],
        };
      }

      case 'load_custom_keywords': {
        const { filePath } = LoadCustomSchema.parse(args);
        const config = await loadKeywordsConfig(filePath);
        clearConfigCache();
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: true, message: `Loaded custom keywords from ${filePath}`, investorKeywords: config.investors.keywords.length, startupCreditKeywords: config.startupCredits.keywords.length }, null, 2),
          }],
        };
      }

      case 'health_check': {
        const grokConfigured = !!process.env.GROK_API_KEY;
        try {
          await getInvestorKeywords();
          await getStartupCreditKeywords();
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({ success: true, status: 'healthy', grokConfigured, keywordsLoaded: true }, null, 2),
            }],
          };
        } catch (error) {
          return {
            content: [{
              type: 'text',
              text: JSON.stringify({ success: false, status: 'unhealthy', grokConfigured, keywordsLoaded: false, error: String(error) }, null, 2),
            }],
          };
        }
      }

      default:
        return {
          content: [{
            type: 'text',
            text: JSON.stringify({ success: false, error: `Unknown tool: ${name}` }, null, 2),
          }],
        };
    }
  } catch (error) {
    return {
      content: [{
        type: 'text',
        text: JSON.stringify({ success: false, error: error instanceof Error ? error.message : 'Unknown error' }, null, 2),
      }],
    };
  }
});

// Run the server
export async function runServer() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('MCP Investor Research Server running on stdio');
}
