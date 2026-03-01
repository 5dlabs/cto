/**
 * Iterative PRD parsing - processes one service/component at a time.
 * 
 * Instead of generating all tasks at once (which overwhelms the model),
 * this approach:
 * 1. First pass: Analyze the structured JSON PRD
 * 2. Iterate: Generate tasks for each service/component
 * 3. Build up: Accumulate tasks with proper dependencies
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

const logger = createLogger('parse-prd-iterative');

interface StructuredPrd {
  name: string;
  description: string;
  requirements?: string[];
  tech_stack?: Record<string, {
    agent?: string;
    language?: string;
    framework?: string;
    runtime?: string;
    database?: string | string[];
    ui?: string;
    [key: string]: unknown;
  }>;
  constraints?: string[];
  success_criteria?: string[];
}

interface ServiceContext {
  name: string;
  agent: string;
  stack: Record<string, unknown>;
  requirements: string[];
}

/**
 * Extract text from assistant message content.
 */
function extractAssistantText(message: SDKAssistantMessage): string {
  const content = message.message.content;
  if (!Array.isArray(content)) return '';
  
  return content
    .filter((block): block is { type: 'text'; text: string } => block.type === 'text')
    .map((block) => block.text)
    .join('');
}

/**
 * Generate system prompt for a single service.
 */
function getServiceSystemPrompt(service: ServiceContext, nextId: number, existingTasks: GeneratedTask[]): string {
  const existingDeps = existingTasks.map(t => `- ID ${t.id}: ${t.title}`).join('\n');
  
  return `You are a task generator. Generate tasks for a specific service in a larger project.

## Service: ${service.name}
Agent: ${service.agent}
Stack: ${JSON.stringify(service.stack, null, 2)}

## Related Requirements
${service.requirements.map(r => `- ${r}`).join('\n')}

${existingTasks.length > 0 ? `## Existing Tasks (can depend on these)
${existingDeps}
` : ''}

## Output Format
Generate 2-5 tasks for this service, starting from ID ${nextId}. Each task:
{
  "id": number,
  "title": "Action (${service.agent} - Stack)",
  "description": "Brief description",
  "status": "pending",
  "dependencies": [task_ids],
  "priority": "high" | "medium" | "low",
  "details": "Implementation steps",
  "testStrategy": "How to test"
}

## Rules
1. Generate 2-5 tasks specific to this service
2. Dependencies can reference existing task IDs (< ${nextId})
3. Include the agent name in the title: (${service.agent} - ...)
4. Focus on this service only

Output ONLY the JSON, no explanations.`;
}

/**
 * Generate tasks for a single service.
 */
async function generateServiceTasks(
  service: ServiceContext,
  nextId: number,
  existingTasks: GeneratedTask[],
  model: string,
  cliPath: string
): Promise<{ tasks: GeneratedTask[]; usage: TokenUsage }> {
  const systemPrompt = getServiceSystemPrompt(service, nextId, existingTasks);
  const userPrompt = `Generate tasks for the ${service.name} service (${service.agent}).`;

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

  for await (const message of query({ prompt: userPrompt, options: sdkOptions })) {
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

  const result = parseJsonResponse<GeneratedTask>(responseText, 'tasks', isValidTask as (item: unknown) => item is GeneratedTask);
  
  if (!result.success) {
    logger.warn(`Failed to parse tasks for ${service.name}`, { error: result.error });
    return { tasks: [], usage };
  }

  return { tasks: result.items, usage };
}

/**
 * Extract services from structured PRD.
 */
function extractServices(prd: StructuredPrd): ServiceContext[] {
  const services: ServiceContext[] = [];
  
  if (!prd.tech_stack) return services;

  // Map requirements to services based on keywords
  const reqMap: Record<string, string[]> = {};
  for (const [name] of Object.entries(prd.tech_stack)) {
    reqMap[name] = (prd.requirements || []).filter(r => 
      r.toLowerCase().includes(name.replace('_', ' ')) ||
      r.toLowerCase().includes(prd.tech_stack![name]?.agent?.toLowerCase() ?? '')
    );
  }

  // Infrastructure first
  if (prd.tech_stack.infrastructure) {
    services.push({
      name: 'infrastructure',
      agent: prd.tech_stack.infrastructure.agent || 'Bolt',
      stack: prd.tech_stack.infrastructure,
      requirements: reqMap['infrastructure'] || ['Setup Kubernetes infrastructure'],
    });
  }

  // Backend services next
  const backendOrder = ['notification_router', 'admin_api', 'integration_service'];
  for (const name of backendOrder) {
    if (prd.tech_stack[name]) {
      services.push({
        name,
        agent: prd.tech_stack[name].agent || 'Unknown',
        stack: prd.tech_stack[name],
        requirements: reqMap[name] || [],
      });
    }
  }

  // Frontend apps last
  const frontendOrder = ['web_console', 'mobile_app', 'desktop_client'];
  for (const name of frontendOrder) {
    if (prd.tech_stack[name]) {
      services.push({
        name,
        agent: prd.tech_stack[name].agent || 'Unknown',
        stack: prd.tech_stack[name],
        requirements: reqMap[name] || [],
      });
    }
  }

  return services;
}

/**
 * Parse PRD iteratively - one service at a time.
 */
export async function parsePrdIterative(
  payload: ParsePrdPayload,
  model: string,
  _options: GenerateOptions
): Promise<AgentResponse<ParsePrdData>> {
  const prdContent = payload.prd_content;
  
  // Parse the structured JSON PRD
  let prd: StructuredPrd;
  try {
    prd = JSON.parse(prdContent);
  } catch {
    return {
      success: false,
      error: 'PRD content is not valid JSON. Use parse_prd for markdown PRDs.',
      error_type: 'validation_error',
    };
  }

  logger.info('Starting iterative PRD parsing', { 
    name: prd.name,
    services: Object.keys(prd.tech_stack || {}).length,
  });

  const services = extractServices(prd);
  logger.info('Extracted services', { count: services.length, services: services.map(s => s.name) });

  const cliPath = getClaudeCliOrThrow();
  const allTasks: GeneratedTask[] = [];
  let totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  let nextId = payload.next_id ?? 1;

  // Process each service
  for (const service of services) {
    logger.info(`Processing service: ${service.name}`, { agent: service.agent, nextId });

    try {
      const { tasks, usage } = await generateServiceTasks(
        service,
        nextId,
        allTasks,
        model,
        cliPath
      );

      logger.info(`Generated ${tasks.length} tasks for ${service.name}`, { usage });

      allTasks.push(...tasks);
      nextId = Math.max(nextId, ...tasks.map(t => t.id)) + 1;

      totalUsage.input_tokens += usage.input_tokens;
      totalUsage.output_tokens += usage.output_tokens;
      totalUsage.total_tokens += usage.total_tokens;

    } catch (e) {
      logger.error(`Failed to process ${service.name}`, { error: String(e) });
    }
  }

  logger.info('Iterative parsing complete', { 
    totalTasks: allTasks.length, 
    totalUsage 
  });

  return {
    success: true,
    data: { tasks: allTasks },
    usage: totalUsage,
    model,
    provider: 'claude-agent-sdk',
  };
}
