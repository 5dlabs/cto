/**
 * Expand Task operation - breaks down a task into subtasks.
 * Uses minimal prompts based on "Ralph Wiggum technique".
 */

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import type {
  ExpandTaskPayload,
  ExpandTaskData,
  GenerateOptions,
  AgentResponse,
  TokenUsage,
  GeneratedSubtask,
} from '../types';
import { getClaudeCliOrThrow } from '../cli-finder';

/**
 * JSON prefill to force structured output.
 */
const JSON_PREFILL = '{"subtasks":[';

/**
 * Extract JSON from response, handling markdown code blocks and various formats.
 */
function extractJson(text: string): string {
  let content = text.trim();

  // Handle markdown code blocks
  const jsonBlockMatch = content.match(/```json\s*([\s\S]*?)\s*```/);
  if (jsonBlockMatch?.[1]) {
    content = jsonBlockMatch[1].trim();
  } else {
    const codeBlockMatch = content.match(/```\s*([\s\S]*?)\s*```/);
    if (codeBlockMatch?.[1] && (codeBlockMatch[1].startsWith('{') || codeBlockMatch[1].startsWith('['))) {
      content = codeBlockMatch[1].trim();
    }
  }

  // If it's already a valid JSON object with subtasks, return as-is
  if (content.startsWith('{"subtasks"')) {
    return content;
  }

  // Strip echoed prefill if present and re-wrap
  if (content.startsWith(JSON_PREFILL)) {
    return content;
  }

  // If starts with [ (array), wrap it
  if (content.startsWith('[')) {
    return '{"subtasks":' + content + '}';
  }

  // Look for {"id": to find start of subtask array content
  const idMatch = content.indexOf('{"id":');
  if (idMatch >= 0) {
    return JSON_PREFILL + content.slice(idMatch);
  }

  return content;
}

/**
 * Parse and validate subtasks JSON.
 */
function parseSubtasksJson(content: string): { success: true; subtasks: GeneratedSubtask[] } | { success: false; error: string } {
  const trimmed = content.trim();

  if (!trimmed) {
    return { success: false, error: 'AI returned empty response' };
  }

  try {
    const parsed = JSON.parse(trimmed);
    
    if (parsed.subtasks && Array.isArray(parsed.subtasks)) {
      return { success: true, subtasks: parsed.subtasks };
    }
    
    if (Array.isArray(parsed)) {
      return { success: true, subtasks: parsed };
    }

    return { success: false, error: `Parsed JSON does not contain subtasks array` };
  } catch (e) {
    const parseError = e instanceof Error ? e.message : 'Unknown parse error';
    return { success: false, error: `JSON parse failed: ${parseError}` };
  }
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
 * Generate minimal system prompt for subtask expansion.
 */
function getMinimalSystemPrompt(subtaskCount: number, nextId: number): string {
  return `You are a subtask generator. Break down tasks into specific, actionable subtasks.

## Output Format
Generate ${subtaskCount} subtasks starting from ID ${nextId}. Each subtask:
{
  "id": number,
  "title": "Subtask title",
  "description": "What to do",
  "status": "pending",
  "dependencies": [subtask_ids],
  "details": "Implementation steps",
  "testStrategy": "How to verify"
}

## Rules
1. Each subtask should be completable in 1-4 hours
2. Dependencies only reference lower subtask IDs
3. Include clear implementation details
4. All string fields must be valid JSON (escape quotes and newlines)

Output ONLY the JSON, no explanations.`;
}

/**
 * Generate minimal user prompt for subtask expansion.
 */
function getMinimalUserPrompt(task: { id: number; title: string; description: string; details?: string }, subtaskCount: number, nextId: number): string {
  return `Task to expand:
- ID: ${task.id}
- Title: ${task.title}
- Description: ${task.description}
${task.details ? `- Details: ${task.details}` : ''}

Generate ${subtaskCount} subtasks starting from ID ${nextId}.`;
}

/**
 * Expand a task into subtasks using Claude Agent SDK.
 */
export async function expandTask(
  payload: ExpandTaskPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<ExpandTaskData>> {
  const subtaskCount = payload.subtask_count ?? 5;
  const nextId = payload.next_subtask_id ?? 1;
  const task = payload.task;

  const systemPrompt = getMinimalSystemPrompt(subtaskCount, nextId);
  const userPrompt = getMinimalUserPrompt(task, subtaskCount, nextId);

  try {
    const cliPath = getClaudeCliOrThrow();

    const sdkOptions: Options = {
      customSystemPrompt: systemPrompt,
      model,
      maxTurns: 1,
      allowedTools: [],
      permissionMode: 'bypassPermissions',
      pathToClaudeCodeExecutable: cliPath,
    };

    let responseText = '';
    let usage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

    const promptWithPrefill = `${userPrompt}\n\n${JSON_PREFILL}`;

    for await (const message of query({
      prompt: promptWithPrefill,
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

    // Extract and parse JSON
    const jsonContent = extractJson(responseText);
    const result = parseSubtasksJson(jsonContent);

    if (!result.success) {
      return {
        success: false,
        error: result.error,
        error_type: 'parse_error',
        details: jsonContent.slice(0, 500),
      };
    }

    return {
      success: true,
      data: { subtasks: result.subtasks },
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
