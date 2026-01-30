/**
 * Analyze Complexity operation - analyzes task complexity and recommends subtask counts.
 * Uses minimal prompts based on "Ralph Wiggum technique".
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

/**
 * JSON prefill to force structured output.
 */
const JSON_PREFILL = '{"complexityAnalysis":[';

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
 * Extract JSON from response, handling markdown code blocks.
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

  // If already has complexityAnalysis key, return as-is
  if (content.startsWith('{"complexityAnalysis"')) {
    return content;
  }

  // Strip echoed prefill and re-wrap
  if (content.startsWith(JSON_PREFILL)) {
    return content;
  }

  // If starts with [ (array), wrap it
  if (content.startsWith('[')) {
    return '{"complexityAnalysis":' + content + '}';
  }

  // Look for {"taskId" to find start
  const taskIdMatch = content.indexOf('{"taskId"');
  if (taskIdMatch >= 0) {
    return JSON_PREFILL + content.slice(taskIdMatch);
  }

  return content;
}

/**
 * Parse and validate complexity analysis JSON.
 */
function parseAnalysisJson(content: string): { success: true; analysis: TaskComplexityAnalysis[] } | { success: false; error: string } {
  const trimmed = content.trim();

  if (!trimmed) {
    return { success: false, error: 'AI returned empty response' };
  }

  try {
    const parsed = JSON.parse(trimmed);
    
    if (parsed.complexityAnalysis && Array.isArray(parsed.complexityAnalysis)) {
      return { success: true, analysis: parsed.complexityAnalysis };
    }
    
    if (Array.isArray(parsed)) {
      return { success: true, analysis: parsed };
    }

    return { success: false, error: 'Parsed JSON does not contain complexityAnalysis array' };
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
    const result = parseAnalysisJson(jsonContent);

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
      data: { complexity_analysis: result.analysis },
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
