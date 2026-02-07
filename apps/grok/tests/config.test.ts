/**
 * Unit Tests for Keyword Configuration
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import fs from 'fs/promises';
import * as path from 'path';

// Mock fs module
vi.mock('fs/promises');

import {
  loadKeywordsConfig,
  getDefaultInvestorKeywords,
  getDefaultStartupCreditKeywords,
  getInvestorKeywords,
  getStartupCreditKeywords,
  getAllKeywords,
  clearConfigCache,
} from '../src/lib/keywords.js';

describe('Keyword Configuration', () => {
  beforeEach(() => {
    clearConfigCache();
    vi.resetAllMocks();
  });

  describe('getDefaultInvestorKeywords', () => {
    it('should return non-empty array of investor keywords', () => {
      const keywords = getDefaultInvestorKeywords();
      
      expect(Array.isArray(keywords)).toBe(true);
      expect(keywords.length).toBeGreaterThan(0);
    });

    it('should contain common investor types', () => {
      const keywords = getDefaultInvestorKeywords().map(k => k.toLowerCase());
      
      expect(keywords).toContain('angel investor');
      expect(keywords).toContain('vc firm');
      expect(keywords).toContain('seed fund');
      expect(keywords).toContain('venture capital');
    });

    it('should return unique keywords', () => {
      const keywords = getDefaultInvestorKeywords();
      const unique = new Set(keywords);
      
      expect(keywords.length).toBe(unique.size);
    });
  });

  describe('getDefaultStartupCreditKeywords', () => {
    it('should return non-empty array of startup credit keywords', () => {
      const keywords = getDefaultStartupCreditKeywords();
      
      expect(Array.isArray(keywords)).toBe(true);
      expect(keywords.length).toBeGreaterThan(0);
    });

    it('should contain major cloud providers', () => {
      const keywords = getDefaultStartupCreditKeywords().map(k => k.toLowerCase());
      
      expect(keywords).toContain('aws activate');
      expect(keywords).toContain('google for startups');
      expect(keywords).toContain('microsoft for startups');
    });

    it('should return unique keywords', () => {
      const keywords = getDefaultStartupCreditKeywords();
      const unique = new Set(keywords);
      
      expect(keywords.length).toBe(unique.size);
    });
  });

  describe('loadKeywordsConfig', () => {
    it('should load config from keywords.json', async () => {
      const mockConfig = {
        investors: {
          description: 'Test investors',
          keywords: ['test investor 1', 'test investor 2']
        },
        startupCredits: {
          description: 'Test credits',
          keywords: ['test credit 1', 'test credit 2']
        }
      };

      vi.spyOn(fs, 'readFile').mockResolvedValue(JSON.stringify(mockConfig));

      const config = await loadKeywordsConfig('/test/path/keywords.json');

      expect(config).toEqual(mockConfig);
      expect(fs.readFile).toHaveBeenCalledWith('/test/path/keywords.json', 'utf-8');
    });

    it('should cache config after first load', async () => {
      const mockConfig = {
        investors: { description: '', keywords: ['kw1'] },
        startupCredits: { description: '', keywords: ['kw2'] }
      };

      vi.spyOn(fs, 'readFile').mockResolvedValue(JSON.stringify(mockConfig));

      await loadKeywordsConfig('/test/path.json');
      await loadKeywordsConfig('/test/path.json');
      await loadKeywordsConfig('/test/path.json');

      expect(fs.readFile).toHaveBeenCalledTimes(1);
    });

    it('should throw error for missing file', async () => {
      vi.spyOn(fs, 'readFile').mockRejectedValue(new Error('File not found'));

      await expect(loadKeywordsConfig('/nonexistent/keywords.json'))
        .rejects.toThrow('Failed to load keywords');
    });

    it('should throw error for invalid JSON', async () => {
      vi.spyOn(fs, 'readFile').mockResolvedValue('not valid json');

      await expect(loadKeywordsConfig('/invalid.json'))
        .rejects.toThrow();
    });
  });

  describe('getInvestorKeywords', () => {
    it('should return keywords from config when available', async () => {
      const mockConfig = {
        investors: {
          description: 'Test',
          keywords: ['config investor 1', 'config investor 2']
        },
        startupCredits: { description: '', keywords: [] }
      };

      vi.spyOn(fs, 'readFile').mockResolvedValue(JSON.stringify(mockConfig));

      const keywords = await getInvestorKeywords();
      
      expect(keywords).toEqual(['config investor 1', 'config investor 2']);
    });

    it('should fall back to defaults on error', async () => {
      vi.spyOn(fs, 'readFile').mockRejectedValue(new Error('Error'));

      const keywords = await getInvestorKeywords();
      const defaults = getDefaultInvestorKeywords();
      
      expect(keywords).toEqual(defaults);
    });
  });

  describe('getStartupCreditKeywords', () => {
    it('should return keywords from config when available', async () => {
      const mockConfig = {
        investors: { description: '', keywords: [] },
        startupCredits: {
          description: 'Test',
          keywords: ['config credit 1', 'config credit 2']
        }
      };

      vi.spyOn(fs, 'readFile').mockResolvedValue(JSON.stringify(mockConfig));

      const keywords = await getStartupCreditKeywords();
      
      expect(keywords).toEqual(['config credit 1', 'config credit 2']);
    });

    it('should fall back to defaults on error', async () => {
      vi.spyOn(fs, 'readFile').mockRejectedValue(new Error('Error'));

      const keywords = await getStartupCreditKeywords();
      const defaults = getDefaultStartupCreditKeywords();
      
      expect(keywords).toEqual(defaults);
    });
  });

  describe('getAllKeywords', () => {
    it('should combine investor and startup credit keywords', async () => {
      const mockConfig = {
        investors: { description: '', keywords: ['inv1', 'inv2'] },
        startupCredits: { description: '', keywords: ['cred1', 'cred2'] }
      };

      vi.spyOn(fs, 'readFile').mockResolvedValue(JSON.stringify(mockConfig));

      const all = await getAllKeywords();
      
      expect(all).toEqual(['inv1', 'inv2', 'cred1', 'cred2']);
    });
  });

  describe('clearConfigCache', () => {
    it('should force reload of config', async () => {
      const mockConfig1 = {
        investors: { description: '', keywords: ['first'] },
        startupCredits: { description: '', keywords: [] }
      };

      const mockConfig2 = {
        investors: { description: '', keywords: ['second'] },
        startupCredits: { description: '', keywords: [] }
      };

      const readFile = vi.spyOn(fs, 'readFile');
      readFile.mockResolvedValueOnce(JSON.stringify(mockConfig1));

      await loadKeywordsConfig('/test.json');
      clearConfigCache();
      readFile.mockResolvedValueOnce(JSON.stringify(mockConfig2));

      const config = await loadKeywordsConfig('/test.json');
      
      expect(config.investors.keywords).toEqual(['second']);
      expect(readFile).toHaveBeenCalledTimes(2);
    });
  });
});
