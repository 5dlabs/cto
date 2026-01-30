/**
 * Analyze Complexity operation - analyzes task complexity and recommends subtask counts.
 * Uses minimal prompts based on "Ralph Wiggum technique".
 * Includes robust JSON parsing with fallback.
 */

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import type {
  AnalyzeComplexityPayload,
  AnalyzeComplexityData,
  GenerateOptions,
  AgentResponse,
  TokenUsage,
  TaskComplexityAnalysis,
} from '../types';
import { getClaudeCliOrThrow } from '../cli-finder';
import { parseJsonResponse, isValidComplexityAnalysis } from '../utils/json-parser';

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
 * Generate minimal system prompt for complexity analysis.
 */
function getMinimalSystemPrompt(): string {
  return `You are a task complexity analyzer. Evaluate tasks and recommend subtask counts.

## Output Format
For each task, output:
{
  "taskId": number,
  "taskTitle": "task title",
  "complexityScore": 1-10,
  "recommendedSubtasks": number (0 if no expansion needed),
  "expansionPrompt": "guidance for subtask generation",
  "reasoning": "brief explanation"
}

## Scoring Guide
- 1-3: Simple, single-file changes
- 4-6: Moderate, multiple files/components
- 7-10: Complex, architectural changes or integrations

Output ONLY the JSON, no explanations.`;
}

/**
 * Generate minimal user prompt for complexity analysis.
 */
function getMinimalUserPrompt(tasks: Array<{ id: number; title: string; description: string; details?: string }>): string {
  const taskList = tasks.map(t => `- ID ${t.id}: ${t.title}`).join('\n');
  return `Analyze these tasks:
${taskList}

Tasks data:
${JSON.stringify(tasks, null, 2)}`;
}

/**
 * Analyze task complexity using Claude Agent SDK.
 */
export async function analyzeComplexity(
  payload: AnalyzeComplexityPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<AnalyzeComplexityData>> {
  const tasks = payload.tasks;

  const systemPrompt = getMinimalSystemPrompt();
  const userPrompt = getMinimalUserPrompt(tasks);

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
    const result = parseJsonResponse<TaskComplexityAnalysis>(responseText, 'complexityAnalysis', isValidComplexityAnalysis);

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
      data: { complexity_analysis: result.items },
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
