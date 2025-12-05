# OpenMemory Integration Guide

## Overview

OpenMemory has been integrated into our multi-agent orchestration platform to provide persistent, cross-task memory capabilities. This allows agents to learn from past experiences, avoid repeated errors, and share knowledge across the team.

## Architecture

```
┌─────────────────────────────────────┐
│    OpenMemory Server (Centralized)   │
│    Namespace: cto-system             │
│    Service: openmemory:3000         │
│    Storage: 20Gi PVC (SQLite)       │
└─────────────────────────────────────┘
              ▲ HTTP API
              │
    ┌─────────┴─────────┬─────────┬─────────┐
    │                   │         │         │
Rex/Blaze           Cleo      Tess     Atlas/Others
(Implementation)   (Quality)   (QA)    (Docs/Ops)
```

## Quick Start

### 1. Deploy OpenMemory

```bash
# The ArgoCD application will auto-deploy from main branch
# Check status:
kubectl get pods -n cto-system | grep openmemory

# Verify health:
kubectl exec -n cto-system deploy/openmemory -- curl localhost:3000/health
```

### 2. Agent Integration

Each agent container now has access to memory functions via `/agent-templates/shared/memory-functions.sh`:

```bash
# In agent container script (e.g., container-rex.sh.hbs)

# Source memory functions
source /agent-templates/memory-functions.sh

# Initialize memory connection
init_memory

# Query for relevant patterns before starting
PATTERNS=$(query_memory "Docker build patterns for Node.js" 10)

# Store successful patterns
store_success_pattern "npm-docker-fix" "Add .dockerignore with node_modules"

# Check for error solutions when failures occur
if [ $EXIT_CODE -ne 0 ]; then
  check_error_memory "$ERROR_MESSAGE"
fi
```

## Memory Functions Reference

### Core Functions

#### `query_memory(query, limit, include_waypoints)`
Search for relevant memories based on semantic similarity.

```bash
# Find Docker-related memories
MEMORIES=$(query_memory "Docker build failures npm" 10 true)

# Process results
echo "$MEMORIES" | jq -r '.[].content'
```

#### `add_memory(content, pattern_type, success, metadata)`
Store a new memory with metadata.

```bash
# Store a successful pattern
add_memory "Use React Query for data fetching in service-frontend" \
  "architecture" \
  true \
  '{"service": "frontend", "framework": "react"}'
```

#### `reinforce_memory(memory_id, amount)`
Strengthen a memory that proved useful.

```bash
# Reinforce a helpful memory
reinforce_memory "mem_abc123" 2
```

### Pattern-Specific Functions

#### `check_error_memory(error_message)`
Look for solutions to specific errors.

```bash
# When npm install fails
check_error_memory "npm ERR! code EACCES"
# Returns: Previous solutions with success rates
```

#### `load_project_context()`
Load relevant project information at task start.

```bash
# At beginning of task
CONTEXT=$(load_project_context)
# Returns: Tech stack, dependencies, conventions, known issues
```

#### `store_success_pattern(name, solution, details)`
Save a reusable pattern that worked.

```bash
store_success_pattern \
  "kubernetes-resource-limits" \
  "Set memory: 2Gi for Node.js services" \
  "Prevents OOMKilled errors in production"
```

#### `store_error_pattern(error, context, solution)`
Document an error and its solution (if found).

```bash
store_error_pattern \
  "Cannot find module 'tsx'" \
  "TypeScript execution in Docker" \
  "Install tsx globally in Dockerfile"
```

## Integration Examples

### Rex (Implementation Agent)

```bash
#!/bin/bash
# In container-rex.sh.hbs

# Load memory functions
source /agent-templates/memory-functions.sh

# Initialize and load context
init_memory
PROJECT_CONTEXT=$(load_project_context)

# Before implementing, check for patterns
API_PATTERNS=$(query_memory "REST API structure for ${SERVICE_NAME}" 10)

# During implementation
if ! npm install; then
  # Check memory for solutions
  SOLUTIONS=$(check_error_memory "npm install failed")
  
  if [ -n "$SOLUTIONS" ]; then
    echo "Found solutions in memory:"
    echo "$SOLUTIONS"
    # Apply suggested fix...
  else
    # Try common fixes and store what works
    rm -rf node_modules package-lock.json
    if npm install; then
      store_success_pattern "npm-clean-install" \
        "Remove node_modules and package-lock.json before install"
    fi
  fi
fi

# After successful implementation
store_success_pattern "task-${TASK_ID}" \
  "Implemented using React Query and PostgreSQL" \
  "Performance: 200ms average response time"
```

### Cleo (Code Quality Agent)

```bash
# In container-cleo.sh.hbs

# Check for common quality issues
QUALITY_PATTERNS=$(query_memory "linting issues ${SERVICE_NAME}" 15)

# Store recurring issues
if grep -q "no-unused-vars" lint-results.txt; then
  store_error_pattern \
    "Unused variables in ${FILE_PATH}" \
    "ESLint check" \
    "Remove or prefix with underscore"
fi
```

### Tess (QA Agent)

```bash
# In container-tess.sh.hbs

# Load test strategies
TEST_STRATEGIES=$(query_memory "E2E test patterns Kubernetes" 10)

# Store test configurations that work
store_success_pattern \
  "k8s-test-deployment" \
  "Use initContainers for test data setup" \
  "Reduces flakiness by 80%"
```

## Memory Schema

Each memory entry contains:

```json
{
  "id": "mem_abc123",
  "content": "Use React Query instead of Redux for data fetching",
  "metadata": {
    "agent": "rex",
    "task_id": "task-45",
    "service": "frontend",
    "pattern_type": "architecture",
    "success": true,
    "timestamp": "2024-11-24T10:30:00Z"
  },
  "vectors": {
    "factual": [0.1, 0.2, ...],
    "emotional": [0.3, 0.4, ...],
    "temporal": [0.5, 0.6, ...],
    "relational": [0.7, 0.8, ...],
    "behavioral": [0.9, 0.1, ...]
  },
  "salience": 0.85,
  "reinforcements": 3,
  "last_accessed": "2024-11-24T15:45:00Z"
}
```

## Metrics and Monitoring

### Key Performance Indicators

1. **Query Performance**
   - Target: < 100ms average latency
   - Current: ~110ms

2. **Memory Hit Rate**
   - Target: > 60% useful memories found
   - Tracked per agent

3. **Error Recurrence**
   - Target: < 10% repeat failures
   - Measured weekly

4. **Storage Growth**
   - Target: < 1GB/month
   - Auto-decay keeps size manageable

### Grafana Dashboard

Access metrics at: `http://grafana.5dlabs.io/d/openmemory`

Panels include:
- Queries per agent
- Memory hit/miss rates
- Top retrieved patterns
- Error resolution rates
- Storage usage trends

## Best Practices

### Do's ✅

1. **Query before implementing** - Always check for existing patterns
2. **Store both successes and failures** - Learn from everything
3. **Use descriptive content** - Future you will thank you
4. **Reinforce helpful memories** - Boost what works
5. **Include context in metadata** - Service, framework, version

### Don'ts ❌

1. **Don't store sensitive data** - No passwords, tokens, or PII
2. **Don't over-query** - Cache results when possible
3. **Don't store temporary fixes** - Only proven solutions
4. **Don't ignore decay** - Old patterns may be outdated

## Troubleshooting

### Memory Not Available

```bash
# Check service
kubectl get svc openmemory -n cto-system

# Check pods
kubectl get pods -n cto-system | grep openmemory

# View logs
kubectl logs -n cto-system deploy/openmemory
```

### Slow Queries

1. Check memory size: `kubectl exec -n cto-system deploy/openmemory -- ls -lah /data`
2. Review query complexity - use specific terms
3. Reduce `include_waypoints` if not needed

### Memory Not Persisting

1. Verify PVC is mounted: `kubectl describe pod -n cto-system -l app=openmemory`
2. Check write permissions: Should be user 1000:1000
3. Review backup status in logs

## Advanced Usage

### Cross-Agent Learning

```bash
# Rex stores a pattern
REX_MEMORY=$(add_memory "Use connection pooling for PostgreSQL" "performance" true)

# Tess queries and finds it
TESS_QUERY=$(query_memory "database performance" 10)
# Automatically includes Rex's memory if relevant

# Tess reinforces if helpful
reinforce_memory $(echo "$REX_MEMORY" | jq -r '.id')
```

### Memory Chains via Waypoints

```bash
# Memories automatically link when related
# Query with waypoints to get connected memories
CHAIN=$(query_memory "authentication flow" 5 true)
# Returns: Auth setup → JWT config → Session management → Logout flow
```

### Temporal Queries

```bash
# Recent memories have higher salience
RECENT=$(query_memory "changes in last week" 20)

# Older memories decay unless reinforced
OLD=$(query_memory "patterns from task-1" 10)
# May return nothing if too old and unused
```

## Rollout Status

- [x] OpenMemory Helm chart created
- [x] Docker image configuration
- [x] Memory functions library
- [x] ArgoCD deployment manifest
- [ ] Integration into Rex container
- [ ] Integration into Cleo container
- [ ] Integration into Tess container
- [ ] Metrics collection setup
- [ ] Grafana dashboards
- [ ] 2-week pilot measurement
- [ ] Performance analysis report

## Support

For issues or questions:
1. Check the [OpenMemory docs](https://openmemory.cavira.app/docs)
2. Review agent logs for memory-related errors
3. Contact the platform team in #cto-platform Slack channel



