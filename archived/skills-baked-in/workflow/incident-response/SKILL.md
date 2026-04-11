---
name: incident-response
description: Incident response and remediation patterns including observability, diagnosis, and targeted fixes.
agents: [rex, grizz, nova, blaze, tap, spark, bolt, morgan]
triggers: [healer, incident, alert, production issue, remediation, diagnosis]
---

# Incident Response and Remediation

Patterns for diagnosing and fixing production issues.

## Healer Mode Workflow

1. **Investigate** - Gather metrics, logs, and system state
2. **Diagnose** - Identify root cause before fixing
3. **Fix** - Implement minimal targeted fix
4. **Validate** - Confirm metrics improve after deployment
5. **Document** - Store learnings for future incidents

## Tool Usage Priority

1. **Observability Tools** - Query Prometheus, Loki, Grafana for metrics and logs
2. **Kubernetes Tools** - Check pod status, events, deployments
3. **ArgoCD Tools** - Verify GitOps sync status
4. **Memory Search** - Look for similar past incidents
5. **Code Fix** - Implement minimal targeted fix

## Observability Queries

### Prometheus Metrics

```promql
# Error rate
sum(rate(http_requests_total{status=~"5.."}[5m])) 
/ sum(rate(http_requests_total[5m]))

# Latency P99
histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))

# CPU usage
sum(rate(container_cpu_usage_seconds_total{pod=~"app-.*"}[5m])) by (pod)

# Memory usage
container_memory_working_set_bytes{pod=~"app-.*"}
```

### Loki Log Queries

```logql
# Errors in last hour
{namespace="production", pod=~"app-.*"} |= "error" | json | level="error"

# Stack traces
{namespace="production"} |= "panic" or |= "stack trace"

# Slow requests
{namespace="production"} | json | latency_ms > 1000
```

## Kubernetes Diagnostics

```bash
# Pod status and events
kubectl get pods -n production -l app=myapp
kubectl describe pod <pod-name> -n production
kubectl get events -n production --sort-by='.lastTimestamp'

# Logs
kubectl logs -n production -l app=myapp --tail=100
kubectl logs -n production <pod-name> --previous  # Previous container

# Resource usage
kubectl top pods -n production
kubectl top nodes

# Deployment status
kubectl rollout status deployment/myapp -n production
kubectl rollout history deployment/myapp -n production
```

## ArgoCD Status

```bash
# Application status
argocd app get myapp
argocd app diff myapp

# Sync status
argocd app sync myapp --dry-run

# Rollback
argocd app rollback myapp <revision>
```

## Common Issues and Solutions

### High Error Rate

1. Check recent deployments
2. Review error logs for patterns
3. Check dependency health
4. Verify configuration changes

### High Latency

1. Check database query performance
2. Review external service latency
3. Check resource constraints (CPU/memory)
4. Look for lock contention

### OOMKilled Pods

1. Increase memory limits
2. Check for memory leaks
3. Review recent code changes
4. Consider horizontal scaling

### CrashLoopBackOff

1. Check logs for startup errors
2. Verify secrets and configs exist
3. Check health check endpoints
4. Review recent deployments

### ImagePullBackOff

1. Verify image exists in registry
2. Check image pull secrets
3. Verify image tag is correct
4. Check registry connectivity

## Healing Guidelines

- **Diagnose first** - Understand the root cause before fixing
- **Minimal changes** - Fix only what's broken
- **Document findings** - Store learnings in memory for future incidents
- **Validate fix** - Confirm metrics improve after deployment
- **Rollback if needed** - Don't hesitate to rollback if fix doesn't work

## Post-Incident

1. Update metrics/alerts if needed
2. Document root cause and fix
3. Store learnings in memory for similar incidents
4. Consider preventive measures
5. Update runbooks if applicable
