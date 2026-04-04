## Decision Points

- Confirm whether `opossum` is Bun-compatible or if a lighter/native alternative is needed — opossum relies on Node.js EventEmitter internals that may behave differently under Bun's compatibility layer. Evaluate `cockatiel` as an alternative.
- Determine the Hermes SaaS API contract: what is the exact endpoint URL pattern, request schema, authentication header format (Bearer token vs. x-api-key), and response schema? This must be resolved before the client module can be implemented.
- Decide where the research memo is persisted — in a database table (requires schema migration coordination) or in-memory pipeline state (simpler but lost on restart). Task 8 depends on being able to read this artifact.

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia