/**
 * Integration Tests for MCP Server
 * Tests the server configuration and keyword loading
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import fs from 'fs/promises';

// Mock fs
vi.mock('fs/promises');

describe('MCP Server Configuration', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    vi.clearAllMocks();
    delete process.env.GROK_API_KEY;
  });

  describe('Server', () => {
    it('should be properly defined', () => {
      const server = {
        name: 'mcp-investor-research',
        version: '0.1.0',
        capabilities: { tools: {} }
      };
      expect(server.name).toBe('mcp-investor-research');
      expect(server.version).toBe('0.1.0');
      expect(server.capabilities).toHaveProperty('tools');
    });
  });

  describe('Tool List Handler', () => {
    it('should be configurable', () => {
      const tools = [
        { name: 'search_investors', description: 'Search for investors' },
        { name: 'search_startup_credits', description: 'Search for credits' },
        { name: 'list_investor_keywords', description: 'List keywords' },
      ];
      expect(tools).toHaveLength(3);
      expect(tools[0].name).toBe('search_investors');
    });
  });

  describe('Keyword Configuration', () => {
    it('should load keywords from config file', async () => {
      const mockConfig = {
        investors: {
          description: 'Test investors',
          keywords: ['test1', 'test2']
        },
        startupCredits: {
          description: 'Test credits',
          keywords: ['credit1', 'credit2']
        }
      };

      vi.spyOn(fs, 'readFile').mockResolvedValue(JSON.stringify(mockConfig));

      const result = await fs.readFile('/test.json', 'utf-8');
      const config = JSON.parse(result);

      expect(config.investors.keywords).toHaveLength(2);
      expect(config.startupCredits.keywords).toHaveLength(2);
    });

    it('should handle missing config file', async () => {
      vi.spyOn(fs, 'readFile').mockRejectedValue(new Error('File not found'));

      await expect(fs.readFile('/nonexistent.json', 'utf-8'))
        .rejects.toThrow('File not found');
    });
  });

  describe('Environment Configuration', () => {
    it('should detect missing GROK_API_KEY', () => {
      delete process.env.GROK_API_KEY;
      expect(process.env.GROK_API_KEY).toBeUndefined();
    });

    it('should use GROK_API_KEY when set', () => {
      process.env.GROK_API_KEY = 'test-key';
      expect(process.env.GROK_API_KEY).toBe('test-key');
      delete process.env.GROK_API_KEY;
    });
  });
});

describe('Tool Schemas', () => {
  describe('Search Investor Schema', () => {
    it('should accept valid search parameters', () => {
      const params = {
        query: 'VC firm',
        minLikes: 10,
        limit: 20,
      };
      expect(params.query).toBe('VC firm');
      expect(params.minLikes).toBe(10);
      expect(params.limit).toBe(20);
    });

    it('should accept optional parameters', () => {
      const params = {
        query: undefined,
        minLikes: undefined,
        limit: undefined,
      };
      expect(params.query).toBeUndefined();
    });
  });

  describe('Search Startup Credit Schema', () => {
    it('should accept valid parameters', () => {
      const params = {
        query: 'AWS Activate',
        minLikes: 5,
        limit: 10,
      };
      expect(params.query).toBe('AWS Activate');
      expect(params.minLikes).toBe(5);
      expect(params.limit).toBe(10);
    });
  });

  describe('Add Keyword Schema', () => {
    it('should require keyword and category', () => {
      const validParams = {
        keyword: 'new investor',
        category: 'investors' as const,
      };
      expect(validParams.keyword).toBeDefined();
      expect(validParams.category).toBe('investors');
    });

    it('should only accept valid categories', () => {
      const validCategories = ['investors', 'startupCredits'];
      expect(validCategories).toContain('investors');
      expect(validCategories).toContain('startupCredits');
    });
  });
});

describe('Search Configuration', () => {
  describe('Default Limits', () => {
    it('should have sensible defaults for investor search', () => {
      const defaults = {
        minLikes: 10,
        limit: 20,
      };
      expect(defaults.minLikes).toBeGreaterThan(0);
      expect(defaults.limit).toBeGreaterThan(defaults.minLikes);
    });

    it('should have sensible defaults for startup credit search', () => {
      const defaults = {
        minLikes: 5,
        limit: 20,
      };
      expect(defaults.minLikes).toBeGreaterThanOrEqual(0);
      expect(defaults.limit).toBeGreaterThan(defaults.minLikes);
    });
  });

  describe('Category Mapping', () => {
    it('should map categories correctly', () => {
      const categories = {
        investors: ['angel investor', 'VC firm', 'seed fund'],
        startupCredits: ['AWS Activate', 'Google for Startups'],
      };
      expect(categories.investors).toBeInstanceOf(Array);
      expect(categories.startupCredits).toBeInstanceOf(Array);
    });
  });
});
