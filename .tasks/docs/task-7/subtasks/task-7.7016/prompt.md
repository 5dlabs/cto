Implement subtask 7016: Implement health check endpoint with LLM, Signal-CLI, and MCP tool connectivity verification

## Objective
Build the /health/ready and /health/live endpoints that verify connectivity to the LLM endpoint, Signal-CLI sidecar registration status, and reachability of critical MCP tools.

## Steps
1. Implement `/health/live` endpoint:
   - Simple 200 OK if the Morgan process is running
   - No dependency checks — just process alive verification
2. Implement `/health/ready` endpoint:
   - Check LLM connectivity: send a minimal test prompt to the configured LLM endpoint, verify 200 response within 5 seconds
   - Check Signal-CLI: GET http://localhost:8080/v1/about → verify 200 and registration status is 'registered'
   - Check critical MCP tools (at least 3 must be reachable):
     - sigma1_catalog_search: HEAD or GET to Equipment Catalog health endpoint
     - sigma1_generate_quote: HEAD or GET to RMS health endpoint
     - sigma1_create_invoice: HEAD or GET to Finance health endpoint
   - Return 200 only if ALL checks pass
   - Return 503 with JSON body detailing which checks failed
3. Response format:
   ```json
   {
     "status": "ready"|"not_ready",
     "checks": {
       "llm": { "status": "ok"|"error", "latency_ms": 150 },
       "signal_cli": { "status": "ok"|"error", "registered": true },
       "mcp_tools": { "reachable": 11, "total": 11, "failed": [] }
     }
   }
   ```
4. Cache health check results for 10 seconds to avoid hammering dependencies on frequent probe calls.
5. Log health check failures at WARN level for monitoring/alerting.

## Validation
Call /health/live and verify 200 response. Call /health/ready with all dependencies available and verify 200 with all checks passing. Disable LLM mock and verify /health/ready returns 503 with llm check failed. Stop signal-cli sidecar and verify /health/ready returns 503 with signal_cli check failed. Verify response caching: call twice within 10 seconds and verify second call is faster.