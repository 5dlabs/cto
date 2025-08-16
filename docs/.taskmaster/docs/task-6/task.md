# Task 6: Argo Events GitHub EventSource and Sensors

## Overview
This task implements a comprehensive GitHub webhook integration using Argo Events, providing EventSource and Sensor configurations that automatically trigger appropriate workflows based on GitHub repository events. The system maps GitHub events to specific workflows with proper parameter extraction and rate limiting.

## Architecture
The system consists of three main components:
1. **GitHub EventSource**: Receives and validates GitHub webhooks
2. **Event Sensors**: Process events and trigger appropriate workflows  
3. **Parameter Mapping**: Extracts context from event payloads for workflow parameters

## Key Features
- **Multi-Event Support**: Handles pull requests, issues, comments, pushes, workflow runs, and security events
- **Intelligent Routing**: Routes events to appropriate workflows based on event type and content
- **Rate Limiting**: Prevents event storms through configurable rate limits
- **Parameter Extraction**: Automatically extracts repo, PR, branch, and other context from events
- **Security Validation**: Validates webhook signatures to prevent unauthorized events

## Implementation Details

### GitHub EventSource Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: github-es
  namespace: argo-events
spec:
  github:
    main:
      repositories:
        - owner: "*"
          repository: "*"
      webhook:
        endpoint: /events
        port: 12000
        method: POST
        secret:
          name: github-webhook-secret
          key: secret
      events:
        - pull_request
        - issues
        - issue_comment
        - pull_request_review_comment
        - push
        - workflow_run
        - check_run
        - security_advisory
```

### Event-to-Workflow Mapping

#### PR Events → PR Validation
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-pr-to-pr-validation
spec:
  dependencies:
    - name: pr
      eventSourceName: github-es
      eventName: main
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["pull_request"]
          - path: body.action
            type: string
            value: ["opened", "reopened", "synchronize", "ready_for_review"]
        exprs:
          - expression: "body.pull_request.draft == false"
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

#### Comments → Rex Agent
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-comments-to-rex
spec:
  dependencies:
    - name: comment
      eventSourceName: github-es
      eventName: main
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["issue_comment", "pull_request_review_comment"]
          - path: body.action
            type: string
            value: ["created"]
        exprs:
          - expression: "body.comment.user.type != 'Bot'"
  triggers:
    - template:
        name: coderun-rex
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              spec:
                workflowTemplateRef:
                  name: coderun-template
                arguments:
                  parameters:
                    - name: github-app
                      value: rex
                    - name: event
                      value: "{{events.github-es.main.body}}"
```

#### CI Failures → Triage Agent
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-failures-to-triage
spec:
  dependencies:
    - name: failure
      eventSourceName: github-es
      eventName: main
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["workflow_run", "check_run"]
          - path: body.action
            type: string
            value: ["completed"]
        exprs:
          - expression: "body.workflow_run.conclusion in ['failure','timed_out'] || body.check_run.conclusion in ['failure','timed_out']"
  triggers:
    - template:
        name: coderun-triage
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              spec:
                workflowTemplateRef:
                  name: coderun-template
                arguments:
                  parameters:
                    - name: github-app
                      value: triage
                    - name: event
                      value: "{{events.github-es.main.body}}"
```

#### Issues → Implementation Flow
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-issues-to-implementation
spec:
  dependencies:
    - name: issue
      eventSourceName: github-es
      eventName: main
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["issues"]
          - path: body.action
            type: string
            value: ["opened"]
  triggers:
    - template:
        name: implementation
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              spec:
                workflowTemplateRef:
                  name: implementation-dag
                arguments:
                  parameters:
                    - name: event
                      value: "{{events.github-es.main.body}}"
```

#### Security Events → Security Agent
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-security-to-agent
spec:
  dependencies:
    - name: security
      eventSourceName: github-es
      eventName: main
      filters:
        data:
          - path: headers.X-GitHub-Event
            type: string
            value: ["security_advisory"]
        exprs:
          - expression: "body.action in ['published','updated']"
  triggers:
    - template:
        name: coderun-security
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              spec:
                workflowTemplateRef:
                  name: coderun-template
                arguments:
                  parameters:
                    - name: github-app
                      value: security
                    - name: event
                      value: "{{events.github-es.main.body}}"
```

## Rate Limiting and Concurrency Control

### Rate Limiting Configuration
```yaml
policy:
  rateLimit:
    unit: minute
    requestsPerUnit: 30
synchronization:
  semaphore:
    configMapKeyRef:
      name: workflow-semaphores
      key: coderun
```

### Semaphore Configuration
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-semaphores
  namespace: argo
data:
  pr-validation: "5"
  coderun: "10" 
  implementation: "5"
  orchestrator: "2"
```

## Parameter Contract
All sensors follow a consistent parameter contract:

### Standard Parameters
- **event**: Full raw event payload as JSON string
- **owner**: Repository owner login (.repository.owner.login)
- **repo**: Repository name (.repository.name)
- **pr**: Pull request number (.pull_request.number)
- **issue**: Issue number (.issue.number)
- **branch**: Branch reference (.pull_request.head.ref or .ref)
- **agent**: GitHub App to use (rex, clippy, qa, triage, security)

### Event-Specific Parameters
- **includeComments**: Boolean for comment-driven workflows
- **commentBody**: Original comment content
- **workflowRunId**: Workflow run ID for CI events
- **commitSha**: Commit SHA from various event types

## Security Implementation

### Webhook Secret Management
```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-webhook-secret-es
  namespace: argo-events
spec:
  secretStoreRef:
    name: aws-secrets
    kind: ClusterSecretStore
  target:
    name: github-webhook-secret
  data:
    - secretKey: secret
      remoteRef:
        key: github/webhook-secret
```

### RBAC Configuration
```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: argo-events-workflow-submit-role
  namespace: argo
rules:
  - apiGroups: ["argoproj.io"]
    resources: ["workflows"]
    verbs: ["create", "get", "list"]
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get"]
```

## Monitoring and Observability

### Key Metrics
- Event processing rate by type
- Workflow trigger success/failure rates
- Rate limiting activations
- Event processing latency
- Webhook signature validation failures

### Logging
- All events logged with correlation IDs
- Successful workflow triggers logged
- Rate limiting events logged
- Security validation failures logged

## Integration Testing

### Webhook Testing
```bash
# Test webhook endpoint
curl -X POST https://events.example.com/events \
  -H "X-GitHub-Event: ping" \
  -H "X-Hub-Signature-256: sha256=..." \
  -d '{"zen": "test"}'

# Test PR event
curl -X POST https://events.example.com/events \
  -H "X-GitHub-Event: pull_request" \
  -H "X-Hub-Signature-256: sha256=..." \
  -d @pr-opened-event.json
```

### Event Validation
```bash
# Check EventSource status
kubectl get eventsource github-es -n argo-events

# Check Sensor status
kubectl get sensors -n argo-events

# View event logs
kubectl logs -n argo-events deployment/eventbus-controller
```

## Troubleshooting

### Common Issues

#### EventSource Not Receiving Events
- Verify webhook URL accessibility from GitHub
- Check webhook secret configuration
- Validate network policies and ingress

#### Events Not Triggering Workflows
- Check Sensor filters and expressions
- Verify WorkflowTemplate references
- Review RBAC permissions

#### Rate Limiting Issues
- Monitor rate limit configurations
- Adjust requestsPerUnit as needed
- Check semaphore configurations

### Debug Commands
```bash
# Check event processing
kubectl logs -n argo-events -l eventsource-name=github-es

# Monitor workflow creation
kubectl get workflows -w

# Check sensor status
kubectl describe sensor github-pr-to-pr-validation -n argo-events
```

## Dependencies
- Argo Events 1.9+
- External Secrets Operator
- GitHub webhook configuration
- Network ingress for webhook endpoint
- RBAC permissions for workflow creation

## References
- [Argo Events GitHub EventSource](https://argoproj.github.io/argo-events/eventsources/github/)
- [GitHub Webhook Events](https://docs.github.com/en/developers/webhooks-and-events/webhooks/webhook-events-and-payloads)
- [Argo Events Sensors](https://argoproj.github.io/argo-events/sensors/)