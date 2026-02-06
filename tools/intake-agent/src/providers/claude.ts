/**
 * Claude Provider - Supports both Anthropic API (API key) and Claude Code SDK (OAuth).
 * 
 * When ANTHROPIC_API_KEY is set, uses the Anthropic REST API directly.
 * Otherwise falls back to @anthropic-ai/claude-code SDK which uses OAuth.
 */

import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';

// ============================================================================
// Anthropic API Implementation (when API key is set)
// ============================================================================

const ANTHROPIC_API_BASE = 'https://api.anthropic.com/v1';
const API_KEY_ENV = 'ANTHROPIC_API_KEY';

/**
 * Anthropic API provider implementation.
 */
class AnthropicApiProvider implements ModelProvider {
  readonly name: ProviderName = 'claude';
  readonly defaultModel: string = 'claude-sonnet-4-20250514';
  
  isAvailable(): boolean {
    return !!process.env[API_KEY_ENV];
  }
  
  private getApiKey(): string {
    const apiKey = process.env[API_KEY_ENV];
    if (!apiKey) {
      throw new Error(`${API_KEY_ENV} environment variable not set`);
    }
    return apiKey;
  }
  
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
      const messages = systemPrompt
        ? [{ role: 'user', content: `${systemPrompt}\n\n${prompt}` }]
        : [{ role: 'user', content: prompt }];
      
      const requestBody: Record<string, unknown> = {
        model: actualModel,
        max_tokens: options?.maxTokens || 4096,
        messages,
        temperature: options?.temperature ?? 0.7,
      };
      
      if (options?.stopSequences) {
        requestBody.stop_sequences = options.stopSequences;
      }
      
      const response = await fetch(`${ANTHROPIC_API_BASE}/messages`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': apiKey,
          'anthropic-version': '2023-06-01',
        },
        body: JSON.stringify(requestBody),
      });
      
      const latencyMs = Date.now() - startTime;
      
      if (!response.ok) {
        const errorText = await response.text();
        return {
          success: false,
          error: `Anthropic API error: ${response.status} - ${errorText.slice(0, 200)}`,
          usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
          model: actualModel,
          provider: 'claude',
          latencyMs,
        };
      }
      
      const data = await response.json() as {
        content: Array<{ type: string; text: string }>;
        usage: { input_tokens: number; output_tokens: number };
      };
      
      const responseText = data.content
        .filter((b: { type: string }) => b.type === 'text')
        .map((b: { text: string }) => b.text)
        .join('');
      
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
      return {
        success: false,
        error: `Anthropic API error: ${e instanceof Error ? e.message : 'unknown'}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'claude',
        latencyMs: Date.now() - startTime,
      };
    }
  }
}

// ============================================================================
// Claude Code SDK Implementation (fallback when no API key)
// ============================================================================

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import { getClaudeCliOrThrow, isClaudeCliAvailable } from '../cli-finder';

const SDK_DEFAULT_MODEL = 'claude-sonnet-4-20250514';

function extractAssistantText(message: SDKAssistantMessage): string {
  const content = message.message.content;
  if (!Array.isArray(content)) return '';
  return content
    .filter((block): block is { type: 'text'; text: string } => block.type === 'text')
    .map((block) => block.text)
    .join('');
}

class ClaudeSdkProvider implements ModelProvider {
  readonly name: ProviderName = 'claude';
  readonly defaultModel: string = SDK_DEFAULT_MODEL;
  
  private cliPath: string | null = null;
  
  isAvailable(): boolean {
    try {
      if (this.cliPath === null) {
        this.cliPath = isClaudeCliAvailable() ? getClaudeCliOrThrow() : null;
      }
      return this.cliPath !== null;
    } catch {
      return false;
    }
  }
  
  async generate(
    prompt: string,
    systemPrompt: string,
    options?: ProviderOptions,
    model?: string
  ): Promise<ProviderResponse> {
    const startTime = Date.now();
    const actualModel = model || this.defaultModel;
    
    try {
      const cliPath = this.cliPath || getClaudeCliOrThrow();
      this.cliPath = cliPath;
      
      const sdkOptions: Options = {
        customSystemPrompt: systemPrompt,
        model: actualModel,
        maxTurns: 1,
        allowedTools: [],
        permissionMode: 'bypassPermissions',
        pathToClaudeCodeExecutable: cliPath,
      };
      
      let responseText = '';
      let inputTokens = 0;
      let outputTokens = 0;
      
      for await (const message of query({ prompt, options: sdkOptions })) {
        if (message.type === 'assistant') {
          responseText += extractAssistantText(message);
        }
        if (message.type === 'result') {
          const resultMsg = message as SDKResultMessage;
          if ('usage' in resultMsg) {
            inputTokens = resultMsg.usage.input_tokens;
            outputTokens = resultMsg.usage.output_tokens;
          }
        }
      }
      
      const latencyMs = Date.now() - startTime;
      
      if (!responseText) {
        return {
          success: false,
          error: 'Claude returned empty response',
          usage: { input_tokens: inputTokens, output_tokens: outputTokens, total_tokens: inputTokens + outputTokens },
          model: actualModel,
          provider: 'claude',
          latencyMs,
        };
      }
      
      return {
        success: true,
        text: responseText,
        usage: { input_tokens: inputTokens, output_tokens: outputTokens, total_tokens: inputTokens + outputTokens },
        model: actualModel,
        provider: 'claude',
        latencyMs,
      };
    } catch (e) {
      return {
        success: false,
        error: `Claude SDK error: ${e instanceof Error ? e.message : 'unknown'}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'claude',
        latencyMs: Date.now() - startTime,
      };
    }
  }
}

// ============================================================================
// Factory - chooses between API key and SDK based on environment
// ============================================================================

/**
 * Create the Claude provider.
 * Uses Anthropic API if ANTHROPIC_API_KEY is set, otherwise uses Claude Code SDK.
 */
export function createClaudeProvider(): ModelProvider {
  if (process.env[API_KEY_ENV]) {
    console.error('[Claude] Using Anthropic API (ANTHROPIC_API_KEY set)');
    return new AnthropicApiProvider();
  }
  console.error('[Claude] Using Claude Code SDK (OAuth)');
  return new ClaudeSdkProvider();
}

/**
 * @deprecated Use createClaudeProvider() instead
 */
export class ClaudeProvider extends AnthropicApiProvider {}

// NOTE: Removed eagerly initialized claudeProvider export to avoid side-effect logging.
// Use createClaudeProvider() or get the provider from the registry in ./index.ts instead.
