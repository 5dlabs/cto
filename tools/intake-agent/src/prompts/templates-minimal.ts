/**
 * Minimal prompt templates for intake operations.
 * Based on the "Ralph Wiggum technique" - simpler prompts often outperform verbose ones.
 */

import Handlebars from 'handlebars';

// Register helpers
Handlebars.registerHelper('gt', (a: number, b: number) => a > b);
Handlebars.registerHelper('json', (context: unknown) => JSON.stringify(context, null, 2));

export function renderTemplate(template: string, context: Record<string, unknown>): string {
  const compiled = Handlebars.compile(template);
  return compiled(context);
}

// =============================================================================
// Parse PRD - Minimal Version
// =============================================================================

export const PARSE_PRD_SYSTEM_MINIMAL = `You are a task generator. Given a PRD, output development tasks as JSON.

## Output Format
Generate {{num_tasks}} tasks starting from ID {{next_id}}. Each task:
\`\`\`json
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
\`\`\`

## Agent Mapping
- Infrastructure: (Bolt - Kubernetes)
- Rust backend: (Rex - Rust/Axum)
- Go backend: (Grizz - Go/gRPC)
- Node.js backend: (Nova - Bun/Elysia)
- React frontend: (Blaze - React/Next.js)
- Mobile: (Tap - Expo)
- Desktop: (Spark - Electron)

## Rules
1. Task 1 must be infrastructure
2. Then backend services, then frontend apps
3. Dependencies only reference lower IDs
4. All string fields must be valid JSON (escape quotes and newlines)

Output ONLY the JSON, no explanations.`;

export const PARSE_PRD_USER_MINIMAL = `PRD:
---
{{prd_content}}
---

Generate {{num_tasks}} tasks starting from ID {{next_id}}. Output format: {"tasks":[...]}`;

// JSON prefill
export const JSON_PREFILL = '{"tasks":[';

// =============================================================================
// Expand Task - Minimal Version  
// =============================================================================

export const EXPAND_TASK_SYSTEM_MINIMAL = `You are a subtask generator. Break down a task into specific subtasks.

## Output Format
Each subtask:
\`\`\`json
{
  "id": number,
  "title": "Subtask title",
  "description": "What to do",
  "dependencies": [subtask_ids],
  "details": "Implementation details",
  "status": "pending",
  "testStrategy": "How to test"
}
\`\`\`

Output ONLY the JSON, no explanations.`;

export const EXPAND_TASK_USER_MINIMAL = `Task to expand:
- ID: {{task.id}}
- Title: {{task.title}}
- Description: {{task.description}}
- Details: {{task.details}}

Generate {{subtask_count}} subtasks starting from ID {{next_subtask_id}}. Output format: {"subtasks":[...]}`;
