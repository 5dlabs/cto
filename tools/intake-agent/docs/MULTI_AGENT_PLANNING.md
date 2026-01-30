# Multi-Agent Planning System

## Overview

This document outlines the evolution from single-agent task generation to a multi-perspective planning system with research, debate, and consensus.

---

## Current State

### Task Generation Flow
```
PRD → parse_prd → Tasks (with decisionPoints)
         ↓
     expand_task → Subtasks (with subagentType, parallelizable)
```

### Dual-Agent Pattern (Quality Validation)
```
Generator (Claude) → Content
         ↓
Critic (Claude/MiniMax) → Issues, Confidence Score
         ↓
Refiner (Claude) → Improved Content
         ↓
[Loop until approved or max iterations]
```

**Limitation:** The critic validates quality/correctness, but doesn't take a strategic stance (pessimistic vs optimistic). It's error-checking, not perspective-based reasoning.

### Research Capability
- `research` operation exists
- MCP servers configured: Firecrawl, OctoCode, Context7, websearch
- **Status:** Not wired into task generation flow

---

## Desired State

### Complete Planning Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    PHASE 1: RESEARCH                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   PRD Input                                                      │
│       ↓                                                          │
│   Research Agent (uses ALL sources)                              │
│   ┌─────────────┬─────────────┬─────────────┬─────────────┐    │
│   │ Firecrawl   │ OctoCode    │ Context7    │ WebSearch   │    │
│   │ (web docs)  │ (code pats) │ (lib docs)  │ (general)   │    │
│   └─────────────┴─────────────┴─────────────┴─────────────┘    │
│       ↓                                                          │
│   Research Summary (shared context for all agents)               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                 PHASE 2: PROPOSAL GENERATION                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Research Summary                                               │
│       ↓                                                          │
│   ┌─────────────────────┐   ┌─────────────────────┐            │
│   │  PESSIMISTIC AGENT  │   │  OPTIMISTIC AGENT   │            │
│   │                     │   │                     │            │
│   │  - Risk-aware       │   │  - Opportunity-     │            │
│   │  - Conservative     │   │    focused          │            │
│   │  - What could go    │   │  - What could go    │            │
│   │    wrong?           │   │    right?           │            │
│   │  - Safety margins   │   │  - Efficiency gains │            │
│   │  - Fallback plans   │   │  - Innovation       │            │
│   └──────────┬──────────┘   └──────────┬──────────┘            │
│              ↓                         ↓                        │
│        Solution A                Solution B                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    PHASE 3: DEBATE                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Round 1:                                                       │
│   ┌─────────────────────┐   ┌─────────────────────┐            │
│   │  Pessimist reviews  │   │  Optimist reviews   │            │
│   │  Solution B         │   │  Solution A         │            │
│   │  → Identifies risks │   │  → Identifies       │            │
│   │  → Proposes fixes   │   │    missed opps      │            │
│   └──────────┬──────────┘   └──────────┬──────────┘            │
│              ↓                         ↓                        │
│        Critique A→B              Critique B→A                   │
│                                                                  │
│   Round 2 (if needed):                                          │
│   ┌─────────────────────┐   ┌─────────────────────┐            │
│   │  Pessimist responds │   │  Optimist responds  │            │
│   │  to B→A critique    │   │  to A→B critique    │            │
│   └──────────┬──────────┘   └──────────┬──────────┘            │
│              ↓                         ↓                        │
│        Rebuttal A                Rebuttal B                     │
│                                                                  │
│   MAX_DEBATE_ROUNDS = 3 (configurable)                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                  PHASE 4: CONSENSUS                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Synthesis Agent reads:                                         │
│   - Solution A + Solution B                                      │
│   - All debate rounds                                            │
│   - Points of agreement                                          │
│   - Points of disagreement                                       │
│       ↓                                                          │
│   TENTATIVE PLAN                                                 │
│   - Merges best of both approaches                               │
│   - Documents remaining tradeoffs                                │
│   - Flags unresolved decisions for human review                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   PHASE 5: CRITIQUE                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Critic Agent reviews TENTATIVE PLAN                            │
│   - Structural issues                                            │
│   - Missing components                                           │
│   - Feasibility concerns                                         │
│   - Dependency problems                                          │
│       ↓                                                          │
│   CRITIQUE REPORT                                                │
│   - Issues (critical, major, minor)                              │
│   - Suggested remediations                                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                 PHASE 6: REMEDIATION (ONE PASS)                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Remediation Agent:                                             │
│   - Applies critique fixes                                       │
│   - ONE PASS ONLY (no iteration)                                 │
│       ↓                                                          │
│   FINAL PLAN                                                     │
│   - Tasks with subtasks                                          │
│   - Decision points documented                                   │
│   - Ready for implementation                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Gap Analysis

| Component | Current State | Desired State | Gap |
|-----------|--------------|---------------|-----|
| **Research** | `research` op exists, MCP configured | All sources used before planning | Need to wire into flow |
| **Pessimistic Agent** | ❌ Not implemented | Agent with risk-aware persona | Need new prompt/persona |
| **Optimistic Agent** | ❌ Not implemented | Agent with opportunity-focused persona | Need new prompt/persona |
| **Debate Protocol** | ❌ Not implemented | Structured back-and-forth with rounds | Need new orchestration |
| **Consensus Synthesis** | ❌ Not implemented | Merge solutions from debate | Need synthesis logic |
| **Critique** | ✅ Exists (`runCritic`) | Review tentative plan | Exists, needs wiring |
| **Remediation** | ✅ Exists (`runRefiner`) | One-pass fix | Exists, limit to 1 pass |

---

## Implementation Tasks

### Task 1: Wire Research into Flow
- [ ] Call `research` operation before task generation
- [ ] Pass research findings to all subsequent agents
- [ ] Ensure all MCP sources are used (Firecrawl, OctoCode, Context7, websearch)
- [ ] Test each research source works

### Task 2: Create Agent Personas
- [ ] Define `PessimisticAgentPrompt` - risk-aware, conservative
- [ ] Define `OptimisticAgentPrompt` - opportunity-focused, innovative
- [ ] Both receive same research context
- [ ] Both output structured solution proposals

### Task 3: Implement Debate Protocol
- [ ] Create `DebateRound` type
- [ ] Implement cross-critique (A reviews B, B reviews A)
- [ ] Implement rebuttal mechanism
- [ ] Add `MAX_DEBATE_ROUNDS` limit (default: 3)
- [ ] Track points of agreement/disagreement

### Task 4: Implement Consensus Synthesis
- [ ] Create `SynthesisAgent` that reads all debate output
- [ ] Merge best elements from both solutions
- [ ] Document remaining tradeoffs
- [ ] Output `TentativePlan`

### Task 5: Wire Critique and Remediation
- [ ] Apply existing `runCritic` to tentative plan
- [ ] Apply existing `runRefiner` with `maxIterations: 1`
- [ ] Output `FinalPlan`

### Task 6: Create Unified Operation
- [ ] New operation: `generate_plan_with_debate`
- [ ] Orchestrates all phases
- [ ] Returns full trace of reasoning

---

## Configuration (Future)

```typescript
interface PlanningConfig {
  research: {
    enabled: boolean;
    sources: ('firecrawl' | 'octocode' | 'context7' | 'websearch')[];
    maxResearchTime: number; // seconds
  };
  debate: {
    enabled: boolean;
    maxRounds: number; // default: 3
    pessimisticWeight: number; // 0-1, how much to weight pessimist
    optimisticWeight: number; // 0-1, how much to weight optimist
  };
  critique: {
    enabled: boolean;
    remediationPasses: number; // default: 1
  };
}
```

---

## Success Criteria

1. **Research Phase**: All 4 MCP sources return relevant findings
2. **Proposal Phase**: Both agents produce structurally valid solutions
3. **Debate Phase**: Agents identify genuine tradeoffs, not just agree
4. **Consensus Phase**: Merged solution incorporates both perspectives
5. **Critique Phase**: Identifies real issues, not false positives
6. **Final Output**: Production-ready task list

---

## Next Steps

1. Review this document
2. Prioritize implementation tasks
3. Start with Task 1 (Research wiring) - foundational
4. Then Task 2 (Personas) - enables debate
5. Then Tasks 3-5 (Debate, Consensus, Wiring)
6. Finally Task 6 (Unified operation)
