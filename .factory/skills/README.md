# Agent Skills

These skills provide domain-specific knowledge for AI coding agents working on the CTO platform. Skills are CLI-agnostic and work with any agent CLI.

## Available Skills

### Platform Skills

| Skill | Description | When to Use |
|-------|-------------|-------------|
| **cto-platform** | Play workflow orchestration, intake processing, MCP tools | Managing development lifecycle from PRD to deployed code |
| **healer** | Detection patterns, API endpoints, dual-model architecture | Monitoring Play workflows, debugging agent failures |
| **linear-agent-api** | Sessions, activities, signals, best practices | Working with Linear agent integration |
| **telemetry** | Prometheus metrics, Loki logs, Grafana dashboards | Querying logs, metrics, debugging via observability |

### Quality & Process Skills

| Skill | Description | When to Use |
|-------|-------------|-------------|
| **systematic-debugging** | 4-phase root cause process, evidence gathering, hypothesis testing | Any bug, test failure, or unexpected behavior - BEFORE proposing fixes |
| **test-driven-development** | RED-GREEN-REFACTOR cycle with strict enforcement | Implementing any feature or bugfix - BEFORE writing code |
| **verification-before-completion** | Evidence-based completion claims | BEFORE claiming any work is complete, fixed, or passing |

### Technology Skills

| Skill | Description | When to Use | Agents |
|-------|-------------|-------------|--------|
| **react-best-practices** | React/Next.js performance optimization (45+ rules) | Writing, reviewing, or refactoring React/Next.js code | Blaze, Spark, Tap |

## Skill Format

Each skill is a markdown file (`SKILL.md`) with YAML frontmatter:

```yaml
---
name: skill-name
description: What this skill does and when to use it.
---
```

## Iron Laws

Several skills define "Iron Laws" - non-negotiable rules that must always be followed:

| Skill | Iron Law |
|-------|----------|
| **systematic-debugging** | NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST |
| **test-driven-development** | NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST |
| **verification-before-completion** | NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE |

## Synchronization

These skills are the source of truth and sync to `.claude/skills/` for Claude Code compatibility. The `.claude/` directory is gitignored for local customization, while `.factory/skills/` is tracked.

## Adding New Skills

1. Create a new directory under `.factory/skills/`
2. Add a `SKILL.md` file with proper frontmatter
3. Include clear sections for:
   - When to use
   - Key concepts/rules (with Iron Laws if applicable)
   - Examples (good and bad)
   - Red flags (when to stop and reconsider)
   - Common rationalizations to avoid
4. Copy to `.claude/skills/` for local use

## Skill Design Principles

Based on research from [obra/superpowers](https://github.com/obra/superpowers):

1. **Description = When to Use, NOT What the Skill Does**
   - Descriptions should only contain triggers ("Use when...")
   - Workflow summaries in descriptions cause agents to skip the skill body

2. **Iron Laws Over Guidelines**
   - Non-negotiable rules are more effective than suggestions
   - "Never" is clearer than "try to avoid"

3. **Red Flags Section**
   - List signals that indicate the agent is about to violate the skill
   - Helps catch rationalization before it happens

4. **Rationalization Prevention**
   - Table mapping common excuses to reality
   - Preemptively addresses "but this is different because..."
