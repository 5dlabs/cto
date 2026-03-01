#!/usr/bin/env bun
/**
 * Intake Agent - Claude Agent SDK wrapper for PRD parsing and task generation.
 *
 * This binary reads JSON requests from stdin and writes JSON responses to stdout.
 * It uses the official Claude Agent SDK for AI operations with MCP support.
 *
 * Usage:
 *   echo '{"operation":"ping"}' | ./intake-agent
 *   echo '{"operation":"parse_prd","payload":{"prd_content":"..."}}' | ./intake-agent
 */

import type {
  AgentRequest,
  AgentResponse,
  AgentErrorResponse,
  ParsePrdPayload,
  ExpandTaskPayload,
  AnalyzeComplexityPayload,
  PingData,
  ErrorType,
  GeneratedTask,
} from './types';
import { validateRequest } from './types';
import { parsePrd } from './operations/parse-prd';
import { parsePrdIterative } from './operations/parse-prd-iterative';
import { expandTask } from './operations/expand-task';
import { analyzeComplexity } from './operations/analyze';
import { generate, type GeneratePayload } from './operations/generate';
import { generatePrompts, type GeneratePromptsPayload } from './operations/generate-prompts';
import { research, getResearchCapabilities, type ResearchPayload } from './operations/research';
import {
  generateWithCriticOperation,
  validateContentOperation,
  getProviderStatus,
} from './operations/generate-with-critic';
import { generateWithDebate } from './operations/generate-with-debate';
import { generateDocs } from './operations/generate-docs';
import type { GenerateWithCriticPayload, ProviderName } from './providers/types';

/**
 * Package version - read from package.json at build time.
 */
const VERSION = '1.0.0';

/**
 * Claude Agent SDK version.
 */
const SDK_VERSION = '1.0.0';

/**
 * Default model if not specified in request.
 */
const DEFAULT_MODEL = 'claude-sonnet-4-20250514';

/**
 * Create an error response.
 */
function errorResponse(error: string, errorType: ErrorType = 'unknown', details?: string): AgentErrorResponse {
  return {
    success: false,
    error,
    error_type: errorType,
    ...(details ? { details } : {}),
  };
}

/**
 * Handle ping operation for health check.
 */
function handlePing(): AgentResponse<PingData> {
  return {
    success: true,
    data: {
      status: 'ok',
      version: VERSION,
      sdk_version: SDK_VERSION,
    },
    usage: {
      input_tokens: 0,
      output_tokens: 0,
      total_tokens: 0,
    },
    model: 'none',
    provider: 'intake-agent',
  };
}

/**
 * Route request to appropriate handler.
 */
async function handleRequest(request: AgentRequest): Promise<AgentResponse<unknown>> {
  const model = request.model ?? DEFAULT_MODEL;
  const options = request.options ?? {};

  switch (request.operation) {
    case 'ping':
      return handlePing();

    case 'parse_prd': {
      const payload = request.payload as ParsePrdPayload;
      if (!payload?.prd_content) {
        return errorResponse('Missing prd_content in payload', 'validation_error');
      }
      return parsePrd(payload, model, options);
    }

    case 'parse_prd_iterative': {
      const payload = request.payload as ParsePrdPayload;
      if (!payload?.prd_content) {
        return errorResponse('Missing prd_content in payload', 'validation_error');
      }
      return parsePrdIterative(payload, model, options);
    }

    case 'expand_task': {
      const payload = request.payload as ExpandTaskPayload;
      if (!payload?.task) {
        return errorResponse('Missing task in payload', 'validation_error');
      }
      return expandTask(payload, model, options);
    }

    case 'analyze_complexity': {
      const payload = request.payload as AnalyzeComplexityPayload;
      if (!payload?.tasks || !Array.isArray(payload.tasks)) {
        return errorResponse('Missing tasks array in payload', 'validation_error');
      }
      return analyzeComplexity(payload, model, options);
    }

    case 'generate': {
      const payload = request.payload as GeneratePayload;
      if (!payload?.user_prompt) {
        return errorResponse('Missing user_prompt in payload', 'validation_error');
      }
      return generate(payload, model, options);
    }

    case 'generate_prompts': {
      const payload = request.payload as GeneratePromptsPayload;
      if (!payload?.tasks || !Array.isArray(payload.tasks)) {
        return errorResponse('Missing tasks array in payload', 'validation_error');
      }
      return generatePrompts(payload);
    }

    case 'research': {
      const payload = request.payload as ResearchPayload;
      if (!payload?.topic) {
        return errorResponse('Missing topic in payload', 'validation_error');
      }
      return research(payload, model, options);
    }

    case 'research_capabilities':
      return getResearchCapabilities();

    case 'generate_with_critic': {
      const payload = request.payload as GenerateWithCriticPayload;
      if (!payload?.user_prompt) {
        return errorResponse('Missing user_prompt in payload', 'validation_error');
      }
      return generateWithCriticOperation(payload);
    }

    case 'validate_content': {
      const payload = request.payload as {
        content: string;
        critic?: ProviderName;
        critic_model?: string;
        context?: string;
        content_type?: string;
        criteria?: string;
      };
      if (!payload?.content) {
        return errorResponse('Missing content in payload', 'validation_error');
      }
      return validateContentOperation(payload);
    }

    case 'provider_status':
      return getProviderStatus();

    case 'deliberate': {
      const { runDeliberation } = await import('./operations/deliberation');
      const payload = request.payload as import('./types').DeliberatePayload;
      if (!payload?.prd_content) {
        return errorResponse('Missing prd_content in payload', 'validation_error');
      }
      const sessionId = payload.session_id ?? crypto.randomUUID();
      return runDeliberation({ ...payload, session_id: sessionId });
    }

    case 'prd_research': {
      const { prdResearch } = await import('./operations/prd-research');
      const payload = request.payload as { prd_content: string };
      if (!payload?.prd_content) {
        return errorResponse('Missing prd_content in payload', 'validation_error');
      }
      const memos = await prdResearch(payload);
      return {
        success: true,
        data: memos,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: 'tavily',
        provider: 'tavily',
      };
    }

    case 'generate_docs': {
      const payload = request.payload as {
        tasks: GeneratedTask[];
        base_path: string;
        project_root?: string;
      };
      if (!payload?.tasks || !Array.isArray(payload.tasks)) {
        return errorResponse('Missing tasks array in payload', 'validation_error');
      }
      if (!payload?.base_path) {
        return errorResponse('Missing base_path in payload', 'validation_error');
      }
      return generateDocs(payload as unknown as Parameters<typeof generateDocs>[0]) as unknown as AgentResponse<unknown>;
    }

    case 'generate_with_debate': {
      const payload = request.payload as {
        user_prompt: string;
        system_prompt?: string;
        prefill?: string;
        context?: string;
        content_type?: string;
        config?: {
          generator?: ProviderName;
          critic?: ProviderName;
          generator_model?: string;
          critic_model?: string;
          max_refinements?: number;
          critic_threshold?: number;
        };
      };
      if (!payload?.user_prompt) {
        return errorResponse('Missing user_prompt in payload', 'validation_error');
      }
      return generateWithDebate({
        systemPrompt: payload.system_prompt || 'You are a helpful assistant.',
        userPrompt: payload.user_prompt,
        prefill: payload.prefill,
        context: payload.context,
        contentType: payload.content_type as 'tasks' | 'code' | 'docs' | 'general' | undefined,
        config: payload.config,
      });
    }

    default:
      return errorResponse(`Unknown operation: ${request.operation}`, 'validation_error');
  }
}

/**
 * Read JSON from stdin.
 * Supports both single-line JSON and multi-line JSON with EOF.
 */
async function readStdin(): Promise<string> {
  const chunks: Uint8Array[] = [];

  // Read all data from stdin
  for await (const chunk of Bun.stdin.stream()) {
    chunks.push(chunk);
  }

  // Combine chunks and decode
  const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
  const combined = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    combined.set(chunk, offset);
    offset += chunk.length;
  }

  return new TextDecoder().decode(combined);
}

/**
 * Write JSON to stdout.
 */
function writeStdout(response: AgentResponse<unknown>): void {
  const json = JSON.stringify(response);
  console.log(json);
}

/**
 * Display help message.
 */
function printHelp(): void {
  console.log(`intake-agent v${VERSION}

PRD parsing and task generation agent using Claude Agent SDK.

This binary reads JSON requests from stdin and writes JSON responses to stdout.

Usage:
  echo '{"operation":"ping"}' | intake-agent
  cat request.json | intake-agent
  intake-agent < request.json

Operations:
  ping                   Health check
  parse_prd              Parse PRD into tasks
  parse_prd_iterative    Parse PRD iteratively with streaming
  expand_task            Expand task into subtasks
  analyze_complexity     Analyze task complexity
  generate               Generate content with AI
  generate_prompts       Generate prompts for tasks
  research               Perform research on a topic
  research_capabilities  List research capabilities
  generate_with_critic   Generate with critic feedback
  validate_content       Validate content with AI
  provider_status        Get provider status
  generate_docs          Generate documentation for tasks
  generate_with_debate   Generate with debate pattern
  deliberate             Run deliberation session on PRD
  prd_research           Research PRD context via Tavily (pre-debate)

Options:
  -h, --help             Show this help message
  -V, --version          Show version

Environment:
  ANTHROPIC_API_KEY      API key for Claude (optional with OAuth)

Examples:
  # Health check
  echo '{"operation":"ping"}' | intake-agent

  # Parse a PRD
  echo '{"operation":"parse_prd","payload":{"prd_content":"..."}}' | intake-agent`);
}

/**
 * Display version.
 */
function printVersion(): void {
  console.log(`intake-agent ${VERSION}`);
}

/**
 * Main entry point.
 */
async function main(): Promise<void> {
  // Check for CLI flags first, before reading stdin
  const args = process.argv.slice(2);
  if (args.includes('-h') || args.includes('--help')) {
    printHelp();
    process.exit(0);
  }
  if (args.includes('-V') || args.includes('--version')) {
    printVersion();
    process.exit(0);
  }

  try {
    // Read request from stdin
    const input = await readStdin();
    const trimmed = input.trim();

    if (!trimmed) {
      writeStdout(errorResponse('Empty input', 'validation_error'));
      process.exit(1);
    }

    // Parse JSON
    let request: unknown;
    try {
      request = JSON.parse(trimmed);
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown parse error';
      writeStdout(errorResponse(`Invalid JSON: ${error}`, 'parse_error', trimmed.slice(0, 200)));
      process.exit(1);
    }

    // Validate request structure
    if (!validateRequest(request)) {
      writeStdout(
        errorResponse(
          'Invalid request structure. Expected { operation: "ping" | "parse_prd" | "parse_prd_iterative" | "expand_task" | "analyze_complexity" | "generate" | "generate_prompts" | "generate_docs" | "generate_with_debate" | "generate_with_critic" | "validate_content" | "research" | "research_capabilities" | "provider_status" | "deliberate" | "prd_research", payload?: {...} }',
          'validation_error'
        )
      );
      process.exit(1);
    }

    // Handle request
    const response = await handleRequest(request);
    writeStdout(response);

    // Exit with appropriate code
    process.exit(response.success ? 0 : 1);
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    const stack = e instanceof Error ? e.stack : undefined;
    writeStdout(errorResponse(`Unhandled error: ${error}`, 'unknown', stack));
    process.exit(1);
  }
}

// Run main
main();
