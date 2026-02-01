/**
 * Expand Task Prompt - Full-featured with subagent-aware metadata.
 * 
 * Features:
 * - Subagent types (implementer, reviewer, tester, documenter, researcher, debugger)
 * - Parallelizable flag for concurrent execution
 * - Expansion prompt from complexity analysis
 * - Complexity reasoning context
 */

export interface ExpandTaskPromptContext {
  subtaskCount: number;
  nextSubtaskId: number;
  task: {
    id: string | number;
    title: string;
    description: string;
    details?: string;
  };
  expansionPrompt?: string;
  complexityReasoning?: string;
  additionalContext?: string;
  enableSubagents?: boolean;
  useResearch?: boolean;
}

/**
 * Build the system prompt for task expansion.
 */
export function buildExpandTaskSystemPrompt(ctx: ExpandTaskPromptContext): string {
  const subagentSection = ctx.enableSubagents !== false ? `

## Subagent Types
Each subtask MUST include a subagentType:
- **implementer**: Write/implement code (default for most coding)
- **reviewer**: Review code quality, patterns, best practices
- **tester**: Write and run tests
- **documenter**: Write documentation
- **researcher**: Research and exploration tasks
- **debugger**: Debug issues and fix bugs

## Parallelization
Set parallelizable=true for subtasks that can run concurrently with others at the same dependency level.

## Subagent Optimization
1. Maximize parallelism: Group independent work units
2. Minimize dependencies: Only add when strictly necessary
3. Match types to work: implementer for coding, tester for tests
4. Context isolation: Each subagent works alone, subtasks must be self-contained
5. Plan review phases: Include reviewer subtasks after implementation` : '';

  const researchSection = ctx.useResearch ? `

## Research Mode
Use current best practices and latest technical information for subtask generation.` : '';

  return `## Role
You are an AI assistant breaking down high-level tasks into specific, actionable subtasks${ctx.enableSubagents !== false ? ' optimized for parallel subagent execution' : ''}.${researchSection}

## Output Schema
Each subtask MUST have ALL fields:
\`\`\`json
{
  "id": number (sequential from ${ctx.nextSubtaskId}),
  "title": "Clear, actionable title",
  "description": "Detailed description (min 10 chars)",
  "status": "pending",
  "dependencies": [subtask_ids],
  "details": "Implementation details (min 20 chars)",
  "testStrategy": "Testing approach"${ctx.enableSubagents !== false ? `,
  "subagentType": "implementer" | "reviewer" | "tester" | "documenter" | "researcher" | "debugger",
  "parallelizable": boolean` : ''}
}
\`\`\`
${subagentSection}

## Rules
1. IDs MUST be sequential starting from ${ctx.nextSubtaskId}
2. Each subtask should be completable in 1-4 hours
3. Dependencies only reference lower subtask IDs
4. Include clear implementation details
5. All string fields must be valid JSON (escape quotes and newlines)

Output ONLY the JSON array contents. No markdown, no explanations.`;
}

/**
 * Build the user prompt for task expansion.
 */
export function buildExpandTaskUserPrompt(ctx: ExpandTaskPromptContext): string {
  const sections: string[] = [
    `## Task to Expand`,
    `- ID: ${ctx.task.id}`,
    `- Title: ${ctx.task.title}`,
    `- Description: ${ctx.task.description}`,
  ];

  if (ctx.task.details) {
    sections.push(`- Details: ${ctx.task.details}`);
  }

  if (ctx.expansionPrompt) {
    sections.push(`\n## Expansion Guidance\n${ctx.expansionPrompt}`);
  }

  if (ctx.complexityReasoning) {
    sections.push(`\n## Complexity Analysis\n${ctx.complexityReasoning}`);
  }

  if (ctx.additionalContext) {
    sections.push(`\n## Additional Context\n${ctx.additionalContext}`);
  }

  sections.push(`\nGenerate ${ctx.subtaskCount} subtasks starting from ID ${ctx.nextSubtaskId}.`);

  if (ctx.enableSubagents !== false) {
    sections.push(`
## Subagent Requirements
- Include subagentType for EVERY subtask
- Set parallelizable=true for independent subtasks
- Minimize dependencies to maximize parallel execution
- Include at least one reviewer subtask after implementation
- Include tester subtasks for validation`);
  }

  sections.push(`\nOutput ONLY the JSON array contents.`);

  return sections.join('\n');
}

/**
 * Export prompts for external use.
 */
export const ExpandTaskPrompt = {
  buildSystemPrompt: buildExpandTaskSystemPrompt,
  buildUserPrompt: buildExpandTaskUserPrompt,
};
