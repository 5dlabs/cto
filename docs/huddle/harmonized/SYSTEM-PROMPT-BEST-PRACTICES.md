# System Prompt Best Practices

**Status**: Consensus Achieved  
**Date**: December 5, 2025  
**Contributors**: Claude Opus 4, Composer (Sonnet 4.5), Gemini 3 Pro Preview, Grok Code Fast 1, Sonnet 4.5

---

## Executive Summary

Research consensus on system prompt optimization:

| Principle | Recommendation | Evidence |
|-----------|---------------|----------|
| **Length** | < 150 lines | LLM instruction-following research |
| **Instructions** | < 100 custom | Claude Code has ~50 built-in |
| **Structure** | WHAT → WHY → HOW | Industry best practice |
| **Disclosure** | Progressive | Reduces context bloat |
| **Style Rules** | Use linters, not prompts | "Never send an LLM to do a linter's job" |

---

## The Science

### Instruction Following Limits

From academic research cited by HumanLayer:

> **Frontier LLMs can follow ~150-200 instructions with reasonable consistency.**

**Key findings:**
- More instructions = uniform degradation (ALL instructions followed worse)
- Position bias: Beginning and end of prompts get more attention
- Smaller models: Exponential decay in instruction-following
- Larger models: Linear decay (more graceful)

### Implications for AGENTS.md

| Component | Instructions | Notes |
|-----------|-------------|-------|
| Claude Code System Prompt | ~50 | Built-in, not modifiable |
| Your AGENTS.md | ~100 max | Leaves buffer for user messages |
| **Total Budget** | ~150 | Hard limit based on research |

**Bottom line**: Every line in AGENTS.md competes for limited instruction-following capacity.

---

## Content Guidelines

### ✅ WHAT TO INCLUDE

```markdown
## Build & Test Commands
- Build: `cargo build --release`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`

## Architecture Overview
[1-2 paragraphs max]

## Project Structure
├── src/           # Source code
├── tests/         # Integration tests
└── docs/          # Documentation

## Conventions
- [Only non-obvious conventions]
- [Things not enforced by linters]

## Security
- API keys in environment variables
- Never commit secrets

## Git Workflow
- Branch from main
- PR required for all changes
```

### ❌ WHAT TO AVOID

| Anti-Pattern | Why | Alternative |
|-------------|-----|-------------|
| Code style guidelines | Use linters | `cargo fmt`, ESLint, Prettier |
| Long code snippets | Bloats context | Reference files with line numbers |
| Task-specific instructions | Only sometimes relevant | Progressive disclosure |
| Duplicated README content | Redundant | Link to README |
| Auto-generated content | Usually bloated | Craft intentionally |

---

## Progressive Disclosure Pattern

### The Problem

Dumping everything into AGENTS.md:
- Exceeds instruction limits
- Most content irrelevant to current task
- Agents ignore or misapply instructions

### The Solution

Tell agents **WHERE to find information**, not everything upfront:

```markdown
# MyProject

## Quick Reference
- Build: `npm run build`
- Test: `npm test`

## Detailed Guides
When working on specific areas, read the relevant guide:
- Building: `docs/agents/building.md`
- Testing: `docs/agents/testing.md`  
- Architecture: `docs/agents/architecture.md`
- Database: `docs/agents/database.md`

Read the relevant guide BEFORE starting work on that area.
```

### Benefits

1. **Concise AGENTS.md**: Under 60 lines
2. **On-demand context**: Agents load when needed
3. **Easier maintenance**: Docs stay in one place
4. **No duplication**: Single source of truth

---

## Hierarchical Organization

### For Monorepos

```
project/
├── AGENTS.md                    # High-level: structure, commands
├── packages/
│   ├── api/
│   │   └── AGENTS.md           # API-specific: endpoints, auth
│   ├── web/
│   │   └── AGENTS.md           # Frontend-specific: components
│   └── shared/
│       └── AGENTS.md           # Shared: utilities, types
```

### Discovery Rule

**Closest file to edited code wins.**

All 6 CLIs support this pattern (native or configurable).

---

## System Prompt vs Agent Identity

### Three-Tier Architecture

```
┌─────────────────────────────────────────┐
│ Layer 1: System Prompt                  │
│ (CLI-provided or via flag)              │
│ - Agent personality                     │
│ - Tool capabilities                     │
│ - Output format                         │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Layer 2: AGENTS.md                      │
│ (Auto-discovered)                       │
│ - Project conventions                   │
│ - Build/test commands                   │
│ - Architecture overview                 │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│ Layer 3: User Message                   │
│ (Task-specific)                         │
│ - Actual task description               │
│ - Files to modify                       │
│ - Expected outcomes                     │
└─────────────────────────────────────────┘
```

### Separation of Concerns

| Layer | Contains | Example |
|-------|----------|---------|
| **System Prompt** | WHO the agent is | "You are Rex, a Rust expert..." |
| **AGENTS.md** | WHAT the project is | "This is a Rust + TypeScript monorepo..." |
| **User Message** | WHAT to do | "Implement feature X in file Y" |

---

## CLI-Specific System Prompt Mechanisms

### Claude Code (Most Flexible)

| Flag | Behavior | When to Use |
|------|----------|-------------|
| `--system-prompt` | Replaces entire default | Complete control |
| `--append-system-prompt` | Adds to default | **Recommended** |
| `--system-prompt-file` | Replaces with file | Print mode only |

```bash
# Recommended: Append to preserve built-in capabilities
claude --append-system-prompt "You are Rex, a Rust expert. Always use idiomatic Rust."
```

### Other CLIs

Most CLIs don't have system prompt flags—they rely entirely on AGENTS.md:

| CLI | System Prompt Control | Agent Identity |
|-----|----------------------|----------------|
| Codex | None (file-based only) | AGENTS.md |
| Cursor | None (rules-based) | AGENTS.md or .cursor/rules |
| Gemini | `GEMINI_SYSTEM_MD` env var | GEMINI.md or configured |
| OpenCode | Agent config JSON/Markdown | `.opencode/agent/` |
| Aider | None (file-based) | `--read` flag |

---

## Template Recommendations

### AGENTS.md Template (< 60 lines)

```markdown
# {{project_name}}

{{project_description}}

## Commands
- Build: `{{build_command}}`
- Test: `{{test_command}}`
- Lint: `{{lint_command}}`

## Structure
{{project_structure}}

## Conventions
{{#each conventions}}
- {{this}}
{{/each}}

## Before Committing
1. Run `{{lint_command}}`
2. Run `{{test_command}}`
3. Ensure all checks pass

## Detailed Guides
{{#each guides}}
- {{name}}: `{{path}}`
{{/each}}
```

### Agent Identity Template

```markdown
# {{agent_name}}

You are {{agent_name}}, {{agent_role}}.

## Specialization
{{agent_specialization}}

## Core Rules
{{#each agent_rules}}
- {{this}}
{{/each}}

## Tools
You have access to: {{agent_tools}}

## Output Style
{{agent_output_style}}
```

---

## Prompt Caching Optimization

From @dejavucoder's research:

> **Prompt caching is the most bang-for-buck optimization for LLM workflows.**

### Best Practices for Cache Hits

1. **Stable prefix**: Put unchanging content at the START
2. **AGENTS.md first**: Load before task-specific content
3. **Consistent formatting**: Same structure every time
4. **Avoid timestamps**: Dynamic content breaks cache

### Container Image Implications

```
# Cached (stable, loaded first)
1. AGENTS.md content
2. Agent identity
3. Project context

# Not cached (dynamic, loaded last)
4. Task-specific prompt
5. User message
```

---

## Research References

### Academic Papers
- **AI Agents That Matter** (arXiv:2407.01502) - Cost vs accuracy trade-offs
- **Instruction Following Research** - 150-200 instruction limit

### Industry Best Practices
- **HumanLayer Blog**: https://www.humanlayer.dev/blog/writing-a-good-claude-md
- **Anthropic Best Practices**: https://www.anthropic.com/engineering/claude-code-best-practices
- **AGENTS.md Spec**: https://agents.md/

### X Posts (Research Resources)
- @omarsar0: Deep Agents architecture patterns
- @rohanpaul_ai: Anthropic multi-agent research system
- @googleaidevs: Gemini 3 Pro system instructions (~5% improvement)
- @dejavucoder: Prompt caching optimization
- @sayashk: CORE-Bench solved with scaffold improvements

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────┐
│                 SYSTEM PROMPT CHECKLIST                 │
├─────────────────────────────────────────────────────────┤
│ □ Under 150 lines                                       │
│ □ Under 100 custom instructions                         │
│ □ Build/test commands present                           │
│ □ Architecture overview (1-2 paragraphs)                │
│ □ No code style rules (use linters)                     │
│ □ Progressive disclosure for details                    │
│ □ Stable content at start (for caching)                 │
│ □ Tested across all target CLIs                         │
└─────────────────────────────────────────────────────────┘
```

