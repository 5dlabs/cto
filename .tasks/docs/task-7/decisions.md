## Decision Points

- What is the primary inter-service communication paradigm for backend microservices (synchronous REST/gRPC vs asynchronous event-driven messaging)?
- What API paradigm should be used for Morgan's tool-server interface to backend services (REST, gRPC, or GraphQL)?
- What authentication and authorization mechanism should be used for internal service-to-service communication?
- What access control model should be used for the web frontend and Morgan agent (role-based vs attribute-based)?
- Should the Signal integration for Morgan be self-hosted (Signal-CLI) or use a third-party SaaS relay?

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP