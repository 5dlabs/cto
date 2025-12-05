# Template Structure Options

## Current Understanding

### Workflows
- **Code (Play)**: Feature implementation from TaskMaster tasks
  - Prompt comes from GitHub repo (docs service)
  - Full feature branch workflow
  - System prompt: "Implement this feature according to acceptance criteria"

- **Healer**: CI failure remediation
  - Prompt comes from PVC files (`${HEAL_PROMPT_FILE}`)
  - Git worktree isolation for concurrent agents
  - System prompt: "Fix this specific CI failure"

### CLI Differences

| Feature | Claude Code | Factory (Droid) | OpenAI Codex |
|---------|-------------|-----------------|--------------|
| **Command** | `claude -p` | `droid exec` | `codex exec` |
| **System Prompt** | `--system-prompt-file` | Custom droids / inline | `AGENTS.md` file |
| **Tool Permissions** | `--allowedTools` patterns | `--auto level` + `--enabled-tools` | `--sandbox` + `--ask-for-approval` |
| **Output Format** | `--output-format stream-json` | `-o stream-json` | `--json` |
| **MCP Config** | `--mcp-config path` | `/mcp add` | `codex mcp add` |
| **Model Override** | managed-settings.json | `-m model-id` | `-m model-id` |
| **Continue Session** | `--continue` | `-s session-id` | `codex resume` |

### What CLIs Should Contain
CLIs should **only** account for:
1. Config file formats (settings.json vs config.toml vs AGENTS.md)
2. Command structure differences (flags, arguments)
3. Tool permission syntax
4. Output format handling

---

## Option A: Workflow-First

```
templates/
├── shared/                    # Truly shared utilities
│   ├── functions/             # GitHub auth, git ops
│   │   ├── github-auth.sh.hbs
│   │   └── git-operations.sh.hbs
│   └── bootstrap/             # Environment setup
│       └── rust-env.sh.hbs
│
├── agents/                    # Agent identities (shared across workflows)
│   ├── rex.md.hbs             # "You are Rex, Rust specialist..."
│   ├── blaze.md.hbs           # "You are Blaze, Frontend specialist..."
│   ├── bolt.md.hbs
│   ├── cipher.md.hbs
│   ├── atlas.md.hbs
│   ├── cleo.md.hbs
│   ├── tess.md.hbs
│   ├── spark.md.hbs
│   └── morgan.md.hbs
│
├── code/                      # Play workflow
│   ├── system-prompt.hbs      # "Implement this feature..."
│   ├── claude/
│   │   ├── container.sh.hbs
│   │   ├── config.json.hbs
│   │   └── settings.json.hbs
│   ├── factory/
│   │   ├── container.sh.hbs
│   │   └── factory-cli-config.json.hbs
│   └── codex/
│       ├── container.sh.hbs
│       └── config.toml.hbs
│
└── healer/                    # Healer workflow
    ├── system-prompt.hbs      # "Fix this CI failure..."
    ├── claude/
    │   ├── container.sh.hbs
    │   └── config.json.hbs
    ├── factory/
    │   ├── container.sh.hbs
    │   └── factory-cli-config.json.hbs
    └── codex/
        ├── container.sh.hbs
        └── config.toml.hbs
```

### Pros
- Clear separation of workflows
- System prompts close to their containers
- Easy to understand which files belong to which workflow

### Cons
- CLI container logic duplicated across workflows
- Config files duplicated (e.g., `settings.json` in both code/ and healer/)

---

## Option B: CLI-First with Workflow Prompts

```
templates/
├── shared/                    # Truly shared utilities
│   ├── functions/
│   │   ├── github-auth.sh.hbs
│   │   └── git-operations.sh.hbs
│   └── bootstrap/
│       └── rust-env.sh.hbs
│
├── agents/                    # Agent identities with workflow variants
│   ├── rex/
│   │   ├── identity.md.hbs    # Core: "You are Rex, Rust specialist"
│   │   ├── code.hbs           # Code workflow additions
│   │   └── healer.hbs         # Healer workflow additions
│   ├── blaze/
│   │   ├── identity.md.hbs
│   │   ├── code.hbs
│   │   └── healer.hbs
│   └── ...
│
├── clis/                      # CLI-specific (shared across workflows)
│   ├── claude/
│   │   ├── container-base.sh.hbs   # Common container setup
│   │   ├── execute.sh.hbs          # How to invoke claude CLI
│   │   ├── config.json.hbs
│   │   └── settings.json.hbs
│   ├── factory/
│   │   ├── container-base.sh.hbs
│   │   ├── execute.sh.hbs
│   │   └── factory-cli-config.json.hbs
│   └── codex/
│       ├── container-base.sh.hbs
│       ├── execute.sh.hbs
│       └── config.toml.hbs
│
└── workflows/                 # Workflow-specific wrappers
    ├── code/
    │   ├── system-prompt.hbs  # "Implement this feature..."
    │   └── setup.sh.hbs       # Docs service, TaskMaster loading
    └── healer/
        ├── system-prompt.hbs  # "Fix this CI failure..."
        └── setup.sh.hbs       # PVC file loading, worktree setup
```

### Pros
- CLI logic in one place
- Agent prompts can have workflow-specific additions
- Less duplication of CLI configs

### Cons
- Requires composition logic in controller
- Agent directory structure is more complex

---

## Option C: Matrix Composition (Most DRY)

```
templates/
├── core/                      # Universal primitives
│   ├── git-setup.sh.hbs
│   ├── github-auth.sh.hbs
│   └── rust-env.sh.hbs
│
├── agents/                    # WHO - Agent identity only
│   ├── rex.md.hbs             # "You are Rex, Rust specialist..."
│   ├── blaze.md.hbs           # "You are Blaze, Frontend specialist..."
│   ├── bolt.md.hbs
│   ├── cipher.md.hbs
│   ├── atlas.md.hbs
│   ├── cleo.md.hbs
│   ├── tess.md.hbs
│   ├── spark.md.hbs
│   └── morgan.md.hbs
│
├── clis/                      # HOW - CLI execution only
│   ├── claude/
│   │   ├── invoke.sh.hbs      # claude -p --output-format stream-json...
│   │   ├── config.json.hbs
│   │   └── settings.json.hbs
│   ├── factory/
│   │   ├── invoke.sh.hbs      # droid exec -o stream-json...
│   │   └── factory-cli-config.json.hbs
│   └── codex/
│       ├── invoke.sh.hbs      # codex exec --json...
│       └── config.toml.hbs
│
├── workflows/                 # WHAT - Workflow context only
│   ├── code/
│   │   ├── system-prompt.hbs  # Feature implementation context
│   │   ├── setup.sh.hbs       # Docs service, TaskMaster, feature branches
│   │   └── teardown.sh.hbs    # PR creation, cleanup
│   └── healer/
│       ├── system-prompt.hbs  # Remediation context
│       ├── setup.sh.hbs       # PVC files, worktrees, failure context
│       └── teardown.sh.hbs    # Worktree cleanup
│
└── container.sh.hbs           # Master template that composes:
                               # core/* + agents/{agent} + clis/{cli}/* + workflows/{workflow}/*
```

### Composition Logic
```
Final Container = 
  core/git-setup.sh.hbs +
  core/github-auth.sh.hbs +
  workflows/{workflow}/setup.sh.hbs +
  agents/{agent}.md.hbs (into system prompt) +
  workflows/{workflow}/system-prompt.hbs +
  clis/{cli}/invoke.sh.hbs +
  workflows/{workflow}/teardown.sh.hbs
```

### Pros
- Maximum DRY - each concept defined once
- Clear separation: WHO (agent) × HOW (CLI) × WHAT (workflow)
- Adding new CLI = add one folder
- Adding new workflow = add one folder
- Adding new agent = add one file

### Cons
- Most complex composition logic required
- Harder to understand full container without reading multiple files
- Debugging requires tracing through composition

---

## Option D: Hybrid (Balanced)

```
templates/
├── shared/                    # Common utilities (partials)
│   ├── git.sh.hbs             # Git setup, auth
│   ├── rust-env.sh.hbs
│   ├── node-env.sh.hbs
│   └── mcp.json.hbs
│
├── agents/                    # Agent identities (single files, shared)
│   ├── rex.md.hbs
│   ├── blaze.md.hbs
│   ├── bolt.md.hbs
│   ├── cipher.md.hbs
│   ├── atlas.md.hbs
│   ├── cleo.md.hbs
│   ├── tess.md.hbs
│   ├── spark.md.hbs
│   └── morgan.md.hbs
│
├── clis/                      # CLI configs + execution partials only
│   ├── claude/
│   │   ├── config.json.hbs
│   │   ├── settings.json.hbs
│   │   └── invoke.sh.hbs      # Just: claude -p --output-format...
│   ├── factory/
│   │   ├── factory-cli-config.json.hbs
│   │   └── invoke.sh.hbs      # Just: droid exec -o stream-json...
│   └── codex/
│       ├── config.toml.hbs
│       └── invoke.sh.hbs      # Just: codex exec --json...
│
├── code/                      # Complete Play workflow containers
│   ├── system-prompt.hbs      # "Implement this feature..."
│   └── container.sh.hbs       # Full container using partials:
│                              # {{> shared/git}}
│                              # {{> shared/rust-env}}
│                              # ... docs service setup ...
│                              # {{> clis/{cli}/invoke}}
│
└── healer/                    # Complete Healer workflow containers
    ├── system-prompt.hbs      # "Fix this CI failure..."
    └── container.sh.hbs       # Full container using partials:
                               # {{> shared/git}}
                               # {{> shared/rust-env}}
                               # ... PVC/worktree setup ...
                               # {{> clis/{cli}/invoke}}
```

### Pros
- Workflows have complete, readable containers
- CLI-specific logic shared via small partials
- Agent identities fully shared
- Easier to debug (can read one container file)
- Less complex composition than Option C

### Cons
- Some duplication between code/container.sh.hbs and healer/container.sh.hbs
- Need to maintain partials in sync

---

## Recommendation

**Option D (Hybrid)** strikes the best balance:

1. **Workflows own their containers** - Easy to understand and debug
2. **CLIs provide execution partials** - No duplication of CLI invocation logic
3. **Agents are simple single files** - Shared across all workflows
4. **Shared utilities via partials** - Git, env setup used everywhere

### Migration Path
1. Create `clis/` with just config files and `invoke.sh.hbs` partials
2. Keep `agents/` as simple identity files
3. Refactor `code/container.sh.hbs` to use `{{> clis/{cli}/invoke}}`
4. Create `healer/container.sh.hbs` using same partials

### Controller Changes
Minimal - just need to:
1. Select the right `clis/{cli}/invoke.sh.hbs` partial based on CLI type
2. Include the right `agents/{agent}.md.hbs` in system prompt
3. Render the workflow-specific container template

---

## Decision Matrix

| Criteria | Option A | Option B | Option C | Option D |
|----------|----------|----------|----------|----------|
| DRY (less duplication) | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Readability | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Controller complexity | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Adding new CLI | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| Adding new workflow | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Adding new agent** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Debugging ease | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |

**Legend**: ⭐ = Poor, ⭐⭐⭐⭐⭐ = Excellent

---

## 🚀 Best Option for Adding New Agents

**Options A, C, and D are all excellent for adding new agents** because they use single-file agent definitions.

### Why Single-File Agents Win

In Options A, C, and D, adding a new agent requires:
```bash
# Just create ONE file:
templates/agents/nova.md.hbs
```

The file contains:
- Agent identity ("You are Nova, the AI/ML specialist...")
- Domain expertise and rules
- Validation commands
- Definition of done

**That's it.** No other files to create, no workflow-specific variants.

### Option B is Worse for New Agents

Option B requires creating a **directory with multiple files** per agent:
```bash
templates/agents/nova/
├── identity.md.hbs    # Core identity
├── code.hbs           # Code workflow additions  
└── healer.hbs         # Healer workflow additions
```

This means:
- 3 files per agent instead of 1
- Must remember to create workflow-specific variants
- Higher chance of forgetting one, causing runtime errors

### Recommended: Option C or D

| Aspect | Option C | Option D |
|--------|----------|----------|
| Files to create | 1 (`agents/nova.md.hbs`) | 1 (`agents/nova.md.hbs`) |
| Controller changes | None | None |
| Works immediately | ✅ Yes | ✅ Yes |
| Complexity tradeoff | More complex composition | Simpler, workflows own containers |

**For maximum agent scalability, choose Option C or D.**

### Example: Adding a New Agent

```bash
# 1. Create the agent file
cat > templates/agents/nova.md.hbs << 'EOF'
# Agent Identity: Nova (AI/ML Specialist)

You are **Nova**, the **AI/ML specialist** agent.

## Core Specialization
- Machine Learning: PyTorch, TensorFlow, scikit-learn
- Data: Pandas, NumPy, data pipelines
- MLOps: Model deployment, monitoring, versioning
- LLMs: Fine-tuning, RAG, prompt engineering

## Execution Rules
1. **Reproducibility first.** Set random seeds, version data.
2. **Document experiments.** Track metrics, hyperparameters.
3. **Test thoroughly.** Unit tests for data transforms.
...
EOF

# 2. That's it! The agent is now available for all workflows and CLIs.
```

### Agent Checklist Template

When adding a new agent, include:
- [ ] **Identity**: Name and specialization
- [ ] **Core expertise**: Technologies, frameworks, tools
- [ ] **Execution rules**: Domain-specific best practices
- [ ] **Validation commands**: How to verify work (e.g., `pytest`, `cargo test`)
- [ ] **Definition of done**: Clear completion criteria

