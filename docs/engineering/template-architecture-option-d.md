# Template Architecture: Option D Implementation

**Status**: In Progress  
**Decision**: Option D (Hybrid) - Multi-agent consensus (4-2 vote)  
**Date**: December 2024

---

## Executive Summary

This document outlines the transition from the current template structure to Option D (Hybrid), which balances DRY principles with operational simplicity. The key insight is separating **what's shared** (tools config, MCP servers) from **what's CLI-specific** (invocation flags, output formats).

---

## Current State

### Directory Structure (Before)

```
templates/
├── agents/                    # Agent identities ✅
│   ├── rex/
│   │   ├── identity.md.hbs
│   │   └── tools.hbs
│   ├── blaze/
│   ├── bolt/
│   ├── cipher/
│   ├── cleo/
│   ├── tess/
│   └── morgan/
│
├── clis/                      # CLI configs (duplicated with code/)
│   ├── claude/
│   │   ├── config.json.hbs
│   │   ├── container.sh.hbs   # Full container (duplication!)
│   │   └── settings.json.hbs
│   ├── factory/
│   ├── codex/
│   └── ...
│
├── code/                      # Play workflow (agent/CLI matrix)
│   ├── rex/
│   │   ├── claude/container.sh.hbs
│   │   ├── factory/container.sh.hbs
│   │   └── codex/container.sh.hbs
│   ├── blaze/
│   ├── cipher/
│   └── ...
│
├── healer/                    # Healer workflow (incomplete)
│   ├── rex/
│   │   ├── claude/container.sh.hbs
│   │   └── factory/container.sh.hbs
│   └── mcp.json.hbs
│
└── shared/                    # Shared utilities ✅
    ├── bootstrap/
    │   └── rust-env.sh.hbs
    ├── functions/
    │   ├── github-auth.sh.hbs
    │   ├── git-operations.sh.hbs
    │   └── ...
    ├── mcp.json.hbs           # MCP server config
    └── tools-config.json.hbs  # Remote tools list
```

### Problems with Current State

1. **Duplication**: CLI container logic duplicated between `clis/` and `code/`
2. **Agent × CLI Matrix**: Full containers per agent/CLI combo = maintenance burden
3. **Healer Incomplete**: Only Rex templates exist for Healer workflow
4. **No CLI Invoke Partials**: Can't share invocation logic across workflows

---

## Finalized State (Option D)

### Design Principles

1. **Single-file agents**: Adding new agent = 1 identity file
2. **CLI invoke partials**: CLI-specific logic in small, focused partials
3. **Workflow containers**: Complete, readable files per workflow
4. **Shared configs**: Tools/MCP config defined once, used everywhere

### Directory Structure (After)

```
templates/
├── agents/                    # Agent identities (unchanged)
│   ├── rex/
│   │   ├── identity.md.hbs    # Who Rex is, specialization
│   │   └── tools.hbs          # Agent-specific tool config
│   ├── blaze/
│   ├── bolt/
│   ├── cipher/
│   ├── cleo/
│   ├── tess/
│   └── morgan/
│
├── clis/                      # CLI-specific configs + invoke partials
│   ├── claude/
│   │   ├── config.json.hbs    # Claude Code config
│   │   ├── settings.json.hbs  # Enterprise settings
│   │   └── invoke.sh.hbs      # ✨ NEW: CLI invocation ONLY
│   ├── factory/
│   │   ├── factory-cli-config.json.hbs
│   │   └── invoke.sh.hbs      # ✨ NEW: CLI invocation ONLY
│   ├── codex/
│   │   ├── config.toml.hbs
│   │   └── invoke.sh.hbs      # ✨ NEW: CLI invocation ONLY
│   └── ...
│
├── code/                      # Play workflow (simplified)
│   ├── container.sh.hbs       # ✨ Single container using {{> clis/{cli}/invoke}}
│   └── system-prompt.md.hbs   # Play-specific system prompt
│
├── healer/                    # Healer workflow (complete)
│   ├── container.sh.hbs       # ✨ NEW: Uses {{> clis/{cli}/invoke}}
│   ├── system-prompt.md.hbs   # ✨ NEW: Healer-specific system prompt
│   └── mcp.json.hbs
│
└── shared/                    # Shared utilities (unchanged)
    ├── bootstrap/
    ├── functions/
    ├── mcp.json.hbs           # MCP server config (COMMON)
    └── tools-config.json.hbs  # Remote tools list (COMMON)
```

---

## Tools Configuration Architecture

### What's Shared

The toolman-client configuration is **common across all CLIs**:

```
templates/shared/
├── mcp.json.hbs           # MCP server definitions
│                          # - toolman-client command
│                          # - server URL
│                          # - working directory
│
└── tools-config.json.hbs  # Available remote tools
                           # - brave_search
                           # - openmemory_query/store
                           # - github tools
                           # - kubernetes tools
```

### What's CLI-Specific

Each CLI has different flags to **enable/consume** the shared tools:

| CLI | Config File | Tool Enable Flags | Output Format |
|-----|-------------|-------------------|---------------|
| **Claude** | `config.json.hbs`, `settings.json.hbs` | `--mcp-config`, `--allowedTools`, `--disallowedTools` | `--output-format stream-json` |
| **Factory** | `factory-cli-config.json.hbs` | `--enabled-tools`, `--disabled-tools` | `-o stream-json` |
| **Codex** | `config.toml.hbs` | MCP section in config.toml | `--json` |

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    SHARED CONFIGS                               │
│  ┌────────────────────┐    ┌────────────────────────────────┐  │
│  │    mcp.json.hbs    │    │    tools-config.json.hbs       │  │
│  │  (MCP servers)     │    │    (remote tools list)         │  │
│  └─────────┬──────────┘    └───────────────┬────────────────┘  │
└────────────┼───────────────────────────────┼────────────────────┘
             │                               │
             ▼                               ▼
┌─────────────────────────────────────────────────────────────────┐
│              WORKFLOW CONTAINER (code/ or healer/)              │
│                                                                 │
│  1. Copy shared configs to /workspace/.mcp.json                 │
│  2. Set up environment (git, auth, repo clone)                  │
│  3. Load prompt (from docs service OR healer server)            │
│  4. Include CLI invoke partial: {{> clis/{cli}/invoke}}         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CLI INVOKE PARTIALS                          │
├─────────────────┬─────────────────────┬─────────────────────────┤
│     Claude      │       Factory       │         Codex           │
├─────────────────┼─────────────────────┼─────────────────────────┤
│ claude -p       │ droid exec          │ codex exec              │
│ --mcp-config    │ --auto medium       │ --full-auto             │
│ --output-format │ --enabled-tools     │ --sandbox workspace-    │
│   stream-json   │ -o stream-json      │   write                 │
│ --system-prompt │ --skip-permissions  │ --json                  │
│ --dangerously-  │   -unsafe           │                         │
│   skip-perms    │                     │                         │
└─────────────────┴─────────────────────┴─────────────────────────┘
```

---

## CLI Invoke Partial Details

### Claude (`clis/claude/invoke.sh.hbs`)

Based on latest docs (Dec 2024): https://code.claude.com/docs/en/cli-reference

```bash
# Key flags:
claude -p \
  --output-format stream-json \
  --input-format stream-json \
  --mcp-config /workspace/.mcp.json \
  --system-prompt-file /path/to/prompt.md \
  --dangerously-skip-permissions \
  --verbose
```

### Factory (`clis/factory/invoke.sh.hbs`)

Based on latest docs (Dec 2024): https://docs.factory.ai/reference/cli-reference

```bash
# Key flags:
droid exec \
  --auto medium \              # Autonomy: low|medium|high
  -o stream-json \
  --enabled-tools "tool1,tool2" \
  --skip-permissions-unsafe \
  --cwd /workspace \
  "prompt text"
```

### Codex (`clis/codex/invoke.sh.hbs`)

Based on latest docs (Dec 2024): https://developers.openai.com/codex/cli/reference/

```bash
# Key flags:
codex exec \
  --full-auto \                # Or: --sandbox workspace-write --ask-for-approval on-failure
  --cd /workspace \
  --json \
  "prompt text"
```

---

## Workflow Differences

### Code/Play Workflow

- **Prompt source**: Docs repository (TaskMaster files)
- **Git strategy**: Feature branches
- **System prompt**: Task implementation focused
- **Use case**: Implementing new features from PRD

### Healer Workflow

- **Prompt source**: Healer server (static prompts)
- **Git strategy**: Git worktrees for isolation
- **System prompt**: CI remediation focused
- **Use case**: Fixing CI failures automatically

---

## Active Agents

The following agents are currently in the stack:

| Agent | Specialization | Status |
|-------|---------------|--------|
| **Rex** | Rust backend | ✅ Active |
| **Blaze** | Frontend (Next.js, React) | ✅ Active |
| **Bolt** | Infrastructure (K8s, Helm) | ✅ Active |
| **Cipher** | Security | ✅ Active |
| **Cleo** | Code quality | ✅ Active |
| **Tess** | Testing | ✅ Active |
| **Morgan** | Project management | ✅ Active |
| **Atlas** | Git/GitHub operations | ✅ Active |

**Removed** (not in current stack):
- ~~Spark~~ (Research - future)
- ~~Nova~~ (AI - future)

---

## Implementation Status

### ✅ Completed

- [x] CLI invoke partials created (`clis/{cli}/invoke.sh.hbs`)
- [x] Healer workflow container (`healer/container.sh.hbs`)
- [x] Healer system prompt (`healer/system-prompt.md.hbs`)
- [x] Agent identities in place (`agents/{name}/identity.md.hbs`)
- [x] Shared configs in place (`shared/mcp.json.hbs`, `shared/tools-config.json.hbs`)
- [x] Removed Spark/Nova agents (not in stack)
- [x] Multi-agent consensus documented (`huddle/CONSENSUS.md`)

### 🔲 Pending

- [ ] Update controller template composition logic
- [ ] Refactor `code/container.sh.hbs` to use CLI partials
- [ ] Test all agent × CLI × workflow combinations
- [ ] BACKLOG: Wrap Healer CRD in Argo Workflow (match Play API pattern)

---

## Adding New Agents (Option D Benefit)

To add a new agent (e.g., "Nova"):

```bash
# 1. Create agent identity (1 file)
templates/agents/nova/identity.md.hbs

# 2. Create agent tools config (1 file)
templates/agents/nova/tools.hbs

# Done! Works across all CLIs and workflows automatically.
```

**No controller changes required** - agent is injected via Handlebars context.

---

## Adding New CLIs (Option D Benefit)

To add a new CLI (e.g., "Gemini"):

```bash
# 1. Create CLI invoke partial
templates/clis/gemini/invoke.sh.hbs

# 2. Create CLI config files
templates/clis/gemini/config.json.hbs

# 3. Update controller CLI enum (one change)
```

**All agents automatically work with new CLI.**

---

## References

- [Template Structure Options](./template-structure-options.md)
- [Huddle Consensus](../huddle/CONSENSUS.md)
- [Claude CLI Reference](https://code.claude.com/docs/en/cli-reference)
- [Factory CLI Reference](https://docs.factory.ai/reference/cli-reference)
- [Codex CLI Reference](https://developers.openai.com/codex/cli/reference/)






