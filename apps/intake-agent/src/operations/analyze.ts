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
 * Generate system prompt for complexity analysis.
 */
function getSystemPrompt(): string {
  return `You are a task complexity analyzer. Evaluate tasks and recommend subtask counts for parallel subagent execution.

## Output Schema
For each task, provide:
{
  "taskId": number,
  "taskTitle": "task title",
  "complexityScore": 1-10,
  "recommendedSubtasks": number (0 if no expansion needed),
  "expansionPrompt": "detailed guidance for subtask generation",
  "reasoning": "explanation of complexity factors"
}

## Scoring Guide
- 1-3: Simple, single-file changes, isolated scope
- 4-6: Moderate, multiple files/components, some integration
- 7-10: Complex, architectural changes, multiple services, significant integration

## Expansion Guidance
For tasks scoring 5+, the expansionPrompt should provide:
- Key areas to break down
- Suggested parallel work streams
- Critical dependencies to consider
- Subagent types needed (implementer, tester, reviewer)

CRITICAL OUTPUT FORMAT:
- The JSON structure \`{"complexityAnalysis":[\` has already been started for you
- You must CONTINUE by outputting analysis objects directly as array elements
- Do NOT repeat the opening structure - just output the analysis objects
- No markdown formatting, no explanatory text before or after`;
}

/**
 * Generate user prompt for complexity analysis.
 */
function getUserPrompt(tasks: Array<{ id: string; title: string; description: string; details?: string }>, threshold: number): string {
  const taskList = tasks.map(t => `- ID ${t.id}: ${t.title}`).join('\n');
  return `Analyze these tasks for complexity:
${taskList}

Tasks data:
${JSON.stringify(tasks, null, 2)}

Threshold: ${threshold} (recommend subtasks for tasks scoring >= ${threshold})

OUTPUT: Continue the JSON array by outputting analysis objects directly. Start with the first analysis object's opening brace { - do NOT output {"complexityAnalysis":[ again as that is already provided. End with ]} to close the array and object.`;
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
  const threshold = payload.threshold ?? 5;

  const systemPrompt = getSystemPrompt();
  const userPrompt = getUserPrompt(tasks, threshold);

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

    // Prepend the JSON structure that the prompt tells the model is "already provided"
    // Handle both cases:
    // 1. Model outputs array contents directly (needs wrapping)
    // 2. Model outputs full JSON (use as-is)
    const trimmed = responseText.trim();
    let wrappedResponse: string;
    
    if (trimmed.startsWith('{"complexityAnalysis"') || trimmed.startsWith('{ "complexityAnalysis"')) {
      wrappedResponse = trimmed;
    } else if (trimmed.startsWith('[')) {
      wrappedResponse = '{"complexityAnalysis":' + trimmed + '}';
    } else if (trimmed.startsWith('{')) {
      wrappedResponse = '{"complexityAnalysis":[' + trimmed;
    } else {
      wrappedResponse = '{"complexityAnalysis":[' + trimmed;
    }
    
    // Parse with robust JSON parser
    const result = parseJsonResponse<TaskComplexityAnalysis>(wrappedResponse, 'complexityAnalysis', isValidComplexityAnalysis);

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
