/**
 * Research operation - gathers context using MCP tools before task generation.
 * 
 * This operation uses MCP servers (Firecrawl, OctoCode, Context7) to research
 * technologies, best practices, and code patterns relevant to a PRD.
 */

import { query, type Options, type SDKResultMessage, type SDKAssistantMessage } from '@anthropic-ai/claude-code';
import type {
  GenerateOptions,
  AgentResponse,
  TokenUsage,
} from '../types';
import { getResearchMcpServers, hasResearchCapability, listAvailableMcpServers } from '../mcp/config';
import { getClaudeCliOrThrow } from '../cli-finder';

/**
 * Payload for research operation.
 */
export interface ResearchPayload {
  /** The PRD content or topic to research */
  topic: string;
  /** Specific areas to focus research on */
  focus_areas?: string[];
  /** Maximum number of research turns */
  max_turns?: number;
  /** Enable specific MCP servers */
  enable_servers?: {
    firecrawl?: boolean;
    octocode?: boolean;
    context7?: boolean;
    websearch?: boolean;
  };
}

/**
 * Research result containing gathered context.
 */
export interface ResearchData {
  /** Research findings summary */
  summary: string;
  /** Detailed findings by topic */
  findings: Array<{
    topic: string;
    content: string;
    source?: string;
  }>;
  /** Technology recommendations */
  recommendations: string[];
  /** List of MCP servers used */
  servers_used: string[];
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
 * Build research prompt.
 */
function buildResearchPrompt(payload: ResearchPayload): string {
  const focusAreas = payload.focus_areas?.length 
    ? `\n\nFocus your research on these specific areas:\n${payload.focus_areas.map(a => `- ${a}`).join('\n')}`
    : '';

  return `You are a technical researcher. Research the following topic and provide comprehensive findings.

## Topic
${payload.topic}
${focusAreas}

## Instructions
1. Use available tools to research current best practices, libraries, and implementation patterns
2. Look up relevant documentation for any technologies mentioned
3. Search for code examples and patterns from reputable sources
4. Identify potential technical challenges and solutions

## Output Format
Provide your findings in the following JSON structure:
{
  "summary": "Brief executive summary of key findings",
  "findings": [
    {
      "topic": "Topic area",
      "content": "Detailed findings",
      "source": "Source URL or reference (if applicable)"
    }
  ],
  "recommendations": [
    "Specific technology or approach recommendation"
  ]
}

Start your research now.`;
}

/**
 * Parse research results from response.
 */
function parseResearchResults(text: string): ResearchData | null {
  // Try to extract JSON from response
  const jsonMatch = text.match(/```json\s*([\s\S]*?)\s*```/);
  if (jsonMatch?.[1]) {
    try {
      const parsed = JSON.parse(jsonMatch[1]);
      return {
        summary: parsed.summary ?? '',
        findings: parsed.findings ?? [],
        recommendations: parsed.recommendations ?? [],
        servers_used: [],
      };
    } catch {
      // Fall through to text parsing
    }
  }

  // Try direct JSON parse
  const firstBrace = text.indexOf('{');
  const lastBrace = text.lastIndexOf('}');
  if (firstBrace >= 0 && lastBrace > firstBrace) {
    try {
      const parsed = JSON.parse(text.slice(firstBrace, lastBrace + 1));
      return {
        summary: parsed.summary ?? '',
        findings: parsed.findings ?? [],
        recommendations: parsed.recommendations ?? [],
        servers_used: [],
      };
    } catch {
      // Fall through to text-based result
    }
  }

  // Return text as summary if JSON parsing failed
  return {
    summary: text.slice(0, 2000),
    findings: [],
    recommendations: [],
    servers_used: [],
  };
}

/**
 * Research a topic using MCP tools.
 */
export async function research(
  payload: ResearchPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<ResearchData>> {
  // Check research capability
  if (!hasResearchCapability()) {
    const servers = listAvailableMcpServers();
    const unavailable = servers.filter(s => !s.available);
    return {
      success: false,
      error: `No research MCP servers available. Configure at least one of: ${unavailable.map(s => `${s.name} (${s.reason})`).join(', ')}`,
      error_type: 'mcp_error',
    };
  }

  try {
    // Find Claude CLI executable
    const cliPath = getClaudeCliOrThrow();

    // Get MCP servers
    const mcpServers = getResearchMcpServers();
    const serverNames = Object.keys(mcpServers);

    // Configure Claude Agent SDK options with MCP servers
    const sdkOptions: Options = {
      model,
      maxTurns: payload.max_turns ?? 5,
      mcpServers,
      permissionMode: 'bypassPermissions',
      pathToClaudeCodeExecutable: cliPath,
      // Allow common tools for research
      allowedTools: [
        'WebFetch',
        'WebSearch', 
        'Read',
        'Glob',
        'Grep',
      ],
    };

    let responseText = '';
    let usage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

    const prompt = buildResearchPrompt(payload);

    for await (const message of query({
      prompt,
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

    // Parse results
    const results = parseResearchResults(responseText);
    if (!results) {
      return {
        success: false,
        error: 'Failed to parse research results',
        error_type: 'parse_error',
        details: responseText.slice(0, 500),
      };
    }

    // Add servers used
    results.servers_used = serverNames;

    return {
      success: true,
      data: results,
      usage,
      model,
      provider: 'claude-agent-sdk',
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    return {
      success: false,
      error: `Research failed: ${error}`,
      error_type: 'api_error',
    };
  }
}

/**
 * Check available research capabilities.
 */
export function getResearchCapabilities(): AgentResponse<{
  available: boolean;
  servers: Array<{ name: string; available: boolean; reason?: string }>;
}> {
  const servers = listAvailableMcpServers();
  const available = servers.some(s => s.available);

  return {
    success: true,
    data: {
      available,
      servers,
    },
    usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
    model: 'none',
    provider: 'intake-agent',
  };
}
