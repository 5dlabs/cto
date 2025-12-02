# CTO Platform Template System Design

> **Version**: 1.0.0  
> **Status**: Phase 1-2 Implemented  
> **Authors**: Claude Opus 4.5, Engineering Team  
> **Date**: December 2, 2025  
> **Last Updated**: December 2, 2025

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Design Principles](#design-principles)
3. [Architecture Overview](#architecture-overview)
4. [Directory Structure](#directory-structure)
5. [Inheritance Model](#inheritance-model)
6. [Naming Conventions](#naming-conventions)
7. [Partial Library](#partial-library)
8. [Variable Contract](#variable-contract)
9. [Anti-Patterns](#anti-patterns)
10. [Migration Strategy](#migration-strategy)
11. [Examples](#examples)
12. [Testing & Validation](#testing--validation)

---

## Executive Summary

The CTO Platform template system uses [Handlebars](https://handlebarsjs.com/) to generate agent prompts, container scripts, and configuration files. This document defines the canonical structure, inheritance patterns, and conventions for maintaining a consistent, DRY, and contributor-friendly template codebase.

### Goals

- **DRY**: Eliminate duplication through strategic use of partials
- **Discoverable**: Clear naming and structure for new contributors
- **Testable**: Templates can be validated in CI
- **Extensible**: Easy to add new agents, CLIs, and workflows

### Current State vs Target

| Metric | Current | Target |
|--------|---------|--------|
| Total Lines | 50,321 | ~22,000 |
| Unique Logic | ~18,000 | ~18,000 |
| Duplication | ~32,000 | ~4,000 |
| Avg Container Script | 2,500 lines | 50-100 lines |

---

## Design Principles

### 1. Composition Over Inheritance

Templates should be composed from small, focused partials rather than relying on deep inheritance chains.

```handlebars
{{!-- GOOD: Explicit composition --}}
{{> shared/bootstrap/rust-env}}
{{> shared/auth/github-app}}
{{> agents/behaviors/implementation}}

{{!-- BAD: Deep inheritance hiding behavior --}}
{{> level3-container-which-includes-level2-which-includes-level1}}
```

### 2. Single Source of Truth

Each piece of logic or instruction should exist in exactly one place.

```handlebars
{{!-- GOOD: Context7 instructions in one place --}}
{{> shared/context7-instructions}}

{{!-- BAD: Same instructions copy-pasted across 6 CLI directories --}}
```

### 3. Explicit Over Implicit

Parameters should be explicitly passed, not implicitly inherited.

```handlebars
{{!-- GOOD: Clear parameter passing --}}
{{> shared/quality-gates 
    language="rust"
    strict=true
    coverage_threshold=95}}

{{!-- BAD: Magic globals --}}
{{> shared/quality-gates}}
{{!-- Where does language come from? Who knows! --}}
```

### 4. Fail Fast, Fail Loud

Templates should validate required parameters and produce clear error messages.

```handlebars
{{!-- GOOD: Parameter validation --}}
{{#unless task_id}}
  {{!-- Template error: task_id is required --}}
  echo "❌ FATAL: task_id not provided to template"
  exit 1
{{/unless}}
```

### 5. CLI-Agnostic Core

Core logic (git operations, GitHub auth, quality gates) should work across all CLIs. CLI-specific code goes in thin adapter layers.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         TEMPLATE LAYERS                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐    Thin wrappers with CLI-specific params          │
│  │ CLI Adapters │    codex/, cursor/, factory/, claude/              │
│  │  (10-50 LOC) │    Just include base + pass parameters             │
│  └──────┬───────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌──────────────┐    Agent-specific behavior & prompts               │
│  │   Agents     │    Rex, Cleo, Tess, Cipher, Blaze, Spark           │
│  │ (50-200 LOC) │    Composed from behavior + tool partials          │
│  └──────┬───────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌──────────────┐    Reusable behavior blocks                        │
│  │  Behaviors   │    implementation, review, testing, security       │
│  │ (100-300 LOC)│    Can be mixed and matched                        │
│  └──────┬───────┘                                                    │
│         │                                                            │
│         ▼                                                            │
│  ┌──────────────┐    Shell functions, auth, git operations           │
│  │   Shared     │    github-auth, quality-gates, git-ops             │
│  │ (Primitives) │    CLI-agnostic building blocks                    │
│  └──────────────┘                                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

### Proposed Canonical Structure

```
templates/
├── README.md                           # This design doc summary
│
├── shared/                             # 🧱 Building blocks (CLI-agnostic)
│   ├── functions/                      # Shell function libraries
│   │   ├── github-auth.sh.hbs          # JWT auth, token refresh
│   │   ├── git-ops.sh.hbs              # Clone, branch, commit, push
│   │   ├── quality-gates.sh.hbs        # Lint, test, format runners
│   │   ├── task-setup.sh.hbs           # Task file operations
│   │   └── memory.sh.hbs               # OpenMemory integration
│   │
│   ├── prompts/                        # Reusable prompt fragments
│   │   ├── context7-instructions.md.hbs
│   │   ├── openmemory-usage.md.hbs
│   │   ├── pr-creation-guide.md.hbs
│   │   ├── quality-commands.md.hbs
│   │   └── design-system.md            # Static content (no .hbs)
│   │
│   ├── bootstrap/                      # Environment setup blocks
│   │   ├── rust-env.sh.hbs             # Cargo/rustup initialization
│   │   ├── node-env.sh.hbs             # NVM/Node setup
│   │   └── python-env.sh.hbs           # Pyenv/venv setup
│   │
│   └── container-core.sh.hbs           # Universal container skeleton
│
├── agents/                             # 🤖 Agent definitions
│   ├── _base.md.hbs                    # Base agent template structure
│   │
│   ├── behaviors/                      # Role-specific behavior blocks
│   │   ├── implementation.md.hbs       # Rex-like: ship code
│   │   ├── code-review.md.hbs          # Cleo-like: review PRs
│   │   ├── testing.md.hbs              # Tess-like: QA verification
│   │   ├── security.md.hbs             # Cipher-like: security scan
│   │   ├── frontend.md.hbs             # Blaze-like: UI implementation
│   │   └── integration.md.hbs          # Atlas/Bolt: environment ops
│   │
│   ├── rex/                            # Agent-specific overrides
│   │   ├── system-prompt.md.hbs        # Full system prompt
│   │   └── memory.md.hbs               # CLI memory file (if needed)
│   ├── cleo/
│   ├── tess/
│   ├── cipher/
│   ├── blaze/
│   ├── spark/
│   ├── atlas/
│   ├── bolt/
│   └── nova/
│
├── cli/                                # 🖥️ CLI-specific adapters
│   ├── claude/
│   │   ├── container.sh.hbs            # Thin wrapper → shared/container-core
│   │   ├── config.json.hbs             # Claude config format
│   │   ├── hooks/                      # CLI-specific hooks
│   │   │   ├── stop-commit.sh.hbs
│   │   │   └── stop-pr-creation.sh.hbs
│   │   └── agents/                     # CLI + Agent combinations
│   │       ├── rex.sh.hbs              # container.sh + rex params
│   │       ├── cleo.sh.hbs
│   │       └── ...
│   │
│   ├── codex/
│   │   ├── container.sh.hbs
│   │   ├── config.toml.hbs
│   │   └── agents/
│   │
│   ├── cursor/
│   │   ├── container.sh.hbs
│   │   ├── config.json.hbs
│   │   ├── mcp.json.hbs
│   │   └── agents/
│   │
│   ├── factory/
│   ├── gemini/
│   └── opencode/
│
├── workflows/                          # 📋 Workflow-specific templates
│   ├── code/                           # Implementation workflow
│   │   ├── coding-guidelines.md.hbs
│   │   ├── github-guidelines.md.hbs
│   │   └── mcp.json.hbs
│   │
│   ├── intake/                         # Task intake workflow
│   │   ├── unified-intake.sh.hbs
│   │   └── ...
│   │
│   ├── review/                         # Code review workflow
│   ├── heal/                           # Remediation workflow
│   ├── docs/                           # Documentation workflow
│   └── pm/                             # Project management
│
└── security/                           # 🔒 Security documentation
    ├── CIPHER_QUICK_REFERENCE.md
    └── CIPHER_SECURITY_GUIDELINES.md
```

### Key Changes from Current Structure

| Current | Proposed | Rationale |
|---------|----------|-----------|
| `code/{cli}/container-{agent}.sh.hbs` | `cli/{cli}/agents/{agent}.sh.hbs` | Clearer hierarchy |
| `code/{cli}/agents-{agent}.md.hbs` | `agents/{agent}/memory.md.hbs` | Agent owns its prompts |
| Duplicated 3000-line bases | `shared/container-core.sh.hbs` | Single source of truth |
| `agents/rex-system-prompt.md.hbs` | `agents/rex/system-prompt.md.hbs` | Agent directory grouping |
| Scattered partial usage | Consistent `shared/` + `behaviors/` | Predictable locations |

---

## Inheritance Model

### Container Script Inheritance

```
┌────────────────────────────────────────────────────────────────┐
│                     shared/container-core.sh.hbs               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ 1. Parameter validation                                  │  │
│  │ 2. Environment bootstrap ({{> shared/bootstrap/{{lang}}})│  │
│  │ 3. GitHub auth ({{> shared/functions/github-auth}})      │  │
│  │ 4. Repository clone/setup                                │  │
│  │ 5. Task file preparation                                 │  │
│  │ 6. ════════ {{> @partial-block}} ═══════════             │  │
│  │    ↑ This is where agent-specific logic goes             │  │
│  │ 7. Cleanup & completion                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                     cli/claude/container.sh.hbs                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ {{#> shared/container-core                               │  │
│  │     cli_name="claude"                                    │  │
│  │     retry_env_var="CLAUDE_MAX_RETRIES"                   │  │
│  │     default_retries=5}}                                  │  │
│  │                                                          │  │
│  │   # Claude-specific initialization                       │  │
│  │   setup_claude_environment                               │  │
│  │                                                          │  │
│  │   # Run the agent (partial-block content goes here)      │  │
│  │   claude --print "$CLAUDE_WORK_DIR" ...                  │  │
│  │                                                          │  │
│  │ {{/shared/container-core}}                               │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                     cli/claude/agents/rex.sh.hbs               │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ {{> cli/claude/container                                 │  │
│  │     agent_name="rex"                                     │  │
│  │     agent_banner="🔧 Rex implementation starting"        │  │
│  │     agent_behavior="implementation"                      │  │
│  │     completion_marker="/workspace/.rex-complete"}}       │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### Agent Prompt Inheritance

```
┌────────────────────────────────────────────────────────────────┐
│                      agents/_base.md.hbs                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ # {{cli_name}} Project Memory — {{agent_role}} ({{name}})│  │
│  │                                                          │  │
│  │ ## Agent Identity & Boundaries                           │  │
│  │ {{> agents/partials/identity                             │  │
│  │     github_app=github_app                                │  │
│  │     model=model                                          │  │
│  │     task_id=task_id}}                                    │  │
│  │                                                          │  │
│  │ ## Mission-Critical Execution Rules                      │  │
│  │ {{> agents/behaviors/{{behavior}}}}                      │  │
│  │                                                          │  │
│  │ {{#if rust_project}}                                     │  │
│  │ ## Documentation Tools                                   │  │
│  │ {{> shared/prompts/context7-instructions}}               │  │
│  │ {{/if}}                                                  │  │
│  │                                                          │  │
│  │ {{#if frontend_project}}                                 │  │
│  │ ## Frontend Guidelines                                   │  │
│  │ {{> shared/prompts/design-system}}                       │  │
│  │ {{/if}}                                                  │  │
│  │                                                          │  │
│  │ ## Tooling                                               │  │
│  │ {{> agents/partials/tooling-block tools=tools}}          │  │
│  │                                                          │  │
│  │ ## Memory Extensions                                     │  │
│  │ {{> agents/partials/memory-block cli_config=cli_config}} │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                    agents/rex/system-prompt.md.hbs             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ {{> agents/_base                                         │  │
│  │     name="Rex"                                           │  │
│  │     agent_role="Implementation Agent"                    │  │
│  │     behavior="implementation"                            │  │
│  │     rust_project=true                                    │  │
│  │     cli_name=cli_name}}                                  │  │
│  │                                                          │  │
│  │ ## Rex-Specific Additions                                │  │
│  │ {{!-- Any Rex-only content goes here --}}                │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

---

## Naming Conventions

### Files

| Pattern | Example | Use Case |
|---------|---------|----------|
| `{name}.sh.hbs` | `container.sh.hbs` | Shell script template |
| `{name}.md.hbs` | `system-prompt.md.hbs` | Markdown/prompt template |
| `{name}.json.hbs` | `config.json.hbs` | JSON configuration |
| `{name}.toml.hbs` | `config.toml.hbs` | TOML configuration |
| `_base.{ext}.hbs` | `_base.md.hbs` | Abstract base (not used directly) |
| `{name}.md` | `design-system.md` | Static content (no variables) |

### Directories

| Pattern | Example | Purpose |
|---------|---------|---------|
| `shared/` | - | CLI-agnostic reusable components |
| `cli/{cli_name}/` | `cli/claude/` | CLI-specific adapters |
| `agents/{agent_name}/` | `agents/rex/` | Agent-specific files |
| `workflows/{workflow}/` | `workflows/code/` | Workflow-specific templates |
| `{noun}s/` | `agents/`, `behaviors/` | Pluralized for collections |

### Partials

Partial names should be descriptive paths that indicate location:

```handlebars
{{!-- GOOD: Clear path indicates source --}}
{{> shared/functions/github-auth}}
{{> agents/behaviors/implementation}}
{{> cli/claude/hooks/stop-commit}}

{{!-- BAD: Unclear flat names --}}
{{> github_auth}}
{{> impl}}
{{> claude_hook}}
```

### Variables

| Convention | Example | Use |
|------------|---------|-----|
| `snake_case` | `task_id`, `repository_url` | All template variables |
| `SCREAMING_SNAKE` | `MAX_RETRIES` | Environment variables in scripts |
| Boolean prefixes | `is_rust`, `has_frontend` | Conditionals |
| Consistent plurals | `tools.tools` → `tool_list` | Avoid awkward paths |

---

## Partial Library

### Shared Function Partials

#### `shared/functions/github-auth.sh.hbs`

```handlebars
{{!--
  GitHub App JWT Authentication
  
  Required params:
    - github_app_id: GitHub App ID (from env)
    - github_app_private_key: PEM key (from env)
    - repository_url: Target repository
  
  Provides:
    - GITHUB_TOKEN: Installation access token
    - REPO_OWNER: Parsed owner
    - REPO_NAME: Parsed repo name
--}}
github_app_authenticate() {
    local app_id="${GITHUB_APP_ID:?Missing GITHUB_APP_ID}"
    local private_key="${GITHUB_APP_PRIVATE_KEY:?Missing GITHUB_APP_PRIVATE_KEY}"
    local repo_url="{{repository_url}}"
    
    echo "🔐 Authenticating with GitHub App..."
    
    # JWT creation logic (extracted from container bases)
    # ... 60 lines of auth logic ...
    
    export GITHUB_TOKEN="$token"
    export REPO_OWNER="$owner"
    export REPO_NAME="$name"
    
    echo "✓ Authenticated as installation $INSTALLATION_ID"
}
```

#### `shared/functions/quality-gates.sh.hbs`

```handlebars
{{!--
  Quality Gate Runner
  
  Params:
    - language: "rust" | "typescript" | "python"
    - strict: boolean (fail on warnings)
    - coverage_threshold: number (0-100)
--}}
run_quality_gates() {
    local language="{{language}}"
    local strict="{{#if strict}}true{{else}}false{{/if}}"
    local coverage_threshold="{{coverage_threshold}}"
    
    echo "🔍 Running quality gates for $language..."
    
    {{#if (eq language "rust")}}
    echo "📐 Formatting..."
    cargo fmt --all -- --check || { echo "❌ Format failed"; return 1; }
    
    echo "📎 Linting..."
    cargo clippy --workspace --all-targets --all-features \
        -- -D warnings {{#if strict}}-W clippy::pedantic{{/if}} \
        || { echo "❌ Clippy failed"; return 1; }
    
    echo "🧪 Testing..."
    cargo test --workspace --all-features \
        || { echo "❌ Tests failed"; return 1; }
    {{/if}}
    
    {{#if (eq language "typescript")}}
    # TypeScript quality gates...
    {{/if}}
    
    echo "✅ All quality gates passed"
}
```

### Prompt Partials

#### `shared/prompts/context7-instructions.md.hbs`

```handlebars
{{!--
  Context7 Documentation Tool Instructions
  
  Include in any agent that may need library documentation.
  Works for both Rust and frontend projects.
--}}
## Context7 Documentation Tools

You have access to **Context7** for real-time, up-to-date library documentation.

### Two-Step Workflow

1. **Resolve**: `resolve_library_id({ libraryName: "tokio rust" })`
2. **Get docs**: `get_library_docs({ context7CompatibleLibraryID: "...", topic: "..." })`

{{#if rust_project}}
### Pre-Resolved Rust Library IDs

| Library | ID | Score | Use Case |
|---------|-----|-------|----------|
| Tokio | `/websites/rs_tokio_tokio` | 93.8 | Async runtime |
| Anyhow | `/dtolnay/anyhow` | 89.3 | Error handling |
| Serde | `/websites/serde_rs` | 80.2 | Serialization |
| Thiserror | `/dtolnay/thiserror` | 83.1 | Custom errors |
| Tracing | `/tokio-rs/tracing` | 69.6 | Logging |
{{/if}}

{{#if frontend_project}}
### Pre-Resolved Frontend Library IDs

| Library | ID | Use Case |
|---------|-----|----------|
| React | `/facebook/react` | UI components |
| Next.js | `/vercel/next.js` | Full-stack framework |
{{/if}}

**Always query Context7 before implementing unfamiliar patterns.**
```

### Behavior Partials

#### `agents/behaviors/implementation.md.hbs`

```handlebars
{{!--
  Implementation Behavior
  
  Used by: Rex, Blaze, Spark
  Focus: Shipping production-ready code
--}}
## Mission-Critical Execution Rules

1. **No mocks or placeholders.** All integrations use real services and configurable parameters.
2. **Parameterize everything.** No hard-coded endpoints, thresholds, or secrets.
3. **Document-as-you-build.** Update README and task docs for downstream agents.
4. **Own the git history.** Clean commits, never leave workspace dirty.
5. **Stay on the feature branch.** Never push to main. Use `git push origin HEAD`.
6. **Operate without supervision.** Make decisions, document rationale, keep moving.
7. **Task isolation is absolute.** Only implement Task {{task_id}}.

## Implementation Playbook

1. **Read the docs**: `task/task.md`, `task/acceptance-criteria.md`
{{#if rust_project}}
2. **Query Context7**: Get documentation for unfamiliar patterns
3. **Plan**: Summarize approach before coding
4. **Implement**: Production-ready code with real integrations
5. **Verify**: `cargo fmt`, `cargo clippy -D warnings -W clippy::pedantic`, `cargo test`
{{else}}
2. **Plan**: Summarize approach before coding
3. **Implement**: Production-ready code with real integrations
4. **Verify**: Run linters, formatters, and tests
{{/if}}
5. **Create the PR**: `gh pr create` with labels and detailed body

## Definition of Done

- All acceptance criteria satisfied with evidence
- Zero lint/test failures
- PR opened and labeled: `task-{{task_id}}`, `service-{{service}}`
```

---

## Variable Contract

### Global Variables (Always Available)

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `task_id` | string | Unique task identifier | `"TASK-1234"` |
| `service` | string | Target service name | `"controller"` |
| `repository_url` | string | Git repository URL | `"https://github.com/..."` |
| `workflow_name` | string | Current workflow name | `"code-workflow"` |
| `github_app` | string | GitHub App name | `"Rex"` |
| `model` | string | LLM model identifier | `"claude-sonnet-4"` |
| `cli_type` | string | CLI being used | `"claude"` |

### Optional Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `docs_repository_url` | string | null | Docs repo if separate |
| `docs_branch` | string | `"main"` | Docs branch |
| `working_directory` | string | `"/workspace"` | Container work dir |
| `rust_project` | boolean | false | Is this a Rust project? |
| `frontend_project` | boolean | false | Is this a frontend project? |
| `telemetry.enabled` | boolean | false | Enable OTEL tracing |

### CLI-Specific Variables

| CLI | Variable | Description |
|-----|----------|-------------|
| Claude | `claude_work_dir` | Claude's working directory |
| Codex | `codex_config_path` | Path to config.toml |
| Cursor | `cursor_mcp_path` | Path to mcp.json |
| Factory | `factory_telemetry_endpoint` | OTEL endpoint |

---

## Anti-Patterns

### ❌ Monolithic Templates

**Problem**: 3000-line container scripts with no reuse.

```handlebars
{{!-- BAD: Everything in one file --}}
#!/bin/bash
# Line 1-80: GitHub auth
# Line 81-180: Git setup
# Line 181-280: Task setup
# Line 281-380: Quality gates
# ... 2600 more lines of duplicated logic
```

**Solution**: Extract to partials.

```handlebars
{{!-- GOOD: Composed from focused partials --}}
#!/bin/bash
source /shared/functions/github-auth.sh
source /shared/functions/git-ops.sh
source /shared/functions/quality-gates.sh

github_app_authenticate
setup_repository
run_quality_gates
```

### ❌ Copy-Paste Across CLIs

**Problem**: Same agent logic duplicated 6 times.

```
code/codex/agents-rex.md.hbs    (87 lines)
code/cursor/agents-rex.md.hbs   (59 lines)  ← Different content!
code/factory/agents-rex.md.hbs  (114 lines) ← Different content!
```

**Solution**: Single source with CLI parameter.

```handlebars
{{!-- agents/rex/system-prompt.md.hbs --}}
{{> agents/_base
    name="Rex"
    behavior="implementation"
    cli_name=cli_name}}  {{!-- CLI passed as param --}}
```

### ❌ Implicit Dependencies

**Problem**: Partial assumes variables exist without documentation.

```handlebars
{{!-- BAD: What is task_id? Where does it come from? --}}
echo "Working on {{task_id}}"
```

**Solution**: Document at partial top.

```handlebars
{{!--
  @requires task_id - The task identifier (string)
  @requires service - The service name (string)
--}}
echo "Working on {{task_id}} for {{service}}"
```

### ❌ Logic in Two Places

**Problem**: Context7 instructions in both container script AND agent prompt.

**Solution**: Choose one. Prefer agent prompt for LLM instructions.

### ❌ Environment-Specific Hardcoding

**Problem**: Hardcoded URLs, paths, or values that differ per environment.

```handlebars
{{!-- BAD --}}
curl https://production.api.internal/...
```

**Solution**: Parameterize.

```handlebars
{{!-- GOOD --}}
curl {{api_base_url}}/...
```

---

## Migration Strategy

### Phase 1: Foundation (Week 1)

1. **Create `shared/functions/`** directory
2. **Extract GitHub auth** from container-base files → `shared/functions/github-auth.sh.hbs`
3. **Extract git operations** → `shared/functions/git-ops.sh.hbs`
4. **Update ONE CLI** (OpenCode - least used) to source from shared
5. **Validate** with actual container runs

### Phase 2: Container Consolidation (Week 2)

1. **Create `shared/container-core.sh.hbs`** with partial-block pattern
2. **Convert Codex** containers to use core (they already have partial system)
3. **Convert Cursor, Factory, Gemini** containers
4. **Fix Claude containers** - biggest win, most duplication
5. **Delete old duplicated bases**

### Phase 3: Agent Prompt Unification (Week 3)

1. **Create `agents/_base.md.hbs`** with composable structure
2. **Create `agents/behaviors/`** partials
3. **Migrate Rex** prompts to new structure
4. **Migrate remaining agents**
5. **Delete CLI-duplicated agent files**

### Phase 4: Cleanup & Documentation (Week 4)

1. **Remove orphaned files**
2. **Update this design doc** with learnings
3. **Add template validation** to CI
4. **Create contributor guide**

### Migration Validation

For each migration step:

```bash
# 1. Render template with test values
cargo run --bin template-render -- \
  --template templates/cli/claude/agents/rex.sh.hbs \
  --values test-fixtures/rex-values.json \
  --output /tmp/rendered-script.sh

# 2. Compare against known-good output
diff /tmp/rendered-script.sh test-fixtures/expected/claude-rex.sh

# 3. Run actual container test
kubectl apply -f test-fixtures/test-job-rex.yaml
kubectl logs -f job/test-rex-container
```

---

## Examples

### Example 1: Adding a New Agent (Nova)

1. **Create behavior** (if new type):
   ```
   agents/behaviors/orchestration.md.hbs
   ```

2. **Create agent directory**:
   ```
   agents/nova/
   ├── system-prompt.md.hbs
   └── memory.md.hbs (optional)
   ```

3. **System prompt uses base**:
   ```handlebars
   {{> agents/_base
       name="Nova"
       agent_role="Orchestration Agent"
       behavior="orchestration"}}
   
   ## Nova-Specific Instructions
   ...
   ```

4. **Create CLI adapters** (one per supported CLI):
   ```
   cli/claude/agents/nova.sh.hbs
   cli/codex/agents/nova.sh.hbs
   ```

### Example 2: Adding a New CLI (Aider)

1. **Create CLI directory**:
   ```
   cli/aider/
   ├── container.sh.hbs      # Uses shared/container-core
   ├── config.yaml.hbs       # Aider-specific config
   └── agents/               # Agent wrappers
       ├── rex.sh.hbs
       └── cleo.sh.hbs
   ```

2. **Container uses shared core**:
   ```handlebars
   {{#> shared/container-core
       cli_name="aider"
       retry_env_var="AIDER_MAX_RETRIES"}}
   
   # Aider-specific initialization
   aider --model {{model}} ...
   
   {{/shared/container-core}}
   ```

3. **Register in controller** for template selection

### Example 3: Updating Quality Gates for All Agents

1. **Edit single file**: `shared/functions/quality-gates.sh.hbs`

2. **Change propagates automatically** to all CLIs and agents

3. **No need to touch** 50+ container files

---

## Testing & Validation

### Template Rendering Tests

```rust
#[test]
fn test_rex_container_renders() {
    let template = include_str!("../templates/cli/claude/agents/rex.sh.hbs");
    let values = json!({
        "task_id": "TEST-123",
        "service": "test-service",
        "repository_url": "https://github.com/test/repo",
        // ... required values
    });
    
    let rendered = handlebars.render_template(template, &values)?;
    
    assert!(rendered.contains("#!/bin/bash"));
    assert!(rendered.contains("TEST-123"));
    assert!(!rendered.contains("{{"));  // No unresolved placeholders
}
```

### CI Validation

```yaml
# .github/workflows/template-validation.yaml
template-lint:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    
    - name: Check for unresolved variables
      run: |
        find templates -name "*.hbs" -exec grep -l '{{[^{]' {} \; | \
        xargs -I {} bash -c 'echo "Checking {}"; grep -n "{{[^#/>!]" {}'
    
    - name: Render all templates
      run: cargo run --bin template-validate
    
    - name: Check for duplicated content
      run: |
        # Find files >70% similar
        ./scripts/find-duplicates.sh templates/ 70
```

### Pre-Commit Hook

```yaml
# .pre-commit-config.yaml
- repo: local
  hooks:
    - id: template-syntax
      name: Validate template syntax
      entry: cargo run --bin template-validate --
      files: \.hbs$
      language: system
```

---

## Appendix: Current Files to Migrate

### High Priority (Most Duplication)

| File | Lines | Action |
|------|-------|--------|
| `code/claude/container-rex.sh.hbs` | 1,908 | Extract to partial + wrapper |
| `code/claude/container-tess.sh.hbs` | 2,226 | Extract to partial + wrapper |
| `code/claude/container-cleo.sh.hbs` | 2,522 | Extract to partial + wrapper |
| `code/claude/container-blaze.sh.hbs` | 2,525 | Extract to partial + wrapper |
| `code/claude/container-cipher.sh.hbs` | 2,855 | Extract to partial + wrapper |
| `code/codex/container-base.sh.hbs` | 3,180 | Become shared core |
| `code/cursor/container-base.sh.hbs` | 2,712 | Use shared core |
| `code/factory/container-base.sh.hbs` | 3,101 | Use shared core |

### Medium Priority (Agent Prompts)

All `code/{cli}/agents-{agent}.md.hbs` files (27 total) → Consolidate to `agents/{agent}/`

### Low Priority (Already OK)

- `shared/` files are good, just underutilized
- `agents/` system prompts are good, just not integrated
- Security docs are fine as static markdown

---

## Conclusion

This design provides a clear path from the current 50k-line template codebase to a maintainable ~22k-line system with:

- **Single source of truth** for all shared logic
- **Clear inheritance** through Handlebars partials
- **Consistent naming** that indicates file location and purpose
- **Easy extension** for new agents, CLIs, and workflows
- **Testable structure** with validation in CI

The migration can be done incrementally, validating each phase before proceeding.

---

*Document maintained by the CTO Platform Engineering Team*

---

## Appendix B: Template Review Feedback (December 2, 2025)

> This section contains observations from a comprehensive review of all templates in the `templates/` folder, analyzing their interaction with the Controller, Workflows, and CRDs.

### Executive Summary of Current State

After reviewing the entire template codebase, I can confirm the design document's assessment is accurate. The current state shows:

1. **Significant duplication** across CLI-specific container scripts (claude, codex, cursor, factory)
2. **Well-structured shared components** that are underutilized (`shared/`, `agents/`)
3. **Strong workflow specialization** with clear separation (code, intake, review, heal, remediate, pm)
4. **Inconsistent adoption** of the DRY principles - some templates compose beautifully, others are monolithic

### Controller Integration Analysis

The controller (`crates/controller/src/tasks/code/templates.rs`) orchestrates template rendering through:

#### 1. Template Selection Logic

The controller maps `github_app` and `run_type` to specific templates:

```rust
// Agent-specific template selection
match run_type {
    "documentation" => format!("code/{}/container-{}.sh.hbs", cli, github_app_lower),
    "intake" => "intake/unified-intake.sh.hbs",
    "remediation" => format!("remediate/{}/container.sh.hbs", cli),
    "review" => format!("review/{}/container.sh.hbs", cli),
    // Default to agent-specific container
    _ => format!("code/{}/container-{}.sh.hbs", cli, github_app_lower),
}
```

This means adding a new agent requires creating templates in **multiple** CLI directories—a prime target for the proposed consolidation.

#### 2. Context Injection

The controller builds a comprehensive Handlebars context from `CodeRun` CRD specs:

| CRD Field | Template Variable | Description |
|-----------|-------------------|-------------|
| `spec.taskId` | `{{task_id}}` | Task identifier |
| `spec.repositoryUrl` | `{{repository_url}}` | Git repository |
| `spec.githubApp` | `{{github_app}}` | Agent name (Rex, Cleo, etc.) |
| `spec.model` | `{{model}}` | LLM model identifier |
| `spec.cli` | `{{cli_type}}` | CLI type (claude, codex, etc.) |
| `spec.prNumber` | `{{pr_number}}` | PR number for review workflows |
| `spec.tools` | `{{tools}}` | MCP tools configuration |

#### 3. Dynamic Partial Registration

The controller registers shared partials at runtime:

```rust
handlebars.register_partial("shared/context7-instructions", ...)?;
handlebars.register_partial("shared/design-system", ...)?;
handlebars.register_partial("shared/memory-functions", ...)?;
```

**Observation**: The controller already supports the partial system proposed in this document, but the templates don't fully leverage it.

### Workflow Integration Patterns

Each workflow type has distinct template requirements:

#### Code Workflow (Primary)

- **Container Script**: CLI + Agent specific (e.g., `code/claude/container-rex.sh.hbs`)
- **System Prompt**: Agent-specific prompts from `agents/`
- **Guidelines**: Generated `coding-guidelines.md.hbs`, `github-guidelines.md.hbs`
- **MCP Config**: `mcp.json.hbs` for tool discovery

**Key Finding**: The code workflow templates show the most duplication, with ~2,000-3,000 lines repeated across each CLI/agent combination.

#### Intake Workflow

- **Single Script**: `intake/unified-intake.sh.hbs` (well-designed, ~760 lines)
- **TaskMaster Integration**: Parses PRD, generates tasks, creates documentation
- **PR Creation**: Automated with structured body

**Key Finding**: This is a good example of a unified template that works across use cases without CLI-specific variants.

#### Review Workflow

- **Container**: `review/{cli}/container.sh.hbs`
- **Agent Prompts**: `review/{cli}/agents.md.hbs`
- **Post-Processing**: `review/factory/post_review.py`

**Key Finding**: Review templates are compact (~120 lines) and CLI-agnostic in behavior—a model for the proposed refactor.

#### Heal Workflow

- **Container**: `heal/{cli}/container.sh.hbs`
- **Completion Probe**: Built-in acceptance criteria verification with retry loop
- **Issue Management**: Automatic GitHub Issue commenting and closure

**Key Finding**: The heal workflow has a unique "completion probe" pattern that verifies acceptance criteria—this pattern should be extracted to a shared behavior partial.

#### Remediate Workflow

- **Container**: `remediate/{cli}/container.sh.hbs`
- **CI Alert Fetching**: Automatic retrieval of CI failures

**Key Finding**: Remediate templates reuse much of the review infrastructure but have diverged slightly—candidate for consolidation.

### Agent System Prompt Analysis

The agent system prompts in `templates/agents/` are well-structured and follow a consistent pattern:

| Agent | Primary Behavior | Key Differentiators |
|-------|------------------|---------------------|
| Rex | Implementation | Rust/backend focus, PR creation mandatory |
| Nova | Implementation | Node.js/TypeScript focus |
| Blaze | Frontend | shadcn/ui, React/Next.js focus |
| Cleo | Quality Review | Zero tolerance for lint/test failures |
| Tess | Testing/QA | CI verification gate |
| Cipher | Security | CodeQL, Dependabot enforcement |
| Atlas | PR Guardian | Long-running merge automation |
| Bolt | Deployment | ArgoCD, ngrok integration |

**Key Finding**: All agent prompts share these patterns:

1. Tool discovery mandate (MCP tools)
2. Context7 documentation requirement
3. OpenMemory integration
4. Completion marker file creation
5. GitHub PR/Review actions

These shared patterns are perfect candidates for the proposed `agents/behaviors/` partials.

### Shared Components Review

#### Already Well-Structured

| Component | Location | Status |
|-----------|----------|--------|
| Memory Functions | `shared/memory-functions.sh` | ✅ Used across agents |
| Task Setup | `shared/task-setup-functions.sh` | ✅ Robust with retry logic |
| Context7 Instructions | `shared/context7-instructions.md.hbs` | ✅ Recently refactored |
| Design System | `shared/design-system.md` | ✅ Static, well-documented |

#### Opportunities for New Shared Partials

1. **GitHub App Authentication** - Currently duplicated ~150 lines in every container script
2. **Quality Gate Runner** - Similar logic in Rex, Cleo, Tess with slight variations
3. **PR Creation Flow** - 200+ lines duplicated in stop hooks
4. **Repository Clone & Setup** - ~100 lines repeated everywhere
5. **Completion Marker Pattern** - Each agent has its own version

### CRD Interaction Points

The templates receive data from two CRD types:

#### CodeRun CRD

Primary context source for agent execution:

```yaml
spec:
  taskId: "TASK-123"
  repositoryUrl: "https://github.com/..."
  githubApp: "Rex"
  model: "claude-sonnet-4"
  cli: "claude"
  tools:
    url: "http://tools-service:8080"
  # ... additional fields
```

#### Workflow CRD (Argo)

Provides workflow-level context:

```yaml
metadata:
  labels:
    task-id: "TASK-123"
    current-stage: "implementation-in-progress"
    parent-workflow: "play-workflow-xyz"
```

**Key Finding**: The templates currently extract repository owner/name by parsing `repository_url` string. This could be simplified if the controller pre-computed these values.

### Specific Recommendations

#### 1. Immediate Wins (Low Effort, High Impact)

1. **Extract GitHub Auth** - Create `shared/functions/github-auth.sh.hbs`
   - Currently ~150 lines duplicated in every container
   - Single extraction saves ~2,400 lines across 16 container templates

2. **Consolidate Quality Gates** - Create `shared/functions/quality-gates.sh.hbs`
   - Parameterize for language (rust/typescript/python)
   - Include all tools: fmt, clippy, test, security scans
   - Single file replaces 5-6 duplicated blocks per agent

3. **Unify Completion Marker** - Standard pattern for all agents

   ```handlebars
   {{> shared/completion-marker agent=github_app task_id=task_id}}
   ```

#### 2. Medium-Term Refactoring

1. **Agent Behavior Partials** - As proposed in Section 5
   - `implementation.md.hbs` for Rex, Nova, Blaze
   - `code-review.md.hbs` for Cleo
   - `testing.md.hbs` for Tess
   - `security.md.hbs` for Cipher
   - `integration.md.hbs` for Atlas, Bolt

2. **CLI Adapter Layer** - Thin wrappers as proposed
   - Each CLI adapter should be <100 lines
   - Include only CLI-specific initialization

3. **Workflow Template Consolidation**
   - Review and Remediate workflows share 80% of logic
   - Create `review-remediate-base.sh.hbs`

#### 3. Controller Enhancements

Recommend updating the controller to:

1. **Pre-compute repository metadata**:

   ```rust
   context.insert("repo_owner", parse_owner(&repository_url));
   context.insert("repo_name", parse_name(&repository_url));
   ```

2. **Provide project type hints**:

   ```rust
   context.insert("is_rust_project", detect_rust_project(&repo_path));
   context.insert("is_frontend_project", detect_frontend_project(&repo_path));
   ```

3. **Register all shared partials automatically**:

   ```rust
   for entry in glob("templates/shared/**/*.hbs") {
       handlebars.register_partial(partial_name(entry), read(entry)?)?;
   }
   ```

### Metrics After Review

Based on the actual file analysis:

| Metric | Actual Current | Design Doc Estimate | Notes |
|--------|----------------|---------------------|-------|
| Container Script Avg | ~1,900 lines | 2,500 lines | Estimate was high |
| Agent Prompts | ~200-400 lines | Not specified | Well-structured |
| Shared Components | ~800 lines | Underestimated | Memory/task functions |
| GitHub Auth (per file) | ~150 lines | ~60 lines | Includes retry logic |
| Quality Gates (per file) | ~80 lines | ~50 lines | Varies by language |

### Files Requiring Immediate Attention

| File | Lines | Priority | Issue |
|------|-------|----------|-------|
| `code/claude/container-rex.sh.hbs` | 1,909 | HIGH | Largest, most duplicated |
| `code/codex/container-base.sh.hbs` | 3,180+ | HIGH | Token limit exceeded on read |
| `code/cursor/container-base.sh.hbs` | 2,712+ | HIGH | Token limit exceeded on read |
| `pm/morgan-pm.sh.hbs` | 2,185 | MEDIUM | Complex but unique logic |
| `intake/unified-intake.sh.hbs` | 766 | LOW | Good structure, minor cleanup |

### Review Summary

The design document accurately identifies the problems and proposes appropriate solutions. The key insight from this review is that:

1. **The partial infrastructure exists** - The controller supports it, the Handlebars engine supports it
2. **Migration is safe** - We can validate rendered output against current behavior
3. **Shared components work well** - The existing `shared/` components prove the pattern
4. **Agent prompts are the model** - The `agents/` directory structure is clean and should be the reference

The proposed 4-week migration plan is realistic given the scope. I recommend starting with Phase 1 (GitHub auth extraction) as it provides the fastest validation of the approach with the least risk.

---

Feedback compiled from comprehensive template review on December 2, 2025

---

## Appendix C: Implementation Summary (December 2, 2025)

### Completed Work

#### Phase 1: Extract Shared Functions ✅

Created the foundational shared partial library:

```
templates/shared/
├── bootstrap/
│   └── rust-env.sh.hbs           # Rust toolchain initialization
├── functions/
│   ├── github-auth.sh.hbs        # GitHub App JWT authentication
│   ├── docker-sidecar.sh.hbs     # Docker sidecar management
│   └── completion-marker.sh.hbs  # Task completion signaling
├── container-core.sh.hbs         # Core container orchestration
├── context7-instructions.md.hbs  # Context7 documentation tool usage
└── design-system.md              # Frontend design principles
```

#### Phase 2: Convert CLI Containers ✅

Updated all CLI container-base files to use shared partials:
- `code/opencode/container-base.sh.hbs`
- `code/codex/container-base.sh.hbs`
- `code/cursor/container-base.sh.hbs`
- `code/factory/container-base.sh.hbs`
- `code/claude/container.sh.hbs`

#### Phase 3: Agent Prompts Cleanup ✅

- Removed all `rustdocs_query_rust_docs` references (deprecated)
- Updated Rex, Cleo, and Tess system prompts to use Context7
- Standardized documentation tool usage across all agents

#### Phase 4: Cleanup ✅

Deleted orphaned files:
- `templates/context7-instructions-snippet.md`
- `templates/design-system.md` (duplicate of `shared/design-system.md`)
- `templates/effect-solutions-instructions.md`
- `templates/shadcn-instructions-snippet.md`

### Controller Updates

Registered all shared partials in `crates/controller/src/tasks/code/templates.rs`:
- `shared/bootstrap/rust-env`
- `shared/functions/github-auth`
- `shared/functions/docker-sidecar`
- `shared/functions/completion-marker`
- `shared/context7-instructions`
- `shared/design-system`
- `shared/container-core`

Updated test binary (`src/bin/test_templates.rs`) to register shared partials.

### Validation

All templates render successfully:
- ✅ `cargo run -p controller --bin test-templates` passes
- ✅ `cargo clippy --all-targets -- -D warnings` passes
- ✅ `cargo test -p controller` passes (142 tests)
- ✅ `cargo fmt --all --check` passes

### Files Modified

| File | Change |
|------|--------|
| `templates/shared/bootstrap/rust-env.sh.hbs` | Created |
| `templates/shared/functions/github-auth.sh.hbs` | Created |
| `templates/shared/functions/docker-sidecar.sh.hbs` | Created |
| `templates/shared/functions/completion-marker.sh.hbs` | Created |
| `templates/shared/container-core.sh.hbs` | Created |
| `templates/shared/context7-instructions.md.hbs` | Created |
| `templates/code/claude/container.sh.hbs` | Uses shared partials |
| `templates/code/codex/container-base.sh.hbs` | Uses shared partials |
| `templates/code/cursor/container-base.sh.hbs` | Uses shared partials |
| `templates/code/factory/container-base.sh.hbs` | Uses shared partials |
| `templates/code/opencode/container-base.sh.hbs` | Uses shared partials |
| `templates/agents/rex-system-prompt.md.hbs` | Context7 migration |
| `templates/agents/cleo-system-prompt.md.hbs` | Context7 migration |
| `templates/agents/tess-system-prompt.md.hbs` | Context7 migration |
| `crates/controller/src/tasks/template_paths.rs` | Added SHARED_* constants |
| `crates/controller/src/tasks/code/templates.rs` | Partial registration |
| `crates/controller/src/bin/test_templates.rs` | Shared partial test support |

### Next Steps

1. **Further Deduplication**: The container scripts still have significant duplication in the agent-specific sections. Phase 3 of the original plan (Agent Behavior Partials) could further reduce duplication.

2. **Snapshot Testing**: Add golden file testing to validate rendered template output against known-good baselines.

3. **Documentation**: Update contributor guidelines to reference this design document.

---

Implementation completed December 2, 2025
