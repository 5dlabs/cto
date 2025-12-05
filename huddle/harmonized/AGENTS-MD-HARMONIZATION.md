# AGENTS.md Harmonization Strategy

**Status**: Consensus Achieved  
**Date**: December 5, 2025  
**Contributors**: Claude Opus 4, Composer (Sonnet 4.5), Gemini 3 Pro Preview, Grok Code Fast 1, Sonnet 4.5

---

## Executive Summary

All researchers agree: **AGENTS.md is the universal standard** for agent instructions across all 6 supported CLIs. This document provides the harmonization strategy for implementing cross-CLI compatibility.

---

## The Standard: AGENTS.md

### Why AGENTS.md?

| Metric | Evidence |
|--------|----------|
| **Adoption** | 20,000+ open source projects |
| **CLI Support** | All 6 CLIs (native or configurable) |
| **Industry Backing** | OpenAI (Codex), Anthropic (Claude), Google (Gemini) |
| **Format** | Plain Markdown (universal) |
| **Discovery** | Hierarchical (monorepo-friendly) |

### CLI Compatibility Matrix

| CLI | Native File | AGENTS.md Support | Configuration Method |
|-----|-------------|-------------------|---------------------|
| **Claude Code** | `CLAUDE.md` | ✅ Via import or symlink | `@AGENTS.md` in CLAUDE.md |
| **OpenAI Codex** | `AGENTS.md` | ✅ Native | Auto-discovered |
| **Cursor** | `.cursor/rules/` | ✅ Supported | Place in project root |
| **Gemini CLI** | `GEMINI.md` | ✅ Configurable | `contextFileName: "AGENTS.md"` |
| **OpenCode** | `AGENTS.md` | ✅ Native | Auto-discovered or `rules` config |
| **Aider** | `CONVENTIONS.md` | ✅ Configurable | `read: AGENTS.md` in .aider.conf.yml |

---

## Implementation Strategy

### Option 1: Symlinks (Simplest)

```bash
# Create AGENTS.md as source of truth
# Symlink for CLI compatibility
ln -s AGENTS.md CLAUDE.md
ln -s AGENTS.md GEMINI.md
```

### Option 2: Import Syntax (Claude Code)

```markdown
# CLAUDE.md
@AGENTS.md
```

### Option 3: Configuration Files

**Gemini CLI** (`.gemini/settings.json`):
```json
{
  "context": {
    "fileName": ["AGENTS.md", "GEMINI.md"]
  }
}
```

**Aider** (`.aider.conf.yml`):
```yaml
read: AGENTS.md
```

**OpenCode** (`opencode.json`):
```json
{
  "instructions": ["AGENTS.md"]
}
```

---

## Container Image Structure

Based on consensus from all researchers:

```
/workspace/
├── AGENTS.md                  # Primary (single source of truth)
├── CLAUDE.md -> AGENTS.md     # Symlink for Claude Code
├── GEMINI.md -> AGENTS.md     # Symlink for Gemini CLI
│
├── .claude/
│   └── CLAUDE.md -> ../AGENTS.md
│
├── .gemini/
│   └── settings.json          # contextFileName: "AGENTS.md"
│
├── .cursor/
│   └── rules/
│       └── agent.mdc -> ../../AGENTS.md
│
├── .aider.conf.yml            # read: AGENTS.md
│
└── opencode.json              # instructions: ["AGENTS.md"]
```

---

## Template Integration (Option D)

Aligned with the existing Option D consensus:

```
templates/
├── agents/                    # Single-file agent identities
│   ├── rex.md.hbs            # Contains agent personality + rules
│   ├── blaze.md.hbs
│   └── ...
│
├── shared/
│   └── agents-md.hbs         # AGENTS.md generator template
│
├── clis/                      # CLI-specific invoke partials
│   ├── claude/
│   │   └── invoke.sh.hbs     # --append-system-prompt + CLAUDE.md
│   ├── codex/
│   │   └── invoke.sh.hbs     # AGENTS.md auto-discovered
│   ├── cursor/
│   │   └── invoke.sh.hbs     # .cursor/rules or AGENTS.md
│   ├── gemini/
│   │   └── invoke.sh.hbs     # GEMINI.md or settings.json config
│   ├── opencode/
│   │   └── invoke.sh.hbs     # AGENTS.md auto-discovered
│   └── aider/
│       └── invoke.sh.hbs     # --read AGENTS.md
│
└── code/
    └── container.sh.hbs      # Generates AGENTS.md + symlinks
```

---

## CLI Invocation Patterns

### Claude Code
```bash
# AGENTS.md content loaded via CLAUDE.md symlink or import
claude --append-system-prompt "Agent: ${AGENT_NAME}" \
       --print "${TASK_PROMPT}"
```

### OpenAI Codex
```bash
# AGENTS.md auto-discovered in workspace
codex --ask-for-approval never "${TASK_PROMPT}"
```

### Cursor
```bash
# AGENTS.md auto-discovered
cursor chat "${TASK_PROMPT}"
```

### Gemini CLI
```bash
# GEMINI.md symlinked to AGENTS.md, or configured via settings.json
gemini -p "${TASK_PROMPT}"
```

### OpenCode
```bash
# AGENTS.md auto-discovered
opencode run "${TASK_PROMPT}"
```

### Aider
```bash
# Explicit read flag
aider --read AGENTS.md --message "${TASK_PROMPT}"
```

---

## Migration Checklist

- [ ] Create base `AGENTS.md` template in `templates/shared/agents-md.hbs`
- [ ] Update container setup to generate symlinks
- [ ] Add CLI config files (`.gemini/settings.json`, `.aider.conf.yml`, etc.)
- [ ] Test with all 6 CLIs
- [ ] Update documentation

---

## References

- **AGENTS.md Spec**: https://agents.md/
- **OpenAI Codex Guide**: https://developers.openai.com/codex/guides/agents-md
- **Claude Code Memory**: https://code.claude.com/docs/en/memory
- **Gemini CLI Config**: https://geminicli.com/docs/get-started/configuration/
- **OpenCode Rules**: https://opencode.ai/docs/rules/
- **Aider Conventions**: https://aider.chat/docs/usage/conventions.html

