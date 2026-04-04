## Decision Points

- Where is the agent-hint-to-Linear-user-ID mapping stored? Options: hardcoded map in code, a ConfigMap, or queried dynamically from the Linear API at startup. This affects how new agents are onboarded.
- Should resolve_agent_delegates() cache Linear user lookups for the lifetime of a pipeline run or re-resolve on every invocation? Affects correctness vs. performance if the mapping changes mid-run.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia