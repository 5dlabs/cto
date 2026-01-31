# Task 1: Setup Kubernetes infrastructure and databases

## Priority
high

## Description
Provision all required databases and infrastructure services including PostgreSQL, Redis/Valkey, Kafka, MongoDB, RabbitMQ, and SeaweedFS using CRDs

## CRITICAL: Use Subagents for Parallel Work

**You MUST delegate each database deployment to a subagent using the `Task` tool.**

Structure your work as follows:
1. **Main agent (you):** Orchestrate and coordinate
2. **Subagent 1:** Deploy PostgreSQL with CloudNative-PG (use `Task` tool)
3. **Subagent 2:** Deploy Redis/Valkey cluster (use `Task` tool)
4. **Subagent 3:** Deploy MongoDB with Percona (use `Task` tool)
5. **Subagent 4:** Deploy Kafka with Strimzi (use `Task` tool)
6. **Subagent 5:** Deploy RabbitMQ cluster (use `Task` tool)
7. **Subagent 6:** Deploy SeaweedFS for object storage (use `Task` tool)

Example of spawning a subagent:
```
Task({
  task: "Create PostgreSQL CRD manifest using CloudNative-PG operator. Create the file at /workspace/infra/postgresql/cluster.yaml with a 3-instance HA cluster.",
  agentName: "general-purpose"
})
```

After all subagents complete, harmonize their work:
- Create the main kustomization.yaml
- Configure namespaces and network policies
- Document the deployment

## Dependencies
None

## Implementation Details
Deploy CloudNative-PG PostgreSQL cluster, Valkey Redis, Strimzi Kafka, Percona MongoDB, RabbitMQ cluster, and SeaweedFS for object storage. Configure namespaces, network policies, and resource quotas.

## Acceptance Criteria
All CRDs deploy successfully, databases are reachable from test pods, health checks pass, persistent storage is provisioned

## Workspace
You are working in `/workspace` which is a Git repository. Create all manifests and configurations there.

## Notes
- This is a LOCAL Docker test environment - you will NOT have a real Kubernetes cluster
- Focus on CREATING the manifest files and configurations
- **USE SUBAGENTS** - This is critical for testing parallel execution
