#!/usr/bin/env bun
/**
 * Intake Agent - PRD research orchestrator.
 *
 * This binary reads JSON requests from stdin and writes JSON responses to stdout.
 *
 * Remaining operations (everything else is now handled by Lobster workflows):
 *   - ping           — health check
 *   - prd_research    — multi-source research APIs (Exa/Perplexity/Hermes/Tavily/Firecrawl)
 *   - design_intake   — design input normalization + frontend detection + optional Stitch generation
 *
 * Deliberation is now a full Lobster workflow (deliberation.lobster.yaml).
 *
 * Usage:
 *   echo '{"operation":"ping"}' | ./intake-agent
 *   echo '{"operation":"prd_research","payload":{"prd_content":"..."}}' | ./intake-agent
 */

import type {
  AgentRequest,
  AgentResponse,
  AgentErrorResponse,
  PingData,
  ErrorType,
} from './types';
import { validateRequest } from './types';

// Note: deliberation.ts is no longer imported — deliberation is now a
// Lobster-native workflow (intake/workflows/deliberation.lobster.yaml).

/**
 * Package version - read from package.json at build time.
 */
const VERSION = '1.0.0';

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
  switch (request.operation) {
    case 'ping':
      return handlePing();

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
        model: 'multi-source',
        provider: 'exa+perplexity+hermes+tavily+firecrawl',
      };
    }

    case 'design_intake': {
      const { designIntake } = await import('./operations/design-intake');
      const payload = request.payload as {
        prd_content: string;
        design_prompt?: string;
        design_artifacts_path?: string;
        design_urls?: string;
        design_mode?: 'ingest_only' | 'ingest_plus_stitch';
        output_dir?: string;
        project_name?: string;
      };
      if (!payload?.prd_content) {
        return errorResponse('Missing prd_content in payload', 'validation_error');
      }

      const designContext = await designIntake(payload);
      return {
        success: true,
        data: designContext,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: 'detector+stitch',
        provider: '@google/stitch-sdk',
      };
    }

    case 'design_variants': {
      const { generateDesignVariants } = await import('./operations/design-intake');
      const payload = request.payload as {
        candidates_path: string;
        output_dir?: string;
        variant_count?: number;
        creative_range?: 'REFINE' | 'EXPLORE' | 'REIMAGINE';
      };
      if (!payload?.candidates_path) {
        return errorResponse('Missing candidates_path in payload', 'validation_error');
      }

      const result = await generateDesignVariants(payload);
      return {
        success: true,
        data: result,
        usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
        model: 'stitch-variants',
        provider: '@google/stitch-sdk',
      };
    }

    default:
      return errorResponse(`Unknown operation: ${request.operation}`, 'validation_error');
  }
}

/**
 * Read JSON from stdin.
 */
async function readStdin(): Promise<string> {
  const chunks: Uint8Array[] = [];

  for await (const chunk of Bun.stdin.stream()) {
    chunks.push(chunk);
  }

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

PRD research orchestrator.

This binary reads JSON requests from stdin and writes JSON responses to stdout.
Deliberation is now a full Lobster workflow (deliberation.lobster.yaml).

Usage:
  echo '{"operation":"ping"}' | intake-agent
  cat request.json | intake-agent

Operations:
  ping                   Health check
  prd_research           Research PRD context via Exa/Perplexity/Tavily/Firecrawl
  design_intake          Normalize design inputs + optional Stitch generation
  design_variants        Generate design variants from existing Stitch candidates

Options:
  -h, --help             Show this help message
  -V, --version          Show version

Examples:
  echo '{"operation":"ping"}' | intake-agent
  echo '{"operation":"prd_research","payload":{"prd_content":"..."}}' | intake-agent
  echo '{"operation":"design_intake","payload":{"prd_content":"...","design_urls":"https://example.com"}}' | intake-agent
  echo '{"operation":"design_variants","payload":{"candidates_path":".intake/design/stitch/candidates.json"}}' | intake-agent`);
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
    const input = await readStdin();
    const trimmed = input.trim();

    if (!trimmed) {
      writeStdout(errorResponse('Empty input', 'validation_error'));
      process.exit(1);
    }

    let request: unknown;
    try {
      request = JSON.parse(trimmed);
    } catch (e) {
      const error = e instanceof Error ? e.message : 'Unknown parse error';
      writeStdout(errorResponse(`Invalid JSON: ${error}`, 'parse_error', trimmed.slice(0, 200)));
      process.exit(1);
    }

    if (!validateRequest(request)) {
      writeStdout(
        errorResponse(
          'Invalid request structure. Expected { operation: "ping" | "prd_research" | "design_intake", payload?: {...} }',
          'validation_error'
        )
      );
      process.exit(1);
    }

    const response = await handleRequest(request);
    writeStdout(response);
    process.exit(response.success ? 0 : 1);
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    const stack = e instanceof Error ? e.stack : undefined;
    writeStdout(errorResponse(`Unhandled error: ${error}`, 'unknown', stack));
    process.exit(1);
  }
}

main();
