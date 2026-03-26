# CTO Deliberation Agent Values

This directory contains Helm values files for CTO-owned OpenClaw agents.
These agents are deployed via `applications/workloads/deliberation-agents.yaml`
and use the `charts/openclaw` chart from the
[openclaw-platform](https://github.com/5dlabs/openclaw-platform) repo.

## Agents

| Agent | Model | Role |
|-------|-------|------|
| optimist | Claude Sonnet 4.6 (Anthropic) | Debate — advocates for ambitious approaches |
| pessimist | GPT-5.3 | Debate — devil's advocate, finds failure modes |
| morgan | GPT-5.3 | Lean PM persona for the avatar demo and remote API |
| committee-1 | claude-sonnet-4-6 | Vote — impartial pragmatist |
| committee-2 | gpt-5.3 | Vote — systems thinking, scalability |
| committee-3 | MiniMax-M2.5 | Vote — DX / maintainability |
| committee-4 | Claude Opus 4.6 (Anthropic) | Vote — security / reliability |
| committee-5 | gpt-5.3 | Vote — velocity / MVP |

## How They Fit In

```
PRD → Intake (me) → deliberation phase
         ├─ NATS → optimist (proposes approach)
         ├─ NATS → pessimist (challenges it)
         │   └─ decision point raised
         │       ├─ NATS → committee-1..5 (parallel vote)
         │       └─ tally → winner → back to debate
         └─ design brief compiled → task generation
```

The deliberation agents are NATS-only (no Discord) and stateless
(`memorySearch.enabled: false`). Morgan is the exception: it is a lean HTTP-first
agent exposed separately for the talking-avatar proof of concept.
