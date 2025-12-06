# System Prompt Research: CLI Comparison and Findings

**Model**: Claude Sonnet 4.5  
**Date**: December 5, 2025  
**Status**: Complete  
**Research Scope**: How 6 major CLIs handle system prompts and agent identity

---

## Executive Summary

This research examines how six major coding CLIs (Claude Code, Factory/Droid, OpenAI Codex, Cursor, Gemini CLI, and OpenCode) handle system prompts and agent configuration. Key findings:

1. **AGENTS.md is emerging as the cross-CLI standard** - Supported by all 6 CLIs with varying levels of integration
2. **Three primary approaches exist**: CLI flags, configuration files, and markdown-based instructions
3. **Strong convergence** toward AGENTS.md format for agent instructions across the ecosystem
4. **System prompts are distinct from agent identity** - Most CLIs separate these concerns
5. **Best practices emphasize brevity** - <300 lines, <150 optimal, highly focused content

**Recommendation**: Adopt AGENTS.md as primary format with CLI-specific wrappers only when necessary. This maximizes portability and reduces maintenance overhead.

---

## Table of Contents

1. [The AGENTS.md Standard](#the-agentsmd-standard)
2. [CLI-by-CLI Analysis](#cli-by-cli-analysis)
3. [Comparison Matrix](#comparison-matrix)
4. [Best Practices from Research](#best-practices-from-research)
5. [Harmonization Strategy](#harmonization-strategy)
6. [Container & Prompt Structure Recommendations](#container--prompt-structure-recommendations)
7. [Key Research Papers](#key-research-papers)

---

## The AGENTS.md Standard

### What is AGENTS.md?

AGENTS.md is an **open, cross-platform format** for providing instructions to AI coding agents. Initiated by OpenAI (for Codex), it has been adopted by 20,000+ open-source projects and is supported by all major coding CLIs.

**Official Website**: https://agents.md/

### Core Principles

1. **Complement, don't replace README.md** - README is for humans, AGENTS.md is for agents
2. **Plain Markdown** - No special syntax or metadata (with some CLI extensions)
3. **Hierarchical discovery** - Nearest file to edited code takes precedence
4. **Nested support** - Monorepos can have multiple AGENTS.md files per package

### Standard Structure

```markdown
# Build & Test
- Build: `npm run build`
- Test: `npm test`

# Architecture Overview
Brief description of major modules and data flow

# Conventions & Patterns
Naming, folder structure, code style

# Security
API keys, endpoints, auth flows

# Git Workflows
Branching strategy, commit conventions
```

### Discovery Hierarchy (typical)

1. Current working directory
2. Nearest parent directory up to repo root
3. Subdirectory-specific files
4. User home directory override (CLI-specific)

### Supported Tools

**Full Support (20+ tools)**:
- Claude Code (CLAUDE.md or AGENTS.md)
- OpenAI Codex (AGENTS.md)
- Factory/Droid (AGENTS.md)
- Cursor (AGENTS.md)
- Gemini CLI (configurable)
- OpenCode (AGENTS.md or custom path)
- Aider, Zed, Warp, VS Code, GitHub Copilot, Devin, Jules, Phoenix, RooCode, and more

---

## CLI-by-CLI Analysis

### 1. Claude Code CLI

**System Prompt Mechanism**: CLI Flags  
**Agent Identity**: CLAUDE.md or AGENTS.md  
**Documentation**: https://code.claude.com/docs/en/cli-reference

#### System Prompt Options

Claude Code provides **three distinct flags** for system prompt customization:

| Flag | Behavior | Modes | Use Case |
|------|----------|-------|----------|
| `--system-prompt` | Replaces entire default prompt | Interactive + Print | Complete control over Claude's behavior |
| `--system-prompt-file` | Replaces with file contents | Print only | Load prompts from files for version control |
| `--append-system-prompt` | Appends to default prompt | Interactive + Print | Add instructions while keeping defaults |

**Examples**:

```bash
# Complete replacement
claude --system-prompt "You are a Python expert who only writes type-annotated code"

# Load from file
claude -p --system-prompt-file ./prompts/code-review.txt "Review this PR"

# Append (recommended)
claude --append-system-prompt "Always use TypeScript and include JSDoc comments"
```

#### Agent Configuration

**Subagents via --agents flag** (JSON format):

```bash
claude --agents '{
  "code-reviewer": {
    "description": "Expert code reviewer. Use proactively after code changes.",
    "prompt": "You are a senior code reviewer. Focus on code quality, security, and best practices.",
    "tools": ["Read", "Grep", "Glob", "Bash"],
    "model": "sonnet"
  }
}'
```

#### CLAUDE.md / AGENTS.md

- **Location**: Project root (discovered automatically)
- **Format**: Plain Markdown
- **Behavior**: Injected into context with system reminder:

```
<system-reminder>
IMPORTANT: this context may or may not be relevant to your tasks.
You should not respond to this context unless it is highly relevant to your task.
</system-reminder>
```

**Key Insight**: Claude Code tells the model to **ignore CLAUDE.md if not relevant** to reduce noise. This means:
- Keep content universally applicable
- Avoid task-specific instructions that only apply sometimes
- Use progressive disclosure (reference other docs rather than including everything)

#### Best Practices (from Anthropic)

- **<300 lines** (ideally <60 lines)
- Focus on WHAT (tech stack, structure), WHY (purpose), HOW (commands, verification)
- Avoid code style guidelines (use linters instead)
- Don't use `/init` or auto-generation
- Progressive disclosure - link to specialized docs rather than embedding

**Official Guide**: https://www.humanlayer.dev/blog/writing-a-good-claude-md

---

### 2. Factory (Droid) CLI

**System Prompt Mechanism**: AGENTS.md  
**Agent Identity**: AGENTS.md  
**Documentation**: https://docs.factory.ai/cli/configuration/agents-md

#### Core Philosophy

Factory is a **strong advocate** for the AGENTS.md standard and positions it as:
- Briefing packet for AI agents
- Separate from README.md (which is for humans)
- Cross-platform standard that works with all agents

#### File Discovery

Priority order:
1. `./AGENTS.md` in current working directory
2. Nearest parent directory up to repo root
3. Subdirectory-specific AGENTS.md files
4. `~/.factory/AGENTS.md` (personal override)

**Multiple files coexist** - closer files take precedence.

#### Recommended Structure

```markdown
# MyProject

Overview of the project

## Core Commands
• Type-check and lint: `pnpm check`
• Auto-fix style: `pnpm check:fix`
• Run full test suite: `pnpm test --run --no-color`

## Project Layout
├─ client/ → React + Vite frontend
├─ server/ → Express backend

## Development Patterns & Constraints
• TypeScript strict mode, single quotes, trailing commas
• 100-char line limit
• Tests first when fixing logic bugs

## Git Workflow Essentials
1. Branch from `main` with descriptive name
2. Run `pnpm check` locally before committing
3. Force pushes allowed only on feature branch

## Evidence Required for Every PR
- All tests green
- Lint & type check pass
- Proof artifact (failing test → now passes)
```

#### Best Practices (from Factory)

- **≤150 lines** (shorter is better)
- Concrete commands wrapped in backticks
- Update alongside code (treat like code in PRs)
- One source of truth (link to docs, don't duplicate)
- Make requests precise

#### Integration Modes

- **Specification Mode**: Uses AGENTS.md for planning context
- **Auto-Run**: Relies on accurate build/test commands from AGENTS.md

---

### 3. OpenAI Codex CLI

**System Prompt Mechanism**: AGENTS.md  
**Agent Identity**: AGENTS.md  
**Documentation**: https://developers.openai.com/codex/guides/agents-md/

#### Discovery Mechanism

Codex builds an **instruction chain** on every run:

1. **Global scope**: 
   - Checks `~/.codex/` (or `$CODEX_HOME`)
   - If `AGENTS.override.md` exists → use it
   - Otherwise → use `AGENTS.md`

2. **Project scope**:
   - Walks from repo root to current directory
   - In each directory: `AGENTS.override.md` > `AGENTS.md` > fallback filenames

3. **Merge order**: 
   - Files concatenated root → leaf
   - Later files override earlier guidance

#### Configuration

**Fallback filenames** (in `~/.codex/config.toml`):

```toml
project_doc_fallback_filenames = ["TEAM_GUIDE.md", ".agents.md"]
project_doc_max_bytes = 65536
```

**Multiple profiles** via `CODEX_HOME`:

```bash
CODEX_HOME=$(pwd)/.codex codex exec "List active instruction sources"
```

#### Override Pattern

```
~/.codex/
  AGENTS.md              # Base global guidance
  AGENTS.override.md     # Temporary override

project/
  AGENTS.md              # Project-wide
  services/
    AGENTS.md            # Ignored if override exists
    AGENTS.override.md   # Takes precedence
```

#### Key Features

- **Empty files ignored** - Must contain content
- **Size limit** - Default 32 KiB, configurable
- **No caching** - Rebuilds instruction chain on every run
- **Validation**: Check active sources with:

```bash
codex --ask-for-approval never "Summarize the current instructions."
```

---

### 4. Cursor CLI

**System Prompt Mechanism**: Rules system  
**Agent Identity**: AGENTS.md or .cursor/rules  
**Documentation**: https://cursor.com/docs/context/rules

#### Four Rule Types

1. **Project Rules** - `.cursor/rules/*.mdc` (version controlled)
2. **User Rules** - Global preferences in Cursor Settings
3. **Team Rules** - Organization-wide (Team/Enterprise plans)
4. **AGENTS.md** - Simple markdown alternative

#### Project Rules (MDC Format)

`.cursor/rules` uses **MDC** (Markdown with metadata):

```markdown
---
globs: ["*.ts", "*.tsx"]
alwaysApply: false
---

- Use our internal RPC pattern when defining services
- Always use snake_case for service names.

@service-template.ts
```

**Rule Types**:
- **Always Apply** - Every chat session
- **Apply Intelligently** - When Agent decides it's relevant
- **Apply to Specific Files** - When file matches glob pattern
- **Apply Manually** - When @-mentioned

#### AGENTS.md Support

Cursor supports AGENTS.md as a **simpler alternative** to `.cursor/rules`:

```markdown
# Project Instructions

## Code Style
- Use TypeScript for all new files
- Prefer functional components in React

## Architecture
- Follow the repository pattern
- Keep business logic in service layers
```

**Location**: Project root and subdirectories

**Advantages over .cursor/rules**:
- Plain markdown (no metadata)
- Cross-CLI compatibility
- Simpler for straightforward use cases

#### Nested Rules

Organize rules in subdirectories:

```
project/
  .cursor/rules/        # Project-wide
  backend/
    server/
      .cursor/rules/    # Backend-specific
  frontend/
    .cursor/rules/      # Frontend-specific
```

#### Team Rules (Enterprise)

- Managed from Cursor dashboard
- Can be enforced (required) or optional
- Plain text, no MDC format
- Included in model context for Agent (Chat)

**Precedence**: Team Rules → Project Rules → User Rules

#### Migration Path

`.cursorrules` (legacy) is still supported but **deprecated**. Recommended migration:
1. Migrate to `.cursor/rules` for structured needs
2. Or migrate to `AGENTS.md` for simplicity and portability

---

### 5. Gemini CLI

**System Prompt Mechanism**: Configuration file  
**Agent Identity**: Configurable (defaults to .gemini/context.md)  
**Documentation**: https://developers.google.com/gemini-code-assist/docs/gemini-cli

#### Overview

Gemini CLI is Google's **open-source AI agent** with ReAct (Reason and Act) loop. It focuses on:
- Complex use cases (bug fixing, features, test coverage)
- Built-in tools + MCP servers
- Versatile beyond coding (content generation, research, task management)

#### Agent Mode Features

Available in VS Code via Gemini Code Assist:
- Model Context Protocol (MCP) servers
- Commands: `/memory`, `/stats`, `/tools`, `/mcp`
- Yolo mode (auto-approve)
- Built-in tools (grep, terminal, file operations)
- Web search and fetch

#### AGENTS.md Configuration

**Settings location**: `.gemini/settings.json`

```json
{
  "contextFileName": "AGENTS.md"
}
```

**Default**: Gemini CLI looks for context file at `.gemini/context.md` but can be configured to use `AGENTS.md` for cross-CLI compatibility.

#### Privacy

- **Standard/Enterprise**: Covered by Cloud security/privacy
- **Individuals**: Separate privacy notice applies

#### Integration Points

- Available in Cloud Shell without setup
- Quota shared with Gemini Code Assist agent mode
- ReAct loop for multi-step task execution

---

### 6. OpenCode CLI

**System Prompt Mechanism**: Agent configuration (JSON or Markdown)  
**Agent Identity**: Agent system (primary + subagents)  
**Documentation**: https://opencode.ai/docs/agents/

#### Agent System

OpenCode has a **sophisticated agent system** with two types:

1. **Primary Agents** - Main assistants you interact with directly
   - Cycle with Tab key or `switch_agent` keybind
   - Built-in: Build (default, all tools), Plan (restricted)

2. **Subagents** - Specialized assistants invoked by primary agents
   - Automatic invocation based on descriptions
   - Manual invocation via @-mention
   - Built-in: General (research), Explore (codebase search)

#### Configuration Methods

**JSON Configuration** (`opencode.json`):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "agent": {
    "build": {
      "mode": "primary",
      "model": "anthropic/claude-sonnet-4-20250514",
      "prompt": "{file:./prompts/build.txt}",
      "tools": {
        "write": true,
        "edit": true,
        "bash": true
      }
    },
    "code-reviewer": {
      "description": "Reviews code for best practices and potential issues",
      "mode": "subagent",
      "prompt": "You are a code reviewer. Focus on security, performance, and maintainability.",
      "tools": {
        "write": false,
        "edit": false
      }
    }
  }
}
```

**Markdown Configuration**:

Place in:
- Global: `~/.config/opencode/agent/`
- Project: `.opencode/agent/`

Filename becomes agent name (e.g., `review.md` → `review` agent):

```markdown
---
description: Reviews code for quality and best practices
mode: subagent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.1
tools:
  write: false
  edit: false
  bash: false
---

You are in code review mode. Focus on:
- Code quality and best practices
- Potential bugs and edge cases
- Performance implications
- Security considerations
```

#### Permission System

Fine-grained control over tool usage:

```json
{
  "permission": {
    "edit": "ask",  // "ask", "allow", or "deny"
    "bash": {
      "git push": "ask",
      "git status": "allow",
      "git *": "ask",
      "*": "deny"
    },
    "webfetch": "deny"
  }
}
```

#### AGENTS.md Support

OpenCode also supports AGENTS.md via:

```json
{
  "rules": "./AGENTS.md"
}
```

Or place in project root for auto-discovery (similar to other CLIs).

#### Temperature Control

Per-agent temperature settings:

```json
{
  "agent": {
    "analyze": { "temperature": 0.1 },  // Deterministic
    "build": { "temperature": 0.3 },    // Balanced
    "brainstorm": { "temperature": 0.7 } // Creative
  }
}
```

**Defaults**: 0 for most models, 0.55 for Qwen models

#### Create Agent Command

```bash
opencode agent create
```

Interactive workflow:
1. Choose location (global or project)
2. Provide description
3. Generate prompt and identifier
4. Select available tools
5. Creates markdown file with configuration

---

## Comparison Matrix

| Feature | Claude Code | Factory | Codex | Cursor | Gemini CLI | OpenCode |
|---------|------------|---------|--------|---------|-----------|----------|
| **AGENTS.md Support** | ✅ Yes (CLAUDE.md) | ✅ Yes (Primary) | ✅ Yes (Primary) | ✅ Yes (Simple alt) | ✅ Yes (Config) | ✅ Yes (Rules) |
| **System Prompt Flags** | ✅ 3 flags | ❌ No | ❌ No | ❌ No | ❌ No | Via config |
| **Nested Files** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | Unknown | ✅ Yes |
| **Override Pattern** | ❌ No | ❌ No | ✅ Yes | ❌ No | Unknown | Via config |
| **Agent Configuration** | JSON flag | AGENTS.md | AGENTS.md | .mdc or AGENTS.md | Config file | JSON + Markdown |
| **Metadata Support** | No | No | No | Yes (.mdc) | No | Yes (frontmatter) |
| **File Size Limit** | None specified | ≤150 lines rec | 32 KiB (config) | None specified | Unknown | None specified |
| **Recommended Length** | <300 lines (<60 ideal) | ≤150 lines | No limit | <500 lines | Unknown | No limit |
| **Glob Patterns** | ❌ No | ❌ No | ❌ No | ✅ Yes | Unknown | ✅ Yes (tools) |
| **Temperature Control** | Per-model | No | No | No | No | ✅ Per-agent |
| **Permission System** | Tool allowlist | No | No | No | No | ✅ Fine-grained |
| **Subagent System** | ✅ JSON config | No | No | No | No | ✅ Built-in |
| **Fallback Filenames** | No | No | ✅ Configurable | No | ✅ Configurable | ✅ Via rules |
| **Team/Org Rules** | No | No | No | ✅ Enterprise | No | No |
| **Progressive Disclosure** | ✅ Encouraged | ✅ Encouraged | Links | Links | Unknown | Unknown |
| **Auto-ignore if irrelevant** | ✅ Yes | Unknown | No | Unknown | Unknown | Unknown |
| **MCP Integration** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |

### Key Takeaways

1. **AGENTS.md is universal** - All 6 CLIs support it (some via configuration)
2. **Claude Code most flexible** - Only CLI with dedicated system prompt flags
3. **Cursor most structured** - .mdc format with metadata and glob patterns
4. **OpenCode most sophisticated** - Advanced agent system with permissions and temperature control
5. **Codex most explicit** - Clear override pattern and instruction chain
6. **Factory strongest advocate** - Best documentation for AGENTS.md standard

---

## Best Practices from Research

### Length and Structure

**Consensus across all CLIs**:
- **Optimal**: <150 lines
- **Maximum**: <300 lines  
- **Ideal**: <60 lines (HumanLayer/Claude Code)

**Why brevity matters** (from research):
1. LLMs can follow ~150-200 instructions reliably (frontier models)
2. More instructions = uniform degradation in following ALL of them
3. Claude Code system prompt already contains ~50 instructions
4. Context window bloat reduces performance on actual task

### Content Guidelines

**What to Include**:
- ✅ Build and test commands (exact, copy-pasteable)
- ✅ Architecture overview (brief, 1 paragraph)
- ✅ Project structure (folder layout, what goes where)
- ✅ Conventions that aren't obvious from code
- ✅ Security considerations (API keys, auth flows)
- ✅ Git workflow (branching, commit format, PR requirements)

**What to Avoid**:
- ❌ Code style guidelines (use linters instead)
- ❌ Task-specific instructions (only sometimes relevant)
- ❌ Duplicated content from README or other docs
- ❌ Auto-generated content (craft intentionally)
- ❌ Long code snippets (reference files with line numbers instead)

### Progressive Disclosure

**Principle**: Don't tell agents everything upfront. Tell them **how to find** information when needed.

**Pattern**:

```markdown
# Development Guide

For detailed information, see:
- Building the project: `docs/agents/building.md`
- Running tests: `docs/agents/testing.md`
- Code conventions: `docs/agents/conventions.md`
- Database schema: `docs/agents/schema.md`

When working on specific tasks, read the relevant guide first.
```

**Benefits**:
- Keeps AGENTS.md concise
- Agents only load context when needed
- Easier to maintain (docs stay in one place)
- Avoids context window bloat

### Hierarchical Organization

**For monorepos and large projects**:

```
project/
  AGENTS.md                    # High-level: what, why, structure
  packages/
    api/
      AGENTS.md                # API-specific: endpoints, auth, database
    web/
      AGENTS.md                # Frontend-specific: components, routing
    shared/
      AGENTS.md                # Shared code: utils, types
```

**Discovery rule**: Closest file to edited code wins.

### Example Patterns

**Minimal (good for small projects)**:

```markdown
# MyProject

Simple API server with React frontend.

## Commands
- Install: `npm install`
- Dev: `npm run dev`
- Test: `npm test`
- Build: `npm run build`

## Structure
- `src/api/` - Express backend
- `src/web/` - React frontend
- `src/shared/` - Shared utilities

## Testing
Run full suite before committing. Add tests for new features.
```

**Comprehensive (monorepo)**:

```markdown
# MyCompany Platform

Monorepo with 12 microservices and 3 frontend apps.

## Getting Started
- Install: `pnpm install`
- Navigate: `pnpm dlx turbo run where <project>`
- Dev: `pnpm dev --filter <project>`

## Project Structure
See `docs/architecture.md` for service map.

Each package has its own AGENTS.md with specific guidance.

## Commands by Task
- Test one package: `pnpm turbo run test --filter <project>`
- Lint: `pnpm lint --filter <project>`
- Build: `pnpm turbo run build --filter <project>`

## Before PR
1. Run `pnpm test` (must be green)
2. Run `pnpm lint`
3. Title format: `[project] description`
4. Include proof: test evidence or screenshot

## Security
- API keys in .env.local (never commit)
- Use Vault for production secrets
- See security team guide: `docs/security.md`
```

---

## Harmonization Strategy

### Goal

Create a **single source of truth** for agent instructions that works across all 6 CLIs with minimal CLI-specific customization.

### Recommended Approach

**Primary Format**: AGENTS.md (standard markdown)

```
project/
  AGENTS.md                    # Main instructions (works for all CLIs)
  .agents/                     # CLI-specific extensions (optional)
    claude/
      subagents.json           # Claude-specific subagent definitions
    cursor/
      rules/                   # Cursor .mdc rules if glob patterns needed
    opencode/
      agents/                  # OpenCode agent configurations
  docs/
    agents/                    # Progressive disclosure docs
      building.md
      testing.md
      architecture.md
```

### Migration Path

**Phase 1**: Consolidate to AGENTS.md
- Move all agent instructions to `AGENTS.md` at project root
- Use standard markdown (no metadata)
- Keep <150 lines
- Support nested AGENTS.md for monorepo packages

**Phase 2**: CLI-specific wrappers (only if needed)
- Claude Code: Use `--agents` flag for dynamic subagent configuration
- Cursor: Create .mdc rules only if glob patterns required
- OpenCode: Create agent definitions for advanced features (temperature, permissions)
- Others: Standard AGENTS.md sufficient

**Phase 3**: Container image optimization
- Include AGENTS.md in container working directory
- Mount additional docs at `/workspace/docs/agents/`
- CLI-specific configs in `/etc/<cli>/config.*`

### Symbolic Links for Compatibility

If existing workflows reference `CLAUDE.md` or `.cursorrules`:

```bash
ln -s AGENTS.md CLAUDE.md
ln -s AGENTS.md .cursorrules
```

### Template Structure (Option D - Recommended)

Based on research, the optimal template structure is:

```
templates/
  agents/
    base/
      AGENTS.md.hbs              # Universal agent instructions
    specialized/
      rex-extra.md.hbs           # Agent-specific additions
      blaze-extra.md.hbs
  clis/
    claude/
      config.json.hbs            # Claude-specific agent JSON
    cursor/
      rules.mdc.hbs              # Cursor rules if needed
    opencode/
      agents.json.hbs            # OpenCode agent config
  shared/
    instructions/
      building.md.hbs            # Progressive disclosure docs
      testing.md.hbs
      architecture.md.hbs
```

**Benefits**:
- Single AGENTS.md source of truth
- CLI-specific extensions only when needed
- Progressive disclosure for detailed instructions
- Easy to maintain and version control

---

## Container & Prompt Structure Recommendations

### Container Image Layout

**Recommended structure for agent containers**:

```
/workspace/
  AGENTS.md                      # Auto-discovered by all CLIs
  .agents/                       # CLI configs (if needed)
  docs/
    agents/                      # Progressive disclosure
      building.md
      testing.md
      conventions.md
  /etc/
    claude/
      config.json                # Claude-specific settings
    cursor/
      settings.json
    opencode/
      opencode.json
  /src/                          # Actual codebase
```

**Benefits**:
- CLIs auto-discover AGENTS.md in /workspace
- Additional docs available for progressive disclosure
- CLI-specific configs isolated in /etc
- Single AGENTS.md works for all CLIs

### Prompt Injection Strategy

**Three-tier approach**:

1. **Base System Prompt** (CLI-provided or via flag)
   - Agent personality and capabilities
   - Tool usage guidelines
   - Output format requirements

2. **AGENTS.md** (Project-specific)
   - Project structure and conventions
   - Build/test commands
   - Domain-specific knowledge

3. **User Message** (Task-specific)
   - Actual task description
   - Specific files to modify
   - Expected outcomes

**Example for Claude Code**:

```bash
claude \
  --append-system-prompt "$(cat /etc/agents/rex-identity.txt)" \
  --agent rex \
  --agents "$(cat /etc/agents/rex-config.json)" \
  "Implement the feature described in TASK.md"
```

Where:
- `/etc/agents/rex-identity.txt` = Agent personality
- `/etc/agents/rex-config.json` = Subagent configuration
- `AGENTS.md` = Auto-discovered project instructions
- User message = Task

### Multi-CLI Support Pattern

**Single AGENTS.md + CLI detection**:

```dockerfile
# In container entrypoint
case "$CLI" in
  claude)
    exec claude --append-system-prompt "$(cat /etc/agents/$AGENT-identity.txt)" \
                --agents "$(cat /etc/agents/$AGENT-config.json)" \
                "$@"
    ;;
  codex)
    # Codex reads AGENTS.md automatically
    exec codex "$@"
    ;;
  cursor)
    # Cursor reads .cursor/rules or AGENTS.md
    exec cursor agent "$@"
    ;;
  gemini)
    # Gemini reads .gemini/settings.json contextFileName
    exec gemini "$@"
    ;;
  opencode)
    # OpenCode reads opencode.json and AGENTS.md
    exec opencode "$@"
    ;;
  *)
    # Default to universal AGENTS.md
    exec "$CLI" "$@"
    ;;
esac
```

### Document Generation Best Practices

**For generated documentation consumed by CLIs**:

1. **Keep it modular** - One concept per file
2. **Use progressive disclosure** - Reference from AGENTS.md
3. **Include line numbers** - `file:line-range` for precise references
4. **Avoid duplication** - Single source of truth for each concept
5. **Version control** - Track changes to agent instructions like code

**Example generated structure**:

```
docs/
  agents/
    00-index.md                 # Table of contents
    01-quickstart.md            # New contributor onboarding
    02-architecture.md          # System design
    03-api-patterns.md          # API conventions
    04-database.md              # Schema and queries
    05-testing.md               # Test strategy
    06-deployment.md            # Release process
```

Referenced from AGENTS.md:

```markdown
# MyProject

## Getting Started
See `docs/agents/01-quickstart.md` for setup.

## Architecture
See `docs/agents/02-architecture.md` for system design.

When working on specific areas:
- API changes: Read `docs/agents/03-api-patterns.md`
- Database changes: Read `docs/agents/04-database.md`
- Testing: Read `docs/agents/05-testing.md`
```

---

## Key Research Papers

### AI Agents That Matter (arXiv:2407.01502)

**Authors**: Sayash Kapoor, Benedikt Stroebl, Zachary S. Siegel, Nitya Nadgir, Arvind Narayanan  
**Date**: July 1, 2024  
**URL**: https://arxiv.org/abs/2407.01502

**Key Findings**:

1. **Focus on accuracy ignores cost** - SOTA agents are needlessly complex and costly
2. **Joint optimization** - Should optimize accuracy AND cost together
3. **Benchmark overfitting** - Many agents take shortcuts and are fragile
4. **Lack of standardization** - Pervasive lack of reproducibility
5. **Real-world vs benchmark performance** - Big gap between benchmark accuracy and practical usefulness

**Relevance to System Prompts**:
- Simpler, shorter prompts often perform as well as complex ones
- Cost-conscious design matters for production agents
- Standardization (like AGENTS.md) helps reproducibility

### Instruction Following Research

**From HumanLayer blog** (citing academic research):

**Key Finding**: Frontier LLMs can follow **~150-200 instructions** with reasonable consistency.

**Details**:
- **Smaller models**: Exponential decay in instruction-following as count increases
- **Larger models**: Linear decay (more graceful degradation)
- **Position bias**: LLMs bias toward instructions at beginning and end of prompt
- **Uniform degradation**: More instructions = worse at following ALL of them

**Impact on AGENTS.md Design**:
- Claude Code system prompt already has ~50 instructions
- Leaves ~100-150 instructions for AGENTS.md + user rules
- Every line in AGENTS.md "competes" for instruction-following capacity
- Brevity is not just aesthetic - it's functional

**Implication**: <150 lines is a **hard recommendation** based on LLM capabilities.

---

## Conclusions and Recommendations

### For the CTO Platform

1. **Adopt AGENTS.md as standard** across all agent templates
   - Universal compatibility
   - Industry momentum
   - Minimal vendor lock-in

2. **Keep agent instructions concise** (<150 lines)
   - Backed by research on instruction-following limits
   - Better performance than lengthy prompts
   - Easier to maintain

3. **Use progressive disclosure** for detailed docs
   - Reference specialized docs from AGENTS.md
   - Don't duplicate content
   - Agents load context when needed

4. **CLI-specific wrappers only when necessary**
   - Most features work with standard AGENTS.md
   - Add CLI-specific configs for advanced features only
   - Maintain portability as priority

5. **Hierarchical organization for monorepos**
   - Root AGENTS.md for overall structure
   - Package-level AGENTS.md for specifics
   - Closest file to edited code wins

6. **Container image standard layout**
   - AGENTS.md in /workspace (auto-discovered)
   - Progressive disclosure docs in /workspace/docs/agents/
   - CLI configs in /etc/<cli>/ (when needed)

7. **Template structure** (Option D)
   - `templates/agents/base/AGENTS.md.hbs` - universal
   - `templates/agents/specialized/` - agent-specific additions
   - `templates/clis/` - CLI-specific wrappers
   - `templates/shared/instructions/` - progressive disclosure

### Migration Priority

**Immediate** (Week 1-2):
1. Create base AGENTS.md template
2. Extract common instructions from existing templates
3. Keep under 150 lines

**Short-term** (Week 3-4):
1. Add progressive disclosure docs
2. Create CLI-specific wrappers for advanced features
3. Test across all 6 CLIs

**Long-term** (Month 2+):
1. Refine based on agent performance
2. Gather metrics on instruction following
3. Iterate on brevity and clarity

---

## References

1. **AGENTS.md Official**: https://agents.md/
2. **OpenAI Codex Docs**: https://developers.openai.com/codex/guides/agents-md/
3. **Claude Code CLI Reference**: https://code.claude.com/docs/en/cli-reference
4. **Factory AGENTS.md Guide**: https://docs.factory.ai/cli/configuration/agents-md
5. **Cursor Rules Documentation**: https://cursor.com/docs/context/rules
6. **OpenCode Agents Documentation**: https://opencode.ai/docs/agents/
7. **Gemini CLI Documentation**: https://developers.google.com/gemini-code-assist/docs/gemini-cli
8. **HumanLayer Blog**: https://www.humanlayer.dev/blog/writing-a-good-claude-md
9. **AI Agents That Matter Paper**: https://arxiv.org/abs/2407.01502
10. **Anthropic Best Practices**: https://www.anthropic.com/engineering/claude-code-best-practices

---

**Research Completed**: December 5, 2025  
**Model**: Claude Sonnet 4.5  
**Next Steps**: Review with team, finalize template structure, begin implementation

