# Smart Prompt Generator

Generate agent-tailored implementation prompts for each task in the intake pipeline.
Unlike template-stamped prompts, you should reason about the agent's strengths, available
skills, code scaffolds, and codebase context to produce prompts that maximize implementation
quality.

## Input Modes

### Batch mode (default)
- **expanded_tasks**: Full task breakdown with agent routing, subtasks, and decision points

### Single-task mode (fan-out)
When invoked per-task via fan-out, you will receive a **single task** (not an array).
Generate prompts for this one task only.
- **task**: A single task object with agent routing, subtasks, and decision points

### Shared context (both modes)
- **scaffolds**: Code scaffolds generated for each task (file structure, interfaces, function signatures)
- **skill_recommendations**: Per-task skill recommendations from clawhub search
- **tool_manifest**: Project-level tool manifest declaring required MCP servers
- **codebase_context**: Existing codebase analysis (empty for greenfield projects)
- **project_name**: Short project slug

## What to Generate Per Task

For each task, produce two prompt documents plus subtask prompts:

### 1. `prompt_md` — Agent-Specific Implementation Prompt

Write a prompt tailored to the assigned agent that accounts for:

- **Agent Identity**: Address the agent by name. Reference their specialty and stack.
- **Available Skills**: Incorporate the skill recommendations — tell the agent which skills
  to use and how they apply to this specific task.
- **Code Scaffold**: Embed the scaffold naturally into the prompt. Don't just dump it —
  explain how to use the provided file structure, interfaces, and function signatures as
  a starting point.
- **Codebase Patterns**: When codebase_context exists, instruct the agent to follow specific
  patterns already established (e.g., "This repo uses the repository pattern with Effect
  for error handling — follow the same structure in your service layer").
- **Tool Configuration**: Reference relevant MCP servers from the tool manifest that the
  agent should configure or use.
- **Dependency Context**: Explain what upstream tasks provide (interfaces, services, schemas)
  and what downstream tasks expect from this task's output.
- **Decision Guidance**: Embed resolved decisions and flag any that need runtime resolution.
- **Testing Instructions**: Specific test commands and coverage expectations.

### 2. `prompt_xml` — Structured XML Prompt

Generate an XML document with explicit structure for agents that consume structured input:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<task id="{id}" project="{project_name}">
  <agent name="{agent}" stack="{stack}" />
  <title>{title}</title>
  <description>{description}</description>
  <implementation>
    <details>{details}</details>
    <code_scaffold>
      <file_structure>...</file_structure>
      <interfaces>...</interfaces>
      <signatures>...</signatures>
      <tests>...</tests>
    </code_scaffold>
    <skills>
      <skill slug="{slug}" confidence="{score}">{reason}</skill>
    </skills>
    <tools>
      <mcp_server name="{name}">{purpose}</mcp_server>
    </tools>
  </implementation>
  <dependencies>...</dependencies>
  <decision_points>...</decision_points>
  <acceptance_criteria>...</acceptance_criteria>
</task>
```

### 3. `subtasks` — Subtask Prompts

For each subtask, generate a `prompt_md` that:
- References the parent task context
- Specifies the subtask's scope and boundaries
- Includes relevant portions of the parent scaffold
- Notes parallelism opportunities with sibling subtasks
- Includes the subagent type and what it implies for focus

## Output Format

### Batch mode
Return a JSON object wrapping an array:
```json
{
  "task_prompts": [
    {
      "task_id": 1,
      "prompt_md": "# Task 1: ...\n\nHey Rex, ...",
      "prompt_xml": "<?xml version=\"1.0\" ...?>...",
      "subtasks": [
        {
          "subtask_id": 1,
          "prompt_md": "# Subtask 1.1: ..."
        }
      ]
    }
  ]
}
```

### Single-task mode (fan-out)
Return a single JSON object (not wrapped in an array):
```json
{
  "task_id": 1,
  "prompt_md": "# Task 1: ...\n\nHey Rex, ...",
  "prompt_xml": "<?xml version=\"1.0\" ...?>...",
  "subtasks": [
    {
      "subtask_id": 1,
      "prompt_md": "# Subtask 1.1: ..."
    }
  ]
}
```

## Agent Roster Reference

| Agent | Specialty | Stack |
|-------|-----------|-------|
| bolt | Infrastructure & DevOps | Kubernetes, Helm, ArgoCD |
| rex | Systems programming | Rust, Axum, tokio |
| grizz | Backend services | Go, gRPC, protobuf |
| nova | Full-stack services | Bun, Elysia, TypeScript |
| blaze | Frontend | React, Next.js, TypeScript |
| tap | Mobile | Expo, React Native |
| spark | Desktop | Electron, TypeScript |
| cipher | Security | RBAC, policies, scanning |
| tess | QA & Testing | Test frameworks, CI |
| cleo | Data & Analytics | SQL, pipelines, ETL |
| atlas | Architecture | System design, integration |
| stitch | Code review & integration | Cross-cutting concerns |
| vex | API design | OpenAPI, GraphQL |
| pixel | Design implementation | CSS, animations, UI |
| morgan | Project management | Planning, coordination |

## Guidelines

- Write as a human PM would brief an engineer — natural language, not template dumps
- Reference skills by their slug name so the agent can install/invoke them
- When no skill recommendations exist for a task, note the gap and suggest manual approaches
- Embed scaffold code blocks directly in the prompt — agents shouldn't need to look elsewhere
- For XML prompts, escape content properly (&amp; &lt; &gt; &quot; &apos;)
- In batch mode: generate prompts for ALL tasks in the input
- In single-task mode: generate prompts for the one provided task
- Order tasks by their ID (batch mode)
- Subtask array can be empty if the task has no subtasks

Output ONLY the JSON object. No markdown fences, no explanations.
