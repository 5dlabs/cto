Implement subtask 7010: Implement end-to-end lead-to-invoice flow and observability

## Objective
Wire together all skills into the complete lead-to-invoice orchestration flow, add comprehensive logging and observability hooks, and validate the full flow end-to-end.

## Steps
1. Define the lead-to-invoice orchestration flow: incoming lead → sales-qual → customer-vet → quote-gen → (optional upsell) → invoice creation. Ensure the agent can drive this flow conversationally across all channels.
2. Implement flow state management: track where each customer is in the flow, handle interruptions and resumptions, and support branching (e.g., customer declines quote → re-quote).
3. Add observability hooks: structured logs for each flow stage, metrics for stage transition latencies, and error counters.
4. Implement health check endpoint for the agent runtime.
5. Test the complete flow on each channel (Signal, voice, web chat) and verify <10s response time for simple queries.
6. Profile and optimize hot paths if response times exceed targets (e.g., parallel tool calls where possible).
7. Verify all actions are logged with sufficient detail for debugging and audit.

## Validation
Complete lead-to-invoice flow executes successfully on Signal, voice, and web chat channels; simple queries respond in <10s; flow state correctly tracks and resumes after interruptions; health check endpoint returns 200; observability logs and metrics are emitted for every flow stage; all 10 MCP tools are invoked at least once during the full flow test.