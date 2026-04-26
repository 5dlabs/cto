# AlertHub E2E Lifecycle Test

End-to-end test of the CTO platform lifecycle using the AlertHub PRD as the test project.

## Purpose

This test validates the complete CTO workflow from PRD intake through deployment:

1. **Intake** - PRD parsing, task generation, Linear integration
2. **Implementation** - Play workflow execution with specialized agents
3. **Quality** - Code review (Cleo)
4. **Security** - Security scanning (Cipher)
5. **Testing** - Test execution (Tess)
6. **Integration** - PR merge (Atlas)
7. **Deploy** - ArgoCD deployment (Bolt)
8. **Postflight** - Telemetry and verification

## Project Configuration

From `lifecycle-test/ralph-cto.json`:

```json
{
  "project": {
    "name": "alerthub-e2e-test",
    "namespace": "cto",
    "service": "prd-alerthub-e2e-test",
    "prdPath": "tests/intake/alerthub-e2e-test/prd.md",
    "architecturePath": "tests/intake/alerthub-e2e-test/architecture.md",
    "numTasks": 30
  }
}
```

## Phases and Gates

### Phase 1: Intake

**Objective**: Submit intake for AlertHub and verify tasks.json is generated with testStrategy for each task.

| Gate | Command | Purpose |
|------|---------|---------|
| `intake-coderun-created` | `kubectl get coderuns -n cto -o json \| jq -e '[.items[] \| select(.spec.runType == "intake")] \| length > 0'` | CodeRun CRD exists |
| `linear-sidecar-running` | `kubectl get pods -n cto ... \| jq -e '... "Running" or "Pending"'` | Linear sync sidecar active |
| `intake-succeeded` | `kubectl get pods -n cto ... \| select(.status.phase == "Succeeded")` | Intake pod completed |
| `linear-issues-created` | `curl -sf http://localhost:8081/api/linear/project/.../issues` | Linear issues exist |
| `linear-issues-have-subtasks` | Check children count > 0 | Subtasks generated |
| `linear-activities-posted` | Check activities endpoint | Agent activity logged |
| `tasks-json-exists` | `gh api repos/.../contents/.tasks/tasks/tasks.json` | Tasks file in repo |
| `tasks-have-test-strategy` | Verify all tasks have testStrategy field | Test coverage planned |
| `task-docs-created` | Check .tasks/docs folder | Documentation generated |
| `cto-config-attached-to-issue` | Check config attached to Linear | Config propagated |
| `cto-config-configmap-exists` | `kubectl get configmap ... cto-config-project-*` | K8s ConfigMap created |
| `intake-pr-created` | `gh pr list ... intake-*` | PR opened |

### Phase 2: Implementation

**Objective**: Verify Play is triggered via webhook, uses CTO config from Linear, and agent executes correctly.

Key gates:
- Play workflow triggered by GitHub webhook (not MCP)
- CTO config mounted from ConfigMap
- Correct agent assigned based on task type
- Agent completes task and creates PR

### Phase 3: Quality (Cleo)

**Objective**: Verify Cleo review completes with language-appropriate checks.

Key gates:
- Cleo CodeRun created
- Review comments posted to PR
- Language-specific linting applied

### Phase 4: Security (Cipher)

**Objective**: Verify Cipher security scan runs and reports no critical issues.

Key gates:
- Cipher CodeRun created
- Security scan completed
- No critical vulnerabilities

### Phase 5: Testing (Tess)

**Objective**: Verify Tess runs tests per testStrategy and records results.

Key gates:
- Tess CodeRun created
- Tests executed per testStrategy
- Results recorded in Linear

### Phase 6: Integration (Atlas)

**Objective**: Verify Atlas merges after checks pass and updates Linear.

Key gates:
- All prior stages passed
- PR merged to main
- Linear issue status updated

### Phase 7: Deploy (Bolt)

**Objective**: Verify Bolt deploy task runs, applies manifests via ArgoCD.

Key gates:
- Deploy CodeRun created
- ArgoCD sync triggered
- Health checks pass

### Phase 8: Postflight

**Objective**: Verify telemetry, Linear timeline completeness, and Ralph iteration.

Key gates:
- All metrics recorded
- Linear timeline complete
- End-to-end success

## File Structure

```
lifecycle-test/
├── ralph-cto.json           # Main configuration (phases, gates, settings)
├── ralph-cto.state.json     # Current state (phase, attempts, completed)
├── ralph-coordination.json  # Dual agent coordination state
├── current-objective.md     # Current phase objective and gates
├── progress.txt             # Human-readable progress log
├── report.json              # Structured event log
├── prompt.md                # Execution prompt template
├── monitor-prompt.md        # Monitor agent system prompt
├── remediation-prompt.md    # Remediation agent system prompt
├── pin.md                   # Pinned context for agents
├── pin.lookup.md            # Lookup table for pins
├── observe/
│   ├── state.json           # Observed system state
│   └── events.jsonl         # Event stream
└── archive/                 # Previous run logs
```

## Prerequisites

### Local Services (via launchd)

```bash
# Check services are running
just launchd-status

# Required services:
# - agent-controller (port 8080)
# - pm-server (port 8081)
# - healer (port 8082)
```

### Port Forwards

```bash
# If running against remote cluster
kubectl port-forward svc/prometheus-server -n observability 9090:80
kubectl port-forward svc/argocd-server -n argocd 8080:80
kubectl port-forward svc/argo-workflows-server -n automation 2746:2746
```

### Environment

- `kubectl` configured for target cluster
- `gh` CLI authenticated
- `claude` CLI available
- `droid` CLI available (for GPT-5.2 monitor)

## Running the Test

### With Dual Ralph (Recommended)

```bash
# Start dual agent system
./scripts/ralph-dual.sh start

# Monitor status
./scripts/ralph-dual.sh status

# Watch logs
./scripts/ralph-dual.sh logs

# Or attach to see live output
./scripts/ralph-dual.sh attach monitor
```

### Manual Gate Checking

```bash
# Run a single gate
kubectl get coderuns -n cto -o json | jq -e '[.items[] | select(.spec.runType == "intake")] | length > 0'

# Check all intake gates
jq '.phases[] | select(.id == "intake") | .gates[] | .command' lifecycle-test/ralph-cto.json
```

## Current State

Check `lifecycle-test/ralph-cto.state.json`:

```bash
cat lifecycle-test/ralph-cto.state.json | jq .
```

Example:
```json
{
  "phase": "intake",
  "attempts": {},
  "completedObjectives": [],
  "attendedCompleted": 0,
  "last_success": null
}
```

## Progress Log

The `lifecycle-test/progress.txt` contains a detailed log of all test runs, failures, and remediations. Key sections:

- **Phase transitions** - When phases start/complete
- **Gate results** - Pass/fail for each gate
- **Failures** - Root cause analysis of failures
- **Remediations** - Fixes applied

## Common Issues

### Intake Failures

| Issue | Cause | Fix |
|-------|-------|-----|
| AI returns summary instead of JSON | MCP enabled during intake | Ensure `disable_mcp: true` in intake code |
| Output token limit exceeded | PRD too large | Reduce `numTasks` or use smaller model |
| Linear sidecar not running | Pod completed | Gate should check for Succeeded status too |

### Play Workflow Failures

| Issue | Cause | Fix |
|-------|-------|-----|
| Wrong agent assigned | Task metadata missing | Check agent_hint in tasks.json |
| Config not mounted | ConfigMap not synced | Verify cto-config-project-* exists |
| Webhook not triggered | PM server not receiving | Check cloudflare tunnel |

### Cluster Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| Pods stuck Pending | Node capacity | Check `kubectl describe node` |
| PVC attach errors | Volume still attached | Force delete old pods |
| CodeRun stuck Running | Controller not watching | Check controller logs |

## Test Repository

The test creates/uses: `github.com/5dlabs/prd-prd-alerthub-e2e-test`

Structure after intake:
```
prd-prd-alerthub-e2e-test/
├── .tasks/
│   ├── tasks/
│   │   └── tasks.json      # Generated tasks
│   └── docs/
│       └── task-*/         # Task documentation
├── src/                    # Implementation (after Play)
└── README.md
```

## Cleanup

```bash
# Delete all CodeRuns
kubectl delete coderuns -n cto --all

# Delete stuck pods
kubectl delete pods -n cto --field-selector=status.phase=Failed
kubectl delete pods -n cto --field-selector=status.phase=Succeeded

# Reset Ralph state
./scripts/ralph-dual.sh reset

# Archive current progress
mv lifecycle-test/progress.txt lifecycle-test/archive/$(date +%Y%m%d_%H%M%S)/
```

## Monitoring

### Healer Dashboard

The Healer monitors Play workflows and can detect stuck agents:

```bash
curl -sf http://localhost:8082/health
curl -sf http://localhost:8082/api/plays
```

### Grafana

Access via port-forward:
```bash
kubectl port-forward svc/grafana -n observability 3000:80
# Open http://localhost:3000
```

### Logs

```bash
# Controller logs
tail -f /tmp/cto-launchd/controller.log

# PM Server logs
tail -f /tmp/cto-launchd/pm-server.log

# Healer logs
tail -f /tmp/cto-launchd/healer.log
```

## Related Documentation

- [Dual Ralph System](dual-ralph-system.md) - Agent coordination details
- [Play Workflow](play-workflow.md) - Multi-agent orchestration
- [MCP Tools Reference](mcp-tools.md) - Available tools
- [Troubleshooting](troubleshooting.md) - Common issues
