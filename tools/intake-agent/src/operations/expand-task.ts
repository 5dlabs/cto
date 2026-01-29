/**
 * Expand Task operation - breaks down a task into subtasks.
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
import { renderTemplate, EXPAND_TASK_SYSTEM, EXPAND_TASK_USER } from '../prompts/templates';
import { getClaudeCliOrThrow } from '../cli-finder';

/**
 * JSON prefill to force structured output.
 */
const JSON_PREFILL = '{"subtasks":[';

/**
 * Extract JSON continuation from response.
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

  // If starts with [ (array), return as-is
  if (content.startsWith('[')) {
    return content;
  }

  // Look for {"id": to find start of valid subtask JSON
  const idMatch = content.indexOf('{"id":');
  if (idMatch > 0) {
    return content.slice(idMatch);
  } else if (idMatch === 0) {
    return content;
  }

  // Check if starts with { and has "id" as first key
  if (content.startsWith('{')) {
    const afterBrace = content.slice(1).trim();
    if (afterBrace.startsWith('"id"')) {
      return content;
    }
  }

  // Fallback
  const firstBrace = content.indexOf('{');
  if (firstBrace >= 0) {
    return content.slice(firstBrace);
  }

  return content;
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
 * Expand a task into subtasks using Claude Agent SDK.
 */
export async function expandTask(
  payload: ExpandTaskPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<ExpandTaskData>> {
  const context = {
    subtask_count: payload.subtask_count ?? 5,
    task: payload.task,
    next_subtask_id: payload.next_subtask_id ?? 1,
    use_research: payload.use_research ?? false,
    expansion_prompt: payload.expansion_prompt,
    additional_context: payload.additional_context ?? '',
    complexity_reasoning_context: payload.complexity_reasoning_context ?? '',
    gathered_context: '',
    project_root: payload.project_root ?? '',
    enable_subagents: payload.enable_subagents ?? false,
  };

  const systemPrompt = renderTemplate(EXPAND_TASK_SYSTEM, context);
  const userPrompt = renderTemplate(EXPAND_TASK_USER, context);

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
    const jsonContent = extractJsonContinuation(responseText);
    const fullJson = `${JSON_PREFILL}${jsonContent}`;

    let parsed: { subtasks: GeneratedSubtask[] };
    try {
      parsed = JSON.parse(fullJson);
    } catch (e) {
      const parseError = e instanceof Error ? e.message : 'Unknown parse error';
      return {
        success: false,
        error: `JSON parse failed: ${parseError}`,
        error_type: 'parse_error',
        details: fullJson.slice(0, 500),
      };
    }

    if (!Array.isArray(parsed.subtasks)) {
      return {
        success: false,
        error: 'Parsed JSON does not contain subtasks array',
        error_type: 'parse_error',
      };
    }

    return {
      success: true,
      data: { subtasks: parsed.subtasks },
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
