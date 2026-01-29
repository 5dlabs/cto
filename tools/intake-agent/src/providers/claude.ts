/**
 * Claude Provider - Wraps the official Claude Agent SDK.
 * 
 * Uses @anthropic-ai/claude-code for generation with full MCP support.
 */

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import type { ModelProvider, ProviderOptions, ProviderResponse, ProviderName } from './types';
import { getClaudeCliOrThrow, isClaudeCliAvailable } from '../cli-finder';

/**
 * Default model for Claude provider.
 */
const DEFAULT_MODEL = 'claude-sonnet-4-20250514';

/**
 * Extract text content from Claude SDK assistant message.
 */
function extractAssistantText(message: SDKAssistantMessage): string {
  const content = message.message.content;
  if (!Array.isArray(content)) {
    return '';
  }
  
  return content
    .filter((block): block is { type: 'text'; text: string } => block.type === 'text')
    .map((block) => block.text)
    .join('');
}

/**
 * Claude model provider implementation.
 */
export class ClaudeProvider implements ModelProvider {
  readonly name: ProviderName = 'claude';
  readonly defaultModel: string = DEFAULT_MODEL;
  
  private cliPath: string | null = null;
  
  /**
   * Check if Claude CLI is available.
   */
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
  
  /**
   * Generate text using Claude Agent SDK.
   */
  async generate(
    prompt: string,
    systemPrompt: string,
    options?: ProviderOptions,
    model?: string
  ): Promise<ProviderResponse> {
    const startTime = Date.now();
    const actualModel = model || this.defaultModel;
    
    try {
      // Ensure CLI is available
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
      
      // Apply additional options
      if (options?.maxTokens) {
        // Note: Claude SDK uses different option naming
        // We pass it through customSystemPrompt or let it use defaults
      }
      
      let responseText = '';
      let inputTokens = 0;
      let outputTokens = 0;
      
      for await (const message of query({
        prompt,
        options: sdkOptions,
      })) {
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
        usage: {
          input_tokens: inputTokens,
          output_tokens: outputTokens,
          total_tokens: inputTokens + outputTokens,
        },
        model: actualModel,
        provider: 'claude',
        latencyMs,
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown error';
      return {
        success: false,
        error: `Claude API error: ${error}`,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: actualModel,
        provider: 'claude',
        latencyMs: Date.now() - startTime,
      };
    }
  }
}

/**
 * Singleton instance of the Claude provider.
 */
export const claudeProvider = new ClaudeProvider();
