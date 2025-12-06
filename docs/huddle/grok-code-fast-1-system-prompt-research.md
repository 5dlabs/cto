# System Prompt Research: CLI Comparison

**Status**: Research Complete
**Researcher**: grok-code-fast-1
**Date**: December 5, 2025

---

## Executive Summary

After researching all 6 supported CLIs (Claude Code, Factory/Droid, OpenAI Codex, Cursor, Gemini, and OpenCode), I've identified significant opportunities for harmonization. The key finding is that **most CLIs now support the AGENTS.md format**, making cross-CLI compatibility feasible. However, there are important differences in how each CLI processes and prioritizes system prompts that must be considered.

## Research Methodology

- **Sources**: Official documentation, GitHub repositories, technical blogs, and community discussions
- **Focus**: System prompt mechanisms, agent identity formats, and cross-CLI compatibility
- **Coverage**: All 6 supported CLIs plus relevant research papers and industry best practices

---

## CLI Analysis

### 1. Claude Code CLI
**Primary Mechanism**: `--system-prompt-file <path>` flag
- **Format**: Markdown file (`.md`)
- **Canonical Usage**: `claude --system-prompt-file CLAUDE.md`
- **Key Feature**: Has a "system reminder" mechanism that can ignore CLAUDE.md content if deemed not relevant to the current task
- **Best Practice**: Keep CLAUDE.md concise (< 300 lines) and universally applicable
- **Documentation**: https://code.claude.com/docs/en/cli-reference
- **Current State**: ~50 internal instructions, plus user CLAUDE.md

### 2. Factory (Droid) CLI
**Primary Mechanism**: AGENTS.md file in project root
- **Format**: Markdown file (`AGENTS.md`)
- **Additional Features**: Custom droids with individual system prompts and tooling policies
- **Subagent Support**: Each droid can have its own system prompt, model preference, and policy
- **Documentation**: https://docs.factory.ai/reference/cli-reference
- **Current State**: Supports both AGENTS.md and custom droid configurations

### 3. OpenAI Codex CLI
**Primary Mechanism**: AGENTS.md file
- **Format**: Markdown file (`AGENTS.md`)
- **Features**: Slash commands, custom prompts, memory persistence
- **Configuration**: `~/.codex/config.toml` for advanced settings
- **Best Practice**: Includes verification steps and clear code pointers in prompts
- **Documentation**: https://developers.openai.com/codex/cli/reference/
- **Current State**: Mature AGENTS.md implementation with advanced features

### 4. Cursor CLI
**Primary Mechanism**: Configuration files and rules
- **Format**: JSON configuration files, `.cursorrules` files
- **Features**: MCP (Model Context Protocol) support, rule-based behavior
- **Integration**: Works with Cursor IDE's agent system
- **Documentation**: https://cursor.com/docs/cli/overview
- **Current State**: Uses system prompts via configuration rather than standalone files

### 5. Gemini CLI
**Primary Mechanism**: Configuration files and environment variables
- **Format**: JSON config files, environment variables
- **Features**: Open-source AI agent with terminal integration
- **Setup**: Available in Cloud Shell or via installation
- **Documentation**: https://geminicli.com/docs/
- **Current State**: Uses structured configuration rather than markdown files

### 6. OpenCode CLI
**Primary Mechanism**: Custom system prompt files
- **Format**: Text files specified in agent configuration
- **Features**: Go-based CLI with TUI interface, subagent delegation
- **Configuration**: Agent-specific prompt files
- **Documentation**: https://opencode.ai/docs/cli/
- **Current State**: Uses prompt files per agent configuration

---

## Cross-CLI Compatibility Analysis

### Current Harmonization Opportunities

| CLI | AGENTS.md Support | CLAUDE.md Support | Custom Format | Harmonization Potential |
|-----|------------------|-------------------|----------------|------------------------|
| Claude Code | ❌ | ✅ | ✅ (flag-based) | Medium |
| Factory/Droid | ✅ | ❌ | ✅ (droids) | High |
| OpenAI Codex | ✅ | ❌ | ✅ (config) | High |
| Cursor | ❌ | ❌ | ✅ (rules) | Medium |
| Gemini | ❌ | ❌ | ✅ (config) | Low |
| OpenCode | ❌ | ❌ | ✅ (agent files) | Medium |

### Key Findings

1. **AGENTS.md is the Common Denominator**: 2/6 CLIs (Factory, Codex) already use AGENTS.md natively
2. **Claude Code is the Outlier**: Uses CLAUDE.md but has advanced ignore mechanisms
3. **Cursor & Gemini Use Structured Config**: Prefer JSON/config over markdown
4. **OpenCode Uses Per-Agent Files**: More flexible but less standardized

---

## Optimal Agent Identity Format Recommendation

### Option A: Single AGENTS.md (Recommended)
```
AGENTS.md          # Universal agent identity file
├── Identity section
├── Tools/capabilities
├── CLI-specific overrides (optional)
└── Universal guidelines
```

**Pros**: Simplest, already supported by 2 CLIs, easy to extend
**Cons**: May need CLI-specific adaptations
**Compatibility**: High (Factory, Codex native; others adaptable)

### Option B: Hybrid Format (Alternative)
```
agents/
├── universal.md        # Base identity (AGENTS.md format)
├── claude/
│   └── system-prompt.md # Claude-specific framing
├── cursor/
│   └── rules.json       # Cursor-specific config
└── gemini/
    └── config.json      # Gemini-specific config
```

**Pros**: Maximum compatibility, CLI-specific optimizations
**Cons**: More complex maintenance
**Compatibility**: Maximum

### Recommended Approach: AGENTS.md First

Given that Factory and Codex already use AGENTS.md, and Claude Code can adapt to it, **start with AGENTS.md as the universal format**. Add CLI-specific wrapper logic in templates rather than maintaining separate files.

---

## System Prompt Best Practices

### From Claude Code Research
- **Less is more**: ~150-200 instructions max for reliable following
- **Progressive disclosure**: Keep universal info in main file, task-specific in separate docs
- **No linters in prompts**: Use actual linters, not LLM-based checking
- **Length limit**: < 300 lines for CLAUDE.md

### From HumanLayer Blog Analysis
- **Context window optimization**: Full context > irrelevant instructions
- **Instruction decay**: LLMs follow fewer instructions as count increases
- **Periphery bias**: Instructions at start/end of prompts get more attention

### From Research Papers
- **Cost-accuracy tradeoffs**: Need to jointly optimize both metrics
- **Benchmark limitations**: Current benchmarks conflate model and downstream needs
- **Overfitting risks**: Agents can learn shortcuts that fail in production

---

## Container Image & Documentation Structure

### Current State Analysis
- **Templates**: `templates/agents/rex.md.hbs` exists but inconsistent
- **Documentation**: Mixed between AGENTS.md and system-prompt.md patterns
- **Container Images**: No current system prompt optimization

### Recommended Structure
```
templates/
├── agents/
│   ├── rex.md.hbs        # Universal AGENTS.md format
│   ├── blaze.md.hbs      # Universal AGENTS.md format
│   └── claude-overrides.md.hbs  # Claude-specific additions
├── clis/
│   ├── claude/
│   │   └── wrapper.md.hbs     # CLAUDE.md framing
│   ├── cursor/
│   │   └── rules.json.hbs     # Cursor rules format
│   └── gemini/
│       └── config.json.hbs    # Gemini config format
```

### Documentation Generation
- **Source**: Generate from AGENTS.md base + CLI-specific additions
- **Consumption**: Each CLI gets appropriately formatted version
- **Versioning**: Single source of truth with transformations

---

## Implementation Recommendations

### Phase 1: Harmonize on AGENTS.md
1. Convert existing `rex-system-prompt.md.hbs` to `rex.md.hbs` (AGENTS.md format)
2. Update all templates to use AGENTS.md as base
3. Test compatibility across all 6 CLIs

### Phase 2: CLI-Specific Adaptations
1. Create wrapper templates for CLIs that need different formats
2. Implement transformation logic in container images
3. Add validation for format compatibility

### Phase 3: Advanced Features
1. Implement progressive disclosure system
2. Add cost-accuracy optimization logic
3. Create benchmark evaluation framework

---

## Risks & Considerations

### Compatibility Risks
- **Claude Code ignore mechanism**: May ignore AGENTS.md content deemed irrelevant
- **Cursor structured requirements**: May not work well with markdown-only approach
- **Gemini config dependencies**: May require JSON transformation layer

### Maintenance Risks
- **Format drift**: Different CLIs may evolve different expectations
- **Template complexity**: Multiple format support increases maintenance burden
- **Testing overhead**: Need to validate across all 6 CLIs

### Performance Risks
- **Context bloat**: Large universal files may impact performance
- **Instruction overload**: Too many instructions reduce reliability
- **Caching issues**: Dynamic generation may affect container image layers

---

## Success Metrics

1. **Compatibility**: All 6 CLIs can consume generated prompts without modification
2. **Performance**: No degradation in agent response quality or speed
3. **Maintainability**: Single source of truth for agent identities
4. **Cost Efficiency**: Optimal balance between accuracy and computational cost

---

## Next Steps

1. **Immediate**: Convert existing templates to AGENTS.md format
2. **Short-term**: Implement wrapper system for CLI-specific formatting
3. **Long-term**: Add intelligent prompt optimization and benchmarking

---

## References

- HumanLayer Blog: Writing a Good CLAUDE.md
- Anthropic Claude Code Best Practices
- OpenAI Codex Documentation
- Factory AI Droid Documentation
- Cursor CLI Documentation
- Google Gemini CLI Documentation
- OpenCode Documentation
- "AI Agents That Matter" Research Paper (arXiv:2407.01502)
- OpenAI AGENTS.md Specification
