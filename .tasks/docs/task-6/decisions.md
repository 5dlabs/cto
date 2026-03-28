## Decision Points

- LLM Model Selection: Confirm the specific LLM model and version to be used (`gpt-5.4-pro` is specified, but could be a decision point for future flexibility).
- Signal-CLI Deployment Strategy: Decide if Signal-CLI runs as a sidecar, separate pod, or external service.
- Twilio/ElevenLabs Integration Flow: Define the exact call flow and webhook handling between Twilio, ElevenLabs, and the Morgan agent.
- MCP Tool Definition Granularity: Determine the level of abstraction for MCP tools (e.g., one tool per API endpoint vs. one tool per service).
- Inter-service Authentication: How will the AI Agent authenticate calls to backend services (Catalog, RMS, Finance, Vetting)? (e.g., API keys, JWT, mTLS).
- Cloudflare Tunnel Configuration: Define specific ingress rules, DNS records, and security policies for the tunnel.

## Coordination Notes

- Agent owner: angie
- Primary stack: OpenClaw/MCP