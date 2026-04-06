## Decision Points

- How should service-to-service communication be handled between backend services (e.g., Morgan agent, RMS, Catalog, Finance, Vetting, Social Engine)?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- How should the Signal integration for Morgan be implemented?
- How should the Morgan agent orchestrate backend service actions: via direct API calls or through a tool-server abstraction?

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP