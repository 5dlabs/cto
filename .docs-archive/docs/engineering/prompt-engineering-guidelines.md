# Prompt Engineering Guidelines

This document describes the prompt engineering framework used by the CTO platform
for generating high-quality agent prompts during document ingestion and task generation.

## Overview

The CTO platform applies the **Master Prompting 2026** framework to all generated
prompts. This framework ensures consistent, high-quality prompts that maximize
AI agent effectiveness.

## The 6 Core Principles

### 1. Clarity & Specificity

Eliminate ambiguity by defining the task, context, and constraints immediately.

**Implementation:**

- Structured sections with clear headings
- Explicit constraints and formatting requirements
- Specific output format expectations

**Example:**

```markdown
### Constraints & Formatting
- **Code Style:** Match existing codebase patterns
- **Output Format:** Pull request with clear commits
- **PR Title Format:** `feat(task-{id}): {title}`
```

### 2. Few-Shot Prompting

Include examples of the desired output style/format within the prompt.

**Implementation:**

- Example PR structure in every prompt
- Sample output formats for different task types
- Template patterns for common deliverables

**Example:**

```markdown
## Example Output (Few-Shot)
Your PR should follow this structure:

## Summary
Brief description of what this PR implements.

## Changes
- List of specific changes made

## Testing
- How the changes were tested
```

### 3. Chain of Thought

Instruct the AI to think step-by-step before delivering the final answer.

**Implementation:**

- Numbered steps with clear actions
- Explicit "think before acting" instructions
- Logical progression from analysis to delivery

**Example:**

```markdown
## Steps (Chain of Thought)
Think step-by-step before implementing:

1. **Analyze Context** - Review existing code patterns
2. **Plan Implementation** - Outline approach before coding
3. **Implement Solution** - Write clean, documented code
4. **Write Tests** - Create tests for acceptance criteria
5. **Self-Review** - Check for issues before submitting
6. **Submit PR** - Create comprehensive PR description
```

### 4. Iterative Refinement

Pre-load the prompt with instructions to critique its own work.

**Implementation:**

- Self-critique checklist before submission
- Verification steps for each requirement
- Quality gates before PR creation

**Example:**

```markdown
## Self-Critique Checklist
Before submitting your PR, verify each item:

- [ ] **Functionality:** Does it meet all requirements?
- [ ] **Edge Cases:** Are boundary conditions handled?
- [ ] **Security:** Any potential vulnerabilities?
- [ ] **Performance:** Is it efficient for expected scale?
- [ ] **Maintainability:** Is code readable and documented?
```

### 5. Context & Knowledge Leverage

Provide detailed background information and instructions on using context.

**Implementation:**

- Reference to PRD and architecture documents
- Project-specific conventions and patterns
- Links to relevant documentation

**Example:**

```markdown
## Context (Background)
**Task Priority:** high
**Dependencies:** Task 1, Task 2
**Scope:** Implement user authentication flow

Review the PRD and architecture documents in `.taskmaster/docs/`
to understand the full context before implementing.
```

### 6. Role-Playing (Persona)

Assign a specific, expert persona to the AI based on task domain.

**Implementation:**

- Domain detection from task title/description
- Expertise-specific personas
- Responsibility framing

**Persona Mapping:**

| Domain Keywords | Assigned Persona |
|-----------------|------------------|
| frontend, ui, react, component | Senior Frontend Engineer |
| backend, api, server, rust | Senior Backend Engineer |
| devops, kubernetes, helm | Senior DevOps Engineer |
| test, qa, quality | Senior QA Engineer |
| security, auth, encryption | Senior Security Engineer |
| data, analytics, ml | Senior Data Engineer |

**Example:**

```markdown
## Role (Persona)
You are a Senior Frontend Engineer with expertise in React,
TypeScript, and modern UI/UX. Your primary responsibility is
implementing Task 5 with production-quality code.
```

## Generated Files

The unified intake process generates four documentation files per task:

### task.md

Overview document with task details, priority, dependencies, and test strategy.
Used for human reference and task tracking.

### prompt.md

Full implementation prompt applying all 6 principles. This is the primary
prompt used by coding agents to implement the task.

### acceptance-criteria.md

Structured acceptance criteria with functional requirements, testing requirements,
and definition of done checklist.

### task.xml

XML-structured prompt for LLMs that prefer structured input. Contains the same
information as prompt.md in a machine-parseable format.

## Domain Detection

The system automatically detects task domain from keywords in the title and
description to assign appropriate personas:

```bash
# Frontend detection
grep -qiE 'frontend|ui|react|component|css|tailwind|design'

# Backend detection
grep -qiE 'backend|api|server|database|rust|postgres'

# DevOps detection
grep -qiE 'devops|deploy|kubernetes|helm|infra|ci/cd'

# QA detection
grep -qiE 'test|qa|quality|validation'

# Security detection
grep -qiE 'security|auth|encryption|oauth'

# Data detection
grep -qiE 'data|analytics|ml|ai|model'
```

## Best Practices

### Do

- Always include all 6 principles in generated prompts
- Use domain-specific terminology in personas
- Provide concrete examples, not abstract descriptions
- Include self-verification steps
- Reference project-specific documentation

### Don't

- Use vague instructions like "implement this feature"
- Skip the Chain of Thought steps
- Omit acceptance criteria or testing requirements
- Use generic personas for specialized tasks
- Assume context without explicit references

## Integration Points

The Master Prompting 2026 framework is applied in:

1. **unified-intake.sh.hbs** - Main prompt generation during intake
2. **task.xml generation** - XML-structured prompts
3. **parse_prd.rs** - PRD parsing and task generation
4. **Agent templates** - Runtime prompt enhancement

## References

- [Master Prompting 2026 Framework](https://publishpages313.notion.site/Next-Gen-Prompt-Engineer-2026-2b90584ba03a80bf9d6ae00d00e144db)
- [Unified Intake Architecture](../unified-intake-architecture.md)
- [Agent Templates](../../infra/charts/controller/agent-templates/)

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-29 | Initial framework integration |
