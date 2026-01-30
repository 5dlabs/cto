/**
 * OpenAI/Codex Provider - Uses OpenAI's Chat Completions API.
 * 
 * Supports GPT-4o, GPT-4-turbo, o1-preview, and other OpenAI models.
 * 
 * API Docs: https://platform.openai.com/docs/api-reference/chat
 */

import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';

/**
 * OpenAI API base URL.
 */
const OPENAI_API_BASE = 'https://api.openai.com/v1';

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
 * OpenAI chat message format.
 */
interface OpenAIMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

/**
 * OpenAI chat completion request body.
 * Note: Newer models (GPT-4.1+, GPT-5.x, o-series) use max_completion_tokens.
 */
interface OpenAIRequest {
  model: string;
  messages: OpenAIMessage[];
  max_tokens?: number;
  max_completion_tokens?: number;
  temperature?: number;
  stop?: string[];
  response_format?: { type: 'json_object' | 'text' };
}

/**
 * OpenAI chat completion response.
 */
interface OpenAIResponse {
  id: string;
  object: 'chat.completion';
  created: number;
  model: string;
  choices: Array<{
    index: number;
    message: {
      role: 'assistant';
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
 * Error response from OpenAI API.
 */
interface ApiError {
  error: {
    message: string;
    type: string;
    code: string | null;
  };
}

/**
 * OpenAI/Codex model provider implementation.
 */
export class CodexProvider implements ModelProvider {
  readonly name: ProviderName = 'codex';
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
   * Check if OpenAI API key is configured.
   */
  isAvailable(): boolean {
    return this.getApiKey() !== null;
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
    
    const apiKey = this.getApiKey();
    if (!apiKey) {
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
      const messages: OpenAIMessage[] = [];
      
      // Add system message if provided
      if (systemPrompt) {
        messages.push({ role: 'system', content: systemPrompt });
      }
      
      // Add user message
      messages.push({ role: 'user', content: prompt });
      
      const requestBody: OpenAIRequest = {
        model: actualModel,
        messages,
      };
      
      // Add optional parameters
      // Newer models (GPT-4.1+, GPT-5.x, o-series) use max_completion_tokens
      if (options?.maxTokens) {
        const isNewerModel = /^(gpt-(4\.1|5)|o[0-9])/.test(actualModel);
        if (isNewerModel) {
          requestBody.max_completion_tokens = options.maxTokens;
        } else {
          requestBody.max_tokens = options.maxTokens;
        }
      }
      if (options?.temperature !== undefined) {
        requestBody.temperature = options.temperature;
      }
      if (options?.stopSequences) {
        requestBody.stop = options.stopSequences;
      }
      if (options?.jsonMode) {
        requestBody.response_format = { type: 'json_object' };
      }
      
      const timeout = options?.timeout || DEFAULT_TIMEOUT_MS;
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);
      
      const response = await fetch(`${OPENAI_API_BASE}/chat/completions`, {
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
        const errorData = await response.json().catch(() => ({ error: { message: response.statusText } })) as ApiError;
        return {
          success: false,
          error: `OpenAI API error (${response.status}): ${errorData.error?.message || response.statusText}`,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'codex',
          latencyMs,
        };
      }
      
      const data = await response.json() as OpenAIResponse;
      
      // Extract text from first choice
      const text = data.choices[0]?.message?.content || '';
      
      if (!text) {
        return {
          success: false,
          error: 'OpenAI returned empty response',
          usage: {
            input_tokens: data.usage.prompt_tokens,
            output_tokens: data.usage.completion_tokens,
            total_tokens: data.usage.total_tokens,
          },
          model: data.model || actualModel,
          provider: 'codex',
          latencyMs,
        };
      }
      
      return {
        success: true,
        text,
        usage: {
          input_tokens: data.usage.prompt_tokens,
          output_tokens: data.usage.completion_tokens,
          total_tokens: data.usage.total_tokens,
        },
        model: data.model || actualModel,
        provider: 'codex',
        latencyMs,
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown error';
      const isTimeout = error.includes('abort');
      
      return {
        success: false,
        error: isTimeout ? 'OpenAI API request timed out' : `OpenAI API error: ${error}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'codex',
        latencyMs: Date.now() - startTime,
      };
    }
  }
}

/**
 * Singleton instance of the OpenAI/Codex provider.
 */
export const codexProvider = new CodexProvider();
