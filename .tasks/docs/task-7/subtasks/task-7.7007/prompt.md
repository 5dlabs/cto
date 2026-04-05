Implement subtask 7007: Implement MCP Tool Server HTTP client, JWT auth, and tool registry

## Objective
Build the shared MCP tool server infrastructure: HTTP client wrapper with JWT injection, tool registry that loads all 11 tool definitions, error handling middleware, and request/response logging.

## Steps
1. Create a central tool registry module that loads all 11 MCP tool definitions and exposes them via the MCP tool-server protocol (HTTP SSE or stdio, depending on OpenClaw's MCP client implementation).
2. Implement shared HTTP client wrapper:
   - Inject `Authorization: Bearer ${MORGAN_SERVICE_JWT}` header on all outbound requests
   - JWT token: read from environment variable or Kubernetes secret mount
   - Base URL resolution: read service URLs from sigma1-infra-endpoints ConfigMap environment variables
   - Timeout: 30 seconds per tool call, configurable
   - Retry: 1 retry on 502/503 with exponential backoff
3. Implement request/response logging for all tool calls:
   - Log: tool_name, input_params (redacted sensitive fields), response_status, latency_ms
   - Structured JSON logging for observability
4. Implement tool call error normalization:
   - HTTP 4xx → structured error with user-friendly message
   - HTTP 5xx → 'Service temporarily unavailable' with retry suggestion
   - Network timeout → 'Request timed out, please try again'
5. Implement tool list endpoint for MCP protocol: returns all 11 tools with names, descriptions, and JSON schemas.
6. Validate that all service URLs are configured at startup; log warnings for missing endpoints.

## Validation
Start the MCP tool server and verify the tool list endpoint returns all 11 tools with correct schemas. Send a tool call through the HTTP client wrapper and verify JWT header is injected. Simulate 502, 503, and timeout responses and verify retry/error normalization behavior. Verify structured logs are emitted for each tool call.