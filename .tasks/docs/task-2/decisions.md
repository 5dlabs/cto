## Decision Points

- What is the source of truth for agent → Linear user ID mappings? Options: hardcoded TypeScript map, environment-variable-driven config, or dynamic Linear API lookup by display name. This affects maintainability and whether new agents require a code deploy.
- Should the `agent:pending` label be created on-the-fly via the Linear API if it doesn't already exist in the workspace, or should it be assumed to be pre-provisioned?

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia