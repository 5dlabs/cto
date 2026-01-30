/**
 * Parse PRD operation - generates tasks from a Product Requirements Document.
 * 
 * Uses minimal prompts based on the "Ralph Wiggum technique" - simpler prompts
 * (~40 lines) often outperform verbose prompts (~1,500 words).
 * 
 * Includes robust JSON parsing with streaming support and fallback.
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
import { getClaudeCliOrThrow } from '../cli-finder';
import { parseJsonResponse, isValidTask } from '../utils/json-parser';

/**
 * Maximum retry attempts for PRD parsing when AI returns invalid responses.
 */
const MAX_RETRIES = 3;

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
 * Generate minimal system prompt for task generation.
 * Based on "Ralph Wiggum technique" - simpler prompts work better.
 */
function getMinimalSystemPrompt(numTasks: number, nextId: number): string {
  return `You are a task generator. Given a PRD, output development tasks as JSON.

## Output Format
Generate ${numTasks} tasks starting from ID ${nextId}. Each task:
{
  "id": number,
  "title": "Action (Agent - Stack)",
  "description": "Brief description",
  "status": "pending",
  "dependencies": [task_ids],
  "priority": "high" | "medium" | "low",
  "details": "Implementation steps as escaped string",
  "testStrategy": "How to test"
}

## Agent Mapping
- Infrastructure: (Bolt - Kubernetes)
- Rust backend: (Rex - Rust/Axum)
- Go backend: (Grizz - Go/gRPC)
- Node.js backend: (Nova - Bun/Elysia)
- React frontend: (Blaze - React/Next.js)
- Mobile: (Tap - Expo)
- Desktop: (Spark - Electron)

## Rules
1. Task 1 must be infrastructure setup
2. Then backend services, then frontend apps
3. Dependencies only reference lower IDs
4. All string fields must be valid JSON (escape quotes and newlines)

Output ONLY the JSON array contents, no markdown, no explanations.`;
}

/**
 * Generate minimal user prompt for task generation.
 */
function getMinimalUserPrompt(prdContent: string, numTasks: number, nextId: number): string {
  return `PRD:
---
${prdContent}
---

Generate ${numTasks} tasks starting from ID ${nextId}.`;
}

/**
 * Parse PRD into tasks using Claude Agent SDK.
 */
export async function parsePrd(
  payload: ParsePrdPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<ParsePrdData>> {
  const numTasks = payload.num_tasks ?? 10;
  const nextId = payload.next_id ?? 1;
  const prdContent = payload.prd_content;

  const systemPrompt = getMinimalSystemPrompt(numTasks, nextId);
  const userPrompt = getMinimalUserPrompt(prdContent, numTasks, nextId);

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

      // Query Claude with the prompt
      for await (const message of query({
        prompt: userPrompt,
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

      // Parse and validate with robust JSON parser
      const result = parseJsonResponse<GeneratedTask>(responseText, 'tasks', isValidTask);

      if (!result.success) {
        lastError = result.error;
        console.error(`Attempt ${attempt + 1}/${MAX_RETRIES} failed: ${result.error}`);
        continue;
      }

      // Success!
      return {
        success: true,
        data: { tasks: result.items },
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
