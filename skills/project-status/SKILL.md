---
name: project-status
description: Aggregate project status across CodeRuns, Linear issues, and agent memory. Gives Morgan a unified view of project health.
triggers:
  - "project status"
  - "where are we"
  - "what's the status"
  - "how's the project"
  - "progress report"
  - "task overview"
  - "what's been done"
  - "what's left"
---

# project-status — Project Status Aggregation

## When to Use

Use this skill when asked about overall project health, task progress, or to generate status reports combining data from Kubernetes (CodeRuns), Linear (issues), and agent memory (Qdrant/mem0).

## Data Sources

### 1. CodeRuns (Kubernetes)

Query active and completed CodeRuns for a project:

```bash
# All CodeRuns for a project
kubectl get coderuns -n cto -l "project-id={projectId}" -o wide

# Recent CodeRuns with status
kubectl get coderuns -n cto -o json | jq '.items[] | {name: .metadata.name, task: .spec.taskId, project: .spec.projectId, status: .status.phase, agent: .spec.agent}'
```

### 2. Linear Issues

Query Linear for project tasks and their states:

```bash
# Get project issues via Linear API
curl -s https://api.linear.app/graphql \
  -H "Authorization: $LINEAR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ project(id: \"{projectId}\") { name issues { nodes { title state { name } assignee { name } priority } } } }"}'
```

### 3. Agent Memory (Qdrant/mem0)

Search for project-level memories:

```bash
# Get all handoffs for the project
curl -s http://qdrant.cto.svc.cluster.local:6333/collections/cto_memory/points/scroll \
  -H 'Content-Type: application/json' \
  -d '{
    "limit": 50,
    "with_payload": true,
    "filter": {
      "must": [
        {"key": "user_id", "match": {"text": "jonathon:project:{projectId}"}},
        {"key": "category", "match": {"value": "handoff"}}
      ]
    }
  }'
```

## Status Report Template

When generating a status report, structure it as:

```markdown
# Project: {name}

## Summary
- **Phase**: {current phase}
- **Tasks**: {completed}/{total}
- **Blockers**: {count}
- **Last Activity**: {timestamp}

## Task Breakdown
| Task | Agent | Status | Last Update |
|------|-------|--------|-------------|
| Task 1 - {title} | Rex | ✅ Complete | {date} |
| Task 2 - {title} | Blaze | 🔄 In Progress | {date} |

## Recent Decisions
- {decision 1 from memory}
- {decision 2 from memory}

## Blockers
- {blocker from memory}

## Next Steps
- {based on task_objective memories for incomplete tasks}
```

## Workflow

1. **Identify project** — Get projectId from context or ask
2. **Query CodeRuns** — `kubectl get coderuns` for task pod status
3. **Query memory** — Search project namespace for handoffs, progress, blockers
4. **Query Linear** — Get issue states if LINEAR_API_KEY available
5. **Synthesize** — Combine into status report template
6. **Highlight blockers** — Surface any blocking issues prominently
