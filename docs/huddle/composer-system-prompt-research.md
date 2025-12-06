# System Prompt Research: CLI Comparison & Harmonization Strategy

**Researcher**: Composer (Claude Sonnet 4.5)  
**Date**: 2025-12-05  
**Status**: Complete

---

## Executive Summary

This research analyzes how each of the 6 supported CLIs handles system prompts and agent identity configuration. Key findings:

1. **AGENTS.md/CLAUDE.md/GEMINI.md convergence**: Most CLIs now support markdown-based agent instruction files, creating a harmonization opportunity
2. **Three primary mechanisms**: File-based (AGENTS.md), flag-based (`--system-prompt-file`), and config-based (settings.json)
3. **Hierarchical discovery**: Multiple CLIs support hierarchical file discovery (global → project → subdirectory)
4. **Recommendation**: Adopt AGENTS.md as the universal format with CLI-specific adapters

---

## CLI-by-CLI Analysis

### 1. Claude Code CLI

**Documentation**: https://code.claude.com/docs/en/cli-reference

#### System Prompt Mechanisms

| Mechanism | Flag/Config | Format | Modes | Notes |
|-----------|-------------|--------|-------|-------|
| **File-based** | `--system-prompt-file <path>` | Markdown | Print only | Replaces entire default prompt |
| **Inline** | `--system-prompt <text>` | Text | Interactive + Print | Replaces entire default prompt |
| **Append** | `--append-system-prompt <text>` | Text | Interactive + Print | Adds to default prompt (recommended) |
| **Memory file** | `CLAUDE.md` | Markdown | Auto-loaded | Loaded automatically in every session |

#### Key Findings

- **CLAUDE.md behavior**: Claude Code wraps CLAUDE.md content in a `<system-reminder>` tag that tells Claude to ignore it unless highly relevant. This is intentional to prevent prompt bloat.
- **Best practice**: Use `--append-system-prompt` to preserve Claude Code's built-in capabilities while adding custom instructions
- **Settings.json**: No direct system prompt configuration in settings.json, but supports subagents via `~/.claude/agents/` and `.claude/agents/`
- **Subagents**: Can be defined as markdown files with YAML frontmatter in user/project agent directories

#### CLI Flags (Relevant)

```bash
claude -p --system-prompt-file ./prompt.md "query"
claude --append-system-prompt "Always use TypeScript"
claude --system-prompt "You are a Python expert"
```

#### Documentation Insights

From HumanLayer blog post:
- **Instruction limit**: Frontier thinking LLMs can follow ~150-200 instructions consistently
- **Claude Code system prompt**: Contains ~50 individual instructions
- **Recommendation**: Keep CLAUDE.md < 300 lines, ideally < 60 lines
- **Progressive disclosure**: Use separate markdown files referenced from CLAUDE.md rather than including everything

---

### 2. Factory (Droid) CLI

**Documentation**: https://docs.factory.ai/reference/cli-reference

#### System Prompt Mechanisms

| Mechanism | Flag/Config | Format | Modes | Notes |
|-----------|-------------|--------|-------|-------|
| **File-based** | `agents.md` (project root) | Markdown | Auto-loaded | Similar to AGENTS.md standard |
| **Custom droids** | `/droids` command | Markdown + YAML | Interactive | User-defined specialized agents |
| **Specification mode** | `--use-spec` | N/A | Exec mode | Plan before executing |

#### Key Findings

- **No explicit system prompt flag**: Factory relies on `agents.md` files in project root
- **Custom droids**: Can be created via `/droids` slash command with custom prompts
- **Specification mode**: `--use-spec` enables planning phase before execution
- **Autonomy levels**: Control what operations agent can perform (`--auto low/medium/high`)

#### CLI Flags (Relevant)

```bash
droid exec --use-spec "add user profiles"
droid exec --auto medium "run tests"
```

#### Documentation Insights

- Factory focuses on autonomy levels rather than explicit system prompt configuration
- Custom droids can be configured with specialized prompts for specific tasks
- No mention of hierarchical file discovery (unlike Codex/Gemini)

---

### 3. OpenAI Codex CLI

**Documentation**: https://developers.openai.com/codex/guides/agents-md

#### System Prompt Mechanisms

| Mechanism | Flag/Config | Format | Modes | Notes |
|-----------|-------------|--------|-------|-------|
| **Hierarchical files** | `AGENTS.md` / `AGENTS.override.md` | Markdown | Auto-loaded | Global → Project → Subdirectory |
| **Config fallback** | `project_doc_fallback_filenames` | Config | Auto-loaded | Custom filenames (e.g., `TEAM_GUIDE.md`) |
| **Config limit** | `project_doc_max_bytes` | Config | Auto-loaded | Default: 32 KiB |

#### Key Findings

- **Hierarchical discovery**: Codex walks directory tree loading AGENTS.md files
- **Precedence order**:
  1. Global: `~/.codex/AGENTS.override.md` (if exists) else `~/.codex/AGENTS.md`
  2. Project: Root → current directory, checking `AGENTS.override.md` then `AGENTS.md`
  3. Concatenation: Files merged from root down (later files override earlier)
- **Override pattern**: `AGENTS.override.md` takes precedence over `AGENTS.md` at same level
- **Custom filenames**: Can configure fallback names via `project_doc_fallback_filenames` in `~/.codex/config.toml`

#### CLI Flags (Relevant)

```bash
codex exec "query"  # Automatically loads AGENTS.md files
```

#### Documentation Insights

- **Official AGENTS.md spec**: OpenAI maintains https://agents.md/ as the standard
- **Example format**: Simple markdown with project guidelines, coding conventions, useful commands
- **Discovery stops**: At current working directory (doesn't search subdirectories below)
- **Size limit**: Default 32 KiB combined size, configurable via `project_doc_max_bytes`

---

### 4. Cursor CLI

**Documentation**: https://cursor.com/docs/cli/using

#### System Prompt Mechanisms

| Mechanism | Flag/Config | Format | Modes | Notes |
|-----------|-------------|--------|-------|-------|
| **File-based** | `AGENTS.md` / `CLAUDE.md` | Markdown | Auto-loaded | Automatically detected |
| **Config** | `.cursor/cli-config.json` | JSON | Auto-loaded | CLI-specific configuration |
| **MCP config** | `.cursor/mcp.json` | JSON | Auto-loaded | MCP server configuration |

#### Key Findings

- **Dual format support**: Cursor supports both `AGENTS.md` and `CLAUDE.md` (for compatibility)
- **Automatic detection**: Files are automatically loaded when present
- **Headless mode**: Uses `--print --output-format stream-json --force` for automation
- **No explicit system prompt flag**: Relies on file-based approach

#### CLI Flags (Relevant)

```bash
cursor --print --output-format stream-json --force "query"
```

#### Documentation Insights

- Cursor CLI automatically detects and respects `mcp.json` configuration
- Supports both AGENTS.md and CLAUDE.md for maximum compatibility
- Headless mode designed for automation workflows

---

### 5. Gemini CLI

**Documentation**: https://geminicli.com/docs/get-started/configuration/

#### System Prompt Mechanisms

| Mechanism | Flag/Config | Format | Modes | Notes |
|-----------|-------------|--------|-------|-------|
| **Hierarchical files** | `GEMINI.md` (configurable) | Markdown | Auto-loaded | Global → Project → Subdirectory |
| **Config setting** | `context.fileName` | Config | Auto-loaded | Can specify custom filename(s) |
| **Import syntax** | `@path/to/file.md` | Markdown | Auto-loaded | Import other markdown files |
| **Discovery limit** | `context.discoveryMaxDirs` | Config | Auto-loaded | Default: 200 directories |

#### Key Findings

- **Hierarchical discovery**: Similar to Codex, walks directory tree
- **Loading order**:
  1. Global: `~/.gemini/GEMINI.md`
  2. Project root and ancestors: Searches up to `.git` or home directory
  3. Subdirectories: Scans below current directory (up to 200 dirs by default)
- **Custom filenames**: Can configure via `context.fileName` (single string or array)
- **Import system**: Supports `@path/to/file.md` syntax for modular context files
- **UI indication**: Footer shows count of loaded context files

#### CLI Flags (Relevant)

```bash
gemini --prompt "query"  # Automatically loads GEMINI.md files
gemini --model gemini-3-pro-preview
```

#### Documentation Insights

- **Memory commands**: `/memory refresh` and `/memory show` for managing context
- **Concatenation**: All found files concatenated with separators indicating origin
- **Import format**: Can specify `context.importFormat` for different import processors
- **File filtering**: Respects `.gitignore` and `.geminiignore` by default

---

### 6. OpenCode CLI

**Documentation**: https://opencode.ai/docs/agents/

#### System Prompt Mechanisms

| Mechanism | Flag/Config | Format | Modes | Notes |
|-----------|-------------|--------|-------|-------|
| **Agent config** | `prompt: "{file:./path.md}"` | Markdown | Per-agent | Agent-specific prompts |
| **JSON config** | `opencode.json` | JSON | Auto-loaded | Agent definitions |
| **Markdown agents** | `.opencode/agent/` or `~/.config/opencode/agent/` | Markdown + YAML | Auto-loaded | File-based agent definitions |

#### Key Findings

- **Agent-centric**: OpenCode uses agent configuration rather than global system prompts
- **Two locations**: Global (`~/.config/opencode/agent/`) and project (`.opencode/agent/`)
- **File reference syntax**: `{file:./prompts/build.txt}` for referencing prompt files
- **YAML frontmatter**: Markdown agents use YAML frontmatter for configuration
- **Agent modes**: `primary` (main agent) or `subagent` (invoked via @mention)

#### CLI Flags (Relevant)

```bash
opencode run --agent build "query"
opencode agent create  # Interactive agent creation
```

#### Documentation Insights

- **Built-in agents**: `build` (default), `plan` (restricted), `general` (subagent), `explore` (subagent)
- **Agent options**: `description`, `mode`, `model`, `temperature`, `prompt`, `tools`, `permissions`
- **Prompt path**: Relative to config file location (works for both global and project configs)
- **No global AGENTS.md**: OpenCode doesn't use a single global instruction file

---

## Comparison Table

| CLI | Primary Mechanism | File Format | Hierarchical? | Override Support | Config File | Flag Support |
|-----|------------------|-------------|---------------|-----------------|-------------|--------------|
| **Claude Code** | `CLAUDE.md` + flags | Markdown | No | `--system-prompt-file` | `settings.json` | ✅ Yes |
| **Factory** | `agents.md` | Markdown | No | Custom droids | N/A | ❌ No |
| **Codex** | `AGENTS.md` | Markdown | ✅ Yes | `AGENTS.override.md` | `config.toml` | ❌ No |
| **Cursor** | `AGENTS.md` / `CLAUDE.md` | Markdown | No | N/A | `cli-config.json` | ❌ No |
| **Gemini** | `GEMINI.md` | Markdown | ✅ Yes | Configurable filename | `settings.json` | ❌ No |
| **OpenCode** | Agent config | Markdown + YAML | No | Per-agent prompts | `opencode.json` | ❌ No |

---

## Harmonization Opportunities

### 1. AGENTS.md as Universal Standard

**Finding**: 5 out of 6 CLIs support markdown-based agent instruction files:
- Codex: `AGENTS.md` (official standard)
- Cursor: `AGENTS.md` / `CLAUDE.md`
- Factory: `agents.md` (lowercase)
- Gemini: `GEMINI.md` (configurable to `AGENTS.md`)
- Claude Code: `CLAUDE.md` (but can use via `--system-prompt-file`)

**Recommendation**: 
- Use `AGENTS.md` as the primary format
- For Claude Code, generate `AGENTS.md` and reference via `--system-prompt-file`
- For Gemini, configure `context.fileName: ["AGENTS.md", "GEMINI.md"]` for compatibility
- For Factory, use `agents.md` (lowercase) or check if uppercase is supported

### 2. Hierarchical Discovery Pattern

**Finding**: Codex and Gemini support hierarchical file discovery (global → project → subdirectory)

**Recommendation**:
- Implement hierarchical discovery in our template generation
- Create structure:
  ```
  ~/.cto/AGENTS.md                    # Global defaults
  <project-root>/AGENTS.md            # Project-specific
  <project-root>/services/<service>/AGENTS.md  # Service-specific
  ```

### 3. Progressive Disclosure Pattern

**Finding**: HumanLayer research recommends progressive disclosure (pointers to separate files)

**Recommendation**:
- Keep `AGENTS.md` files concise (< 300 lines, ideally < 60)
- Use `@path/to/file.md` syntax (Gemini) or file references
- Create modular instruction files:
  ```
  AGENTS.md                    # Main file with pointers
  agent-docs/
    building.md                # Build instructions
    testing.md                 # Test patterns
    code-conventions.md        # Style guide
  ```

### 4. Agent Identity vs System Prompt Separation

**Finding**: Current codebase mixes agent identity (`identity.md.hbs`) with system prompts (`*-system-prompt.md.hbs`)

**Recommendation** (aligned with Option D consensus):
- **Agent identity**: `templates/agents/{agent}.md.hbs` - Who the agent is, specialization, core rules
- **System prompt**: Generated from agent identity + workflow context + CLI-specific framing
- **CLI adapters**: `templates/clis/{cli}/invoke.sh.hbs` - Handles CLI-specific system prompt injection

---

## Best Practices from Research

### 1. Instruction Count Management

From HumanLayer research and "AI Agents That Matter" paper:
- **Limit**: ~150-200 instructions for frontier thinking models
- **Claude Code baseline**: ~50 instructions in default system prompt
- **Recommendation**: Keep agent instructions < 100 individual instructions
- **Progressive decay**: More instructions = uniform degradation (not just ignoring later ones)

### 2. File Length Guidelines

- **HumanLayer**: < 300 lines, ideally < 60 lines
- **Codex**: 32 KiB default limit (configurable)
- **Recommendation**: Target < 200 lines per AGENTS.md file

### 3. Content Organization

**WHAT** (Project structure, tech stack):
- Tech stack and versions
- Project structure (especially monorepos)
- Key directories and their purposes

**WHY** (Project purpose):
- What the project does
- Purpose of different components
- Business context

**HOW** (Working with the project):
- Build commands
- Test commands
- Development workflow
- Quality gates

### 4. What NOT to Include

- **Code style guidelines**: Use linters/formatters instead (LLMs are expensive linters)
- **Task-specific instructions**: Use progressive disclosure
- **Outdated code snippets**: Use file references instead
- **Every possible command**: Focus on universally applicable instructions

---

## Recommendations for CTO Platform

### 1. Template Structure (Option D Alignment)

```
templates/
├── agents/                    # Agent identities (single files)
│   ├── rex.md.hbs            # Agent identity + core specialization
│   ├── blaze.md.hbs
│   └── ...
│
├── clis/                      # CLI configs + invoke partials
│   ├── claude/
│   │   └── invoke.sh.hbs     # Uses --system-prompt-file with generated AGENTS.md
│   ├── codex/
│   │   └── invoke.sh.hbs     # Ensures AGENTS.md exists in workspace
│   ├── factory/
│   │   └── invoke.sh.hbs     # Ensures agents.md exists
│   ├── cursor/
│   │   └── invoke.sh.hbs     # Ensures AGENTS.md exists
│   ├── gemini/
│   │   └── invoke.sh.hbs     # Ensures GEMINI.md or AGENTS.md exists
│   └── opencode/
│       └── invoke.sh.hbs     # Generates agent config with prompt reference
│
└── code/                      # Play workflow containers
    └── container.sh.hbs      # Generates AGENTS.md, then invokes CLI partial
```

### 2. AGENTS.md Generation Strategy

**Template**: `templates/shared/agents-md.hbs`

```markdown
# {{agent_name}} Agent Guidelines

{{> agents/{{agent_name}}}}

## Execution Context
- **GitHub App**: {{github_app}}
- **Task ID**: {{task_id}}
- **Service**: {{service}}
- **Repository**: {{repository_url}}

## Universal Rules
{{> shared/universal-rules}}

{{#if cli_specific_instructions}}
## CLI-Specific Instructions
{{{cli_specific_instructions}}}
{{/if}}
```

### 3. CLI-Specific Adapters

Each CLI invoke partial should:

1. **Claude Code**: Generate `AGENTS.md`, then use `--system-prompt-file AGENTS.md`
2. **Codex**: Ensure `AGENTS.md` exists in workspace root (auto-discovered)
3. **Factory**: Ensure `agents.md` exists (lowercase)
4. **Cursor**: Ensure `AGENTS.md` exists (auto-discovered)
5. **Gemini**: Generate `GEMINI.md` or configure `context.fileName: ["AGENTS.md"]`
6. **OpenCode**: Generate agent config JSON with `prompt: "{file:./AGENTS.md}"`

### 4. Memory Optimization

Based on research findings:

- **Keep agent identity files concise**: Focus on WHO the agent is, not HOW to do everything
- **Use progressive disclosure**: Reference separate files for detailed instructions
- **Leverage hierarchical discovery**: Where supported (Codex, Gemini), use directory structure
- **Count instructions**: Aim for < 100 individual instructions per agent
- **Avoid duplication**: Don't repeat instructions across multiple files

---

## Implementation Plan

### Phase 1: Harmonize File Format
1. Standardize on `AGENTS.md` as primary format
2. Update all CLI invoke partials to generate/ensure `AGENTS.md` exists
3. For CLIs requiring different names, create symlinks or copies:
   - Factory: `agents.md` → `AGENTS.md` (or vice versa)
   - Gemini: Configure `context.fileName: ["AGENTS.md"]` or generate both

### Phase 2: Implement Hierarchical Discovery
1. Generate `AGENTS.md` at multiple levels:
   - Global defaults (if needed)
   - Project root
   - Service-specific (where applicable)
2. Test with Codex and Gemini to verify hierarchical loading

### Phase 3: Optimize Content
1. Audit current agent identity files for instruction count
2. Split large files using progressive disclosure
3. Remove code style guidelines (delegate to linters)
4. Focus on universally applicable instructions

### Phase 4: CLI-Specific Enhancements
1. **Claude Code**: Use `--append-system-prompt` for workflow-specific additions
2. **OpenCode**: Create agent configs that reference `AGENTS.md`
3. **Gemini**: Leverage import syntax (`@path/to/file.md`) for modularity

---

## Research Sources

1. **Claude Code Documentation**: https://code.claude.com/docs/en/cli-reference
2. **Factory Documentation**: https://docs.factory.ai/reference/cli-reference
3. **Codex Documentation**: https://developers.openai.com/codex/guides/agents-md
4. **Gemini CLI Documentation**: https://geminicli.com/docs/get-started/configuration/
5. **OpenCode Documentation**: https://opencode.ai/docs/agents/
6. **HumanLayer Blog**: https://www.humanlayer.dev/blog/writing-a-good-claude-md
7. **AGENTS.md Standard**: https://github.com/openai/agents.md
8. **AI Agents That Matter Paper**: https://arxiv.org/abs/2407.01502

---

## Conclusion

The CLI ecosystem is converging on markdown-based agent instruction files, with `AGENTS.md` emerging as the de facto standard. By adopting `AGENTS.md` as our universal format and creating CLI-specific adapters, we can:

1. **Harmonize** agent identity across all 6 CLIs
2. **Leverage** hierarchical discovery where supported
3. **Optimize** instruction count and file length
4. **Maintain** CLI-specific optimizations where beneficial

The Option D template structure aligns well with this approach, allowing us to maintain agent identities as single files while generating CLI-appropriate system prompts at invocation time.

