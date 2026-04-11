---
name: context-optimization
description: Extend effective context capacity through compression, masking, caching, and partitioning techniques.
agents: [blaze, rex, nova, tap, spark, grizz, bolt, cleo, cipher, tess, morgan, atlas, stitch]
triggers: [optimize context, reduce tokens, token costs, context limits, observation masking, context budgeting]
---

# Context Optimization Techniques

Context optimization extends effective capacity through strategic compression, masking, caching, and partitioning. Effective optimization can double or triple effective context capacity.

## When to Activate

- Context limits constrain task complexity
- Optimizing for cost reduction (fewer tokens = lower costs)
- Reducing latency for long conversations
- Building production systems at scale

## Core Strategies

### Compaction

Summarize context contents when approaching limits, reinitialize with summary.

**Priority for compression:**
1. Tool outputs → replace with summaries
2. Old turns → summarize early conversation
3. Retrieved docs → summarize if recent versions exist
4. **Never compress system prompt**

**Summary preservation by type:**
- Tool outputs: Key findings, metrics, conclusions
- Conversations: Key decisions, commitments, context shifts
- Documents: Key facts and claims

### Observation Masking

Tool outputs can comprise 80%+ of token usage. Replace verbose outputs with compact references once their purpose is served.

**Masking Strategy:**

| Category | Action |
|----------|--------|
| Never mask | Current task observations, most recent turn, active reasoning |
| Consider masking | 3+ turns ago, verbose outputs with extractable key points |
| Always mask | Repeated outputs, boilerplate, already summarized |

**Example:**
```
if len(observation) > max_length:
    ref_id = store_observation(observation)
    return f"[Obs:{ref_id} elided. Key: {extract_key(observation)}]"
```

### KV-Cache Optimization

Reuse cached computations across requests with identical prefixes.

**Cache-friendly ordering:**
1. System prompt (stable, first)
2. Tool definitions (stable)
3. Frequently reused elements
4. Unique content (last)

**Design tips:**
- Avoid dynamic content like timestamps
- Use consistent formatting
- Keep structure stable across sessions

### Context Partitioning

Split work across sub-agents with isolated contexts. Each operates in clean context focused on its subtask.

**Aggregation pattern:**
1. Validate all partitions completed
2. Merge compatible results
3. Summarize if still too large

## Budget Management

Design explicit token budgets:
- System prompt: X tokens
- Tool definitions: Y tokens
- Retrieved docs: Z tokens
- Message history: W tokens
- Reserved buffer: 10-20%

**Trigger optimization when:**
- Token utilization > 70%
- Response quality degrades
- Costs increase due to long contexts

## Decision Framework

| Dominant component | Apply |
|-------------------|-------|
| Tool outputs | Observation masking |
| Retrieved documents | Summarization or partitioning |
| Message history | Compaction with summarization |
| Multiple | Combine strategies |

## Performance Targets

- Compaction: 50-70% reduction, <5% quality degradation
- Masking: 60-80% reduction in masked observations
- Cache optimization: 70%+ hit rate for stable workloads

## Guidelines

1. Measure before optimizing—know current state
2. Apply compaction before masking when possible
3. Design for cache stability with consistent prompts
4. Partition before context becomes problematic
5. Balance token savings against quality preservation
