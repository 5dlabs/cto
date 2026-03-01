/**
 * Expand Task operation - breaks down a task into subtasks.
 * Full-featured prompts with subagent-aware expansion for parallel execution.
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
import { parseJsonResponse, isValidSubtask, validateSingleConcern } from '../utils/json-parser';

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
 * Generate system prompt for subtask expansion with full subagent support.
 */
function getSystemPrompt(_subtaskCount: number, nextId: number, enableSubagents: boolean, useResearch: boolean): string {
  const researchSection = useResearch ? `

You have access to current best practices and latest technical information to provide research-backed subtask generation.` : '';

  const subagentSection = enableSubagents ? `
- subagentType: The type of specialized subagent to handle this subtask. MUST be one of:
  - "implementer": Write/implement code (default for most coding subtasks)
  - "reviewer": Review code quality, patterns, and best practices
  - "tester": Write and run tests
  - "documenter": Write documentation
  - "researcher": Research and exploration tasks
  - "debugger": Debug issues and fix bugs
- parallelizable: Boolean indicating if this subtask can run in parallel with others at the same dependency level (true for independent work, false for coordination-required tasks)` : '';

  const subagentGuidelines = enableSubagents ? `

## Subagent Optimization Guidelines

When breaking down tasks for subagent execution:
1. **Maximize parallelism**: Group independent work units that can run simultaneously
2. **Minimize dependencies**: Only add dependencies when strictly necessary
3. **Match subagent types to work**: Use implementer for coding, tester for tests, etc.
4. **Consider context isolation**: Each subagent works in isolation, so subtasks should be self-contained
5. **Plan review phases**: Include reviewer subtasks after implementation phases` : '';

  const singleConcernRule = `

## CRITICAL: Single-Concern Subtask Rule ⚠️

Each subtask MUST do exactly ONE thing. VIOLATIONS include:
- "Deploy PostgreSQL, MongoDB, Redis" → SPLIT INTO 3 subtasks!
- "Deploy Kafka and RabbitMQ" → SPLIT INTO 2 subtasks!
- "Configure namespaces, policies, and quotas" → SPLIT INTO 3 subtasks!
- Any subtask with "(X, Y, Z)" or "X and Y" for different systems

PATTERNS THAT INDICATE VIOLATION:
- Multiple operator names (CloudNative-PG, Percona, Strimzi)
- Multiple technology names in parentheses
- The word "and" connecting different systems
- Multiple CRD types in one subtask

✅ CORRECT: "Deploy PostgreSQL Cluster" (one database, one subtask)
✅ CORRECT: "Configure Network Policies" (one concern, one subtask)
❌ WRONG: "Deploy PostgreSQL and MongoDB" (two databases, needs split!)`;

  return `You are an AI assistant helping with task breakdown for software development. Break down high-level tasks into specific, actionable subtasks that can be implemented${enableSubagents ? ' in parallel by specialized subagents' : ' sequentially'}.${singleConcernRule}${researchSection}

IMPORTANT: Each subtask object must include ALL of the following fields:
- id: MUST be sequential integers starting EXACTLY from ${nextId}. First subtask id=${nextId}, second id=${nextId + 1}, etc. DO NOT use any other numbering pattern!
- title: A clear, actionable title (5-200 characters)
- description: A detailed description (minimum 10 characters)
- dependencies: An array of subtask IDs this subtask depends on (can be empty [])
- details: Implementation details (minimum 20 characters)
- status: Must be "pending" for new subtasks
- testStrategy: Testing approach (can be null)${subagentSection}

CRITICAL OUTPUT FORMAT:
- The JSON structure \`{"subtasks":[\` has already been started for you
- You must CONTINUE by outputting subtask objects directly as array elements
- Do NOT repeat the opening structure - just output the subtask objects
- No markdown formatting, no explanatory text before or after
- Do NOT explain your reasoning or summarize the subtasks${subagentGuidelines}`;
}

/**
 * Generate user prompt for subtask expansion with full context.
 */
function getUserPrompt(
  task: { id: string; title: string; description: string; details?: string; test_strategy?: string },
  subtaskCount: number,
  nextId: number,
  enableSubagents: boolean,
  expansionPrompt?: string,
  additionalContext?: string,
  complexityReasoning?: string
): string {
  const subagentRequirements = enableSubagents ? `

SUBAGENT REQUIREMENTS:
- Include subagentType for EVERY subtask (implementer, reviewer, tester, documenter, researcher, or debugger)
- Set parallelizable=true for subtasks that can run concurrently with others at the same dependency level
- Minimize dependencies to maximize parallel execution potential
- Group related implementation work so multiple implementer subagents can work simultaneously
- Include at least one reviewer subtask after implementation subtasks
- Include tester subtasks for validation work

SINGLE-CONCERN RULE:
- Each subtask must deploy/configure ONE system only
- "Deploy PostgreSQL, MongoDB, Redis" → MUST split into 3 subtasks!
- "Configure namespaces, policies, quotas" → MUST split into 3 subtasks!
- Review each subtask - if it mentions multiple technologies, SPLIT IT!` : '';

  return `Break down this task into ${subtaskCount > 0 ? `exactly ${subtaskCount}` : 'an appropriate number of'} specific subtasks${enableSubagents ? ' optimized for parallel subagent execution' : ''}:

Task ID: ${task.id}
Title: ${task.title}
Description: ${task.description}
Current details: ${task.details || 'None'}${task.test_strategy ? `
Test strategy: ${task.test_strategy}` : ''}${expansionPrompt ? `

Expansion guidance: ${expansionPrompt}` : ''}${additionalContext ? `

Additional context: ${additionalContext}` : ''}${complexityReasoning ? `

Complexity Analysis Reasoning: ${complexityReasoning}` : ''}

CRITICAL: You MUST use sequential IDs starting from ${nextId}. The first subtask MUST have id=${nextId}, the second MUST have id=${nextId + 1}, and so on. Do NOT use parent task ID in subtask numbering!${subagentRequirements}

OUTPUT: Continue the JSON array by outputting subtask objects directly. Start with the first subtask's opening brace { - do NOT output {"subtasks":[ again as that is already provided. End with ]} to close the array and object.`;
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
  const enableSubagents = payload.enable_subagents ?? true; // Default to enabled
  const useResearch = payload.use_research ?? false;

  const systemPrompt = getSystemPrompt(subtaskCount, nextId, enableSubagents, useResearch);
  const userPrompt = getUserPrompt(
    task,
    subtaskCount,
    nextId,
    enableSubagents,
    payload.expansion_prompt,
    payload.additional_context,
    payload.complexity_reasoning_context
  );

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

    const MAX_ATTEMPTS = 3;
    let lastParseError = '';
    let lastResponseText = '';
    let finalResult: ReturnType<typeof parseJsonResponse<GeneratedSubtask>> | null = null;
    let finalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

    for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
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

      // Accumulate token usage across all attempts
      finalUsage.input_tokens += usage.input_tokens;
      finalUsage.output_tokens += usage.output_tokens;
      finalUsage.total_tokens += usage.total_tokens;

      lastResponseText = responseText;

      // Handle both cases:
      // 1. Model outputs array contents directly (needs wrapping)
      // 2. Model outputs full JSON with {"subtasks":[...]} (use as-is)
      const trimmed = responseText.trim();
      let wrappedResponse: string;
      
      if (trimmed.startsWith('{"subtasks"') || trimmed.startsWith('{ "subtasks"')) {
        // Model gave us full JSON, use as-is
        wrappedResponse = trimmed;
      } else if (trimmed.startsWith('[')) {
        // Model gave us just the array, wrap it
        wrappedResponse = '{"subtasks":' + trimmed + '}';
      } else if (trimmed.startsWith('{')) {
        // Model started with first object, wrap in array
        wrappedResponse = '{"subtasks":[' + trimmed;
      } else {
        // Unexpected format, try wrapping anyway
        wrappedResponse = '{"subtasks":[' + trimmed;
      }
      
      // Parse with robust JSON parser
      const result = parseJsonResponse<GeneratedSubtask>(wrappedResponse, 'subtasks', isValidSubtask as (item: unknown) => item is GeneratedSubtask);

      if (!result.success) {
        lastParseError = result.error ?? 'Parse failed';
        if (attempt < MAX_ATTEMPTS) {
          continue; // retry
        }
        return {
          success: false,
          error: lastParseError,
          error_type: 'parse_error',
          details: lastResponseText.slice(0, 500),
        };
      }

      // Parse succeeded, capture and break
      finalResult = result;
      break;
    } // end retry loop

    if (!finalResult || !finalResult.success) {
      return {
        success: false,
        error: lastParseError || 'Parse failed',
        error_type: 'parse_error',
        details: lastResponseText.slice(0, 500),
      };
    }

    // Validate single-concern rule
    const validation = validateSingleConcern(finalResult.items);
    if (!validation.valid) {
      const violations = validation.violations.map(v => 
        `Subtask ${v.id} ("${v.title}"): ${v.reason}`
      ).join('\n');
      
      return {
        success: false,
        error: `Combined subtasks detected. Each subtask must do exactly ONE thing.\n\n${violations}\n\nPlease regenerate with SPLIT subtasks.`,
        error_type: 'validation_error',
        details: violations,
      };
    }

    return {
      success: true,
      data: { subtasks: finalResult.items },
      usage: finalUsage,
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
