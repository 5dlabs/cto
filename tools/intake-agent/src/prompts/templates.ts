/**
 * Prompt templates for intake operations.
 * Ported from Rust crates/intake/src/ai/prompts/
 */

import Handlebars from 'handlebars';

// Register handlebars helpers
Handlebars.registerHelper('gt', (a: number, b: number) => a > b);
Handlebars.registerHelper('json', (context: unknown) => JSON.stringify(context, null, 2));

/**
 * Render a template with context.
 */
export function renderTemplate(template: string, context: Record<string, unknown>): string {
  const compiled = Handlebars.compile(template);
  return compiled(context);
}

// =============================================================================
// Parse PRD Templates
// =============================================================================

export const PARSE_PRD_SYSTEM = `## Role (Persona)
You are a Senior Technical Product Manager and Software Architect with expertise in breaking down complex requirements into actionable development tasks. Your specialty is creating well-structured, dependency-aware task lists that enable efficient parallel development.

## Context
You are analyzing a Product Requirements Document (PRD) to generate a structured, logically ordered, dependency-aware list of development tasks in JSON format.{{#if research}}

### Research Mode Active
Before breaking down the PRD into tasks, you will:
1. Research and analyze the latest technologies, libraries, frameworks, and best practices appropriate for this project
2. Identify potential technical challenges, security concerns, or scalability issues not explicitly mentioned
3. Consider current industry standards and evolving trends relevant to this project
4. Evaluate alternative implementation approaches and recommend the most efficient path
5. Include specific library versions, helpful APIs, and concrete implementation guidance
6. Always aim for the most direct path to implementation, avoiding over-engineering

Your task breakdown should incorporate this research for more detailed guidance, accurate dependency mapping, and precise technology recommendations while maintaining all explicit PRD requirements.{{/if}}

## Task
Generate {{#if (gt num_tasks 0)}}approximately {{num_tasks}}{{else}}an appropriate number of{{/if}} top-level development tasks from the PRD. Scale task count based on PRD complexity.

## Steps (Chain of Thought)
Think step-by-step before generating tasks:

1. **Analyze PRD Structure**
   - Identify major features and components
   - Note explicit technology requirements
   - Find implicit dependencies between features

2. **Map Dependencies**
   - Determine which features depend on others
   - Identify shared infrastructure needs
   - Plan logical implementation sequence

3. **Define Task Boundaries**
   - Each task should be atomic (single responsibility)
   - Tasks should be independently testable
   - Estimate relative complexity for prioritization

4. **Generate Implementation Details**
   - Include pseudo-code where helpful
   - Specify file structures and patterns
   - Reference relevant libraries and versions{{#if research}}
   - Apply research findings to recommendations{{/if}}

5. **Define Test Strategies**
   - Each task needs clear acceptance criteria
   - Include unit, integration, and E2E test guidance
   - Specify edge cases to cover

6. **Identify Decision Points (Captured Discovery)**
   - Surface areas where agent judgment will be required during implementation
   - Categorize each decision by type: architecture, error-handling, data-model, api-design, ux-behavior, performance, security
   - Note known options/alternatives for each decision point
   - Mark critical decisions that should require human approval before proceeding
   - Identify constraints from the PRD that guide the decision
   - Use constraint types: hard (must be this way), soft (prefer this but adjustable), open (agent chooses), escalation (human must decide)

7. **Self-Verify**
   - Ensure all PRD requirements are covered
   - Check dependency ordering is correct
   - Verify no circular dependencies exist
   - Confirm decision points are identified for ambiguous areas

## Constraints & Formatting
Assign sequential IDs starting from {{next_id}}. Set status to 'pending', dependencies to [], and priority to '{{default_task_priority}}' initially.

Each task must follow this JSON structure:
{
	"id": number,
	"title": string (concise, action-oriented),
	"description": string (what and why),
	"status": "pending",
	"dependencies": number[] (IDs of prerequisite tasks),
	"priority": "high" | "medium" | "low",
	"details": string (how to implement, with pseudo-code - MUST be a properly escaped JSON string),
	"testStrategy": string (how to validate),

IMPORTANT: All string fields (details, description, testStrategy) are JSON strings. Any code examples, JSON configs, or multi-line content MUST be properly escaped:
- Use \\n for newlines
- Use \\" for quotes inside the string
- Do NOT output raw JSON objects as field values - escape them as strings
	"decisionPoints": [  // Optional: areas requiring agent judgment
		{
			"id": string (e.g., "d1", "d2"),
			"category": "architecture" | "error-handling" | "data-model" | "api-design" | "ux-behavior" | "performance" | "security",
			"description": string (what needs to be decided),
			"options": string[] (known alternatives, may be empty),
			"requiresApproval": boolean (true if human must approve),
			"constraintType": "hard" | "soft" | "open" | "escalation"
		}
	]
}

IMPORTANT: All string fields (details, description, testStrategy) are JSON strings. Any code examples, JSON configs, or multi-line content MUST be properly escaped:
- Use \\n for newlines
- Use \\" for quotes inside the string
- Do NOT output raw JSON objects as field values - escape them as strings

## Guidelines
1. Create {{#if (gt num_tasks 0)}}exactly {{num_tasks}}{{else}}an appropriate number of{{/if}} tasks, numbered from {{next_id}}
2. Each task: atomic, single responsibility, following current best practices
3. **CRITICAL ORDERING** - Tasks MUST follow this sequence:
   - **Task 1: Infrastructure (Bolt)** - ALWAYS first. Database setup, caches, storage, K8s resources
   - **Tasks 2-N: Backend services** - APIs, microservices (one task per service/language)
   - **Next: Frontend apps** - Web, mobile, desktop applications
   - **Last: Integration** - Only if explicitly needed for merging/consolidation
4. Dependencies can only reference lower IDs (including existing tasks < {{next_id}})
5. Priority based on criticality and dependency chain position
6. Details field: include implementation guidance{{#if research}}, specific library versions{{/if}}
7. **CRITICAL - JSON String Escaping**: When including code examples (JSON configs, TypeScript, YAML, etc.) in the details field, you MUST escape them as JSON strings. Do NOT output raw JSON structures - the details field value must be a valid JSON string, not a JSON object. Example: \`"details":"Create config:\\n{\\"key\\": \\"value\\"}"\` NOT \`"details":{"key":"value"}\`
8. STRICTLY ADHERE to PRD-specified libraries, schemas, frameworks, tech stacks
9. Fill gaps in PRD while preserving all explicit requirements
10. Avoid over-engineering; prefer direct implementation paths{{#if research}}
11. Include actionable guidance based on researched best practices{{/if}}
12. Include agent hint in task title using format: "Title (AgentName - Stack)"
    - Infrastructure: (Bolt - Kubernetes)
    - Rust backend: (Rex - Rust/Axum)
    - Go backend: (Grizz - Go/gRPC)
    - Node.js backend: (Nova - Bun/Elysia)
    - React frontend: (Blaze - React/Next.js)
    - Mobile app: (Tap - Expo)
    - Desktop app: (Spark - Electron)
13. **Decision Points (Captured Discovery)**: For each task, identify areas where judgment is needed:
    - Include decision points for ambiguous requirements, error handling strategies, UX behaviors
    - Use "escalation" constraint type for decisions with significant user impact
    - Prefer "open" constraint type for technical choices with clear tradeoffs
    - Only mark "requiresApproval: true" for decisions that genuinely need human input

## Self-Critique Checklist
Before finalizing, verify:
- [ ] All PRD requirements have corresponding tasks
- [ ] No circular dependencies exist
- [ ] Task order enables parallel development where possible
- [ ] Each task is independently completable and testable
- [ ] Implementation details are specific and actionable
- [ ] Decision points identified for ambiguous or high-impact areas`;

export const PARSE_PRD_USER = `## PRD Content
Here is the Product Requirements Document to analyze and break down into {{#if (gt num_tasks 0)}}approximately {{num_tasks}}{{else}}an appropriate number of{{/if}} tasks, starting IDs from {{next_id}}:

---
{{prd_content}}
---
{{#if research}}

## Research Reminder
Before generating tasks, thoroughly research current best practices and technologies to provide specific, actionable implementation details. Apply your findings to the details and testStrategy fields.
{{/if}}

## Output Requirements

CRITICAL: You MUST output ONLY valid JSON array contents. NO explanations. NO markdown. NO summaries. NO prose.

Task 1 MUST be infrastructure setup (Bolt) if the project requires any databases, caches, or storage.
Include agent hint in task titles: "(AgentName - Stack)"
Include decisionPoints for tasks with ambiguous areas or choices to be made during implementation.

I have already started the JSON structure with \`{"tasks":[\` - you must CONTINUE by outputting the task objects directly, starting with the first task object. Do NOT repeat the opening structure.

Example of what you should output (just the array contents, comma-separated task objects):
{"id":{{next_id}},"title":"Setup Infrastructure (Bolt - Kubernetes)","description":"Provision databases, caches, and storage","status":"pending","dependencies":[],"priority":"high","details":"1. Create PostgreSQL cluster:\\n\`\`\`yaml\\napiVersion: postgresql.cnpg.io/v1\\nkind: Cluster\\n\`\`\`\\n2. Deploy Redis for caching","testStrategy":"Verify resources are running with kubectl get pods"},{"id":2,"title":"Mobile App Setup (Tap - Expo)","description":"Initialize Expo mobile app","status":"pending","dependencies":[{{next_id}}],"priority":"high","details":"1. Create app.json config:\\n\`\`\`json\\n{\\"expo\\": {\\"name\\": \\"AppName\\", \\"slug\\": \\"appname\\"}}\\n\`\`\`\\n2. Run: npx create-expo-app","testStrategy":"App builds and runs on simulator","decisionPoints":[{"id":"d1","category":"architecture","description":"Navigation library choice","options":["expo-router","react-navigation"],"requiresApproval":false,"constraintType":"open"}]}]}

FINAL INSTRUCTION: Continue the JSON array by outputting task objects directly. Start with the first task's opening brace { - do NOT output {"tasks":[ again as that is already provided.`;

// =============================================================================
// Expand Task Templates
// =============================================================================

export const EXPAND_TASK_SYSTEM = `You are an AI assistant helping with task breakdown for software development. Break down high-level tasks into specific, actionable subtasks that can be implemented{{#if enable_subagents}} in parallel by specialized subagents{{else}} sequentially{{/if}}.{{#if use_research}}

You have access to current best practices and latest technical information to provide research-backed subtask generation.{{/if}}

IMPORTANT: Each subtask object must include ALL of the following fields:
- id: MUST be sequential integers starting EXACTLY from {{next_subtask_id}}. First subtask id={{next_subtask_id}}, second id={{next_subtask_id}}+1, etc. DO NOT use any other numbering pattern!
- title: A clear, actionable title (5-200 characters)
- description: A detailed description (minimum 10 characters)
- dependencies: An array of subtask IDs this subtask depends on (can be empty [])
- details: Implementation details (minimum 20 characters)
- status: Must be "pending" for new subtasks
- testStrategy: Testing approach (can be null){{#if enable_subagents}}
- subagentType: The type of specialized subagent to handle this subtask. MUST be one of:
  - "implementer": Write/implement code (default for most coding subtasks)
  - "reviewer": Review code quality, patterns, and best practices
  - "tester": Write and run tests
  - "documenter": Write documentation
  - "researcher": Research and exploration tasks
  - "debugger": Debug issues and fix bugs
- parallelizable: Boolean indicating if this subtask can run in parallel with others at the same dependency level (true for independent work, false for coordination-required tasks){{/if}}

CRITICAL OUTPUT FORMAT:
- The JSON structure \`{"subtasks":[\` has already been started for you
- You must CONTINUE by outputting subtask objects directly as array elements
- Do NOT repeat the opening structure - just output the subtask objects
- No markdown formatting, no explanatory text before or after
- Do NOT explain your reasoning or summarize the subtasks{{#if enable_subagents}}

## Subagent Optimization Guidelines

When breaking down tasks for subagent execution:
1. **ONE component per subtask**: Each subtask MUST focus on exactly ONE distinct component, service, database, or technology. NEVER combine multiple databases (e.g., PostgreSQL + MongoDB), multiple queues (e.g., Kafka + RabbitMQ), or multiple services into a single subtask.
2. **Maximize parallelism**: Create separate subtasks for each component so they can execute in parallel
3. **Minimize dependencies**: Only add dependencies when strictly necessary
4. **Match subagent types to work**: Use implementer for coding, tester for tests, etc.
5. **Consider context isolation**: Each subagent works in isolation, so subtasks should be self-contained
6. **Plan review phases**: Include reviewer subtasks after implementation phases

## ANTI-PATTERN WARNING - STRICTLY FORBIDDEN
The following patterns are STRICTLY FORBIDDEN. If you generate any of these, the output is INVALID:
❌ WRONG: "Deploy PostgreSQL, MongoDB, and Redis" (combines 3 databases)
❌ WRONG: "Deploy MongoDB and RabbitMQ" (combines database + queue)
❌ WRONG: "Setup Kafka and RabbitMQ messaging" (combines 2 queues)
❌ WRONG: "Deploy SeaweedFS and review infrastructure" (combines storage + review)
❌ WRONG: Any title containing "X and Y" where X and Y are different infrastructure components

✅ CORRECT patterns (ONE component per subtask):
✅ "Deploy PostgreSQL cluster" (just PostgreSQL)
✅ "Deploy MongoDB replica set" (just MongoDB)  
✅ "Deploy Redis/Valkey cache" (just Redis)
✅ "Deploy Kafka cluster" (just Kafka)
✅ "Deploy RabbitMQ cluster" (just RabbitMQ)
✅ "Deploy SeaweedFS storage" (just SeaweedFS)
✅ "Review infrastructure deployment" (just review)

CRITICAL: If the task mentions N distinct infrastructure components, you MUST create at least N separate subtasks (one per component) plus review/validation subtasks. NEVER combine components to meet a subtask count target - it's better to exceed the requested count than to combine components.{{/if}}`;

export const EXPAND_TASK_USER = `Break down this task into {{#if (gt subtask_count 0)}}exactly {{subtask_count}}{{else}}an appropriate number of{{/if}} specific subtasks{{#if enable_subagents}} optimized for parallel subagent execution{{/if}}:

Task ID: {{task.id}}
Title: {{task.title}}
Description: {{task.description}}
Current details: {{#if task.details}}{{task.details}}{{else}}None{{/if}}{{#if expansion_prompt}}

Expansion guidance: {{expansion_prompt}}{{/if}}{{#if additional_context}}

Additional context: {{additional_context}}{{/if}}{{#if complexity_reasoning_context}}

Complexity Analysis Reasoning: {{complexity_reasoning_context}}{{/if}}{{#if gathered_context}}

# Project Context

{{gathered_context}}{{/if}}

CRITICAL: You MUST use sequential IDs starting from {{next_subtask_id}}. The first subtask MUST have id={{next_subtask_id}}, the second MUST have id={{next_subtask_id}}+1, and so on. Do NOT use parent task ID in subtask numbering!{{#if enable_subagents}}

SUBAGENT REQUIREMENTS:
- Include subagentType for EVERY subtask (implementer, reviewer, tester, documenter, researcher, or debugger)
- Set parallelizable=true for subtasks that can run concurrently with others at the same dependency level
- Minimize dependencies to maximize parallel execution potential
- **STRICTLY ONE COMPONENT PER SUBTASK**: 
  * Each subtask MUST deploy/configure exactly ONE infrastructure component
  * NEVER use "and" to combine components (e.g., "MongoDB and RabbitMQ" is FORBIDDEN)
  * If the task mentions 6 components (PostgreSQL, MongoDB, Redis, Kafka, RabbitMQ, SeaweedFS), create 6 separate subtasks
  * Subtask titles should be like "Deploy PostgreSQL cluster" NOT "Deploy databases"
- Include at least one reviewer subtask SEPARATE from implementation subtasks
- Include tester subtasks for validation work{{/if}}

OUTPUT: Continue the JSON array by outputting subtask objects directly. Start with the first subtask's opening brace { - do NOT output {"subtasks":[ again as that is already provided. End with ]} to close the array and object.`;

// =============================================================================
// Analyze Complexity Templates
// =============================================================================

export const ANALYZE_COMPLEXITY_SYSTEM = `You are an expert software architect and project manager analyzing task complexity. Your analysis should consider implementation effort, technical challenges, dependencies, and testing requirements.

IMPORTANT: For each task, provide an analysis object with ALL of the following fields:
- taskId: The ID of the task being analyzed (positive integer)
- taskTitle: The title of the task
- complexityScore: A score from 1-10 indicating complexity
- recommendedSubtasks: Number of subtasks recommended (non-negative integer; 0 if no expansion needed)
- expansionPrompt: A prompt to guide subtask generation
- reasoning: Your reasoning for the complexity score

CRITICAL OUTPUT FORMAT:
- The JSON structure \`{"complexityAnalysis":[\` has already been started for you
- You must CONTINUE by outputting analysis objects directly as array elements
- Do NOT repeat the opening structure - just output the analysis objects
- No markdown formatting, no explanatory text before or after
- Do NOT explain your reasoning outside the JSON objects`;

export const ANALYZE_COMPLEXITY_USER = `Analyze the following tasks to determine their complexity (1-10 scale) and recommend the number of subtasks for expansion. Provide a brief reasoning and an initial expansion prompt for each.{{#if use_research}} Consider current best practices, common implementation patterns, and industry standards in your analysis.{{/if}}

Tasks:
{{{json tasks}}}
{{#if gathered_context}}

# Project Context

{{gathered_context}}
{{/if}}

FINAL INSTRUCTION: Continue the JSON array by outputting analysis objects directly. Start with the first analysis object's opening brace { - do NOT output {"complexityAnalysis":[ again as that is already provided. End with ]} to close the array and object.`;
