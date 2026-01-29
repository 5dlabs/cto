/**
 * Generic text generation operation.
 * Provides a simple interface for generating text with Claude via the Agent SDK.
 */

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import type {
  GenerateOptions,
  AgentResponse,
  TokenUsage,
} from '../types';
import { getClaudeCliOrThrow } from '../cli-finder';

/**
 * Payload for generic text generation.
 */
export interface GeneratePayload {
  /** System prompt for the conversation */
  system_prompt: string;
  /** User prompt (main input) */
  user_prompt: string;
  /** Optional prefill to start the assistant's response */
  prefill?: string;
}

/**
 * Response data for generate operation.
 */
export interface GenerateData {
  /** Generated text */
  text: string;
}

/**
 * Extract text from assistant message content.
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
 * Generic text generation using Claude Agent SDK.
 */
export async function generate(
  payload: GeneratePayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<GenerateData>> {
  try {
    // Find Claude CLI executable
    const cliPath = getClaudeCliOrThrow();

    const sdkOptions: Options = {
      customSystemPrompt: payload.system_prompt,
      model,
      maxTurns: 1,
      allowedTools: [],
      permissionMode: 'bypassPermissions',
      pathToClaudeCodeExecutable: cliPath,
    };

    let responseText = '';
    let usage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

    // Build prompt with optional prefill
    const prompt = payload.prefill 
      ? `${payload.user_prompt}\n\n${payload.prefill}`
      : payload.user_prompt;

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
          usage.input_tokens = resultMsg.usage.input_tokens;
          usage.output_tokens = resultMsg.usage.output_tokens;
          usage.total_tokens = usage.input_tokens + usage.output_tokens;
        }
      }
    }

    if (!responseText) {
      return {
        success: false,
        error: 'Claude Agent SDK returned empty response',
        error_type: 'api_error',
      };
    }

    return {
      success: true,
      data: { text: responseText },
      usage,
      model,
      provider: 'claude-agent-sdk',
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    return {
      success: false,
      error: `API error: ${error}`,
      error_type: 'api_error',
    };
  }
}
