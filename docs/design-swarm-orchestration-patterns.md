# Design: Swarm Orchestration Patterns for CTO Platform

**Status:** Draft
**Date:** 2026-02-23
**Author:** Competitive analysis of Swarms API (swarms.ai) applied to OpenClaw/CTO
**Priority:** P0-P2 (phased)

---

## Motivation

Competitive analysis of the [Swarms API platform](https://docs.swarms.ai) identified a significant gap in CTO's orchestration layer: while CTO has superior infrastructure (persistent agents, real tool execution, Kubernetes-native), it lacks **formalized multi-agent coordination patterns** as first-class primitives.

Swarms offers 12+ named swarm architectures. CTO currently relies on ad-hoc task decomposition via Morgan and implicit coordination through NATS/Discord. Adding even a subset of these patterns to OpenClaw would be a meaningful capability upgrade.

## Competitive Positioning

### Where CTO is ahead
- Self-hosted EKS with full infrastructure control
- Persistent StatefulSet agents with cross-session memory
- Real tool execution (shell, git, kubectl, CodeRun jobs)
- NATS pub/sub + Discord bridge for real-time inter-agent communication
- Multi-model support natively per agent
- Domain-specific agents (Rex/Rust, Blaze/frontend, Cipher/security)
- No per-token markup — fixed infrastructure cost

### Where Swarms is ahead
- Formalized orchestration pattern taxonomy (12+ named patterns)
- Dynamic sub-agent delegation at runtime
- SSE streaming for real-time output
- Agent marketplace for reusable configurations
- Specialized reasoning agent types

### Swarms weaknesses
- Stateless API-only — no agent persistence between calls
- No real tool execution (`run_bash` is disabled)
- No self-hosting option
- ~3x token price markup over raw API ($18.50/1M output vs ~$6/1M raw)
- Single MCP URL per agent only (multi-MCP "coming soon")

---

## Proposed Orchestration Patterns

### Phase 1: Core Patterns (P0)

#### 1. SequentialWorkflow
**What:** Linear agent pipeline — output of agent A feeds as input to agent B, then C, etc.
**CTO mapping:** Morgan decomposes a task into ordered steps, each assigned to a different agent. The orchestrator ensures outputs chain sequentially.
**Implementation:** New OpenClaw orchestration mode. Morgan (or any coordinator agent) emits a `workflow.sequential` NATS message with an ordered agent list and task chain. A workflow controller processes steps in order, passing each agent's output as context to the next.

```yaml
workflow:
  type: sequential
  agents: [researcher, writer, editor, fact-checker]
  task: "Create a comprehensive security audit report"
```

#### 2. ConcurrentWorkflow
**What:** Fan-out parallel execution — same task (or different tasks) sent to N agents simultaneously, results collected.
**CTO mapping:** Multiple CodeRun jobs or agent tasks dispatched in parallel via NATS, with a barrier that waits for all to complete.
**Implementation:** Coordinator publishes N tasks to NATS concurrently, each targeting a specific agent. A `workflow.barrier` mechanism collects all responses before synthesizing.

```yaml
workflow:
  type: concurrent
  agents: [rex, blaze, grizz]
  task: "Review PR #42 from your domain perspective"
  collect: true
```

#### 3. MajorityVoting
**What:** N agents independently evaluate the same task, then a voting mechanism determines the outcome by majority consensus.
**CTO mapping:** Particularly useful for Stitch (code review), Cipher (security), Cleo (quality) — multiple agents vote on whether a PR is merge-ready.
**Implementation:** Same task dispatched to N agents. Each returns a structured vote (approve/reject + reasoning). Orchestrator tallies votes and applies majority rule.

```yaml
workflow:
  type: majority_voting
  agents: [stitch, cipher, cleo, tess]
  task: "Should PR #42 be merged? Vote YES or NO with reasoning."
  quorum: 3  # need 3/4 to approve
```

#### 4. HierarchicalSwarm
**What:** Supervisor/worker tree — a coordinator agent breaks down work, delegates to workers, and synthesizes results.
**CTO mapping:** This is what Morgan already does informally. Formalize it so any agent can act as a supervisor, spawning sub-tasks to worker agents.
**Implementation:** Extend the CodeRun model so a parent agent can create child CodeRun jobs with explicit supervisor-worker relationships. The supervisor receives all child outputs and produces a synthesis.

```yaml
workflow:
  type: hierarchical
  supervisor: morgan
  workers: [rex, blaze, grizz, bolt]
  task: "Implement the new payment gateway integration"
  synthesis: true  # supervisor compiles final report
```

### Phase 2: Advanced Patterns (P1)

#### 5. GraphWorkflow (DAG)
**What:** Directed acyclic graph of agents — conditional edges, fan-out, fan-in, with dependencies.
**CTO mapping:** Complex multi-step workflows where some steps depend on others, some run in parallel.
**Implementation:** Define a DAG of agent tasks with dependency edges. A workflow engine topologically sorts and executes, respecting dependencies.

```yaml
workflow:
  type: graph
  nodes:
    research: { agent: researcher, task: "Gather requirements" }
    design: { agent: blaze, task: "Create UI mockups", depends_on: [research] }
    backend: { agent: rex, task: "Implement API", depends_on: [research] }
    infra: { agent: bolt, task: "Provision infrastructure", depends_on: [research] }
    integrate: { agent: morgan, task: "Integration testing", depends_on: [design, backend, infra] }
```

#### 6. DebateWithJudge
**What:** Two or more agents argue opposing positions, a judge agent evaluates and decides.
**CTO mapping:** Useful for architecture decisions — have Rex and Grizz debate Rust vs Go for a component, with Morgan as judge.
**Implementation:** Agents are given the same problem with opposing system prompts. Their outputs are collected and passed to a judge agent who renders a final decision.

### Phase 3: Meta Patterns (P2)

#### 7. Dynamic Sub-Agent Delegation
**What:** A coordinator agent can dynamically spawn ephemeral child agents at runtime without pre-defining them.
**CTO mapping:** Add `spawn_agent(name, prompt, task)` and `assign_task(agent_id, task)` as tool functions available to any persistent agent.
**Implementation:** These tools create short-lived CodeRun jobs with custom system prompts. The parent agent decides at runtime how many sub-agents to create and what specialties they need.

```python
# Available as tool functions within any OpenClaw agent
spawn_agent(
    name="base-chain-researcher",
    prompt="You are an expert on Base L2 blockchain architecture...",
    task="Research the current Base node requirements"
)
```

#### 8. Reasoning Templates
**What:** Reusable meta-prompts that wrap any agent in a reasoning loop (self-consistency, dual-perspective, iterative refinement).
**CTO mapping:** Implement as OpenClaw Handlebars templates that can be applied to any agent.
- **Self-consistency:** Run same prompt N times, compare outputs, return consensus
- **Dual-perspective:** Two passes with different framing, then synthesis
- **Iterative refinement:** Agent critiques its own output, then improves it

#### 9. Agent Template Library
**What:** Reusable, composable agent configurations beyond the current Helm roster.
**CTO mapping:** A library of agent templates stored in the repo (or a registry) that can be instantiated on demand — e.g., "code-reviewer", "technical-writer", "security-auditor" templates that any workflow can reference.

---

## Implementation Strategy

### NATS Message Schema

All workflow patterns communicate via NATS. Proposed topic hierarchy:

```
workflow.create.<type>     # Create a new workflow
workflow.<id>.step.<n>     # Individual step execution
workflow.<id>.result       # Step result
workflow.<id>.complete     # Workflow complete with final output
workflow.<id>.vote         # Vote submission (for MajorityVoting)
```

### Workflow Controller

A new lightweight controller (likely a Rust service, or an extension to the OpenClaw orchestrator) that:
1. Accepts workflow definitions via NATS or API
2. Manages execution state (which steps have completed, which are pending)
3. Routes outputs between agents according to the workflow pattern
4. Collects and synthesizes results
5. Publishes final output

### Integration with Existing Agents

- All existing agents (Morgan, Rex, Blaze, etc.) participate in workflows without modification — they just receive tasks and return results via NATS as they already do
- The workflow controller is an additional layer that manages coordination
- Morgan becomes the default coordinator for hierarchical workflows but any agent can be a coordinator

### SSE Streaming Endpoint (P1)

Add an SSE endpoint to the CTO API gateway that streams workflow progress in real-time:

```
GET /api/v1/workflows/<id>/stream

event: step_started
data: {"agent": "rex", "step": 2, "task": "Implement API"}

event: step_completed
data: {"agent": "rex", "step": 2, "output": "..."}

event: workflow_completed
data: {"output": "...", "duration_ms": 45000}
```

---

## Effort Estimates

| Pattern | Complexity | Estimated Effort | Dependencies |
|---------|-----------|-----------------|--------------|
| SequentialWorkflow | Low | 1 week | Workflow controller skeleton |
| ConcurrentWorkflow | Low | 1 week | Workflow controller skeleton |
| MajorityVoting | Medium | 1-2 weeks | Vote collection + quorum logic |
| HierarchicalSwarm | Medium | 1-2 weeks | Formalize existing Morgan behavior |
| GraphWorkflow (DAG) | High | 2-3 weeks | DAG execution engine |
| DebateWithJudge | Medium | 1 week | Concurrent + judge routing |
| Dynamic Sub-Agent Delegation | Medium | 2 weeks | CodeRun tool function integration |
| Reasoning Templates | Low | 3-5 days | Handlebars template authoring |
| Agent Template Library | Low | 1 week | Schema + storage |
| SSE Streaming | Medium | 1-2 weeks | API gateway integration |

**Total estimated effort:** 10-16 weeks for full implementation across all phases.

---

## Open Questions

1. Should the workflow controller be a separate Rust microservice or embedded in the OpenClaw orchestrator?
2. Do we need persistent workflow state (for recovery after crashes) or is in-memory sufficient?
3. Should workflows be definable via YAML files, API calls, or both?
4. How do we handle timeout and failure policies per workflow step?
5. Should MajorityVoting support weighted votes (e.g., Cipher's security vote counts 2x)?

---

## References

- [Swarms API Documentation](https://docs.swarms.ai/docs/documentation)
- [Swarms Architecture](https://docs.swarms.ai/docs/documentation/getting-started/architecture)
- [Swarms Swarm Types](https://docs.swarms.ai/docs/documentation/multi-agent/swarm_types)
- [Swarms Sub-Agent Delegation](https://docs.swarms.ai/docs/documentation/capabilities/sub_agents)
- [Swarms ATP Protocol](https://docs.swarms.ai/docs/atp/vision) (agent-to-agent payments — watch only)
