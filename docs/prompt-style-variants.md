# Prompt Style Variants (A/B Testing)

This document describes the prompt style variant system, which allows A/B testing between different prompt formats for agent tasks.

## Background

Inspired by the [Ralph Wiggum technique](https://ghuntley.com/ralph/) and findings from the [YC Agents hackathon](https://github.com/repomirrorhq/repomirror/blob/main/repomirror.md), this feature enables testing minimal, declarative prompts against the standard detailed prompts.

**Key insight from Ralph:** Simpler prompts (~103 words) often outperform verbose prompts (~1,500 words). The hypothesis is that overly detailed prompts can make agents "slower and dumber."

## Available Variants

### Standard (Default)
The existing detailed prompts with:
- Role definitions and specializations
- Tool usage priorities
- Code examples and patterns
- Detailed checklists
- ~150-200 lines per agent

### Minimal (`prompt_style: minimal`)
Ralph-inspired concise prompts with:
- Brief role statement
- Core constraints only
- Declarative "Definition of Done"
- No code examples (trusts model knowledge)
- ~40-50 lines per agent

Currently implemented for:
- **Rex** (`templates/agents/rex/coder-minimal.md.hbs`) - Rust backend
- **Blaze** (`templates/agents/blaze/coder-minimal.md.hbs`) - React/Next.js frontend
- **Nova** (`templates/agents/nova/coder-minimal.md.hbs`) - Bun/Effect backend
- **Grizz** (`templates/agents/grizz/coder-minimal.md.hbs`) - Go backend
- **Tess** (`templates/agents/tess/test-minimal.md.hbs`) - Testing

## Usage

### Via Linear Label

Add the label `cto:prompt:minimal` to a Linear issue before delegating to the CTO agent:

```
Labels: cto:prompt:minimal
```

### Via CodeRun Spec

Set the `promptStyle` field in the CodeRun resource:

```yaml
apiVersion: agents.platform/v1
kind: CodeRun
spec:
  promptStyle: "minimal"
  # ... other fields
```

### Via Frontmatter (Future)

Can be added to issue description frontmatter:

```yaml
---
cto:
  prompt_style: minimal
---
```

## A/B Testing Workflow

1. **Baseline Run**: Execute a task with standard prompts (no label)
2. **Variant Run**: Execute the same task with `cto:prompt:minimal` label
3. **Compare Results**:
   - Time to completion
   - Token usage / cost
   - Output quality (code correctness, style)
   - PR review feedback

### Metrics to Track

| Metric | How to Measure |
|--------|----------------|
| Completion time | Workflow duration from Argo |
| Token usage | Claude API logs / billing |
| Code quality | Clippy warnings, test coverage |
| Iteration count | Number of retries needed |
| Human intervention | Manual fixes required |

## Implementation Details

### Configuration Flow

```
Linear Label (cto:prompt:minimal)
        ↓
CtoConfig.prompt_style
        ↓
CodeRunSpec.prompt_style
        ↓
get_agent_system_prompt_template()
        ↓
agents/{agent}/{job}-minimal.md.hbs
```

### Template Resolution

In `crates/controller/src/tasks/code/templates.rs`:

```rust
// Check for prompt style variant
let suffix = code_run
    .spec
    .prompt_style
    .as_ref()
    .filter(|s| *s == "minimal")
    .map_or("", |_| "-minimal");

format!("agents/{agent}/{job}{suffix}.md.hbs")
```

### Adding New Variants

To add a minimal variant for another agent:

1. Create `templates/agents/{agent}/{job}-minimal.md.hbs`
2. Follow the minimal template pattern:
   - Brief role statement
   - Essential constraints only
   - Clear Definition of Done
   - No code examples

## Minimal Template Pattern

```markdown
# {Agent} - {Role}

You are {Agent}. Your job is to {primary task} in `task/`.

## Constraints
- {Essential constraint 1}
- {Essential constraint 2}
- {Essential constraint 3}

## Definition of Done
- {Acceptance criteria reference}
- {Required commands/checks}
- {PR requirements}

## Task Context
- Task ID: {{task_id}}
- Service: {{service}}
- Branch: feature/task-{{task_id}}-{job}

Read `task/` directory and implement.
```

## Skills Integration

The Ralph technique is available as a skill at `templates/skills/workflow/ralph-technique/SKILL.md`. This skill:

- Documents the Ralph philosophy and loop-based execution
- Provides the "signs on the playground" tuning methodology
- Is mapped as an optional skill for Rex, Blaze, Nova, Grizz, and Tess

Load the skill via triggers: `minimal`, `ralph`, `loop`, `autonomous`, `simple`

## Future Enhancements

- [x] ~~Add minimal variants for other agents (Blaze, Tess, etc.)~~ ✓ Done
- [ ] Implement metrics collection for automated comparison
- [ ] Add `verbose` variant for complex tasks requiring more guidance
- [ ] Dashboard for comparing variant performance over time
- [ ] Add minimal variants for remaining agents (Bolt, Atlas, Cipher)

## References

- [Ralph Wiggum technique](https://ghuntley.com/ralph/) - Original concept by Geoff Huntley
- [RepoMirror hackathon writeup](https://github.com/repomirrorhq/repomirror/blob/main/repomirror.md) - YC hackathon results
- [Brief History of Ralph](https://www.humanlayer.dev/blog/brief-history-of-ralph) - HumanLayer's experience with the technique
