//! Parse PRD prompt template.
//!
//! Generates tasks from a Product Requirements Document.

use serde::Serialize;

use super::PromptTemplate;

/// Context for parse-prd prompt.
#[derive(Debug, Clone, Serialize)]
pub struct ParsePrdContext {
    /// Target number of tasks to generate (0 = auto)
    pub num_tasks: i32,
    /// Starting ID for tasks
    pub next_id: i32,
    /// Enable research mode
    pub research: bool,
    /// Content of the PRD file
    pub prd_content: String,
    /// Path to the PRD file
    pub prd_path: String,
    /// Default priority for tasks
    pub default_task_priority: String,
    /// Project root path
    pub project_root: String,
}

impl Default for ParsePrdContext {
    fn default() -> Self {
        Self {
            num_tasks: 10,
            next_id: 1,
            research: false,
            prd_content: String::new(),
            prd_path: String::new(),
            default_task_priority: "medium".to_string(),
            project_root: String::new(),
        }
    }
}

/// Get the parse-prd template.
pub fn template() -> PromptTemplate {
    PromptTemplate::new("parse-prd", SYSTEM_PROMPT, USER_PROMPT)
        .with_description("Parse a Product Requirements Document into structured tasks")
}

// Master Prompting 2026 Framework applied:
// - Principle #6: Role-Playing (Persona) - Expert PRD analyst persona
// - Principle #1: Clarity & Specificity - Structured output format
// - Principle #3: Chain of Thought - Step-by-step analysis process
// - Principle #4: Iterative Refinement - Self-verification instructions
// - Principle #5: Context & Knowledge Leverage - Research mode integration
const SYSTEM_PROMPT: &str = r#"## Role (Persona)
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
	"details": string (how to implement, with pseudo-code),
	"testStrategy": string (how to validate),
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
7. STRICTLY ADHERE to PRD-specified libraries, schemas, frameworks, tech stacks
8. Fill gaps in PRD while preserving all explicit requirements
9. Avoid over-engineering; prefer direct implementation paths{{#if research}}
10. Include actionable guidance based on researched best practices{{/if}}
11. Include agent hint in task title using format: "Title (AgentName - Stack)"
    - Infrastructure: (Bolt - Kubernetes)
    - Rust backend: (Rex - Rust/Axum)
    - Go backend: (Grizz - Go/gRPC)
    - Node.js backend: (Nova - Bun/Elysia)
    - React frontend: (Blaze - React/Next.js)
    - Mobile app: (Tap - Expo)
    - Desktop app: (Spark - Electron)
12. **Decision Points (Captured Discovery)**: For each task, identify areas where judgment is needed:
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
- [ ] Decision points identified for ambiguous or high-impact areas"#;

const USER_PROMPT: &str = r#"## PRD Content
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

I have already started the JSON structure with `{"tasks":[` - you must CONTINUE by outputting the task objects directly, starting with the first task object. Do NOT repeat the opening structure.

Example of what you should output (just the array contents, comma-separated task objects):
{"id":{{next_id}},"title":"Setup Infrastructure (Bolt - Kubernetes)","description":"Provision databases, caches, and storage","status":"pending","dependencies":[],"priority":"high","details":"Deploy PostgreSQL, Redis, etc.","testStrategy":"Verify resources are running"},{"id":2,"title":"Backend API (Rex - Rust/Axum)","description":"Core API service","status":"pending","dependencies":[{{next_id}}],"priority":"high","details":"Create Axum router","testStrategy":"Unit and integration tests","decisionPoints":[{"id":"d1","category":"error-handling","description":"Database failure handling","options":["Retry","Fail fast","Circuit breaker"],"requiresApproval":false,"constraintType":"open"}]}]}

FINAL INSTRUCTION: Continue the JSON array by outputting task objects directly. Start with the first task's opening brace { - do NOT output {"tasks":[ again as that is already provided."#;
