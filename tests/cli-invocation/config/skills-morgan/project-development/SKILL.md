---
name: project-development
description: Methodology for identifying LLM-suited tasks, designing pipelines, and iterating with agent-assisted development.
agents: [morgan, bolt, atlas]
triggers: [start LLM project, design batch pipeline, task-model fit, pipeline architecture, cost estimation]
---

# Project Development Methodology

Principles for identifying tasks suited to LLM processing, designing effective architectures, and iterating rapidly using agent-assisted development.

## Task-Model Fit Recognition

### LLM-Suited Tasks

| Characteristic | Why It Fits |
|----------------|-------------|
| Synthesis across sources | LLMs excel at combining information |
| Subjective judgment with rubrics | Grading, evaluation, classification |
| Natural language output | Human-readable text goals |
| Error tolerance | Individual failures don't break system |
| Batch processing | No conversational state needed |
| Domain knowledge in training | Model has relevant context |

### LLM-Unsuited Tasks

| Characteristic | Why It Fails |
|----------------|--------------|
| Precise computation | Math, counting unreliable |
| Real-time requirements | Latency too high |
| Perfect accuracy requirements | Hallucination risk |
| Proprietary data dependence | Model lacks context |
| Sequential dependencies | Heavy step-by-step coupling |
| Deterministic output requirements | Same input ≠ identical output |

## Manual Prototype Step

**Before automation, validate with manual test:**

1. Copy one representative input into model interface
2. Evaluate output quality
3. This takes minutes, prevents hours of waste

**Answers critical questions:**
- Does model have required knowledge?
- Can it produce needed format?
- What quality level to expect at scale?
- What failure modes exist?

## Pipeline Architecture

**Canonical structure:**
```
acquire → prepare → process → parse → render
```

1. **Acquire**: Fetch raw data (APIs, files, databases)
2. **Prepare**: Transform to prompt format
3. **Process**: Execute LLM calls (expensive, non-deterministic)
4. **Parse**: Extract structured data from outputs
5. **Render**: Generate final outputs

Stages 1, 2, 4, 5 are deterministic. Stage 3 is expensive.

## File System as State Machine

Each processing unit gets a directory:
```
data/{id}/
├── raw.json      # acquire complete
├── prompt.md     # prepare complete
├── response.md   # process complete
├── parsed.json   # parse complete
```

**Benefits:**
- Natural idempotency (file existence gates execution)
- Easy debugging (human-readable state)
- Simple parallelization (directories independent)
- Trivial caching (files persist)

## Structured Output Design

**Effective structure includes:**
1. Section markers for parsing
2. Format examples showing exact output
3. Rationale: "I will be parsing this programmatically"
4. Constrained values (enums, ranges, formats)

**Build robust parsers:**
- Use flexible regex patterns
- Provide sensible defaults for missing sections
- Log failures instead of crashing

## Cost Estimation

```
Total cost = (items × tokens_per_item × price_per_token) + overhead
```

**For batch processing:**
- Estimate input tokens (prompt + context)
- Estimate output tokens (typical response)
- Multiply by item count
- Add 20-30% buffer for retries

## Single vs Multi-Agent

**Single-agent works for:**
- Batch processing with independent items
- Non-interacting items
- Simpler cost management

**Multi-agent works for:**
- Parallel exploration
- Tasks exceeding single context window
- Specialized sub-agents improving quality

Primary reason: **context isolation**, not role anthropomorphization.

## Architectural Reduction

Start minimal. Add complexity only when proven necessary.

**Vercel d0 case:**
- Before: 17 specialized tools, 80% success, 274s execution
- After: 2 tools (bash + SQL), 100% success, 77s execution

**When reduction wins:**
- Well-documented data layer
- Sufficient model reasoning capability
- Specialized tools constraining rather than enabling

## Guidelines

1. Validate task-model fit with manual prototyping first
2. Structure pipelines as discrete, idempotent, cacheable stages
3. Use file system for state management
4. Design prompts for structured, parseable outputs
5. Start minimal; add complexity only when proven necessary
6. Estimate costs early and track throughout
7. Build robust parsers handling LLM output variations
8. Expect and plan for multiple architectural iterations
