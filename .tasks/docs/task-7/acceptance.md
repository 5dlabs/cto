## Acceptance Criteria

- [ ] 1. MCP tool connectivity test: each of the 11 MCP tools responds successfully when called with valid test parameters (mock or against running services). 2. Signal integration test: send test message via signal-cli REST API → verify Morgan receives it, processes intent, and sends response back within 10 seconds. 3. Web chat WebSocket test: connect to /ws/chat, send user message, receive streaming agent response within 10 seconds, verify JSON message format. 4. Session continuity test: connect with session token, disconnect, reconnect with same token → verify conversation history preserved. 5. Lead qualification e2e test: simulate Signal conversation with event inquiry → verify Morgan asks qualifying questions, checks availability (via MCP tool), generates quote → verify opportunity created in RMS. 6. Tool authorization test: all MCP tool calls include valid morgan-agent JWT, backend services accept and log the service identity. 7. Health readiness test: /health/ready returns 200 only when LLM endpoint, Signal-CLI, and at least 3 critical MCP tools (catalog_search, generate_quote, create_invoice) are reachable. 8. Conversation context test: send 10 messages in sequence, verify Morgan's responses demonstrate awareness of conversation history.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.