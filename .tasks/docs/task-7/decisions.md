## Decision Points

- Should the Signal integration for Morgan be self-hosted via Signal-CLI or use a third-party Signal gateway service?
- How should the Morgan AI agent orchestrate backend service calls: direct API calls or via a tool-server abstraction?
- Which LLM model should back the Morgan AI agent for conversational orchestration and skill execution?
- How should the web chat channel be exposed: as a WebSocket endpoint on the OpenClaw agent pod, or via a dedicated chat gateway service?

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP