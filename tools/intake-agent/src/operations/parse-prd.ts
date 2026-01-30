/**
 * Parse PRD operation - generates tasks from a Product Requirements Document.
 * 
 * Uses minimal prompts based on the "Ralph Wiggum technique" - simpler prompts
 * (~40 lines) often outperform verbose prompts (~1,500 words).
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

/**
 * Maximum retry attempts for PRD parsing when AI returns invalid responses.
 */
const MAX_RETRIES = 3;

/**
 * JSON prefill to force structured output.
 */
const JSON_PREFILL = '{"tasks":[';

/**
 * Extract JSON from response, handling markdown code blocks and various formats.
 */
function extractJson(text: string): string {
  let content = text.trim();

  // Handle markdown code blocks (```json ... ``` or ``` ... ```)
  const jsonBlockMatch = content.match(/```json\s*([\s\S]*?)\s*```/);
  if (jsonBlockMatch?.[1]) {
    content = jsonBlockMatch[1].trim();
  } else {
    const codeBlockMatch = content.match(/```\s*([\s\S]*?)\s*```/);
    if (codeBlockMatch?.[1] && (codeBlockMatch[1].startsWith('{') || codeBlockMatch[1].startsWith('['))) {
      content = codeBlockMatch[1].trim();
    }
  }

  // Strip echoed prefill if present
  if (content.startsWith(JSON_PREFILL)) {
    content = content.slice(JSON_PREFILL.length).trim();
    // Need to wrap it back
    content = JSON_PREFILL + content;
  }

  // If it's already a valid JSON object with tasks, return as-is
  if (content.startsWith('{"tasks"')) {
    return content;
  }

  // If starts with {" but not tasks, look for {"tasks" or {"id"
  if (content.startsWith('{')) {
    // Check if it's the full object
    if (content.includes('"tasks"')) {
      return content;
    }
    // Check if it starts with task objects (continuation format)
    const afterBrace = content.slice(1).trim();
    if (afterBrace.startsWith('"id"')) {
      return JSON_PREFILL + content;
    }
  }

  // If starts with [, it's an array - wrap it
  if (content.startsWith('[')) {
    return '{"tasks":' + content + '}';
  }

  // Look for {"id": to find start of task array content
  const idMatch = content.indexOf('{"id":');
  if (idMatch >= 0) {
    return JSON_PREFILL + content.slice(idMatch);
  }

  return content;
}

/**
 * Validate and parse JSON response into tasks.
 */
function parseTasksJson(content: string): { success: true; tasks: GeneratedTask[] } | { success: false; error: string } {
  const trimmed = content.trim();

  if (!trimmed) {
    return { success: false, error: 'AI returned empty response - no task JSON generated' };
  }

  // Try to parse as JSON
  try {
    const parsed = JSON.parse(trimmed);
    
    // Check if it has tasks array
    if (parsed.tasks && Array.isArray(parsed.tasks)) {
      return { success: true, tasks: parsed.tasks };
    }
    
    // If it's an array directly, wrap it
    if (Array.isArray(parsed)) {
      return { success: true, tasks: parsed };
    }

    return { 
      success: false, 
      error: `Parsed JSON does not contain tasks array. Got keys: ${Object.keys(parsed).join(', ')}` 
    };
  } catch (e) {
    const parseError = e instanceof Error ? e.message : 'Unknown parse error';
    return { 
      success: false, 
      error: `JSON parse failed: ${parseError}. Content preview: ${trimmed.slice(0, 300)}...` 
    };
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

      // Extract JSON from response (handles code blocks, prefill, etc.)
      const jsonContent = extractJson(responseText);
      
      // Parse and validate
      const result = parseTasksJson(jsonContent);

      if (!result.success) {
        lastError = result.error;
        console.error(`Attempt ${attempt + 1}/${MAX_RETRIES} failed: ${result.error}`);
        continue;
      }

      // Success!
      return {
        success: true,
        data: { tasks: result.tasks },
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
