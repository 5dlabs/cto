# Integration Task Templates

These templates are used by Morgan (docs agent) to auto-generate integration tasks for parallel execution levels.

## Purpose

When the dependency graph analysis reveals multiple tasks can run in parallel (within the same execution level), we need integration validation between levels to ensure they work together.

## Usage

Morgan automatically:
1. Analyzes the dependency graph in `tasks.json`
2. Identifies execution levels
3. For levels with 2+ parallel tasks, creates an integration task
4. Customizes these templates with level-specific information

## Template Files

- **task.txt** - Comprehensive integration instructions
- **task.md** - Formatted task documentation
- **prompt.md** - Agent prompt with step-by-step validation
- **acceptance-criteria.md** - Integration validation checklist
- **task.xml** - Structured metadata

## Placeholders

Templates use these placeholders that Morgan replaces:

- `LEVEL_INDEX` - The execution level number (0, 1, 2, etc.)
- `TASK_LIST_PLACEHOLDER` - Comma-separated list of task IDs in the level
- `TASK_LIST_WITH_PRS_PLACEHOLDER` - Task descriptions with PR references
- `INTEGRATION_TASK_ID` - The ID assigned to this integration task
- `ALL_DEPENDENCY_TASK_IDS` - All tasks this integration task depends on

## Example

**Input:** Level 0 has tasks 1, 2, 3 (all parallel)

**Morgan creates:**
- Task 4: "Integration - Level 0"
- Depends on: [1, 2, 3]
- Customized templates with level 0 information
- Next level's tasks now depend on task 4 instead of [1,2,3]

## Integration Task Flow

```
Level 0: Tasks 1, 2, 3 (parallel)
  ↓ (all complete)
Task 4: Integration - Level 0
  - Validates 1, 2, 3 work together
  - Runs full test suite
  - Checks for conflicts
  ↓ (integration validated)
Level 1: Tasks start with validated foundation
```

## Agent Assignment

Integration tasks are assigned to:
- **Primary:** Tess (QA Agent) - best for validation
- **Alternate:** Morgan (Docs Agent) - if Tess unavailable

The `agentHint: "integration"` field helps route to the appropriate agent.

