/**
 * Expand Task operation - breaks down a task into subtasks.
 * Uses minimal prompts based on "Ralph Wiggum technique".
 * Includes robust JSON parsing with fallback.
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
import { parseJsonResponse, isValidSubtask } from '../utils/json-parser';

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
  "testStrategy": "How to verify",
  "subagentType": "implementer" | "reviewer" | "tester" | "researcher" | "documenter",
  "parallelizable": boolean
}

## Subagent Types (agent hints)
- implementer: Writing code, creating features
- reviewer: Code review, architecture review
- tester: Writing tests, QA validation
- researcher: Research, spikes, investigations
- documenter: Documentation, comments, READMEs

## Rules
1. Each subtask should be completable in 1-4 hours
2. Dependencies only reference lower subtask IDs
3. Include clear implementation details
4. Assign appropriate subagent_type based on work type
5. Mark parallelizable=true if no dependencies on same-level subtasks
6. All string fields must be valid JSON (escape quotes and newlines)

Output ONLY the JSON array, no explanations.`;
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

    for await (const message of query({
      prompt: userPrompt,
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

    // Parse with robust JSON parser
    const result = parseJsonResponse<GeneratedSubtask>(responseText, 'subtasks', isValidSubtask);

    if (!result.success) {
      return {
        success: false,
        error: result.error,
        error_type: 'parse_error',
        details: responseText.slice(0, 500),
      };
    }

    return {
      success: true,
      data: { subtasks: result.items },
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
