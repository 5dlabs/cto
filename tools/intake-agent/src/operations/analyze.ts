/**
 * Analyze Complexity operation - analyzes task complexity and recommends subtask counts.
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
import { renderTemplate, ANALYZE_COMPLEXITY_SYSTEM, ANALYZE_COMPLEXITY_USER } from '../prompts/templates';
import { getClaudeCliOrThrow } from '../cli-finder';

/**
 * JSON prefill to force structured output.
 */
const JSON_PREFILL = '{"complexityAnalysis":[';

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

  // Look for valid JSON start
  if (content.startsWith('[') || content.startsWith('{')) {
    return content;
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
 * Analyze task complexity using Claude Agent SDK.
 */
export async function analyzeComplexity(
  payload: AnalyzeComplexityPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<AnalyzeComplexityData>> {
  const context = {
    tasks: payload.tasks,
    gathered_context: '',
    threshold: payload.threshold ?? 5,
    use_research: payload.use_research ?? false,
    project_root: payload.project_root ?? '',
  };

  const systemPrompt = renderTemplate(ANALYZE_COMPLEXITY_SYSTEM, context);
  const userPrompt = renderTemplate(ANALYZE_COMPLEXITY_USER, context);

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

    let parsed: { complexityAnalysis: TaskComplexityAnalysis[] };
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

    if (!Array.isArray(parsed.complexityAnalysis)) {
      return {
        success: false,
        error: 'Parsed JSON does not contain complexityAnalysis array',
        error_type: 'parse_error',
      };
    }

    return {
      success: true,
      data: { complexity_analysis: parsed.complexityAnalysis },
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
