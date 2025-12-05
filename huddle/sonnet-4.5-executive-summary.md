# System Prompt Research: Executive Summary

**Model**: Claude Sonnet 4.5  
**Date**: December 5, 2025

---

## TL;DR

**AGENTS.md is the winner.** All 6 CLIs support it. Keep it under 150 lines. Use progressive disclosure for detailed docs. CLI-specific configs only when absolutely necessary.

---

## The Landscape

We analyzed 6 major coding CLIs:
1. **Claude Code** - Most flexible (3 system prompt flags)
2. **Factory/Droid** - Strongest AGENTS.md advocate  
3. **OpenAI Codex** - Most explicit (override pattern)
4. **Cursor** - Most structured (.mdc with metadata)
5. **Gemini CLI** - Google's ReAct agent
6. **OpenCode** - Most sophisticated (agent system with permissions)

**Finding**: ALL support AGENTS.md (some natively, others via configuration).

---

## Key Insights

### 1. AGENTS.md is the Cross-CLI Standard

- **20,000+ projects** already using it
- Initiated by OpenAI for Codex
- Adopted by Claude Code, Factory, Cursor, Gemini, OpenCode, Aider, Zed, and 20+ others
- Plain Markdown (mostly), universal compatibility

### 2. Brevity is Critical (Science-Backed)

**Research shows**: LLMs can follow ~150-200 instructions reliably.

**Implications**:
- Claude Code's system prompt already has ~50 instructions
- More instructions = ALL instructions followed worse (uniform degradation)
- <150 lines is a **hard limit** based on LLM capabilities
- <60 lines is ideal (HumanLayer recommendation)

### 3. Progressive Disclosure is the Pattern

**Anti-pattern**: Stuff everything into AGENTS.md

**Best practice**: Tell agents WHERE to find information

```markdown
# MyProject

## Detailed Guides
- Building: `docs/agents/building.md`
- Testing: `docs/agents/testing.md`
- Architecture: `docs/agents/architecture.md`

Read the relevant guide before starting work.
```

### 4. Three-Tier Prompt Strategy

1. **System Prompt** - Agent personality (CLI-provided or via flag)
2. **AGENTS.md** - Project conventions (auto-discovered)
3. **User Message** - Specific task (user-provided)

Each layer serves a distinct purpose.

### 5. Hierarchical Discovery for Monorepos

```
project/
  AGENTS.md              # High-level structure
  packages/
    api/
      AGENTS.md          # API-specific
    web/
      AGENTS.md          # Frontend-specific
```

**Rule**: Closest file to edited code wins.

---

## Comparison at a Glance

| CLI | AGENTS.md | System Flags | Agent Config | Special Features |
|-----|-----------|--------------|--------------|------------------|
| **Claude Code** | ✅ CLAUDE.md | ✅ 3 flags | JSON flag | Auto-ignores if irrelevant |
| **Factory** | ✅ Primary | ❌ | AGENTS.md | Override pattern |
| **Codex** | ✅ Primary | ❌ | AGENTS.md | Instruction chain, override |
| **Cursor** | ✅ Yes | ❌ | .mdc or AGENTS.md | Glob patterns, team rules |
| **Gemini** | ✅ Config | ❌ | settings.json | ReAct loop, MCP |
| **OpenCode** | ✅ Rules | Config | JSON + Markdown | Temperature, permissions |

---

## Recommendations for CTO Platform

### Immediate Actions

1. **Standardize on AGENTS.md** for all agent templates
   - Universal compatibility
   - Industry standard
   - Minimal vendor lock-in

2. **Enforce <150 line limit**
   - Based on LLM instruction-following research
   - Better performance
   - Easier maintenance

3. **Adopt progressive disclosure**
   - Main AGENTS.md references detailed docs
   - Agents load context on-demand
   - Reduces context bloat

### Template Structure (Recommended)

```
templates/
  agents/
    base/
      AGENTS.md.hbs              # Universal (works for all CLIs)
    specialized/
      rex-additions.md.hbs       # Agent-specific extras
      blaze-additions.md.hbs
  clis/
    claude/config.json.hbs       # Only for advanced features
    cursor/rules.mdc.hbs         # Only if glob patterns needed
    opencode/agents.json.hbs     # Only for permissions/temp
  shared/
    instructions/                # Progressive disclosure docs
      building.md.hbs
      testing.md.hbs
```

### Container Image Layout

```
/workspace/
  AGENTS.md                      # Auto-discovered by all CLIs
  docs/agents/                   # Progressive disclosure
  .agents/                       # CLI configs (optional)
```

### Migration Path

**Phase 1**: Consolidate to AGENTS.md
- Move instructions to project root AGENTS.md
- Standard markdown, no metadata
- Keep <150 lines
- Nested AGENTS.md for packages

**Phase 2**: CLI wrappers (only if needed)
- Claude: `--agents` for subagents
- Cursor: .mdc for glob patterns
- OpenCode: agent configs for advanced features

**Phase 3**: Container optimization
- Standard layout (/workspace/AGENTS.md)
- CLI detection in entrypoint
- Progressive disclosure docs

---

## The Science

### From "AI Agents That Matter" (arXiv:2407.01502)

- Simpler agents often perform as well as complex ones
- Cost matters (joint optimization of accuracy + cost)
- Standardization improves reproducibility

### From Instruction Following Research

- Frontier models: ~150-200 instructions max
- Smaller models: Exponential decay beyond that
- Uniform degradation: More instructions = worse at ALL of them
- Position bias: Beginning and end matter most

**Takeaway**: Every line in AGENTS.md competes for limited instruction-following capacity.

---

## What to Include in AGENTS.md

### ✅ DO Include

- Build and test commands (exact, copy-pasteable)
- Architecture overview (brief, 1 paragraph)
- Project structure (folders, what goes where)
- Conventions not obvious from code
- Security considerations (keys, auth)
- Git workflow (branching, PRs)

### ❌ DON'T Include

- Code style guidelines (use linters)
- Task-specific instructions (only sometimes relevant)
- Duplicate content from README
- Auto-generated content
- Long code snippets (reference files instead)

---

## Quick Wins

1. **Today**: Create base AGENTS.md template (<150 lines)
2. **This week**: Extract common instructions from existing templates
3. **Next week**: Add progressive disclosure docs
4. **Month 1**: Test across all 6 CLIs
5. **Month 2**: Gather metrics, iterate

---

## Bottom Line

**AGENTS.md is the standard. Keep it short. Reference detailed docs. CLI-specific stuff only when necessary.**

Simple as that.

---

## Full Report

See `sonnet-4.5-system-prompt-research-findings.md` for complete analysis with:
- Detailed CLI-by-CLI breakdown
- Complete comparison matrix
- Research paper summaries
- Migration strategies
- Container layout recommendations
- Template structure options

---

**Next Steps**: Review with team → finalize template structure → implement → iterate based on metrics

