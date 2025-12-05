# System Prompt Research: CLI Comparison & Harmonization

**Researcher**: gemini-3-pro-preview  
**Date**: December 5, 2025  
**Context**: Investigating system prompt mechanisms across 6 CLIs to recommend a unified agent identity structure.

---

## Executive Summary

Most CLIs support file-based system prompts or "context" files, but they differ in whether they **replace** the core system prompt or **append** to it (context). 

*   **Harmonization Candidate**: `AGENTS.md` is the strongest candidate for a unified standard, natively supported by Codex and easily configurable in Gemini. Claude Code uses `CLAUDE.md` similarly.
*   **Recommendation**: Adopt a **Folder-based Agent Identity** structure (`templates/agents/<agent>/`) that generates a standard `AGENTS.md` (or `CLAUDE.md` symlink) for "context" and specific system prompt files for CLIs that allow full replacement (Gemini, Claude Code via flag).

---

## CLI Comparison Table

| CLI | System Prompt Mechanism | Context/Memory File | key Flags / Config | Behavior |
| :--- | :--- | :--- | :--- | :--- |
| **Claude Code** | `--system-prompt` (replace)<br>`--append-system-prompt` (append)<br>`--system-prompt-file` (replace, print-only) | `CLAUDE.md` | `--system-prompt-file <file>` | `CLAUDE.md` is injected as a user message/system reminder. Flags allow full replacement or appending. |
| **Factory (Droid)** | `droid exec -f <file>` | `~/.factory/prompts/` | `--file`, `-f` | `droid exec -f` loads a prompt file. Custom droids can be defined in config with specific prompts. |
| **OpenAI Codex** | N/A (Built-in logic) | `AGENTS.md`<br>`AGENTS.override.md` | `project_doc_fallback_filenames` | Native `AGENTS.md` support. Hierarchical loading (Global -> Project -> Dir). |
| **Gemini CLI** | `GEMINI_SYSTEM_MD` env var | `GEMINI.md` (default)<br>Configurable via `contextFileName` | `GEMINI_SYSTEM_MD=/path/to/file` | `GEMINI.md` is "context" (appended). `GEMINI_SYSTEM_MD` *replaces* the entire core system firmware. |
| **OpenCode** | `opencode.json` or Markdown files | `.opencode/agent/*.md` | `agent.<name>.prompt` | Agents defined in JSON or MD. Markdown agents use frontmatter for config and body for prompt. |
| **Cursor CLI** | Interactive/Implicit | `.cursorrules` (implied) | `mcp.json` | Less explicit CLI flags for system prompts. Relies on IDE-style context and rules. |

---

## Detailed Findings

### 1. Claude Code
*   **Mechanism**: Offers precise control. `--system-prompt` wipes the slate clean (removing default tools definitions if not careful), while `--append-system-prompt` adds to the default.
*   **Context**: `CLAUDE.md` is the standard "onboarding" file, effectively a persistent context/memory.
*   **Agent Identity**: Best handled via `--append-system-prompt` for identity/personality, or `CLAUDE.md` for project knowledge.
*   **Caveat**: `--system-prompt-file` is print-mode only. For interactive, use `--system-prompt` string or `--append`.

### 2. Factory (Droid)
*   **Mechanism**: `droid exec` is the primary automation interface. It accepts a prompt file via `-f`.
*   **Agent Identity**: "Custom Droids" are the abstraction here. You define a droid in config with a `prompt` field.
*   **Harmonization**: You can generate a markdown prompt file and feed it to `droid exec -f`.

### 3. OpenAI Codex
*   **Mechanism**: Deeply integrated `AGENTS.md` support. It concatenates these files from root down to CWD.
*   **Harmonization**: It *is* the standard for `AGENTS.md`.
*   **Agent Identity**: Can be placed in `AGENTS.md` or an override file.

### 4. Gemini CLI
*   **Mechanism**: Distinguishes between **Firmware** (`SYSTEM.md` / `GEMINI_SYSTEM_MD`) and **Context** (`GEMINI.md`).
*   **Harmonization**: 
    *   Can configure `contextFileName: "AGENTS.md"` in `settings.json` to align with Codex.
    *   `GEMINI_SYSTEM_MD` env var allows complete personality/agent identity replacement.
*   **Quote**: "The `GEMINI_SYSTEM_MD` environment variable is the key to achieving advanced customization... it completely replaces the default system prompt."

### 5. OpenCode
*   **Mechanism**: Very flexible. Supports "Markdown Agents" where the file body is the system prompt.
*   **Harmonization**: `templates/agents/rex.md` could be directly usable if it follows OpenCode's frontmatter + body format.

### 6. Cursor CLI
*   **Mechanism**: Appears to follow the IDE's conventions (`.cursorrules`).
*   **Harmonization**: `AGENTS.md` content might need to be duplicated or symlinked to `.cursorrules` for full effect, though Cursor is adding support for more standard files.

---

## Harmonization Strategy: "The Hybrid Standard"

We should standardize on **`AGENTS.md`** as the primary project context file, but separate **Agent Identity** into dedicated definition files.

### Recommended Structure

```text
.
├── templates/
│   └── agents/
│       └── rex/
│           ├── identity.md.hbs       # Core personality & role (CLI-agnostic)
│           ├── system.md.hbs         # Full system prompt wrapper (includes identity)
│           └── opencode.md.hbs       # OpenCode specific frontmatter variant
```

### CLI Configuration Plan

1.  **Codex**: Use `AGENTS.md` for project rules. Identity via `AGENTS.override.md` or custom config profile.
2.  **Gemini**: 
    *   Set `contextFileName: "AGENTS.md"` in settings.
    *   Use `GEMINI_SYSTEM_MD` pointing to a generated `system.md` for the Agent Identity.
3.  **Claude**: 
    *   Symlink `AGENTS.md` -> `CLAUDE.md` (or inclusion).
    *   Use `--append-system-prompt` with the content of `identity.md`.
4.  **OpenCode**:
    *   Generate `.opencode/agent/rex.md` using `identity.md` content.
5.  **Droid**:
    *   Generate `prompts/rex.md` for use with `droid exec -f`.

### Unified `AGENTS.md` Content
This file should contain **Project Context** (What/Why/How), not **Agent Identity** (Who).
*   Project Architecture
*   Coding Standards
*   Test Commands
*   *Not* "You are Rex..."

### Agent Identity Files
These should contain the **Who**.
*   "You are Rex, a Rust expert..."
*   "Your goal is to..."

### Best Practices for AGENTS.md (Memory Optimization)
Based on research from HumanLayer and OpenAI:
1.  **Less is More**: Limit to ~200 instructions. Agents perform worse as instruction count grows.
2.  **Universal Applicability**: Only include context relevant to *every* task.
3.  **Progressive Disclosure**: Don't dump everything. Point to other docs (`agent_docs/building.md`, `agent_docs/testing.md`) that the agent can read on demand.
4.  **No Linting Rules**: Use linter tools instead of describing style guides in text.
5.  **Structure**: 
    *   **WHAT**: Project structure, tech stack.
    *   **WHY**: Core purpose and architecture.
    *   **HOW**: Build/Test/Verify commands.

---

## Next Steps for Implementation

1.  **Update `settings.json`** for Gemini CLI to use `AGENTS.md`.
2.  **Refactor Templates**: Split current templates into `identity` (personality) and `context` (project rules).
3.  **Generation Script**: Update `task-master` or setup scripts to generate the CLI-specific wrappers from the single `identity.md` source.
