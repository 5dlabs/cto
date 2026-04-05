Implement subtask 7009: Add observability hooks for Prometheus, Loki, and Grafana

## Objective
Instrument the Morgan agent with Prometheus metrics, structured logging for Loki, and create Grafana dashboards for monitoring agent health, performance, and usage patterns.

## Steps
Step 1: Implement Prometheus metrics: request_count (by channel, skill), response_latency_seconds (histogram by channel, skill), active_connections (gauge by channel), mcp_tool_invocation_count (by tool, status), mcp_tool_latency_seconds (histogram by tool), error_count (by channel, error_type). Step 2: Expose a /metrics endpoint in Prometheus exposition format. Step 3: Implement structured JSON logging for Loki ingestion: include correlation IDs, channel type, session ID, skill invoked, MCP tools called, latency, and error details. Step 4: Create a Grafana dashboard with panels: active connections by channel, request rate, response latency heatmap, MCP tool invocation rates, error rates, and top skills by usage. Step 5: Configure alerting rules: response time >10s, error rate >5%, connection count approaching limits.

## Validation
Prometheus /metrics endpoint returns valid metrics after agent processes requests; Loki receives structured log entries with correlation IDs; Grafana dashboard renders with live data; alert rules fire when thresholds are artificially breached.