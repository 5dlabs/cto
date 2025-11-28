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
    PromptTemplate::new(
        "parse-prd",
        SYSTEM_PROMPT,
        USER_PROMPT,
    ).with_description("Parse a Product Requirements Document into structured tasks")
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant specialized in analyzing Product Requirements Documents (PRDs) and generating a structured, logically ordered, dependency-aware and sequenced list of development tasks in JSON format.{{#if research}}
Before breaking down the PRD into tasks, you will:
1. Research and analyze the latest technologies, libraries, frameworks, and best practices that would be appropriate for this project
2. Identify any potential technical challenges, security concerns, or scalability issues not explicitly mentioned in the PRD without discarding any explicit requirements or going overboard with complexity -- always aim to provide the most direct path to implementation, avoiding over-engineering or roundabout approaches
3. Consider current industry standards and evolving trends relevant to this project (this step aims to solve LLM hallucinations and out of date information due to training data cutoff dates)
4. Evaluate alternative implementation approaches and recommend the most efficient path
5. Include specific library versions, helpful APIs, and concrete implementation guidance based on your research
6. Always aim to provide the most direct path to implementation, avoiding over-engineering or roundabout approaches

Your task breakdown should incorporate this research, resulting in more detailed implementation guidance, more accurate dependency mapping, and more precise technology recommendations than would be possible from the PRD text alone, while maintaining all explicit requirements and best practices and all details and nuances of the PRD.{{/if}}

Analyze the provided PRD content and generate {{#if (gt num_tasks 0)}}approximately {{num_tasks}}{{else}}an appropriate number of{{/if}} top-level development tasks. If the complexity or the level of detail of the PRD is high, generate more tasks relative to the complexity of the PRD
Each task should represent a logical unit of work needed to implement the requirements and focus on the most direct and effective way to implement the requirements without unnecessary complexity or overengineering. Include pseudo-code, implementation details, and test strategy for each task. Find the most up to date information to implement each task.
Assign sequential IDs starting from {{next_id}}. Infer title, description, details, and test strategy for each task based *only* on the PRD content.
Set status to 'pending', dependencies to an empty array [], and priority to '{{default_task_priority}}' initially for all tasks.
Generate a response containing a single key "tasks", where the value is an array of task objects adhering to the provided schema.

Each task should follow this JSON structure:
{
	"id": number,
	"title": string,
	"description": string,
	"status": "pending",
	"dependencies": number[] (IDs of tasks this depends on),
	"priority": "high" | "medium" | "low",
	"details": string (implementation details),
	"testStrategy": string (validation approach)
}

Guidelines:
1. {{#if (gt num_tasks 0)}}Unless complexity warrants otherwise{{else}}Depending on the complexity{{/if}}, create {{#if (gt num_tasks 0)}}exactly {{num_tasks}}{{else}}an appropriate number of{{/if}} tasks, numbered sequentially starting from {{next_id}}
2. Each task should be atomic and focused on a single responsibility following the most up to date best practices and standards
3. Order tasks logically - consider dependencies and implementation sequence
4. Early tasks should focus on setup, core functionality first, then advanced features
5. Include clear validation/testing approach for each task
6. Set appropriate dependency IDs (a task can only depend on tasks with lower IDs, potentially including existing tasks with IDs less than {{next_id}} if applicable)
7. Assign priority (high/medium/low) based on criticality and dependency order
8. Include detailed implementation guidance in the "details" field{{#if research}}, with specific libraries and version recommendations based on your research{{/if}}
9. If the PRD contains specific requirements for libraries, database schemas, frameworks, tech stacks, or any other implementation details, STRICTLY ADHERE to these requirements in your task breakdown and do not discard them under any circumstance
10. Focus on filling in any gaps left by the PRD or areas that aren't fully specified, while preserving all explicit requirements
11. Always aim to provide the most direct path to implementation, avoiding over-engineering or roundabout approaches{{#if research}}
12. For each task, include specific, actionable guidance based on current industry standards and best practices discovered through research{{/if}}"#;

const USER_PROMPT: &str = r#"Here's the Product Requirements Document (PRD) to break down into {{#if (gt num_tasks 0)}}approximately {{num_tasks}}{{else}}an appropriate number of{{/if}} tasks, starting IDs from {{next_id}}:{{#if research}}

Remember to thoroughly research current best practices and technologies before task breakdown to provide specific, actionable implementation details.{{/if}}

{{prd_content}}

IMPORTANT: Your response must be a JSON object with a "tasks" property containing an array of task objects. You may optionally include a "metadata" object. Do not include any other properties."#;

