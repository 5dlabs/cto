# Toolman Guide: Simplified CodeRun API with Auto-Detection

## Overview

This guide provides comprehensive instructions for using the simplified CodeRun/DocsRun API system. The tools enable easy creation of AI agent workflows with minimal configuration through intelligent auto-detection of GitHub event parameters.

## Available Tools

### 1. Argo Workflows API (`argo-workflows-api`)

**Purpose**: Direct interaction with Argo Workflows for template and workflow management.

**Base URL**: `https://argo-workflows.workflows.svc.cluster.local:2746`

#### Key Endpoints

##### List WorkflowTemplates
```http
GET /api/v1/workflow-templates/{namespace}
Authorization: Bearer {k8s_token}
```

**Usage Example**:
```bash
# List all templates in workflows namespace
kubectl proxy --port=8080 &
curl -H "Authorization: Bearer $(kubectl get secret -o jsonpath='{.items[0].data.token}' | base64 -d)" \
  http://localhost:8080/api/v1/workflow-templates/workflows
```

##### Submit Workflow from Template
```http
POST /api/v1/workflows/{namespace}
Content-Type: application/json
Authorization: Bearer {k8s_token}

{
  "workflow": {
    "metadata": {"generateName": "test-"},
    "spec": {
      "workflowTemplateRef": {"name": "coderun-template"},
      "arguments": {
        "parameters": [
          {"name": "github-app", "value": "clippy"},
          {"name": "event", "value": "{\"repository\":{\"owner\":{\"login\":\"myorg\"},\"name\":\"myrepo\"}}"}
        ]
      }
    }
  }
}
```

##### Get Workflow Status
```http
GET /api/v1/workflows/{namespace}/{name}
```

**Usage Example**:
```bash
# Get workflow status
argo get my-workflow-name

# Watch workflow progress
argo watch my-workflow-name
```

### 2. CodeRun Template API (`coderun-template-api`)

**Purpose**: Simplified interface for creating CodeRun workflows with minimal parameters.

#### Submit CodeRun Workflow

**Usage Examples**:

```bash
# Minimal usage - only github-app required
argo submit --from workflowtemplate/coderun-template -p github-app=clippy

# With GitHub event payload
argo submit --from workflowtemplate/coderun-template \
  -p github-app=rex \
  -p event='{"repository":{"owner":{"login":"myorg"},"name":"myrepo"},"pull_request":{"number":123,"head":{"ref":"feature-branch"}}}'

# With task reference
argo submit --from workflowtemplate/coderun-template \
  -p github-app=qa \
  -p taskRef="task/integration-tests"

# Complete example
argo submit --from workflowtemplate/coderun-template \
  -p github-app=security \
  -p event='{"repository":{"full_name":"org/repo"},"push":{"ref":"refs/heads/main"}}' \
  -p taskRef="security/scan-vulnerabilities" \
  --watch
```

**Supported GitHub Apps**:
- `rex` - Implementation and code generation
- `clippy` - Code formatting and linting
- `qa` - Quality assurance and testing
- `triage` - Issue triage and analysis
- `security` - Security scanning and analysis

### 3. DocsRun Template API (`docsrun-template-api`)

**Purpose**: Simplified interface for creating DocsRun workflows for documentation tasks.

**Usage Examples**:

```bash
# Generate documentation
argo submit --from workflowtemplate/docsrun-template -p github-app=clippy

# Documentation with specific event
argo submit --from workflowtemplate/docsrun-template \
  -p github-app=clippy \
  -p event='{"repository":{"owner":{"login":"docs-org"},"name":"api-docs"}}'

# Documentation deployment
argo submit --from workflowtemplate/docsrun-template \
  -p github-app=clippy \
  -p taskRef="docs/deploy-preview"
```

### 4. Kubernetes Custom Resource API (`kubernetes-cr-api`)

**Purpose**: Direct access to CodeRun and DocsRun Custom Resources created by templates.

#### Monitor CodeRun Resources

```bash
# List all CodeRuns
kubectl get coderuns -n workflows

# Get specific CodeRun details
kubectl get coderun my-coderun-xyz -o yaml

# Watch CodeRun status changes
kubectl get coderuns -w

# Check CodeRun logs (if available)
kubectl logs -l coderun=my-coderun-xyz
```

#### Monitor DocsRun Resources

```bash
# List all DocsRuns
kubectl get docsruns -n workflows

# Get DocsRun with output
kubectl get docsrun my-docsrun-abc -o yaml

# Monitor documentation builds
kubectl get docsruns -o custom-columns=NAME:.metadata.name,STATUS:.status.phase,CREATED:.metadata.creationTimestamp
```

### 5. Event Payload Processor (`event-payload-processor`)

**Purpose**: Utility service for processing and validating GitHub event payloads locally.

**Base URL**: `http://event-processor.workflows.svc.cluster.local:8080`

#### Process Event Payload

```http
POST /process-event
Content-Type: application/json

{
  "event": {
    "repository": {"owner": {"login": "myorg"}, "name": "myrepo"},
    "pull_request": {"number": 123, "head": {"ref": "feature"}}
  }
}
```

**Response**:
```json
{
  "repoFullName": "myorg/myrepo",
  "owner": "myorg",
  "repo": "myrepo", 
  "ref": "feature",
  "prNumber": "123",
  "issueNumber": "",
  "workflowRunId": "",
  "sha": "",
  "isPR": "1"
}
```

**Usage Example**:
```bash
# Process event payload
curl -X POST http://event-processor.workflows.svc.cluster.local:8080/process-event \
  -H "Content-Type: application/json" \
  -d @sample-pr-event.json
```

#### Validate Event Payload

```bash
# Validate event structure
curl -X POST http://event-processor.workflows.svc.cluster.local:8080/validate-event \
  -H "Content-Type: application/json" \
  -d @webhook-payload.json
```

## Local Development Tools

### Template Tester

**Purpose**: Test WorkflowTemplates locally with various scenarios.

**Command**: `./scripts/test-templates.sh --namespace workflows --templates coderun-template,docsrun-template`

**Usage**:
```bash
# Test all templates with default scenarios
./scripts/test-templates.sh

# Test specific template with custom event
./scripts/test-templates.sh --template coderun-template --event-file pr-event.json

# Dry run without actual submission
./scripts/test-templates.sh --dry-run --template docsrun-template
```

**Configuration** (`.test-templates.config`):
```bash
NAMESPACE=workflows
TEMPLATES="coderun-template,docsrun-template"
EVENT_DIR="./test-events"
TIMEOUT=300
```

### Event Simulator

**Purpose**: Generate realistic GitHub event payloads for testing.

**Command**: `./scripts/simulate-events.sh --output-dir ./test-events --format json`

**Usage**:
```bash
# Generate all event types
./scripts/simulate-events.sh

# Generate specific event type
./scripts/simulate-events.sh --type pull_request --output pr-events/

# Generate events for specific repo
./scripts/simulate-events.sh --repo myorg/myrepo --count 10
```

**Generated Event Types**:
- Pull request events (opened, synchronize, closed)
- Issue events (opened, closed, labeled)
- Push events (to various branches)
- Workflow run events (completed, failed)
- Security advisory events

### Batch Processor (Press-Play)

**Purpose**: Run multiple CodeRun tasks in parallel using the press-play orchestrator.

**Command**: `./scripts/run-press-play.sh --backlog-file ./config/test-backlog.json --dry-run`

**Backlog Configuration** (`./config/test-backlog.json`):
```json
[
  {
    "githubApp": "clippy",
    "taskRef": "lint/format-code"
  },
  {
    "githubApp": "qa", 
    "taskRef": "test/integration"
  },
  {
    "githubApp": "security",
    "taskRef": "scan/vulnerabilities"
  }
]
```

**Usage**:
```bash
# Run batch with default configuration
./scripts/run-press-play.sh

# Custom parallelism and event
./scripts/run-press-play.sh \
  --backlog-file ./custom-backlog.json \
  --parallelism 5 \
  --event-file pr-123-event.json

# Monitor batch execution
./scripts/run-press-play.sh --backlog-file ./large-backlog.json --watch
```

## Common Usage Patterns

### 1. Event-Driven Workflow Creation

**From Argo Events Sensor**:
```yaml
triggers:
  - template:
      name: automated-pr-validation
      k8s:
        operation: create
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: pr-validation-
            spec:
              workflowTemplateRef:
                name: coderun-template
              arguments:
                parameters:
                  - name: github-app
                    value: clippy
                  - name: event
                    value: "{{events.github.body}}"
```

### 2. Manual Workflow Submission

```bash
#!/bin/bash
# submit-coderun.sh
GITHUB_APP=${1:-clippy}
EVENT_FILE=${2:-""}
TASK_REF=${3:-""}

PARAMS="-p github-app=$GITHUB_APP"

if [ -n "$EVENT_FILE" ]; then
  EVENT_JSON=$(cat "$EVENT_FILE" | jq -c .)
  PARAMS="$PARAMS -p event='$EVENT_JSON'"
fi

if [ -n "$TASK_REF" ]; then
  PARAMS="$PARAMS -p taskRef='$TASK_REF'"
fi

echo "Submitting coderun-template with parameters: $PARAMS"
argo submit --from workflowtemplate/coderun-template $PARAMS --watch
```

### 3. Batch Processing with Press-Play

```bash
#!/bin/bash
# batch-process.sh
REPO_OWNER=${1:-myorg}
REPO_NAME=${2:-myrepo}
PR_NUMBER=${3:-}

# Create event payload
EVENT_JSON=$(jq -n \
  --arg owner "$REPO_OWNER" \
  --arg repo "$REPO_NAME" \
  --arg pr "$PR_NUMBER" \
  '{
    repository: {owner: {login: $owner}, name: $repo},
    pull_request: {number: ($pr | tonumber)}
  }')

# Create backlog for PR processing
BACKLOG='[
  {"githubApp":"clippy","taskRef":"format/prettier"},
  {"githubApp":"clippy","taskRef":"lint/eslint"},
  {"githubApp":"qa","taskRef":"test/unit"},
  {"githubApp":"qa","taskRef":"test/integration"}
]'

# Submit press-play workflow
argo submit press-play \
  -p backlog="$BACKLOG" \
  -p event="$EVENT_JSON" \
  -p parallelism=2 \
  --watch
```

### 4. Monitoring and Debugging

```bash
#!/bin/bash
# monitor-coderuns.sh

echo "Active CodeRuns:"
kubectl get coderuns -o custom-columns=NAME:.metadata.name,APP:.spec.github.appName,REPO:.spec.repo,STATUS:.status.phase

echo -e "\nRecent DocsRuns:"
kubectl get docsruns --sort-by=.metadata.creationTimestamp -o custom-columns=NAME:.metadata.name,REPO:.spec.repo,STATUS:.status.phase | tail -10

echo -e "\nWorkflow Status:"
argo list --running

# Check for failed workflows
FAILED_WORKFLOWS=$(argo list --status Failed -o name)
if [ -n "$FAILED_WORKFLOWS" ]; then
  echo -e "\nFailed Workflows:"
  for wf in $FAILED_WORKFLOWS; do
    echo "- $wf"
    argo logs "$wf" --tail 10
  done
fi
```

## Event Payload Examples

### Pull Request Event

```json
{
  "action": "opened",
  "repository": {
    "owner": {"login": "myorg", "type": "Organization"},
    "name": "myrepo",
    "full_name": "myorg/myrepo"
  },
  "pull_request": {
    "number": 123,
    "head": {
      "ref": "feature/new-api",
      "sha": "abc123def456"
    },
    "base": {
      "ref": "main"
    }
  }
}
```

### Issue Event

```json
{
  "action": "opened",
  "repository": {
    "owner": {"login": "myorg"},
    "name": "myrepo"
  },
  "issue": {
    "number": 456,
    "title": "Bug in authentication flow",
    "body": "Steps to reproduce:\n1. Login\n2. Navigate to profile"
  }
}
```

### Push Event

```json
{
  "ref": "refs/heads/main",
  "after": "def789abc123",
  "repository": {
    "owner": {"login": "myorg"},
    "name": "myrepo",
    "full_name": "myorg/myrepo"
  },
  "commits": [
    {
      "id": "def789abc123",
      "message": "Fix authentication bug"
    }
  ]
}
```

### Workflow Run Event

```json
{
  "action": "completed",
  "workflow_run": {
    "id": 789012345,
    "name": "CI",
    "head_branch": "main",
    "head_sha": "ghi456jkl789",
    "conclusion": "failure"
  },
  "repository": {
    "owner": {"login": "myorg"},
    "name": "myrepo"
  }
}
```

## Troubleshooting

### Common Issues

#### 1. Template Not Found

**Symptoms**:
- Error: "WorkflowTemplate 'coderun-template' not found"

**Diagnosis**:
```bash
# Check if templates are installed
argo template list

# Verify template in correct namespace
kubectl get workflowtemplate coderun-template -n workflows
```

**Solutions**:
- Deploy templates: `kubectl apply -f coderun-template.yaml`
- Check namespace: `argo submit --from workflowtemplate/coderun-template -n workflows`

#### 2. Auto-Detection Failures

**Symptoms**:
- Error: "Repository information not available"
- Empty or incorrect parameters in generated CRs

**Diagnosis**:
```bash
# Test event processing locally
echo '$EVENT_JSON' | jq -r '.repository.owner.login // "MISSING"'

# Check workflow logs for context step
argo logs workflow-name -c coderun-context
```

**Solutions**:
- Provide complete event payload with repository information
- Ensure repository exists and is accessible
- Check event payload format against GitHub webhook documentation

#### 3. GitHub App Validation Errors

**Symptoms**:
- Error: "Invalid github-app 'badapp'"

**Diagnosis**:
```bash
# Check available GitHub Apps
kubectl get secrets -n workflows -l app=github-app
```

**Solutions**:
- Use valid GitHub App names: rex, clippy, qa, triage, security
- Ensure GitHub App secrets are properly configured
- Check External Secrets sync status

#### 4. System Prompt Missing

**Symptoms**:
- Error: "System prompt not found"

**Diagnosis**:
```bash
# Check ConfigMap contents
kubectl get configmap controller-agents -n workflows -o yaml

# List available prompt files
kubectl exec deployment/some-pod -- ls -la /etc/agents/
```

**Solutions**:
- Create controller-agents ConfigMap with prompt files
- Ensure prompt files follow naming convention: `{app}_system-prompt.md`
- Add default_system-prompt.md as fallback

### Debug Commands

```bash
# Test template submission with debug output
argo submit --from workflowtemplate/coderun-template \
  -p github-app=clippy \
  -p event='{}' \
  --log

# Check template validation
argo template lint coderun-template.yaml

# Inspect generated CodeRun CR
kubectl get coderuns -o yaml | grep -A20 "spec:"

# Monitor workflow execution
argo watch workflow-name --log

# Check system resources
kubectl get all -n workflows -l app=coderun
```

### Performance Optimization

```bash
# Monitor resource usage
kubectl top pods -n workflows --sort-by=cpu

# Check workflow queue status
argo list --status Pending

# Optimize parallelism
argo submit press-play \
  -p backlog='[...]' \
  -p parallelism=10  # Adjust based on cluster capacity

# Monitor CustomResource creation rate
kubectl get events -n workflows --sort-by=.firstTimestamp | grep CodeRun
```

## Best Practices

### Template Usage

1. **Parameter Minimization**: Only provide github-app for simple cases
2. **Event Payload**: Pass complete GitHub webhook payloads when available
3. **Task References**: Use descriptive taskRef values for better tracking
4. **Namespace Management**: Keep all resources in dedicated workflows namespace

### Batch Processing

1. **Parallelism Tuning**: Start with low parallelism and increase based on cluster capacity
2. **Backlog Size**: Limit backlog items to prevent resource exhaustion
3. **Error Handling**: Monitor batch jobs and handle failures gracefully
4. **Resource Cleanup**: Ensure completed workflows are cleaned up regularly

### Monitoring

1. **Resource Monitoring**: Track CodeRun/DocsRun resource creation and completion
2. **Error Tracking**: Monitor workflow failures and auto-detection errors
3. **Performance Metrics**: Track template execution times and success rates
4. **Capacity Planning**: Monitor resource usage trends for scaling decisions

### Security

1. **Event Validation**: Always validate GitHub event payloads before processing
2. **Secret Management**: Use proper RBAC for GitHub App secret access
3. **Network Security**: Implement NetworkPolicies for pod-to-pod communication
4. **Audit Logging**: Enable audit logging for template submissions and CR operations

## Integration Examples

### With Argo Events

```yaml
# sensor.yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-pr-sensor
spec:
  dependencies:
    - name: github-pr
      eventSourceName: github
      eventName: pull_request
  triggers:
    - template:
        name: pr-validation-workflow
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              metadata:
                generateName: pr-validation-
              spec:
                workflowTemplateRef:
                  name: coderun-template
                arguments:
                  parameters:
                    - name: github-app
                      value: clippy
                    - name: event
                      value: "{{events.github-pr.body}}"
```

### With Custom Controllers

```go
// controller.go - Custom controller integration
func (r *IssueController) processIssue(issue *Issue) error {
    event := map[string]interface{}{
        "repository": map[string]interface{}{
            "owner": map[string]interface{}{"login": issue.RepoOwner},
            "name": issue.RepoName,
        },
        "issue": map[string]interface{}{
            "number": issue.Number,
        },
    }
    
    workflow := &argov1alpha1.Workflow{
        Spec: argov1alpha1.WorkflowSpec{
            WorkflowTemplateRef: &argov1alpha1.WorkflowTemplateRef{
                Name: "coderun-template",
            },
            Arguments: argov1alpha1.Arguments{
                Parameters: []argov1alpha1.Parameter{
                    {Name: "github-app", Value: argov1alpha1.AnyStringPtr("rex")},
                    {Name: "event", Value: argov1alpha1.AnyStringPtr(eventJSON)},
                },
            },
        },
    }
    
    return r.ArgoClient.ArgoprojV1alpha1().Workflows("workflows").Create(ctx, workflow, metav1.CreateOptions{})
}
```

For additional support and advanced usage patterns, consult the main documentation or reach out to the DevOps team.