Implement subtask 7008: Performance optimization and load testing for 500+ concurrent connections

## Objective
Optimize Morgan's response pipeline to achieve <10s response time for simple queries and validate support for 500+ concurrent Signal connections through load testing and bottleneck identification.

## Steps
Step 1: Profile the end-to-end response pipeline for simple queries (e.g., 'What equipment do you have?') — measure time from message receipt → agent processing → MCP tool invocation → response delivery. Step 2: Identify and optimize bottlenecks: connection pooling to MCP tool backends, agent LLM inference latency, message serialization overhead. Step 3: Implement response streaming where possible — start sending partial responses while the agent is still processing. Step 4: Create a load test harness that simulates 500+ concurrent Signal connections sending messages simultaneously. Step 5: Run load tests, capture latency percentiles (p50, p95, p99), throughput, error rates, and resource utilization. Step 6: Tune concurrency settings: connection pool sizes, worker thread counts, message queue depths. Step 7: Document performance baselines and capacity limits.

## Validation
Simple query response time is consistently <10 seconds at p95; load test with 500+ concurrent Signal connections completes without errors; p99 latency for simple queries under load is <15 seconds; no connection drops or timeouts during sustained load; resource utilization stays within pod limits.