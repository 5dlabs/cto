# Task: Create Kubernetes Health Check Module

You are the Bolt infrastructure agent. Create a simple health check module for a Kubernetes deployment.

## Requirements

1. Create a directory structure at `/workspace/health/`
2. Create `health/mod.rs` - Module entry point with pub exports
3. Create `health/probes.rs` - Implement liveness and readiness probe handlers
4. Create `health/types.rs` - Define response types (HealthStatus enum, HealthResponse struct)

## Implementation Details

The module should:
- Return JSON responses
- Include version and uptime information
- Support both `/healthz` (liveness) and `/readyz` (readiness) endpoints

## Definition of Done

Check off items in `/workspace/task/acceptance-criteria.md` as you complete them.
Use the `Read` tool to view the acceptance criteria file first.
