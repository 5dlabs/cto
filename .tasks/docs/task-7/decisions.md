## Decision Points

- What API paradigm should be used for inter-service communication between backend services (e.g., Morgan agent, Equipment Catalog, RMS, Finance, Vetting, Social Engine)?
- What authentication and authorization mechanism should be used for service-to-service and user-to-service API calls?
- Should the Morgan AI agent (OpenClaw) interact with backend services directly via APIs, or exclusively through MCP tool-server abstraction?
- How should the Morgan agent manage conversation state and context across channels (Signal, voice, web chat)? Should there be a unified session store or per-channel state management?
- How should the agent's system prompt and skill definitions be managed — hardcoded in config, stored in a database, or dynamically loaded?

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP