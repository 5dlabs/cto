/**
 * Minimax Provider - Uses Minimax's Anthropic-compatible API.
 * 
 * Minimax offers M2.1 (230B params) and M2.1-lightning (faster inference)
 * with full Anthropic SDK compatibility.
 * 
 * API Docs: https://platform.minimax.io/docs/api-reference/text-anthropic-api
 */

import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';

/**
 * Minimax API base URL (Anthropic-compatible endpoint).
 */
const MINIMAX_API_BASE = 'https://api.minimax.io/v1';

/**
 * Default model for Minimax provider.
 * M2.1-lightning offers faster inference (~100 tps vs ~60 tps).
 */
const DEFAULT_MODEL = 'MiniMax-M2.1-lightning';

/**
 * Environment variable for Minimax API key.
 */
const API_KEY_ENV = 'MINIMAX_API_KEY';

/**
 * Default timeout for API requests (5 minutes).
 */
const DEFAULT_TIMEOUT_MS = 300_000;

/**
 * Anthropic-compatible message format.
 */
interface AnthropicMessage {
  role: 'user' | 'assistant';
  content: string;
}

/**
 * Anthropic-compatible request body.
 */
interface AnthropicRequest {
  model: string;
  max_tokens: number;
  system?: string;
  messages: AnthropicMessage[];
  temperature?: number;
  stop_sequences?: string[];
}

/**
 * Anthropic-compatible response format.
 */
interface AnthropicResponse {
  id: string;
  type: 'message';
  role: 'assistant';
  content: Array<{ type: 'text'; text: string }>;
  model: string;
  stop_reason: string | null;
  stop_sequence: string | null;
  usage: {
    input_tokens: number;
    output_tokens: number;
  };
}

/**
 * Error response from Minimax API.
 */
interface ApiError {
  error: {
    type: string;
    message: string;
  };
}

/**
 * Minimax model provider implementation.
 */
export class MinimaxProvider implements ModelProvider {
  readonly name: ProviderName = 'minimax';
  readonly defaultModel: string = DEFAULT_MODEL;
  
  private apiKey: string | null = null;
  
  /**
   * Get the API key from environment.
   */
  private getApiKey(): string | null {
    if (this.apiKey === null) {
      this.apiKey = process.env[API_KEY_ENV] || null;
    }
    return this.apiKey;
  }
  
  /**
   * Check if Minimax API key is configured.
   */
  isAvailable(): boolean {
    return this.getApiKey() !== null;
  }
  
  /**
   * Generate text using Minimax's Anthropic-compatible API.
   */
  async generate(
    prompt: string,
    systemPrompt: string,
    options?: ProviderOptions,
    model?: string
  ): Promise<ProviderResponse> {
    const startTime = Date.now();
    const actualModel = model || this.defaultModel;
    
    const apiKey = this.getApiKey();
    if (!apiKey) {
      return {
        success: false,
        error: `Minimax API key not configured. Set ${API_KEY_ENV} environment variable.`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'minimax',
        latencyMs: Date.now() - startTime,
      };
    }
    
    try {
      const requestBody: AnthropicRequest = {
        model: actualModel,
        max_tokens: options?.maxTokens || 8192,
        messages: [{ role: 'user', content: prompt }],
      };
      
      // Add system prompt if provided
      if (systemPrompt) {
        requestBody.system = systemPrompt;
      }
      
      // Add optional parameters
      if (options?.temperature !== undefined) {
        requestBody.temperature = options.temperature;
      }
      if (options?.stopSequences) {
        requestBody.stop_sequences = options.stopSequences;
      }
      
      const timeout = options?.timeout || DEFAULT_TIMEOUT_MS;
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);
      
      const response = await fetch(`${MINIMAX_API_BASE}/messages`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': apiKey,
          'anthropic-version': '2023-06-01',
        },
        body: JSON.stringify(requestBody),
        signal: controller.signal,
      });
      
      clearTimeout(timeoutId);
      
      const latencyMs = Date.now() - startTime;
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: { message: response.statusText } })) as ApiError;
        return {
          success: false,
          error: `Minimax API error (${response.status}): ${errorData.error?.message || response.statusText}`,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'minimax',
          latencyMs,
        };
      }
      
      const data = await response.json() as AnthropicResponse;
      
      // Extract text from response
      const text = data.content
        .filter((block) => block.type === 'text')
        .map((block) => block.text)
        .join('');
      
      if (!text) {
        return {
          success: false,
          error: 'Minimax returned empty response',
          usage: {
            input_tokens: data.usage.input_tokens,
            output_tokens: data.usage.output_tokens,
            total_tokens: data.usage.input_tokens + data.usage.output_tokens,
          },
          model: data.model || actualModel,
          provider: 'minimax',
          latencyMs,
        };
      }
      
      return {
        success: true,
        text,
        usage: {
          input_tokens: data.usage.input_tokens,
          output_tokens: data.usage.output_tokens,
          total_tokens: data.usage.input_tokens + data.usage.output_tokens,
        },
        model: data.model || actualModel,
        provider: 'minimax',
        latencyMs,
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown error';
      const isTimeout = error.includes('abort');
      
      return {
        success: false,
        error: isTimeout ? 'Minimax API request timed out' : `Minimax API error: ${error}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'minimax',
        latencyMs: Date.now() - startTime,
      };
    }
  }
}

/**
 * Singleton instance of the Minimax provider.
 */
export const minimaxProvider = new MinimaxProvider();
