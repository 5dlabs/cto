/**
 * Parse PRD Prompt - Full-featured version with decision points and acceptance criteria.
 * 
 * Based on the Rust implementation prompts with all features:
 * - Decision points with categories
 * - Constraint types (hard, soft, open, escalation)
 * - Acceptance criteria (testStrategy)
 * - Agent hints in titles
 * - Research mode support
 */

export interface ParsePrdPromptContext {
  numTasks: number;
  nextId: number;
  research?: boolean;
  defaultPriority?: string;
}

/**
 * Build the system prompt for PRD parsing.
 */
export function buildParsePrdSystemPrompt(ctx: ParsePrdPromptContext): string {
  const researchSection = ctx.research ? `

### Research Mode Active
Before breaking down the PRD into tasks:
1. Research latest technologies, libraries, frameworks appropriate for this project
2. Identify potential technical challenges, security concerns, scalability issues
3. Consider current industry standards and evolving trends
4. Evaluate alternative implementation approaches
5. Include specific library versions and concrete implementation guidance
6. Always aim for the most direct path, avoiding over-engineering` : '';

  return `## Role
You are a Senior Technical Product Manager and Software Architect. You break down complex requirements into actionable development tasks with well-defined acceptance criteria and decision points.

## Context
Analyze a PRD to generate a structured, dependency-aware list of development tasks in JSON format.${researchSection}

## Task
Generate approximately ${ctx.numTasks} top-level development tasks from the PRD, starting from ID ${ctx.nextId}.

## Output Schema
Each task MUST have this structure:
\`\`\`json
{
  "id": number,
  "title": "Action (AgentName - Stack)",
  "description": "What and why",
  "status": "pending",
  "dependencies": [task_ids],
  "priority": "high" | "medium" | "low",
  "details": "Implementation steps (escaped JSON string)",
  "testStrategy": "Acceptance criteria and how to validate",
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
\`\`\`

## Agent Mapping
Include agent hint in title: "(AgentName - Stack)"
- Infrastructure: (Bolt - Kubernetes)
- Rust backend: (Rex - Rust/Axum)
- Go backend: (Grizz - Go/gRPC)
- Node.js backend: (Nova - Bun/Elysia)
- React frontend: (Blaze - React/Next.js)
- Mobile app: (Tap - Expo)
- Desktop app: (Spark - Electron)

## Decision Point Categories
- **architecture**: System design, patterns, service boundaries
- **error-handling**: Error strategies, retry logic, fallbacks
- **data-model**: Schema design, relationships, migrations
- **api-design**: Endpoints, request/response formats, versioning
- **ux-behavior**: User interactions, edge cases, feedback
- **performance**: Caching, optimization, scaling decisions
- **security**: Auth, encryption, access control

## Constraint Types
- **hard**: Must be this way (PRD requirement)
- **soft**: Prefer this but adjustable
- **open**: Agent chooses best approach
- **escalation**: Human must decide before proceeding

## Rules
1. Task 1 MUST be infrastructure setup (Bolt) if databases/storage needed
2. Then backend services, then frontend apps
3. Dependencies only reference lower IDs
4. All string fields must be valid JSON (escape quotes and newlines)
5. Include decision points for ambiguous areas
6. testStrategy must define clear acceptance criteria

## Self-Verification
Before output, verify:
- All PRD requirements have corresponding tasks
- No circular dependencies
- Each task has clear acceptance criteria
- Decision points identified for ambiguous areas

Output ONLY the JSON array contents. No markdown, no explanations.`;
}

/**
 * Build the user prompt for PRD parsing.
 */
export function buildParsePrdUserPrompt(prdContent: string, ctx: ParsePrdPromptContext): string {
  const researchReminder = ctx.research ? `

## Research Reminder
Research current best practices before generating tasks. Apply findings to details and testStrategy fields.` : '';

  return `## PRD Content
---
${prdContent}
---
${researchReminder}

Generate ${ctx.numTasks} tasks starting from ID ${ctx.nextId}.

Include:
- Agent hints in titles: "(AgentName - Stack)"
- Acceptance criteria in testStrategy
- Decision points for ambiguous requirements
- Constraint types for each decision

Output ONLY the JSON array contents, starting with the first task object.`;
}

/**
 * Export prompts for external use.
 */
export const ParsePrdPrompt = {
  buildSystemPrompt: buildParsePrdSystemPrompt,
  buildUserPrompt: buildParsePrdUserPrompt,
};
