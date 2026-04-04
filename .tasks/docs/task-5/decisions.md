## Decision Points

- The API paradigm (HTTP POST vs NATS publish) for bridge communication is pending D2 resolution. This task implements HTTP as the default, but the facade must accommodate a swap to NATS. If D2 resolves to NATS, the HTTP implementation subtask would need rework.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia