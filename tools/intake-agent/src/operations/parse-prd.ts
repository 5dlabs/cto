/**
 * Parse PRD operation - generates tasks from a Product Requirements Document.
 */

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import type {
  ParsePrdPayload,
  ParsePrdData,
  GenerateOptions,
  AgentResponse,
  TokenUsage,
  GeneratedTask,
} from '../types';
import { renderTemplate, PARSE_PRD_SYSTEM, PARSE_PRD_USER } from '../prompts/templates';
import { getClaudeCliOrThrow } from '../cli-finder';

/**
 * Maximum retry attempts for PRD parsing when AI returns invalid responses.
 */
const MAX_RETRIES = 3;

/**
 * JSON prefill to force structured output.
 */
const JSON_PREFILL = '{"tasks":[';

/**
 * Extract JSON continuation from response, handling various edge cases.
 */
function extractJsonContinuation(text: string): string {
  let content = text.trim();

  // Strip echoed prefill if present
  if (content.startsWith(JSON_PREFILL)) {
    content = content.slice(JSON_PREFILL.length).trim();
  }

  // Handle markdown code blocks
  const jsonBlockMatch = content.match(/```json\s*([\s\S]*?)\s*```/);
  if (jsonBlockMatch?.[1]) {
    return jsonBlockMatch[1].trim();
  }

  const codeBlockMatch = content.match(/```\s*([\s\S]*?)\s*```/);
  if (codeBlockMatch?.[1] && (codeBlockMatch[1].startsWith('{') || codeBlockMatch[1].startsWith('['))) {
    return codeBlockMatch[1].trim();
  }

  // If starts with [, it's an array - return as-is
  if (content.startsWith('[')) {
    return content;
  }

  // Look for {"id": to find start of valid task JSON
  const idMatch = content.indexOf('{"id":');
  if (idMatch > 0) {
    return content.slice(idMatch);
  } else if (idMatch === 0) {
    return content;
  }

  // Check if content starts with { but has wrong structure
  if (content.startsWith('{')) {
    const afterBrace = content.slice(1).trim();
    if (afterBrace.startsWith('"id"')) {
      return content;
    }
    // Look for {"id": in the content
    const nestedIdMatch = content.indexOf('{"id":');
    if (nestedIdMatch > 0) {
      return content.slice(nestedIdMatch);
    }
  }

  // Fallback: find first {
  const firstBrace = content.indexOf('{');
  if (firstBrace >= 0) {
    return content.slice(firstBrace);
  }

  return content;
}

/**
 * Validate that extracted JSON is valid task content.
 */
function validateJsonContinuation(content: string): { valid: boolean; error?: string } {
  const trimmed = content.trim();

  if (!trimmed) {
    return { valid: false, error: 'AI returned empty response - no task JSON generated' };
  }

  const firstChar = trimmed[0];

  if (firstChar === '{') {
    const afterBrace = trimmed.slice(1).trim();
    if (!afterBrace.startsWith('"id"')) {
      return {
        valid: false,
        error: `AI response does not contain valid task objects. Expected 'id' as first field. Got: ${trimmed.slice(0, 200)}...`,
      };
    }
    return { valid: true };
  }

  if (firstChar === ']') {
    if (trimmed !== ']}') {
      return { valid: false, error: `Incomplete JSON structure. Expected ']}', got: ${trimmed.slice(0, 50)}...` };
    }
    return { valid: true };
  }

  return {
    valid: false,
    error: `AI returned prose instead of JSON. First 200 chars: ${trimmed.slice(0, 200)}...`,
  };
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
 * Parse PRD into tasks using Claude Agent SDK.
 */
export async function parsePrd(
  payload: ParsePrdPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<ParsePrdData>> {
  const context = {
    num_tasks: payload.num_tasks ?? 10,
    next_id: payload.next_id ?? 1,
    research: payload.research ?? false,
    prd_content: payload.prd_content,
    prd_path: payload.prd_path ?? '',
    default_task_priority: payload.default_task_priority ?? 'medium',
    project_root: payload.project_root ?? '',
  };

  const systemPrompt = renderTemplate(PARSE_PRD_SYSTEM, context);
  const userPrompt = renderTemplate(PARSE_PRD_USER, context);

  let lastError: string | undefined;
  let totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

  // Find Claude CLI executable once
  const cliPath = getClaudeCliOrThrow();

  for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
    try {
      // Configure Claude Agent SDK options
      const sdkOptions: Options = {
        customSystemPrompt: systemPrompt,
        model,
        maxTurns: 1, // Single turn for JSON generation
        allowedTools: [], // No tools for pure JSON output
        permissionMode: 'bypassPermissions',
        pathToClaudeCodeExecutable: cliPath,
      };

      let responseText = '';
      let usage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

      // Use prefill technique by including it in the prompt
      const promptWithPrefill = `${userPrompt}\n\n${JSON_PREFILL}`;

      // Query Claude with the prompt
      for await (const message of query({
        prompt: promptWithPrefill,
        options: sdkOptions,
      })) {
        // Handle assistant messages
        if (message.type === 'assistant') {
          responseText += extractAssistantText(message);
        }
        
        // Extract usage from result message
        if (message.type === 'result') {
          const resultMsg = message as SDKResultMessage;
          if ('usage' in resultMsg) {
            usage.input_tokens = resultMsg.usage.input_tokens;
            usage.output_tokens = resultMsg.usage.output_tokens;
            usage.total_tokens = usage.input_tokens + usage.output_tokens;
          }
        }
      }

      // Accumulate total usage
      totalUsage.input_tokens += usage.input_tokens;
      totalUsage.output_tokens += usage.output_tokens;
      totalUsage.total_tokens += usage.total_tokens;

      // Extract and validate JSON
      const jsonContent = extractJsonContinuation(responseText);
      const validation = validateJsonContinuation(jsonContent);

      if (!validation.valid) {
        lastError = validation.error;
        console.error(`Attempt ${attempt + 1}/${MAX_RETRIES} failed: ${validation.error}`);
        continue;
      }

      // Reconstruct full JSON and parse
      const fullJson = `${JSON_PREFILL}${jsonContent}`;

      let parsed: { tasks: GeneratedTask[] };
      try {
        parsed = JSON.parse(fullJson);
      } catch (e) {
        const parseError = e instanceof Error ? e.message : 'Unknown parse error';
        lastError = `JSON parse failed: ${parseError}. Content: ${fullJson.slice(0, 500)}...`;
        console.error(`Attempt ${attempt + 1}/${MAX_RETRIES} failed: ${lastError}`);
        continue;
      }

      // Validate tasks array
      if (!Array.isArray(parsed.tasks)) {
        lastError = 'Parsed JSON does not contain tasks array';
        continue;
      }

      // Success!
      return {
        success: true,
        data: { tasks: parsed.tasks },
        usage: totalUsage,
        model,
        provider: 'claude-agent-sdk',
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown error';
      lastError = `API error: ${error}`;
      console.error(`Attempt ${attempt + 1}/${MAX_RETRIES} failed with API error: ${error}`);
    }
  }

  // All retries exhausted
  return {
    success: false,
    error: lastError ?? 'Failed to generate tasks after multiple attempts',
    error_type: 'api_error',
  };
}
