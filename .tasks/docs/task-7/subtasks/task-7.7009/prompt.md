Implement subtask 7009: Performance optimization and load testing for 500+ concurrent Signal connections

## Objective
Optimize the Morgan agent and Signal-CLI setup to achieve <10 second response times and handle 500+ concurrent Signal connections, conducting load tests to validate.

## Steps
1. Profile the agent's response pipeline: measure time from message receipt to response sent. Identify bottlenecks (LLM inference, tool calls, Signal-CLI).
2. Optimize: ensure async/non-blocking processing of concurrent messages. Use connection pooling for backend service calls.
3. Configure horizontal pod autoscaling (HPA) for the agent deployment based on CPU/memory or custom metrics (concurrent connections).
4. Tune Signal-CLI for concurrent message handling: configure thread pool size, connection limits.
5. Write a load test script (e.g., using k6 or locust) simulating 500+ concurrent Signal conversations sending messages simultaneously.
6. Run load tests and capture metrics: p50/p95/p99 response times, error rates, message throughput.
7. Iterate on configuration until <10s p95 response time at 500 concurrent connections is achieved.

## Validation
Load test with 500 concurrent simulated Signal conversations. p95 response time is under 10 seconds. Error rate is below 1%. No message loss observed. HPA scales pods appropriately under load.