# Toolman Guide: Argo Events GitHub Integration

## Overview
Guide for managing GitHub webhook integration with Argo Events, including EventSource and Sensor configuration for automated workflow triggering.

## Available Tools

### 1. Argo Events Management
**Purpose**: Manage EventSources and Sensors for GitHub integration

#### Check EventSource Status
```bash
# List all EventSources
kubectl get eventsources -n argo-events

# Check GitHub EventSource details
kubectl describe eventsource github-es -n argo-events

# View EventSource logs
kubectl logs -n argo-events -l eventsource-name=github-es
```

#### Monitor Sensors
```bash
# List all Sensors
kubectl get sensors -n argo-events

# Check specific sensor status
kubectl describe sensor github-pr-to-pr-validation -n argo-events

# View sensor logs
kubectl logs -n argo-events -l sensor-name=github-pr-to-pr-validation
```

### 2. GitHub Webhook Testing
**Purpose**: Test webhook delivery and event processing

#### Test Webhook Endpoint
```bash
# Test basic connectivity
curl -X POST https://events.example.com/events \
  -H "X-GitHub-Event: ping" \
  -H "X-Hub-Signature-256: sha256=test" \
  -d '{"zen":"test"}'

# Test PR event
curl -X POST https://events.example.com/events \
  -H "X-GitHub-Event: pull_request" \
  -H "X-Hub-Signature-256: sha256=..." \
  -d @test-events/pr-opened.json
```

## Local Development Tools

### Webhook Tester
```bash
# Test all event types
./scripts/test-webhooks.sh --endpoint https://events.example.com/events --events all

# Test specific event type
./scripts/test-webhooks.sh --event pull_request --payload pr-test.json
```

### Event Simulator
```bash
# Generate test events
./scripts/simulate-github-events.sh \
  --output-dir ./test-events \
  --types pr,issues,comments

# Replay events to webhook endpoint
./scripts/replay-events.sh --dir ./test-events --endpoint https://events.example.com/events
```

## Configuration Examples

### EventSource Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: github-es
spec:
  github:
    main:
      repositories:
        - owner: "*"
          repository: "*"
      webhook:
        endpoint: /events
        port: 12000
        secret:
          name: github-webhook-secret
          key: secret
      events: ["pull_request", "issues", "issue_comment"]
```

### Sensor Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-pr-sensor
spec:
  dependencies:
    - name: pr
      eventSourceName: github-es
      eventName: main
      filters:
        data:
          - path: headers.X-GitHub-Event
            value: ["pull_request"]
  triggers:
    - template:
        name: pr-validation
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              spec:
                workflowTemplateRef:
                  name: pr-validation
                arguments:
                  parameters:
                    - name: event
                      value: "{{events.github-es.main.body}}"
```

## Troubleshooting

### Common Issues

#### Webhooks Not Received
- Check EventSource pod status and logs
- Verify webhook URL accessibility from GitHub
- Validate network policies and ingress configuration

#### Events Not Triggering Workflows
- Check Sensor filters and CEL expressions
- Verify WorkflowTemplate references exist
- Review RBAC permissions for workflow creation

#### Rate Limiting Issues
- Monitor rate limit configurations in sensors
- Check workflow semaphore limits in ConfigMap
- Adjust requestsPerUnit based on traffic patterns

### Debug Commands
```bash
# Check EventSource connectivity
kubectl port-forward -n argo-events svc/github-es-eventsource-svc 12000:12000

# Monitor event processing
kubectl logs -f -n argo-events -l eventsource-name=github-es

# Check workflow creation
kubectl get workflows -w -n argo

# Validate webhook secrets
kubectl get secret github-webhook-secret -n argo-events -o yaml
```

## Best Practices

### Security
1. **Webhook Validation**: Always validate webhook signatures
2. **RBAC**: Use minimal permissions for service accounts
3. **Secret Management**: Use External Secrets for webhook secrets
4. **Network Policies**: Restrict network access appropriately

### Performance
1. **Rate Limiting**: Configure appropriate rate limits per sensor
2. **Filtering**: Use efficient filters to reduce processing overhead
3. **Semaphores**: Prevent resource exhaustion with workflow semaphores
4. **Monitoring**: Monitor event processing metrics and latency

For additional support, consult the main Argo Events documentation.