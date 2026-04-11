---
name: context-degradation
description: Recognize and mitigate context failures including lost-in-middle, context poisoning, distraction, confusion, and clash.
agents: [blaze, rex, nova, tap, spark, grizz, bolt, cleo, cipher, tess, morgan, atlas, stitch]
triggers: [context problems, lost-in-middle, agent failures, context poisoning, performance degradation, attention patterns]
---

# Context Degradation Patterns

Language models exhibit predictable degradation patterns as context length increases. Understanding these patterns is essential for diagnosing failures and designing resilient systems.

## When to Activate

- Agent performance degrades unexpectedly during long conversations
- Debugging cases where agents produce incorrect outputs
- Designing systems that must handle large contexts reliably
- Investigating "lost in middle" phenomena

## Core Degradation Patterns

### Lost-in-Middle Phenomenon

Models demonstrate U-shaped attention curves. Information at the beginning and end receives reliable attention; middle content suffers 10-40% lower recall accuracy.

**Mitigation:**
- Place critical information at beginning or end
- Use summary structures at attention-favored positions
- Add explicit section headers for navigation

### Context Poisoning

Errors compound through repeated reference. Once poisoned, context creates feedback loops reinforcing incorrect beliefs.

**Symptoms:**
- Degraded output quality on previously successful tasks
- Tool misalignment (wrong tools/parameters)
- Persistent hallucinations despite corrections

**Recovery:**
- Truncate context to before poisoning
- Explicitly note the error and request re-evaluation
- Restart with clean context, preserve only verified info

### Context Distraction

Over-focus on provided information at expense of training knowledge. Even a single irrelevant document reduces performance.

**Mitigation:**
- Apply relevance filtering before loading documents
- Use namespacing to make irrelevant sections easy to ignore
- Consider tool calls instead of loading into context

### Context Confusion

Irrelevant information influences responses inappropriately. Signs include responses addressing wrong query aspects or tool calls appropriate for different tasks.

**Mitigation:**
- Explicit task segmentation
- Clear transitions between task contexts
- State management isolating different objectives

### Context Clash

Accumulated information directly conflicts, creating contradictory guidance.

**Resolution:**
- Explicit conflict marking with clarification requests
- Priority rules establishing source precedence
- Version filtering excluding outdated information

## Degradation Thresholds

| Model | Degradation Onset | Severe Degradation |
|-------|-------------------|-------------------|
| Claude Opus 4.5 | ~100K tokens | ~180K tokens |
| Claude Sonnet 4.5 | ~80K tokens | ~150K tokens |
| GPT-5.2 | ~64K tokens | ~200K tokens |
| Gemini 3 Pro | ~500K tokens | ~800K tokens |

## Four-Bucket Mitigation

1. **Write**: Save context outside window (scratchpads, files)
2. **Select**: Pull relevant context via retrieval/filtering
3. **Compress**: Summarize, abstract, mask observations
4. **Isolate**: Split across sub-agents or sessions

## Guidelines

1. Monitor context length and performance correlation
2. Place critical info at beginning or end
3. Implement compaction before degradation becomes severe
4. Validate retrieved documents for accuracy
5. Use versioning to prevent outdated info clash
6. Test with progressively larger contexts to find thresholds
