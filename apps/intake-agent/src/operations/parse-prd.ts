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
import { createLogger } from '../utils/logger';
import { withTimeout, Timeouts } from '../utils/timeout';

const logger = createLogger('parse-prd');

// Debug helper to log prompts
function logPrompts(system: string, user: string): void {
  if (!logger.isDebug()) return;
  logger.debug('System prompt', { length: system.length, preview: system.slice(0, 200) + '...' });
  logger.debug('User prompt', { length: user.length, preview: user.slice(0, 200) + '...' });
}

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
 * Generate system prompt for task generation with full features.
 * Includes decision points, acceptance criteria, and agent hints.
 */
function getSystemPrompt(numTasks: number, nextId: number, research: boolean = false): string {
  const researchSection = research ? `

### Research Mode Active
Before breaking down the PRD into tasks:
1. Research latest technologies, libraries, frameworks appropriate for this project
2. Identify technical challenges, security concerns, scalability issues
3. Consider current industry standards and trends
4. Include specific library versions and implementation guidance` : '';

  return `## Role
You are a Senior Technical PM and Software Architect breaking down requirements into actionable tasks.
${researchSection}

## Task
Generate ${numTasks} tasks starting from ID ${nextId}.

## Output Schema
{
  "id": number,
  "title": "Action (AgentName - Stack)",
  "description": "What and why",
  "status": "pending",
  "dependencies": [task_ids],
  "priority": "high" | "medium" | "low",
  "details": "Implementation steps (escaped JSON string)",
  "testStrategy": "Acceptance criteria - how to validate this task is complete",
  "decisionPoints": [
    {
      "id": "d1",
      "category": "architecture" | "error-handling" | "data-model" | "api-design" | "ux-behavior" | "performance" | "security",
      "description": "What needs to be decided",
      "options": ["option1", "option2"],
      "requiresApproval": boolean,
      "constraintType": "hard" | "soft" | "open" | "escalation"
    }
  ]
}

## Agent Mapping
- Infrastructure: (Bolt - Kubernetes)
- Rust backend: (Rex - Rust/Axum)
- Go backend: (Grizz - Go/gRPC)
- Node.js backend: (Nova - Bun/Elysia)
- React frontend: (Blaze - React/Next.js)
- Mobile: (Tap - Expo)
- Desktop: (Spark - Electron)

## Decision Point Categories
- architecture: System design, patterns, service boundaries
- error-handling: Error strategies, retry logic, fallbacks
- data-model: Schema design, relationships, migrations
- api-design: Endpoints, request/response formats
- ux-behavior: User interactions, edge cases
- performance: Caching, optimization, scaling
- security: Auth, encryption, access control

## Constraint Types
- hard: PRD requirement (must be this way)
- soft: Prefer this but adjustable
- open: Agent chooses best approach
- escalation: Human must decide

## Rules
1. Task 1 MUST be infrastructure setup (Bolt) if databases/storage needed
2. Then backend services, then frontend apps
3. Dependencies only reference lower IDs
4. testStrategy MUST define clear acceptance criteria
5. Include decisionPoints for ambiguous areas
6. All string fields must be valid JSON (escape quotes/newlines)

Output ONLY the JSON array contents, no markdown, no explanations.`;
}

/**
 * Generate user prompt for task generation with full features.
 */
function getUserPrompt(prdContent: string, numTasks: number, nextId: number, research: boolean = false): string {
  const researchReminder = research ? `

Research current best practices before generating. Apply findings to details and testStrategy.` : '';

  return `PRD:
---
${prdContent}
---
${researchReminder}

Generate ${numTasks} tasks starting from ID ${nextId}.

REQUIREMENTS:
- Include agent hints in titles: "(AgentName - Stack)"
- testStrategy must define acceptance criteria
- Include decisionPoints for ambiguous requirements
- Set constraintType for each decision point

Output ONLY the JSON array contents.`;
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
  const timeout = Timeouts.parsePrd(numTasks);

  logger.info('Starting PRD parsing', { 
    numTasks, 
    nextId, 
    model, 
    prdLength: prdContent.length,
    timeoutMs: timeout 
  });

  const research = payload.research ?? false;
  const systemPrompt = getSystemPrompt(numTasks, nextId, research);
  const userPrompt = getUserPrompt(prdContent, numTasks, nextId, research);
  
  // Log prompts in debug mode
  logPrompts(systemPrompt, userPrompt);

  let lastError: string | undefined;
  let totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

  // Find Claude CLI executable once
  const cliPath = getClaudeCliOrThrow();
  logger.debug('Using Claude CLI', { path: cliPath });
  
  for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
    logger.info(`Attempt ${attempt + 1}/${MAX_RETRIES}`);
    const attemptStart = Date.now();

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
      let chunkCount = 0;

      // Query Claude with the prompt (with timeout)
      const queryPromise = (async () => {
        for await (const message of query({
          prompt: userPrompt,
          options: sdkOptions,
        })) {
          // Handle assistant messages
          if (message.type === 'assistant') {
            const text = extractAssistantText(message);
            responseText += text;
            chunkCount++;
            if (chunkCount % 10 === 0) {
              logger.debug(`Received ${chunkCount} chunks`, { textLength: responseText.length });
            }
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
        return { responseText, usage };
      })();

      const result = await withTimeout(queryPromise, timeout, `PRD parsing (${numTasks} tasks)`);
      responseText = result.responseText;
      usage = result.usage;

      const attemptElapsed = Date.now() - attemptStart;
      logger.info('Query completed', { 
        elapsed_ms: attemptElapsed, 
        responseLength: responseText.length,
        chunks: chunkCount,
        usage 
      });

      // Accumulate total usage
      totalUsage.input_tokens += usage.input_tokens;
      totalUsage.output_tokens += usage.output_tokens;
      totalUsage.total_tokens += usage.total_tokens;

      // Parse and validate with robust JSON parser
      logger.debug('Parsing JSON response');
      const parseResult = parseJsonResponse<GeneratedTask>(responseText, 'tasks', isValidTask as (item: unknown) => item is GeneratedTask);

      if (!parseResult.success) {
        lastError = parseResult.error;
        logger.warn(`Parse failed`, { error: parseResult.error, preview: responseText.slice(0, 200) });
        continue;
      }

      logger.info('PRD parsing successful', { 
        taskCount: parseResult.items.length,
        totalUsage 
      });

      // Success!
      return {
        success: true,
        data: { tasks: parseResult.items },
        usage: totalUsage,
        model,
        provider: 'claude-agent-sdk',
      };
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown error';
      lastError = `API error: ${error}`;
      logger.error(`Attempt failed`, { attempt: attempt + 1, error });
    }
  }

  // All retries exhausted
  logger.error('All retries exhausted', { lastError });
  return {
    success: false,
    error: lastError ?? 'Failed to generate tasks after multiple attempts',
    error_type: 'api_error',
  };
}
