# Database Infrastructure Deployment Test

Deploy database manifests using the custom sub-agents.

## Task

1. Use the `postgres-deployer` sub-agent to create PostgreSQL manifests at `/workspace/infra/postgresql/`
2. Use the `redis-deployer` sub-agent to create Redis manifests at `/workspace/infra/redis/`
3. Use the `mongo-deployer` sub-agent to create MongoDB manifests at `/workspace/infra/mongodb/`

## Important

- Use the Task tool with `subagent_type` set to the specific agent name (e.g., `postgres-deployer`)
- Each sub-agent should create:
  - A main cluster.yaml
  - A kustomization.yaml
- Keep the manifests minimal for this test (just basic structure)

## Expected Sub-agent Usage

```
Task(subagent_type="postgres-deployer", description="Deploy PostgreSQL", prompt="...")
Task(subagent_type="redis-deployer", description="Deploy Redis", prompt="...")
Task(subagent_type="mongo-deployer", description="Deploy MongoDB", prompt="...")
```

Create the base directories first, then spawn all three sub-agents in parallel.
