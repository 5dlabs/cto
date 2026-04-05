Implement subtask 7012: Implement tool call parallelization and end-to-end performance validation

## Objective
Configure MCP tool call parallelization for independent tool invocations and validate the <10 second end-to-end response time target for simple queries across all channels.

## Steps
1. Tool call parallelization:
   a. In the agent's tool-calling logic, identify when multiple tools are independent (e.g., checking availability for 3 different products simultaneously).
   b. Configure OpenClaw/MCP to issue parallel HTTP requests for independent tool calls rather than sequential.
   c. Implement fan-out/fan-in pattern: dispatch parallel tool calls, collect all results, then continue conversation.
   d. Set per-tool timeouts so a slow tool doesn't block the entire parallel batch.
2. Performance profiling:
   a. Measure end-to-end latency for simple queries: user sends 'Do you have LED panels?' → Morgan responds with catalog results.
   b. Break down latency: message receive time + LLM inference time + tool call time + response send time.
   c. Target: total < 10 seconds for single-tool simple queries.
3. Optimization opportunities:
   a. If LLM inference is the bottleneck, consider streaming responses (send partial text as it's generated).
   b. If tool calls are slow, verify cluster DNS resolution and service connectivity latency.
   c. For voice channel: ensure total round-trip (speech-to-text + Morgan + text-to-speech) is conversational (<5 seconds perceived).
4. Load test:
   a. Run 10 concurrent simple queries (catalog search) across web chat WebSocket.
   b. Verify all 10 respond within 10 seconds.
   c. Monitor pod resource utilization during load test.

## Validation
Run 10 concurrent catalog search queries via WebSocket, verify all complete within 10 seconds. Measure and log individual latency components (LLM inference, tool call, message delivery). Verify parallel tool calls (e.g., 3 simultaneous availability checks) complete faster than sequential would. Verify voice round-trip latency is under 5 seconds for simple queries.