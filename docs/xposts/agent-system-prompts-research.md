# X Posts: Agent System Prompts & Best Practices

**Captured:** 2025-12-05  
**Purpose:** Research resource for CTO platform system prompt optimization

---

## Post 1: Deep Agents for Enterprise

**Author:** @omarsar0 (elvis)  
**URL:** https://x.com/omarsar0/status/1980629163976675779  
**Date:** ~2025

> People are sleeping on Deep Agents. Start using them now. This is a fun paper showcasing how to put together advanced deep agents for enterprise use cases. Uses the best techniques: task decomposition, planning, specialized subagents, MCP for NL2SQL, file analysis, and more.

### Key Takeaways
- Task decomposition is essential for complex agents
- Planning layers improve agent reliability
- Specialized subagents for different tasks (NL2SQL, file analysis)
- MCP (Model Context Protocol) for tool integration
- Enterprise use cases require structured approaches

---

## Post 2: Anthropic Multi-Agent Research System

**Author:** @rohanpaul_ai (Rohan Paul)  
**URL:** https://x.com/rohanpaul_ai/status/1933652486520242421  
**Date:** ~2024

> Anthropic just dropped the beautiful explanation of how they built a multi-agent research system using multiple Claude AI agents. A MUST read for anyone building multi-agent system. A lead agent plans research steps, spawns specialized subagents to search in parallel, and...

### Key Takeaways
- Lead agent → subagent architecture
- Research planning as first step
- Parallel execution of specialized tasks
- Spawning subagents for different purposes
- Anthropic's internal best practices for agent systems

---

## Post 3: Gemini 3 Pro System Instructions

**Author:** @googleaidevs (Google AI Developers)  
**URL:** https://x.com/googleaidevs/status/1996271402266017901  
**Date:** ~2025

> Check out these System Instructions for Gemini 3 Pro that improved performance on various agentic benchmarks by up to ~5%.

### Key Takeaways
- System instructions can improve benchmark performance by ~5%
- Google publishes official guidance for Gemini CLI
- Agentic benchmarks are the target for optimization
- Small system prompt changes can have measurable impact

---

## Post 4: Prompt Caching Optimization

**Author:** @dejavucoder (sankalp)  
**URL:** https://x.com/dejavucoder/status/1995247669888078299  
**Date:** ~2025

> Prompt caching is the most bang for buck optimization you can do for your LLM based workflows and agents. In this post, I cover tips to hit the prompt cache more consistently and how it works under the hood (probably the first such resource).

### Key Takeaways
- Prompt caching is highest ROI optimization
- Cache hit consistency requires specific techniques
- Understanding cache mechanics improves hit rate
- Critical for agent workflows with repeated prompts
- AGENTS.md content should be cache-friendly (stable, at the start)

---

## Post 5: CORE-Bench Solved with Claude Code

**Author:** @sayashk (Sayash Kapoor) - co-author of "AI Agents That Matter" paper  
**URL:** https://x.com/sayashk/status/1996334941832089732  
**Date:** ~2025

> CORE-Bench is solved (using Opus 4.5 with Claude Code)
> 
> TL;DR: Last week, we released results for Opus 4.5 on CORE-Bench, a benchmark that tests agents on scientific reproducibility tasks. Earlier this week, Nicholas Carlini reached out to share that an updated scaffold that uses...

### Key Takeaways
- Claude Code with Opus 4.5 achieves new SOTA on CORE-Bench
- Scientific reproducibility tasks as benchmark
- Scaffold/harness matters as much as model capability
- Authors of "AI Agents That Matter" paper continue research
- Agent architecture (scaffold) is key differentiator

---

## Synthesis: Common Themes

### 1. Agent Architecture Matters
- Lead agent + specialized subagents pattern
- Task decomposition and planning layers
- Parallel execution of independent tasks

### 2. System Prompts Have Measurable Impact
- ~5% improvement possible with optimized instructions
- Cache-friendly structure (stable prefix content)
- Keep instructions focused and relevant

### 3. Scaffolding/Harness is Critical
- CORE-Bench solved by improving scaffold, not just model
- AGENTS.md/CLAUDE.md is the primary harness touchpoint
- Tool integration (MCP) essential for real-world tasks

### 4. Cost vs Accuracy Trade-offs
- From "AI Agents That Matter" paper: jointly optimize both
- Prompt caching reduces costs significantly
- Simpler agents can match complex ones at lower cost

---

## Relevance to CTO Platform

### For AGENTS.md Structure
1. Keep stable content at the start (for cache hits)
2. Use specialized subagent patterns in workflow templates
3. Include task decomposition guidance in system prompts

### For Container Images
1. Pre-populate AGENTS.md for cache consistency
2. CLI-specific configs for each supported CLI
3. Harmonize file names across CLIs (symlinks)

### For Workflow Design
1. Lead agent + subagent architecture
2. Parallel execution where possible
3. Clear role definitions in agent identities

---

## Related Resources

- **arXiv Paper:** [AI Agents That Matter](https://arxiv.org/abs/2407.01502) - Sayash Kapoor et al.
- **HumanLayer Blog:** [Writing a Good CLAUDE.md](https://www.humanlayer.dev/blog/writing-a-good-claude-md)
- **AGENTS.md Spec:** https://agents.md/

---

*This document is a snapshot of X posts for research purposes. Content belongs to original authors.*

