## Decision Points

- What API paradigm should be used for inter-service communication between backend services (e.g., Equipment Catalog, RMS, Finance, Vetting)?
- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- Should the Signal integration for Morgan be self-hosted via Signal-CLI or use a third-party Signal gateway service?
- How should the Morgan AI agent orchestrate backend service calls: via direct API calls or through a tool-server abstraction?
- How should API versioning be handled for public and internal APIs?

## Coordination Notes

- Agent owner: Angie
- Primary stack: OpenClaw/MCP