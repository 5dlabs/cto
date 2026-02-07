/**
 * Unit Tests for Grok Client
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock child_process to prevent 1Password CLI calls in tests
vi.mock('child_process', () => ({
  execSync: vi.fn(() => { throw new Error('op not available in tests'); }),
}));

// Mock fetch globally
global.fetch = vi.fn();

import {
  searchX,
  searchInvestors,
  searchStartupCredits,
} from '../src/lib/client.js';

describe('Grok Client', () => {
  const mockFetch = global.fetch as unknown as vi.Mock;

  beforeEach(() => {
    vi.resetAllMocks();
    vi.clearAllMocks();
    delete process.env.GROK_API_KEY;
    delete process.env.GROK_API_URL;
    delete process.env.GROK_MODEL;
  });

  describe('searchX', () => {
    it('should throw error when API key is missing', async () => {
      await expect(searchX({ query: 'test' }))
        .rejects.toThrow('GROK_API_KEY not set');
    });

    it('should make API call with API key', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ output: [] })
      });

      await searchX({ query: 'test' });

      expect(mockFetch).toHaveBeenCalledTimes(1);
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('api.x.ai'),
        expect.objectContaining({
          headers: expect.objectContaining({
            'Authorization': 'Bearer test-api-key',
          }),
        })
      );
    });

    it('should handle empty response', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ output: [] })
      });

      const results = await searchX({ query: 'test' });
      expect(results).toEqual([]);
    });

    it('should handle API error response', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        text: () => Promise.resolve('Unauthorized')
      });

      await expect(searchX({ query: 'test' }))
        .rejects.toThrow('Grok API error: 401');
    });

    it('should parse results with ID and likes', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 1234567890\n50 likes\nTest tweet content\n@testuser'
            }]
          }]
        })
      });

      const results = await searchX({ query: 'test' });

      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('1234567890');
      expect(results[0].likes).toBe(50);
    });

    it('should extract author from result', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 100\n25 likes\nGreat tweet\n@founduser'
            }]
          }]
        })
      });

      const results = await searchX({ query: 'test' });

      expect(results[0].author).toBe('@founduser');
    });

    it('should handle fetch error', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(searchX({ query: 'test' }))
        .rejects.toThrow('Network error');
    });
  });

  describe('searchInvestors', () => {
    it('should return empty array without API key', async () => {
      const results = await searchInvestors(['test'], {}, 10);
      expect(results).toEqual([]);
    });

    it('should return investor results with inv_ prefix', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 100\n50 likes\nAngel investor post'
            }]
          }]
        })
      });

      const results = await searchInvestors(['angel investor'], {}, 10);

      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('inv_100');
      expect(results[0].term).toBe('angel investor');
    });

    it('should include timestamp in results', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 200\n30 likes\nVC post'
            }]
          }]
        })
      });

      const results = await searchInvestors(['vc'], {}, 5);

      expect(results[0].timestamp).toBeDefined();
      expect(new Date(results[0].timestamp!)).toBeInstanceOf(Date);
    });
  });

  describe('searchStartupCredits', () => {
    it('should return empty array without API key', async () => {
      const results = await searchStartupCredits(['test'], {}, 5);
      expect(results).toEqual([]);
    });

    it('should return startup credit results with cred_ prefix', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 456\n30 likes\nAWS Activate credits'
            }]
          }]
        })
      });

      const results = await searchStartupCredits(['AWS Activate'], {}, 5);

      expect(results).toHaveLength(1);
      expect(results[0].id).toBe('cred_456');
      expect(results[0].term).toBe('AWS Activate');
    });

    it('should include URL in results', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 789\n20 likes\nCloud credits'
            }]
          }]
        })
      });

      const results = await searchStartupCredits(['cloud'], {}, 0);

      expect(results[0].url).toContain('789');
    });

    it('should sort results by likes descending', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          output: [{
            content: [{
              type: 'output_text',
              text: 'ID: 500\n100 likes\nPopular startup program'
            }]
          }]
        })
      });

      const results = await searchStartupCredits(['startup'], {}, 0);

      expect(results[0].likes).toBe(100);
    });
  });

  describe('Error Handling', () => {
    it('should handle JSON parse error', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.reject(new Error('Invalid JSON'))
      });

      await expect(searchX({ query: 'test' }))
        .rejects.toThrow('Invalid JSON');
    });

    it('should handle non-200 response', async () => {
      process.env.GROK_API_KEY = 'test-api-key';

      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 429,
        text: () => Promise.resolve('Rate limited')
      });

      await expect(searchX({ query: 'test' }))
        .rejects.toThrow('Grok API error: 429');
    });
  });

  describe('Configuration', () => {
    it('should use default API URL', async () => {
      process.env.GROK_API_KEY = 'test-api-key';
      delete process.env.GROK_API_URL;

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ output: [] })
      });

      await searchX({ query: 'test' });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('api.x.ai'),
        expect.any(Object)
      );
    });

    it('should use default model', async () => {
      process.env.GROK_API_KEY = 'test-api-key';
      delete process.env.GROK_MODEL;

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ output: [] })
      });

      await searchX({ query: 'test' });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          body: expect.stringContaining('grok-4-1-fast-reasoning'),
        })
      );
    });
  });
});
