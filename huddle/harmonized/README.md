# System Prompt Research: Harmonized Findings

**Date**: December 5, 2025  
**Status**: Research Complete, Consensus Achieved

---

## Overview

This folder contains the harmonized findings from multi-model research on system prompts and agent identity across 6 supported CLIs.

## Contributors

| Model | Role | Key Contribution |
|-------|------|------------------|
| **Claude Opus 4** | Primary Researcher | Comprehensive CLI comparison, container structure |
| **Composer (Sonnet 4.5)** | Researcher | Option D alignment, implementation plan |
| **Gemini 3 Pro Preview** | Researcher | Firmware vs context distinction, Gemini config |
| **Grok Code Fast 1** | Researcher | Risk analysis, migration strategy |
| **Sonnet 4.5** | Executive Summary | Brevity guidelines, science backing |

## Documents

### Core Deliverables

| Document | Purpose |
|----------|---------|
| [AGENTS-MD-HARMONIZATION.md](./AGENTS-MD-HARMONIZATION.md) | Cross-CLI compatibility strategy |
| [SYSTEM-PROMPT-BEST-PRACTICES.md](./SYSTEM-PROMPT-BEST-PRACTICES.md) | Prompt optimization guidelines |

### Supporting Research

Located in parent `huddle/` directory:

- `claude-opus-4-system-prompt-research.md` - Detailed CLI analysis
- `composer-system-prompt-research.md` - Implementation focus
- `gemini-3-pro-preview-system-prompt-research.md` - Google ecosystem insights
- `grok-code-fast-1-system-prompt-research.md` - Risk & compatibility analysis
- `sonnet-4.5-executive-summary.md` - TL;DR for stakeholders
- `sonnet-4.5-system-prompt-research-findings.md` - Complete findings

### X Posts Archive

- `../xposts/agent-system-prompts-research.md` - Curated X posts on agent best practices

---

## Key Consensus Points

### 1. AGENTS.md is the Standard

All 6 CLIs support AGENTS.md (native or configurable):

```
✅ Codex      - Native
✅ OpenCode   - Native  
✅ Cursor     - Supported
✅ Claude     - Via symlink/import
✅ Gemini     - Via config
✅ Aider      - Via config
```

### 2. Keep It Short

| Metric | Target |
|--------|--------|
| Lines | < 150 |
| Instructions | < 100 |
| Ideal | < 60 |

### 3. Progressive Disclosure

```markdown
# Instead of dumping everything:
See `docs/agents/building.md` for build instructions.
See `docs/agents/testing.md` for test patterns.
```

### 4. Separate Concerns

| Layer | Contains |
|-------|----------|
| System Prompt | WHO (agent identity) |
| AGENTS.md | WHAT (project context) |
| User Message | WHAT TO DO (task) |

---

## Implementation Priority

### Immediate (This Week)
1. Create `templates/shared/agents-md.hbs`
2. Update container setup for symlinks
3. Test with one CLI (Claude Code)

### Short-term (Next 2 Weeks)
1. Add CLI config files
2. Test all 6 CLIs
3. Update documentation

### Long-term (Month 1+)
1. Gather metrics on agent performance
2. Iterate on content
3. Refine based on feedback

---

## Quick Links

- **AGENTS.md Spec**: https://agents.md/
- **HumanLayer Guide**: https://www.humanlayer.dev/blog/writing-a-good-claude-md
- **AI Agents That Matter**: https://arxiv.org/abs/2407.01502

---

## Next Steps

1. Review harmonized documents with team
2. Finalize template structure
3. Begin implementation
4. Test across CLIs
5. Iterate based on results

