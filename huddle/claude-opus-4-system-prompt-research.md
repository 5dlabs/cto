# System Prompt Research: CLI Comparison for CTO Platform

**Model:** Claude Opus 4  
**Date:** 2025-12-05  
**Status:** Research Complete

---

## Executive Summary

This research analyzes how each of the 6 supported CLIs handles system prompts and agent identity, with the goal of optimizing our `AGENTS.md` structure for cross-CLI compatibility. The industry has largely converged on `AGENTS.md` as the open standard for guiding coding agents, with minor variations in file naming and discovery mechanisms.

### Key Finding: AGENTS.md is the Universal Standard

OpenAI's `AGENTS.md` initiative has achieved broad adoption across the ecosystem. All 6 supported CLIs now support some form of `AGENTS.md`-compatible mechanism:

| CLI | Primary File | Fallback/Alternatives | Discovery Scope |
|-----|-------------|----------------------|-----------------|
| **Claude Code** | `CLAUDE.md` | `AGENTS.md` via imports | Hierarchical (root → cwd → subtrees) |
| **OpenAI Codex** | `AGENTS.md` | `AGENTS.override.md`, custom fallbacks | Global → Project → Subdirectory |
| **Cursor** | `.cursor/rules/*.mdc` | `AGENTS.md` | Project root + nested |
| **Gemini CLI** | `GEMINI.md` | `AGENTS.md` (configurable via `contextFileName`) | Global → Project → Subdirectory |
| **OpenCode** | `AGENTS.md` | Custom instructions via `opencode.json` | Global → Project |
| **Aider** | `CONVENTIONS.md` | `AGENTS.md` (via `read:` config) | Configured paths |

---

## Detailed CLI Analysis

### 1. Claude Code CLI

**Documentation Source:** https://code.claude.com/docs/en/memory

#### Memory Hierarchy (Precedence Order)

1. **Enterprise Policy**: `/Library/Application Support/ClaudeCode/CLAUDE.md` (macOS)
2. **Project Memory**: `./CLAUDE.md` or `./.claude/CLAUDE.md`
3. **User Memory**: `~/.claude/CLAUDE.md`
4. **Project Local**: `./CLAUDE.local.md` (auto-gitignored)

#### System Prompt Mechanisms

| Flag | Behavior | Mode |
|------|----------|------|
| `--system-prompt` | **Replaces** entire default prompt | Interactive + Print |
| `--system-prompt-file` | **Replaces** with file contents | Print only |
| `--append-system-prompt` | **Appends** to default prompt | Interactive + Print |

**Best Practice:** Use `--append-system-prompt` to preserve Claude Code's built-in capabilities while adding custom requirements.

#### AGENTS.md Support

Claude Code supports `@path/to/import` syntax to include external files:

```markdown
# CLAUDE.md
See @README for project overview.
@~/.claude/my-project-instructions.md  # User-specific instructions
```

#### Key Insights

- Memory files are **automatically loaded** into context when launched
- Files higher in hierarchy take precedence
- `CLAUDE.local.md` is ideal for private project-specific preferences
- Supports recursive imports with max-depth of 5 hops

---

### 2. OpenAI Codex CLI

**Documentation Source:** https://developers.openai.com/codex/guides/agents-md

#### Discovery Order (Precedence)

1. **Global Scope**: `~/.codex/AGENTS.override.md` OR `~/.codex/AGENTS.md`
2. **Project Scope**: Walk from repo root to cwd, checking each directory
3. **Merge Order**: Files concatenated root-down (later files override earlier)

#### File Naming

| File | Purpose |
|------|---------|
| `AGENTS.md` | Standard instructions |
| `AGENTS.override.md` | Temporary override (takes precedence) |
| Fallback filenames | Configurable via `project_doc_fallback_filenames` |

#### Configuration

```toml
# ~/.codex/config.toml
project_doc_fallback_filenames = ["TEAM_GUIDE.md", ".agents.md"]
project_doc_max_bytes = 65536  # Default: 32768
```

#### Key Insights

- **No CLI flags** for system prompt - uses file-based discovery only
- Stops reading once combined size reaches `project_doc_max_bytes` (32 KiB default)
- Supports nested `AGENTS.md` files for monorepo subprojects
- **OpenAI Codex repo has 88 `AGENTS.md` files** for their own project

---

### 3. Cursor IDE

**Documentation Source:** https://cursor.com/docs/context/rules

#### Rule Types

| Type | Behavior | Location |
|------|----------|----------|
| **Project Rules** | Version-controlled, scoped to codebase | `.cursor/rules/*.mdc` |
| **User Rules** | Global preferences | Cursor Settings → Rules |
| **Team Rules** | Organization-wide (Team/Enterprise) | Cursor Dashboard |
| **AGENTS.md** | Simple markdown alternative | Project root + subdirectories |

#### MDC Format (Project Rules)

```markdown
---
globs: ["*.ts", "*.tsx"]
alwaysApply: false
---

- Use TypeScript for all new files
- Prefer functional components
```

#### Rule Application Types

- `Always Apply`: Every chat session
- `Apply Intelligently`: When Agent decides relevant
- `Apply to Specific Files`: When file matches glob pattern
- `Apply Manually`: When @-mentioned

#### AGENTS.md Support

Cursor supports `AGENTS.md` as a **simple alternative** to `.cursor/rules`:

```markdown
# AGENTS.md
## Code Style
- Use TypeScript for all new files
- Prefer functional components in React
```

#### Key Insights

- Rules are **included at the start of model context**
- Best practice: Keep rules under 500 lines
- **Supports nested AGENTS.md** in subdirectories
- `.cursorrules` (legacy) will be deprecated

---

### 4. Gemini CLI

**Documentation Source:** https://geminicli.com/docs/get-started/configuration/

#### Configuration Hierarchy

1. **Default Values**: Hardcoded
2. **System Defaults**: `/etc/gemini-cli/system-defaults.json`
3. **User Settings**: `~/.gemini/settings.json`
4. **Project Settings**: `.gemini/settings.json`
5. **System Settings**: `/etc/gemini-cli/settings.json` (override all)
6. **Environment Variables**: `GEMINI_SYSTEM_MD`
7. **Command-line Arguments**: `--model`, etc.

#### Context File Configuration

```json
{
  "context": {
    "fileName": ["GEMINI.md", "AGENTS.md"],  // Supports arrays!
    "discoveryMaxDirs": 200,
    "includeDirectories": ["path/to/dir1"],
    "loadMemoryFromIncludeDirectories": false
  }
}
```

#### System Prompt Override

Set `GEMINI_SYSTEM_MD` environment variable to completely replace the system prompt:

```bash
export GEMINI_SYSTEM_MD="./custom-system.md"
gemini
```

#### Key Insights

- **Supports multiple context file names** in array format
- Can load `AGENTS.md` via `contextFileName` setting
- `/memory refresh` command to reload context files
- Hierarchical loading from global → project → subdirectories
- Import syntax: `@path/to/file.md` supported

---

### 5. OpenCode CLI

**Documentation Source:** https://opencode.ai/docs/rules/

#### File Locations

| Location | Scope | Purpose |
|----------|-------|---------|
| `~/.config/opencode/AGENTS.md` | Global | Personal rules for all sessions |
| `./AGENTS.md` | Project | Team-shared instructions |

#### Discovery Order

1. Traverse up from current directory
2. Check global `~/.config/opencode/AGENTS.md`
3. Combine all found files

#### Custom Instructions

```json
// opencode.json
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": ["CONTRIBUTING.md", "docs/guidelines.md", ".cursor/rules/*.md"]
}
```

#### Key Insights

- **Native AGENTS.md support** (same as Codex pattern)
- `/init` command generates AGENTS.md by scanning project
- Supports referencing external files with `@path/to/file.md` syntax
- Can load existing Cursor rules via glob patterns

---

### 6. Aider

**Documentation Source:** https://aider.chat/docs/usage/conventions.html

#### Configuration

```yaml
# .aider.conf.yml
read: CONVENTIONS.md              # Single file
read: [CONVENTIONS.md, AGENTS.md] # Multiple files
```

#### Usage Pattern

```bash
# Load conventions as read-only (best for caching)
aider --read CONVENTIONS.md
aider --read AGENTS.md
```

#### Key Insights

- **Supports any filename** via `read:` configuration
- Files loaded with `--read` are marked read-only and cached
- [Community conventions repository](https://github.com/Aider-AI/conventions) available
- Can work with `AGENTS.md` seamlessly

---

## Best Practices for Writing AGENTS.md

Based on research from HumanLayer and the arXiv paper "AI Agents That Matter":

### 1. Less is More

> **Frontier thinking LLMs can follow ~150-200 instructions with reasonable consistency.**

Claude Code's system prompt already contains ~50 instructions, leaving ~100-150 for your custom instructions.

**Recommendation:** Keep `AGENTS.md` under 300 lines, ideally under 60 lines for the root file.

### 2. Use Progressive Disclosure

Instead of putting everything in one file:

```markdown
# AGENTS.md

## External Documentation
For TypeScript conventions: @docs/typescript-guidelines.md
For API standards: @docs/api-standards.md
```

### 3. Content Structure (WHAT, WHY, HOW)

| Section | Purpose | Example |
|---------|---------|---------|
| **WHAT** | Project structure, tech stack | "This is a Rust + TypeScript monorepo" |
| **WHY** | Purpose and function | "The controller orchestrates agent workflows" |
| **HOW** | Commands and workflows | "Run `cargo test -p controller` before committing" |

### 4. Don't Duplicate Linters

> **"Never send an LLM to do a linter's job"**

Code style guidelines bloat the context and reduce instruction-following quality. Use:
- Deterministic linters (Clippy, ESLint, Prettier)
- Pre-commit hooks for formatting
- Git hooks for validation

### 5. Avoid Auto-Generation

`/init` commands generate bloated files. Carefully craft each line because `AGENTS.md` is **the highest leverage point** of the harness.

---

## AGENTS.md Compatibility Matrix

### File Naming Support

| CLI | `AGENTS.md` | `CLAUDE.md` | `.cursorrules` | Custom Names |
|-----|-------------|-------------|----------------|--------------|
| Claude Code | Via import | ✅ Native | ❌ | Via import |
| Codex | ✅ Native | ❌ | ❌ | Via config |
| Cursor | ✅ Supported | ❌ | Deprecated | `.cursor/rules/` |
| Gemini | Via config | ❌ | ❌ | Via `fileName` array |
| OpenCode | ✅ Native | ❌ | Via glob | Via `instructions` |
| Aider | Via config | ❌ | ❌ | Any via `read:` |

### Nested File Discovery

| CLI | Subdirectory Discovery | Monorepo Support |
|-----|------------------------|------------------|
| Claude Code | ✅ Automatic on file access | ✅ |
| Codex | ✅ Walk from root to cwd | ✅ |
| Cursor | ✅ Nested `.cursor/rules/` | ✅ |
| Gemini | ✅ Via `discoveryMaxDirs` | ✅ |
| OpenCode | ✅ Traverse up from cwd | ✅ |
| Aider | ❌ Explicit paths only | Partial |

---

## Recommendations for CTO Platform

### 1. Primary File: `AGENTS.md`

Use `AGENTS.md` as the universal file name:
- Supported natively by Codex, OpenCode, Cursor
- Easily configured for Gemini, Claude Code, Aider
- Industry standard with 20k+ open source projects

### 2. Fallback Symlinks for Claude Code

```bash
# For Claude Code compatibility
ln -s AGENTS.md CLAUDE.md
```

Or use Claude Code's import syntax in `CLAUDE.md`:
```markdown
@AGENTS.md
```

### 3. Template Structure (Option D Implementation)

Based on the existing consensus:

```
templates/
├── agents/                    # Single-file agent identities
│   ├── rex.md.hbs            # One file per agent
│   ├── blaze.md.hbs
│   └── ...
│
├── shared/                    # Common utilities (partials)
│   ├── git.sh.hbs
│   ├── rust-env.sh.hbs
│   └── ...
│
├── clis/                      # CLI-specific invoke partials
│   ├── claude/
│   │   └── invoke.sh.hbs     # Uses --append-system-prompt
│   ├── codex/
│   │   └── invoke.sh.hbs     # Uses AGENTS.md discovery
│   ├── cursor/
│   │   └── invoke.sh.hbs     # Uses .cursor/rules/
│   ├── gemini/
│   │   └── invoke.sh.hbs     # Uses GEMINI.md or --model
│   ├── opencode/
│   │   └── invoke.sh.hbs     # Uses AGENTS.md discovery
│   └── aider/
│       └── invoke.sh.hbs     # Uses --read flag
│
└── code/                      # Workflow-specific containers
    └── container.sh.hbs      # {{> clis/{cli}/invoke }}
```

### 4. Container Image Structure

```
/workspace/
├── AGENTS.md                  # Primary (generated from templates/agents/{agent}.md.hbs)
├── CLAUDE.md -> AGENTS.md     # Symlink for Claude Code
├── GEMINI.md -> AGENTS.md     # Symlink for Gemini CLI
├── .claude/
│   └── CLAUDE.md -> ../AGENTS.md
├── .gemini/
│   └── settings.json          # contextFileName: "AGENTS.md"
├── .cursor/
│   └── rules/
│       └── agent.mdc -> ../../AGENTS.md
└── .aider.conf.yml            # read: AGENTS.md
```

### 5. CLI Invocation Patterns

#### Claude Code
```bash
claude --append-system-prompt "$(cat /workspace/system-prompt.txt)" \
       --print "Your task: ${TASK_PROMPT}"
```

#### OpenAI Codex
```bash
# AGENTS.md auto-discovered in /workspace
codex --ask-for-approval never "${TASK_PROMPT}"
```

#### Gemini CLI
```bash
# GEMINI.md or AGENTS.md auto-discovered
gemini -p "${TASK_PROMPT}"
```

#### Cursor CLI
```bash
# .cursor/rules/ auto-discovered
cursor chat "${TASK_PROMPT}"
```

#### OpenCode
```bash
# AGENTS.md auto-discovered
opencode run "${TASK_PROMPT}"
```

#### Aider
```bash
aider --read AGENTS.md --message "${TASK_PROMPT}"
```

---

## Document Consumption Strategy

### For Generated Documentation (TaskMaster)

Based on the linked cto-parallel-test repo pattern, generated docs should:

1. **Store in `.taskmaster/` directory**:
   ```
   docs/.taskmaster/
   ├── tasks.json           # Task definitions
   ├── docs/
   │   ├── architecture.md  # Generated architecture doc
   │   ├── api-reference.md # Generated API docs
   │   └── ...
   └── prompts/
       └── agent-prompt.md  # Task-specific prompts
   ```

2. **Reference via AGENTS.md imports**:
   ```markdown
   # AGENTS.md
   ## Project Documentation
   @docs/.taskmaster/docs/architecture.md
   @docs/.taskmaster/docs/api-reference.md
   ```

3. **Lazy loading pattern** (from OpenCode research):
   ```markdown
   CRITICAL: When you encounter a file reference (e.g., @rules/general.md), 
   use your Read tool to load it on a need-to-know basis.
   ```

---

## Research References

1. **HumanLayer - Writing a Good CLAUDE.md**: https://www.humanlayer.dev/blog/writing-a-good-claude-md
2. **arXiv Paper - AI Agents That Matter**: https://arxiv.org/abs/2407.01502
3. **OpenAI AGENTS.md Specification**: https://agents.md/
4. **Claude Code Memory Documentation**: https://code.claude.com/docs/en/memory
5. **Codex AGENTS.md Guide**: https://developers.openai.com/codex/guides/agents-md
6. **Gemini CLI Configuration**: https://geminicli.com/docs/get-started/configuration/
7. **Cursor Rules Documentation**: https://cursor.com/docs/context/rules
8. **OpenCode Rules**: https://opencode.ai/docs/rules/
9. **Aider Conventions**: https://aider.chat/docs/usage/conventions.html

---

## Conclusion

The industry has converged on `AGENTS.md` as the standard for guiding coding agents. Our CTO platform should:

1. **Adopt `AGENTS.md` as the primary file name** for maximum compatibility
2. **Use symlinks** (`CLAUDE.md`, `GEMINI.md`) for CLI-specific compatibility
3. **Keep agent identities in single files** (`templates/agents/{agent}.md.hbs`)
4. **Generate CLI-specific configs** as part of container setup
5. **Use progressive disclosure** with imports for large documentation sets
6. **Keep root `AGENTS.md` under 300 lines** (ideally <100) for optimal instruction following

The Option D (Hybrid) template structure already aligns well with this research. The main enhancement is ensuring the generated `AGENTS.md` files are compatible with all 6 CLIs through symlinks and configuration files.

---

**Research Complete**  
**Claude Opus 4**  
**2025-12-05**

