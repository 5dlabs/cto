/**
 * Claude Provider - Uses official Anthropic SDK for direct API calls.
 * 
 * Provides simple generation without agentic features for debate planning.
 */

import Anthropic from '@anthropic-ai/sdk';
import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';
import { createLogger } from '../utils/logger';

const logger = createLogger('claude-provider');

/**
 * Default model for Claude provider.
 */
const DEFAULT_MODEL = 'claude-sonnet-4-20250514';

/**
 * Environment variable for Anthropic API key.
 */
const API_KEY_ENV = 'ANTHROPIC_API_KEY';

/**
 * Default timeout for API requests (5 minutes).
 */
const DEFAULT_TIMEOUT_MS = 300_000;

/**
 * Claude model provider implementation using official Anthropic SDK.
 */
export class ClaudeProvider implements ModelProvider {
  readonly name: ProviderName = 'claude';
  readonly defaultModel: string = DEFAULT_MODEL;
  
  private client: Anthropic | null = null;
  
  /**
   * Get or create the Anthropic client.
   */
  private getClient(): Anthropic | null {
    if (this.client === null) {
      const apiKey = process.env[API_KEY_ENV];
      if (!apiKey) {
        return null;
      }
      this.client = new Anthropic({
        apiKey,
        timeout: DEFAULT_TIMEOUT_MS,
      });
    }
    return this.client;
  }
  
  /**
   * Check if Anthropic API key is configured.
   */
  isAvailable(): boolean {
    return !!process.env[API_KEY_ENV];
  }
  
  /**
   * Generate text using Anthropic's Messages API.
   */
  async generate(
    prompt: string,
    systemPrompt: string,
    options?: ProviderOptions,
    model?: string
  ): Promise<ProviderResponse> {
    const startTime = Date.now();
    const actualModel = model || this.defaultModel;
    
    logger.debug('Starting generation', { model: actualModel, promptLength: prompt.length });
    
    const client = this.getClient();
    if (!client) {
      logger.error('API key not configured');
      return {
        success: false,
        error: `Anthropic API key not configured. Set ${API_KEY_ENV} environment variable.`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'claude',
        latencyMs: Date.now() - startTime,
      };
    }
    
    try {
      logger.debug('Sending request', { model: actualModel });

      const response = await client.messages.create({
        model: actualModel,
        max_tokens: options?.maxTokens || 8192,
        system: systemPrompt || undefined,
        messages: [
          { role: 'user', content: prompt }
        ],
        temperature: options?.temperature,
        stop_sequences: options?.stopSequences,
      });
      
      const latencyMs = Date.now() - startTime;
      
      logger.debug('Response received', { 
        latencyMs, 
        model: response.model,
        stopReason: response.stop_reason,
        usage: response.usage,
      });

      // Extract text from response
      const text = response.content
        .filter((block): block is Anthropic.TextBlock => block.type === 'text')
        .map((block) => block.text)
        .join('');
      
      if (!text) {
        logger.warn('Empty response from Anthropic');
        return {
          success: false,
          error: 'Anthropic returned empty response',
          usage: {
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
          },
          model: response.model || actualModel,
          provider: 'claude',
          latencyMs,
        };
      }
      
      return {
        success: true,
        text,
        usage: {
          input_tokens: response.usage.input_tokens,
          output_tokens: response.usage.output_tokens,
          total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        },
        model: response.model || actualModel,
        provider: 'claude',
        latencyMs,
      };
    } catch (e) {
      const latencyMs = Date.now() - startTime;
      
      if (e instanceof Anthropic.APIError) {
        logger.error('Anthropic API error', { 
          status: e.status, 
          message: e.message,
        });
        return {
          success: false,
          error: `Anthropic API error (${e.status}): ${e.message}`,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'claude',
          latencyMs,
        };
      }
      
      const error = e instanceof Error ? e.message : 'Unknown error';
      const isTimeout = error.includes('timeout') || error.includes('ETIMEDOUT');
      
      logger.error('Request failed', { error, isTimeout });
      
      return {
        success: false,
        error: isTimeout ? 'Anthropic API request timed out' : `Anthropic API error: ${error}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'claude',
        latencyMs,
      };
    }
  }
}

/**
 * Singleton instance of the Claude provider.
 */
export const claudeProvider = new ClaudeProvider();
