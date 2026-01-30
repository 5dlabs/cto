/**
 * OpenAI/Codex Provider - Uses official OpenAI SDK.
 * 
 * Supports GPT-4o, GPT-4.1, GPT-5.x, o1, o3, o4-mini and other OpenAI models.
 * Uses official SDK for automatic handling of API differences.
 * 
 * API Docs: https://platform.openai.com/docs/api-reference/chat
 */

import OpenAI from 'openai';
import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';
import { createLogger } from '../utils/logger';

const logger = createLogger('codex-provider');

/**
 * Default model for OpenAI provider.
 * Updated to latest GPT-5.2 (Dec 2025).
 */
const DEFAULT_MODEL = 'gpt-5.2';

/**
 * Environment variable for OpenAI API key.
 */
const API_KEY_ENV = 'OPENAI_API_KEY';

/**
 * Default timeout for API requests (5 minutes).
 */
const DEFAULT_TIMEOUT_MS = 300_000;

/**
 * OpenAI/Codex model provider implementation using official SDK.
 */
export class CodexProvider implements ModelProvider {
  readonly name: ProviderName = 'codex';
  readonly defaultModel: string = DEFAULT_MODEL;
  
  private client: OpenAI | null = null;
  
  /**
   * Get or create the OpenAI client.
   */
  private getClient(): OpenAI | null {
    if (this.client === null) {
      const apiKey = process.env[API_KEY_ENV];
      if (!apiKey) {
        return null;
      }
      this.client = new OpenAI({
        apiKey,
        timeout: DEFAULT_TIMEOUT_MS,
      });
    }
    return this.client;
  }
  
  /**
   * Check if OpenAI API key is configured.
   */
  isAvailable(): boolean {
    return !!process.env[API_KEY_ENV];
  }
  
  /**
   * Check if model is a newer model that uses max_completion_tokens.
   */
  private isNewerModel(model: string): boolean {
    return /^(gpt-(4\.1|5)|o[0-9])/.test(model);
  }
  
  /**
   * Generate text using OpenAI's Chat Completions API.
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
        error: `OpenAI API key not configured. Set ${API_KEY_ENV} environment variable.`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'codex',
        latencyMs: Date.now() - startTime,
      };
    }
    
    try {
      const messages: OpenAI.Chat.ChatCompletionMessageParam[] = [];
      
      // Add system message if provided
      if (systemPrompt) {
        messages.push({ role: 'system', content: systemPrompt });
      }
      
      // Add user message
      messages.push({ role: 'user', content: prompt });

      logger.debug('Sending request', { 
        model: actualModel, 
        messageCount: messages.length,
        isNewerModel: this.isNewerModel(actualModel),
      });

      // Build request params
      const requestParams: OpenAI.Chat.ChatCompletionCreateParamsNonStreaming = {
        model: actualModel,
        messages,
      };

      // Handle max tokens based on model version
      if (options?.maxTokens) {
        if (this.isNewerModel(actualModel)) {
          // Newer models use max_completion_tokens
          (requestParams as any).max_completion_tokens = options.maxTokens;
          logger.debug('Using max_completion_tokens', { value: options.maxTokens });
        } else {
          // Older models use max_tokens
          requestParams.max_tokens = options.maxTokens;
          logger.debug('Using max_tokens', { value: options.maxTokens });
        }
      }

      // Add optional parameters
      if (options?.temperature !== undefined) {
        requestParams.temperature = options.temperature;
      }
      if (options?.stopSequences) {
        requestParams.stop = options.stopSequences;
      }
      if (options?.jsonMode) {
        requestParams.response_format = { type: 'json_object' };
      }

      const response = await client.chat.completions.create(requestParams);
      
      const latencyMs = Date.now() - startTime;
      
      logger.debug('Response received', { 
        latencyMs, 
        model: response.model,
        finishReason: response.choices[0]?.finish_reason,
        usage: response.usage,
      });

      // Extract text from response
      const text = response.choices[0]?.message?.content || '';
      
      if (!text) {
        logger.warn('Empty response from OpenAI');
        return {
          success: false,
          error: 'OpenAI returned empty response',
          usage: {
            input_tokens: response.usage?.prompt_tokens || 0,
            output_tokens: response.usage?.completion_tokens || 0,
            total_tokens: response.usage?.total_tokens || 0,
          },
          model: response.model || actualModel,
          provider: 'codex',
          latencyMs,
        };
      }
      
      return {
        success: true,
        text,
        usage: {
          input_tokens: response.usage?.prompt_tokens || 0,
          output_tokens: response.usage?.completion_tokens || 0,
          total_tokens: response.usage?.total_tokens || 0,
        },
        model: response.model || actualModel,
        provider: 'codex',
        latencyMs,
      };
    } catch (e) {
      const latencyMs = Date.now() - startTime;
      
      if (e instanceof OpenAI.APIError) {
        logger.error('OpenAI API error', { 
          status: e.status, 
          code: e.code,
          message: e.message,
          type: e.type,
        });
        return {
          success: false,
          error: `OpenAI API error (${e.status}): ${e.message}`,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'codex',
          latencyMs,
        };
      }
      
      const error = e instanceof Error ? e.message : 'Unknown error';
      const isTimeout = error.includes('timeout') || error.includes('ETIMEDOUT');
      
      logger.error('Request failed', { error, isTimeout });
      
      return {
        success: false,
        error: isTimeout ? 'OpenAI API request timed out' : `OpenAI API error: ${error}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'codex',
        latencyMs,
      };
    }
  }
}

/**
 * Singleton instance of the OpenAI provider.
 */
export const codexProvider = new CodexProvider();
