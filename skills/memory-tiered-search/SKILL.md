---
name: memory-tiered-search
description: Search agent memory across tiers — portfolio, project, task, agent. Uses Qdrant vector store via mem0 namespace conventions.
triggers:
  - "what's happening across projects"
  - "project status"
  - "task status"
  - "what did Rex do"
  - "what did Blaze do"
  - "memory search"
  - "search memories"
  - "recall"
  - "what do we know about"
  - "handoff"
  - "task history"
  - "project history"
---

# memory-tiered-search — Hierarchical Agent Memory Retrieval

## When to Use

Use this skill when you need to retrieve information from agent memory at any scope level:
- **Portfolio-wide**: "What's happening across all projects?"
- **Project-level**: "How's project X going?"
- **Task-level**: "What's the status of task 1?"
- **Agent-level**: "What did Rex work on in task 2?"

## Namespace Convention

Memory is organized by `user_id` namespaces in Qdrant:

| Tier | Namespace Pattern | Example |
|------|------------------|---------|
| Portfolio | `jonathon` | All memories for the owner |
| Morgan ops | `jonathon:agent:morgan` | Morgan's own operational memories |
| Project | `jonathon:project:{projectId}` | Project-wide summaries, handoffs |
| Task | `jonathon:project:{projectId}:task:{taskId}` | Task-scoped work |
| Agent detail | `jonathon:project:{projectId}:task:{taskId}:{agent}` | Individual agent memories |

## How to Search

### Using mem0 search (preferred)

The mem0 plugin's `search` capability uses these parameters:
- `user_id`: The namespace to search within
- `query`: Natural language search query
- `categories`: Filter by memory category
- `limit`: Number of results (default 5)

### Search by Tier

**Portfolio overview** — search across all project handoffs:
```
search(user_id="jonathon", query="project status update", categories=["task_completion", "handoff"])
```

**Project drill-down** — get details for a specific project:
```
search(user_id="jonathon:project:7c749f56", query="progress blockers decisions")
```

**Task status** — what happened in a specific task:
```
search(user_id="jonathon:project:7c749f56:task:1", query="implementation status", categories=["task_progress", "task_completion"])
```

**Agent audit** — what did a specific agent do:
```
search(user_id="jonathon:project:7c749f56:task:1:rex", query="what was implemented")
```

### Search by Category

Available custom categories:
- `task_objective` — Goals, requirements, acceptance criteria
- `task_progress` — Status updates, milestones, intermediate results
- `task_completion` — Final outcomes, deliverables, PRs merged
- `blocker` — Issues that blocked progress
- `handoff` — Structured summaries from task agents to Morgan
- `intake_decision` — Decisions during project intake
- `architecture` — Architecture decisions and patterns
- `debugging` — Root cause analysis and bug fixes
- `user_preference` — User preferences and conventions

### Direct Qdrant Queries (advanced)

For complex cross-namespace queries, use Qdrant REST API directly:

```bash
# Search across ALL namespaces (portfolio view)
curl -s http://qdrant.cto.svc.cluster.local:6333/collections/cto_memory/points/scroll \
  -H 'Content-Type: application/json' \
  -d '{
    "limit": 20,
    "with_payload": true,
    "filter": {
      "must": [
        {"key": "category", "match": {"value": "handoff"}}
      ]
    }
  }'

# Search within a project namespace
curl -s http://qdrant.cto.svc.cluster.local:6333/collections/cto_memory/points/scroll \
  -H 'Content-Type: application/json' \
  -d '{
    "limit": 20,
    "with_payload": true,
    "filter": {
      "must": [
        {"key": "user_id", "match": {"text": "jonathon:project:7c749f56"}}
      ]
    }
  }'
```

## Workflow

1. **Determine scope** — Is the user asking about portfolio, project, task, or agent?
2. **Build namespace** — Construct the `user_id` from the scope
3. **Choose categories** — Pick relevant categories for the query type
4. **Search** — Use mem0 search or direct Qdrant query
5. **Synthesize** — Combine results into a coherent answer
6. **Drill down** — If the answer is insufficient, search a narrower or broader tier

## Notes

- Morgan has **read access to ALL namespaces** — can search any tier
- Task agents are **scoped** — can only see their own project:task namespace
- Handoff memories are the primary bridge between task agents and Morgan
- If no memories exist for a namespace, the project/task may not have run yet
