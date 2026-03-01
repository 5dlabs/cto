/**
 * Anthropic API Provider - Direct API access with API key.
 * 
 * Uses Anthropic's REST API directly instead of Claude Code SDK.
 * Requires ANTHROPIC_API_KEY environment variable.
 */

import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';

/**
 * Anthropic API base URL.
 */
const ANTHROPIC_API_BASE = 'https://api.anthropic.com/v1';

/**
 * Default model for Anthropic provider.
 */
const DEFAULT_MODEL = 'claude-sonnet-4-20250514';

/**
 * Environment variable for Anthropic API key.
 */
const API_KEY_ENV = 'ANTHROPIC_API_KEY';

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
 * Anthropic API response.
 */
interface AnthropicResponse {
  id: string;
  type: string;
  role: string;
  content: Array<{
    type: string;
    text: string;
  }>;
  model: string;
  stop_reason: string;
  stop_sequence: string | null;
  usage: {
    input_tokens: number;
    output_tokens: number;
  };
}

/**
 * Anthropic API Provider implementation.
 */
export class AnthropicProvider implements ModelProvider {
  readonly name: ProviderName = 'claude';
  readonly defaultModel: string = DEFAULT_MODEL;
  
  /**
   * Check if API key is available.
   */
  isAvailable(): boolean {
    return !!process.env[API_KEY_ENV];
  }
  
  /**
   * Get the API key from environment.
   */
  private getApiKey(): string {
    const apiKey = process.env[API_KEY_ENV];
    if (!apiKey) {
      throw new Error(`${API_KEY_ENV} environment variable not set`);
    }
    return apiKey;
  }
  
  /**
   * Generate text using Anthropic API.
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
    
    try {
      const messages: AnthropicMessage[] = [];
      
      if (systemPrompt) {
        messages.push({ role: 'user', content: `${systemPrompt}\n\n${prompt}` });
      } else {
        messages.push({ role: 'user', content: prompt });
      }
      
      const requestBody: AnthropicRequest = {
        model: actualModel,
        max_tokens: options?.maxTokens || 4096,
        messages,
        temperature: options?.temperature ?? 0.7,
        stop_sequences: options?.stopSequences,
      };
      
      // Remove undefined fields
      if (!systemPrompt) delete (requestBody as { system?: string }).system;
      
      const response = await fetch(`${ANTHROPIC_API_BASE}/messages`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': apiKey,
          'anthropic-version': '2023-06-01',
          'anthropic-dangerous-direct-browser-access': 'true',
        },
        body: JSON.stringify(requestBody),
      });
      
      const latencyMs = Date.now() - startTime;
      
      if (!response.ok) {
        const errorText = await response.text();
        let errorMessage = `Anthropic API error: ${response.status}`;
        
        try {
          const errorJson = JSON.parse(errorText);
          errorMessage = errorJson.error?.message || errorMessage;
        } catch {
          errorMessage = errorText || errorMessage;
        }
        
        return {
          success: false,
          error: errorMessage,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'claude',
          latencyMs,
        };
      }
      
      const data = await response.json() as AnthropicResponse;
      
      const responseText = data.content
        .filter(block => block.type === 'text')
        .map(block => block.text)
        .join('');
      
      if (!responseText) {
        return {
          success: false,
          error: 'Anthropic returned empty response',
          usage: {
            input_tokens: data.usage.input_tokens,
            output_tokens: data.usage.output_tokens,
            total_tokens: data.usage.input_tokens + data.usage.output_tokens,
          },
          model: actualModel,
          provider: 'claude',
          latencyMs,
        };
      }
      
      return {
        success: true,
        text: responseText,
        usage: {
          input_tokens: data.usage.input_tokens,
          output_tokens: data.usage.output_tokens,
          total_tokens: data.usage.input_tokens + data.usage.output_tokens,
        },
        model: actualModel,
        provider: 'claude',
        latencyMs,
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown error';
      return {
        success: false,
        error: `Anthropic API error: ${error}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'claude',
        latencyMs: Date.now() - startTime,
      };
    }
  }
}
