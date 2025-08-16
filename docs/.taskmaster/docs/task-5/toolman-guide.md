# Toolman Guide: Implementation DAG WorkflowTemplate

## Overview
This guide provides instructions for using the Implementation DAG system that orchestrates the complete development lifecycle from task specification through deployment and acceptance testing.

## Available Tools

### 1. Implementation DAG API
**Purpose**: Execute end-to-end implementation workflows

#### Submit Implementation Workflow
```bash
# Submit implementation DAG for PR
argo submit --from workflowtemplate/implementation-dag \
  -p owner=myorg \
  -p repo=myapp \
  -p pr=123 \
  --watch

# Custom configuration
argo submit --from workflowtemplate/implementation-dag \
  -p owner=myorg \
  -p repo=myapp \
  -p pr=456 \
  -p domain=staging.company.com \
  -p chartPath=deploy/helm/app \
  -p imageTag=feature-branch-latest \
  --watch
```

#### Monitor Workflow Progress
```bash
# Watch execution
argo watch implementation-abc123

# Check specific stages
argo logs implementation-abc123 -c implement
argo logs implementation-abc123 -c deploy
argo logs implementation-abc123 -c acceptance

# Get deployment URLs
argo get implementation-abc123 -o json | \
  jq '.status.outputs.parameters[] | select(.name=="preview_urls") | .value'
```

### 2. Preview Environment Management

#### List Preview Environments
```bash
# List all preview namespaces
kubectl get ns -l preview=true

# Get preview environment details
kubectl get ns impl-pr-123 -o yaml

# Check deployed services
kubectl get all -n impl-pr-123
```

#### Access Preview Applications
```bash
# Get ingress URLs
kubectl get ingress -n impl-pr-123

# Test health endpoints
curl -k https://pr-123.preview.example.com/health

# View logs
kubectl logs -n impl-pr-123 deployment/myapp-pr-123
```

## Local Development Tools

### DAG Tester
**Purpose**: Test implementation DAG locally

```bash
# Test complete workflow
./scripts/test-implementation-dag.sh --pr 123 --repo test-app

# Test specific stages
./scripts/test-implementation-dag.sh --stage deploy --pr 123

# Dry run
./scripts/test-implementation-dag.sh --dry-run --pr 123
```

### Acceptance Test Simulator
**Purpose**: Simulate acceptance testing scenarios

```bash
# Run basic acceptance tests
./scripts/simulate-acceptance-tests.sh \
  --host pr-123.preview.example.com \
  --scenarios basic,performance

# Custom thresholds
./scripts/simulate-acceptance-tests.sh \
  --host pr-123.preview.example.com \
  --min-success-pct 0.98 \
  --max-p95-ms 500
```

## Configuration Examples

### Helm Values Override
```yaml
# values-preview.yaml
replicaCount: 1
image:
  repository: myregistry/myapp
  tag: pr-123
ingress:
  enabled: true
  host: pr-123.preview.example.com
resources:
  requests:
    cpu: 100m
    memory: 128Mi
```

### Project Configuration
```yaml
# .implementation-config.yml
preview:
  domain: preview.example.com
  chartPath: charts/application
  valuesFile: values-preview.yaml
deployment:
  timeout: 600
  healthCheck: /health
acceptance:
  endpoints: ["/", "/api/health", "/api/metrics"]
  thresholds:
    successRate: 0.95
    p95Latency: 1000
```

## Common Usage Patterns

### 1. Manual Implementation Flow
```bash
#!/bin/bash
# run-implementation.sh
OWNER=${1:-myorg}
REPO=${2:-myrepo}
PR=${3:-123}

echo "Starting implementation for $OWNER/$REPO #$PR"

WORKFLOW=$(argo submit --from workflowtemplate/implementation-dag \
  -p owner="$OWNER" \
  -p repo="$REPO" \
  -p pr="$PR" \
  -o name)

echo "Submitted: $WORKFLOW"
argo watch "$WORKFLOW"

# Get preview URL on success
if argo get "$WORKFLOW" -o json | jq -e '.status.phase == "Succeeded"' > /dev/null; then
  PREVIEW_URL=$(argo get "$WORKFLOW" -o json | \
    jq -r '.status.outputs.parameters[] | select(.name=="preview_urls") | .value' | \
    jq -r '.hosts[0]')
  echo "Preview available at: https://$PREVIEW_URL"
fi
```

### 2. Batch Implementation
```bash
#!/bin/bash
# batch-implement.sh
BACKLOG_FILE=${1:-backlog.json}

# Process each item in backlog
jq -c '.[]' "$BACKLOG_FILE" | while read item; do
  OWNER=$(echo "$item" | jq -r '.owner')
  REPO=$(echo "$item" | jq -r '.repo') 
  PR=$(echo "$item" | jq -r '.pr')
  
  echo "Processing $OWNER/$REPO #$PR"
  argo submit --from workflowtemplate/implementation-dag \
    -p owner="$OWNER" \
    -p repo="$REPO" \
    -p pr="$PR"
done
```

### 3. Environment Cleanup
```bash
#!/bin/bash
# cleanup-previews.sh
AGE_DAYS=${1:-7}

# Find old preview environments
kubectl get ns -l preview=true -o json | \
  jq -r --argjson age_days "$AGE_DAYS" \
  '.items[] | select(
    (.metadata.creationTimestamp | fromdateiso8601) < (now - ($age_days * 86400))
  ) | .metadata.name' | \
while read ns; do
  echo "Cleaning up old preview: $ns"
  kubectl delete ns "$ns" --ignore-not-found=true
done
```

## Troubleshooting

### Common Issues

#### 1. Deployment Failures
**Symptoms**: Deploy stage fails with Helm errors

**Diagnosis**:
```bash
# Check Helm deployment status
helm list -n impl-pr-123

# Check pod status
kubectl get pods -n impl-pr-123

# View deployment logs
kubectl logs -n impl-pr-123 deployment/myapp-pr-123
```

**Solutions**:
- Verify Helm chart syntax and values
- Check image availability and pull secrets
- Validate resource quotas and limits

#### 2. Acceptance Test Failures
**Symptoms**: Acceptance stage fails with threshold violations

**Diagnosis**:
```bash
# Check acceptance test logs
argo logs workflow-name -c acceptance

# View test responses
kubectl exec workflow-pod -- find /artifacts/acceptance/responses -name "*.txt"

# Check performance metrics
kubectl exec workflow-pod -- cat /artifacts/acceptance/report.json
```

**Solutions**:
- Adjust performance thresholds in workflow parameters
- Investigate application performance issues
- Check network connectivity to preview environment

### Debug Commands
```bash
# Monitor workflow execution
argo watch workflow-name --log

# Check resource usage
kubectl top pods -n impl-pr-123

# Validate Helm deployment
helm get values myapp-pr-123 -n impl-pr-123

# Test endpoints manually
curl -v https://pr-123.preview.example.com/
```

## Best Practices

### Resource Management
1. **Namespace Quotas**: Set appropriate resource quotas for preview environments
2. **Image Optimization**: Use minimal container images for faster deployment
3. **Cleanup Automation**: Implement TTL-based cleanup for unused environments
4. **Monitoring**: Monitor resource usage across preview environments

### Security
1. **Network Policies**: Implement network isolation between preview environments
2. **RBAC**: Use minimal permissions for deployment operations
3. **Secret Management**: Avoid hardcoded secrets in Helm values
4. **TLS**: Ensure all preview environments use TLS encryption

For additional support and advanced usage patterns, consult the main documentation.