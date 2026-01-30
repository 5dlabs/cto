/**
 * Minimax Provider - Uses Minimax's OpenAI-compatible API.
 * 
 * Minimax offers M2.1 (230B params) and M2.1-lightning (faster inference).
 * Uses OpenAI-compatible chat/completions endpoint.
 * 
 * API Docs: https://platform.minimax.io/docs/api-reference
 */

import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';

/**
 * Minimax API base URL (OpenAI-compatible endpoint).
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
 * OpenAI-compatible message format.
 */
interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

/**
 * OpenAI-compatible request body.
 */
interface ChatCompletionsRequest {
  model: string;
  messages: ChatMessage[];
  max_tokens?: number;
  temperature?: number;
  stop?: string[];
}

/**
 * OpenAI-compatible response format.
 */
interface ChatCompletionsResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: Array<{
    index: number;
    message: {
      role: string;
      content: string;
    };
    finish_reason: string;
  }>;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

/**
 * Error response from Minimax API.
 */
interface ApiError {
  error?: {
    type?: string;
    message?: string;
    code?: string;
  };
  base_resp?: {
    status_code?: number;
    status_msg?: string;
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
   * Generate text using Minimax's OpenAI-compatible API.
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
      const messages: ChatMessage[] = [];
      
      // Add system prompt if provided
      if (systemPrompt) {
        messages.push({ role: 'system', content: systemPrompt });
      }
      
      // Add user prompt
      messages.push({ role: 'user', content: prompt });
      
      const requestBody: ChatCompletionsRequest = {
        model: actualModel,
        messages,
        max_tokens: options?.maxTokens || 8192,
      };
      
      // Add optional parameters
      if (options?.temperature !== undefined) {
        requestBody.temperature = options.temperature;
      }
      if (options?.stopSequences) {
        requestBody.stop = options.stopSequences;
      }
      
      const timeout = options?.timeout || DEFAULT_TIMEOUT_MS;
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);
      
      const response = await fetch(`${MINIMAX_API_BASE}/chat/completions`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${apiKey}`,
        },
        body: JSON.stringify(requestBody),
        signal: controller.signal,
      });
      
      clearTimeout(timeoutId);
      
      const latencyMs = Date.now() - startTime;
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({})) as ApiError;
        const errorMessage = errorData.error?.message || 
                            errorData.base_resp?.status_msg || 
                            response.statusText;
        return {
          success: false,
          error: `Minimax API error (${response.status}): ${errorMessage}`,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'minimax',
          latencyMs,
        };
      }
      
      const data = await response.json() as ChatCompletionsResponse;
      
      // Extract text from response
      const text = data.choices?.[0]?.message?.content || '';
      
      if (!text) {
        return {
          success: false,
          error: 'Minimax returned empty response',
          usage: {
            input_tokens: data.usage?.prompt_tokens || 0,
            output_tokens: data.usage?.completion_tokens || 0,
            total_tokens: data.usage?.total_tokens || 0,
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
          input_tokens: data.usage?.prompt_tokens || 0,
          output_tokens: data.usage?.completion_tokens || 0,
          total_tokens: data.usage?.total_tokens || 0,
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
