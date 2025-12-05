# Claude Opus 4.5: Templates Refactor Analysis

> **Model**: Claude Opus 4.5  
> **Date**: December 2, 2025  
> **Purpose**: Comprehensive analysis for production/open-source readiness

---

## Executive Summary

The current templates directory contains **50,321 lines** of Handlebars templates across **162 files**. While there's some partial usage, there's **massive duplication** that creates maintenance burden, consistency issues, and makes the codebase harder to understand for contributors.

**Key Findings:**
- 5 container-base files alone total **11,454 lines** of largely duplicated code
- Claude CLI templates **don't use partials at all** (unlike other CLIs)
- Agent prompts are duplicated across 6 CLI directories with minor variations
- ~40% of template code is redundant

---

## Current Architecture Analysis

### Template Structure

```
templates/
├── agents/           # 8 system prompts (shared across CLIs)
├── code/             # 6 CLI directories × multiple agents
│   ├── claude/       # 15,262 lines (NO PARTIAL INHERITANCE)
│   ├── codex/        # Uses {{> codex_container_base}}
│   ├── cursor/       # Uses {{> cursor_container_base}}
│   ├── factory/      # Uses {{> factory_container_base}}
│   ├── gemini/       # Uses {{> gemini_container_base}}
│   └── opencode/     # Uses {{> opencode_container_base}}
├── shared/           # Only 4 files (~450 lines)
├── docs/             # Documentation templates
├── heal/             # Heal workflow templates
├── intake/           # Intake templates
├── remediate/        # Remediation templates
├── review/           # Review templates
└── security/         # Security docs (plain .md)
```

### Line Count by CLI Container-Base

| CLI | container-base.sh.hbs | Status |
|-----|----------------------|--------|
| Codex | 3,180 lines | Has partial system |
| Cursor | 2,712 lines | Has partial system |
| Factory | 3,101 lines | Has partial system |
| Gemini | 1,261 lines | Has partial system |
| OpenCode | 1,200 lines | Has partial system |
| **Claude** | **N/A** | **NO BASE - each agent is standalone** |

### Claude CLI Problem

Claude templates are **completely standalone** with no inheritance:

| File | Lines | Problem |
|------|-------|---------|
| container-rex.sh.hbs | 1,908 | Full standalone script |
| container-tess.sh.hbs | 2,226 | Full standalone script |
| container-cleo.sh.hbs | 2,522 | Full standalone script |
| container-blaze.sh.hbs | 2,525 | Full standalone script |
| container-cipher.sh.hbs | 2,855 | Full standalone script |
| **Total** | **15,262** | **~80% is duplicated boilerplate** |

Compare to other CLIs where agent containers are 1-3 lines:
```handlebars
{{> codex_container_base
    agent_banner="🔧 Rex Codex implementation workflow starting"
    agent_completion_message="✅ Rex Codex implementation complete"}}
```

---

## Anti-Patterns Identified

### 1. **Claude Templates Bypass Partial System**
- While other CLIs use `{{> cli_container_base}}`, Claude templates are monolithic
- Results in ~12,000 lines of duplicated boilerplate across Claude agent files
- Makes updates error-prone (need to change 5+ files for common logic)

### 2. **Agent Prompts Duplicated Per CLI**
Each CLI directory has its own `agents-rex.md.hbs`, `agents-cleo.md.hbs`, etc.:
```
code/codex/agents-rex.md.hbs     (87 lines)
code/cursor/agents-rex.md.hbs    (59 lines)  
code/factory/agents-rex.md.hbs   (114 lines)
code/opencode/agents-rex.md.hbs  (114 lines)
```
These share 70-90% content but have CLI-specific variations scattered throughout.

### 3. **Inconsistent Use of `templates/agents/` Directory**
- Central agent system prompts exist (`rex-system-prompt.md.hbs`, etc.)
- Only used via `{{> agents/cipher-system-prompt}}` in 4 places (all for Cipher)
- Other agents don't leverage this central location

### 4. **Shared Directory Underutilized**
`templates/shared/` contains only 4 files:
- `context7-instructions.md.hbs` (new)
- `design-system.md`
- `memory-functions.sh`
- `task-setup-functions.sh`

These shared utilities aren't being included in templates that could benefit.

### 5. **Naming Inconsistencies**
- Some use `container-base.sh.hbs`, others use just `container.sh.hbs`
- Mixed patterns: `agents-rex.md.hbs` vs `rex-system-prompt.md.hbs`
- Underscore vs hyphen: `memory-functions.sh` vs `task_setup_functions.sh` (wait, it's hyphenated)

### 6. **Dead/Orphaned Code**
- `code_shared_hooks_stop-code-pr-creation.sh.hbs` at root level (423 lines)
- Integration templates that may be unused
- `container.sh.hbs` (generic) alongside specific containers

---

## Redundancies Identified

### Code Duplication Estimate

| Section | Est. Duplicated Lines | Files Affected |
|---------|----------------------|----------------|
| GitHub App Auth | 80 lines × 10 | All container-base |
| Git Setup | 100 lines × 10 | All container-base |
| Branch Management | 200 lines × 10 | All container-base |
| PR Creation Logic | 300 lines × 10 | All container-base |
| Quality Gates | 150 lines × 10 | All container-base |
| Agent Identity Header | 20 lines × 27 | All agents-*.md.hbs |
| Context7 Instructions | 30 lines × 10 | Various agent files |
| Memory Extensions Block | 15 lines × 27 | All agents-*.md.hbs |
| **Estimated Total** | **~15,000 lines** | |

### Specific Redundant Patterns

1. **GitHub App JWT Authentication** (~80 lines)
   - Identical in every container-base
   - Should be a sourced function

2. **Parse Repo Function** (~15 lines)
   - Duplicated in every container-base
   - Should be in shared utilities

3. **Quality Gate Commands** (~50 lines)
   - Same `cargo fmt`, `cargo clippy`, `cargo test` blocks
   - Should be a parameterized function

4. **PR Creation Template** (~100 lines)
   - Nearly identical across all containers
   - Should be a shared template block

---

## Proposed Modular Architecture

### Tier 1: Core Utilities (Shell Functions)

```
templates/shared/
├── functions/
│   ├── github-auth.sh.hbs      # JWT auth, token refresh
│   ├── git-operations.sh.hbs   # Clone, branch, commit, PR
│   ├── quality-gates.sh.hbs    # Lint, test, format commands
│   ├── agent-lifecycle.sh.hbs  # Init, cleanup, completion markers
│   └── memory-functions.sh     # OpenMemory integration (exists)
```

### Tier 2: Base Container Templates

```
templates/shared/
├── container-base.sh.hbs       # Universal base (CLI-agnostic)
└── cli-bootstrap/
    ├── claude-init.sh.hbs      # Claude-specific setup
    ├── codex-init.sh.hbs       # Codex-specific setup
    ├── cursor-init.sh.hbs      # Cursor wrapper setup
    ├── factory-init.sh.hbs     # Factory telemetry
    ├── gemini-init.sh.hbs      # Gemini setup
    └── opencode-init.sh.hbs    # OpenCode setup
```

### Tier 3: Agent Behavior Templates

```
templates/agents/
├── base-agent.md.hbs           # Common agent structure
├── behaviors/
│   ├── implementation.md.hbs   # Rex-like behavior
│   ├── quality-review.md.hbs   # Cleo-like behavior
│   ├── testing.md.hbs          # Tess-like behavior
│   ├── security.md.hbs         # Cipher-like behavior
│   └── frontend.md.hbs         # Blaze-like behavior
└── tools/
    ├── context7.md.hbs         # Context7 instructions
    ├── openmemory.md.hbs       # OpenMemory usage
    └── cli-tools.md.hbs        # Quality CLI tools
```

### Tier 4: CLI-Specific Minimal Wrappers

```
templates/code/{cli}/
├── container.sh.hbs            # 5-10 lines, includes shared + cli-init
├── agents.md.hbs               # CLI-specific header only
└── config.{ext}.hbs            # CLI config file
```

---

## Implementation Recommendations

### Phase 1: Extract Shared Functions (High Impact)

1. Create `shared/functions/github-auth.sh.hbs`:
```bash
# Extracted from container-base files
github_app_authenticate() {
    # ... 80 lines of JWT logic
}
export -f github_app_authenticate
```

2. Update all container-base to source instead of inline:
```handlebars
source /agent-templates/shared_functions_github-auth.sh
github_app_authenticate
```

**Estimated Savings**: ~800 lines (80 × 10 files)

### Phase 2: Create Universal Container Base

1. Extract common container logic to `shared/container-base.sh.hbs`
2. Each CLI creates thin wrapper:
```handlebars
#!/bin/bash
# CLI-specific initialization
{{> shared/cli-bootstrap/{{cli_type}}-init}}

# Universal container logic
{{> shared/container-base
    cli_name="{{cli_type}}"
    retry_var="{{cli_type}}_MAX_RETRIES"}}
```

**Estimated Savings**: ~8,000 lines

### Phase 3: Unify Agent Prompts

1. Create `agents/base-agent.md.hbs`:
```handlebars
# {{cli_name}} Project Memory — {{agent_role}} ({{agent_name}})

## Agent Identity & Boundaries
{{> agents/partials/identity-block}}

## Mission-Critical Execution Rules
{{> agents/partials/execution-rules}}

{{#if rust_project}}
{{> shared/context7-instructions}}
{{/if}}

## {{agent_role}} Playbook
{{> agents/behaviors/{{agent_behavior}}}}

## Definition of Done
{{> agents/partials/done-criteria}}

{{> agents/partials/tooling-memory}}
```

2. CLI-specific agents become ~5 lines:
```handlebars
{{> agents/base-agent
    cli_name="Codex"
    agent_name="Rex"
    agent_role="Implementation Agent"
    agent_behavior="implementation"
    rust_project=true}}
```

**Estimated Savings**: ~2,000 lines across 27 agent files

### Phase 4: Fix Claude Templates

1. Create `code/claude/container-base.sh.hbs` using shared base
2. Convert all Claude agent containers to use partials:
```handlebars
{{> claude_container_base
    agent_banner="🔧 Rex Claude implementation workflow"
    agent_completion_message="✅ Rex implementation complete"}}
```

**Estimated Savings**: ~12,000 lines (Claude alone!)

---

## Breaking Changes & Migration

### Safe Changes (No Breaking)
- Extract functions to shared directory
- Add new partials
- Create new base templates

### Requires Testing
- Converting existing templates to use partials
- Renaming template files
- Changing partial parameter signatures

### Migration Strategy

1. **Create new structure alongside existing**
2. **Update one CLI at a time** (start with least-used: OpenCode)
3. **Validate each migration** with actual container runs
4. **Remove old templates only after validation**

---

## Inaccuracies Found

### 1. Outdated Tool References
- ~~`rustdocs_query_rust_docs`~~ (removed in this session)
- Some templates still reference deprecated DocsRun CRD

### 2. Inconsistent Context7 Instructions
- Some agents have detailed Context7 sections
- Others have none or abbreviated versions
- Should be standardized via shared partial

### 3. Model/CLI Assumptions
- Some templates assume specific models
- Hardcoded behavior that should be parameterized

### 4. Task ID Placeholders
- `{{task_id}}` sometimes used in strings that should escape
- Inconsistent handling of empty task IDs

---

## Metrics: Before vs After

| Metric | Current | After Refactor |
|--------|---------|----------------|
| Total .hbs Lines | 50,321 | ~28,000 (est.) |
| Unique Logic Lines | ~20,000 | ~20,000 |
| Duplicated Lines | ~30,000 | ~8,000 |
| Files | 162 | ~120 |
| Avg Container Size | 2,500 | 300 |
| Update Touch Points | 10+ files | 1-2 files |

---

## Priority Recommendations

### Immediate (Before Public Release)

1. ✅ **Fix Claude templates** - biggest ROI, most duplication
2. ✅ **Extract github-auth function** - security-critical, duplicated everywhere
3. ✅ **Standardize Context7** - already started, finish propagation

### Short-Term (Next Sprint)

4. Create universal container-base
5. Unify agent prompt structure
6. Add template tests (validate rendering)

### Medium-Term (Before v1.0)

7. Full modular architecture
8. Documentation for template contributors
9. Template validation CI

---

## Conclusion

The templates have grown organically and accumulated significant technical debt. The proposed refactoring would:

- **Reduce code by ~44%** (22,000+ lines)
- **Improve maintainability** (single source of truth)
- **Enable easier contributions** (clear structure)
- **Reduce bugs** (changes propagate automatically)
- **Make open-source ready** (professional, documented)

The most impactful single change is **fixing Claude templates to use partials**, which alone would eliminate ~12,000 lines of duplication.

---

*Generated by Claude Opus 4.5 for the CTO Platform refactor initiative.*

