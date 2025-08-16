# Task 3: Common Argo WorkflowTemplate Wrapper for CodeRun/DocsRun with Simplified API

## Overview

This task extends our existing WorkflowTemplates that create CodeRun/DocsRun Custom Resources, focusing on parameter simplification and auto-detection from event payloads. We already have coderun/docsrun templates installed by the controller chart; refine rather than reinvent.

Important:
- Do NOT re-implement functionality that already exists. Extend the existing templates under `infra/charts/controller/templates/`.
- Testing/verification via Argo tools (Argo CD and Argo Workflows), not Helm.
- Scope: Rust-only for this phase; multi-language support is out of scope.

## Architecture

The solution provides three main components:

1. **coderun-template (existing)**: Creates CodeRun CRs
2. **docsrun-template (existing)**: Creates DocsRun CRs for documentation tasks  
3. **press-play (optional)**: Orchestrator DAG can remain out-of-scope; focus on simplifying inputs to the existing templates

## Key Features

### Simplified API (incremental)
- Reduce required parameters where safe; keep compatibility with templates in `infra/charts/controller/templates/`
- Auto-detect owner/repo/ref/PR number from Argo Events payloads when present; otherwise use existing parameters

### Event Payload Processing
- **Pull Requests**: Extracts owner, repo, ref, PR number from pull_request events
- **Issues**: Processes issue events for owner, repo, issue number
- **Workflow Runs**: Handles workflow_run events with branch and commit info
- **Push Events**: Processes push events for branch and commit details
- **Fallback Logic**: Uses git commands in workspace when event data is unavailable

### Resource Management
- **Workspace Preparation**: Sets up `/work/src` workspace directory
- **MCP Requirements**: Mounts MCP tools configuration at `/work/requirements.yaml` (provided by project-level `docs/requirements.yaml`; per-task `@client-config.json` unification will be handled in a separate task)
- **System Prompts**: Resolves GitHub App-specific system prompts from ConfigMap
- **Token Integration**: Integrates with GitHub App token generator from Task 2

## Implementation Details

### coderun-template Structure

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: coderun-template
spec:
  entrypoint: coderun-main
  arguments:
    parameters:
      - name: github-app        # Required: rex, clippy, qa, triage, security
      - name: event            # Optional: JSON event payload
        value: "{}"
      - name: taskRef          # Optional: task reference
        value: ""
  templates:
    - name: coderun-main
      steps:
        - - name: context
            template: coderun-context
        - - name: validate-prompt
            template: validate-prompt
        - - name: create-coderun
            template: create-coderun
```

### Auto-Detection Logic

The system uses `jq` to parse event payloads and extract relevant information:

```bash
# Extract repository information
OWNER=$(jq -r '.repository.owner.login // .organization.login // empty' /tmp/event.json)
REPO=$(jq -r '.repository.name // empty' /tmp/event.json)
FULL=$(jq -r '.repository.full_name // empty' /tmp/event.json)

# Extract reference information
REF=$(jq -r '.pull_request.head.ref // .ref // .workflow_run.head_branch // empty' /tmp/event.json)

# Extract PR and issue numbers
PRNR=$(jq -r '.pull_request.number // empty' /tmp/event.json)
ISSNR=$(jq -r '.issue.number // empty' /tmp/event.json)

# Extract commit SHA
SHA=$(jq -r '.pull_request.head.sha // .after // .workflow_run.head_sha // empty' /tmp/event.json)

# Fallback to git commands if event data is missing
[ -z "$REF" ] && [ -d /work/src/.git ] && REF=$(cd /work/src && git rev-parse --abbrev-ref HEAD || true)
[ -z "$REF" ] && REF=main
```

### System Prompt Resolution (existing)

Prompts are resolved from the `controller-agents` ConfigMap rendered by Helm:

```bash
SYSTEM="/etc/agents/${GITHUB_APP}_system-prompt.md"
[ -f "$SYSTEM" ] || SYSTEM="/etc/agents/default_system-prompt.md"
[ -f "$SYSTEM" ] || { echo "missing prompt" >&2; exit 2; }
```

### Volume and Secret Mounting (existing)

```yaml
volumes:
  - name: workspace
    emptyDir: {}
  - name: controller-agents
    configMap:
      name: controller-agents
  - name: mcp-requirements
    configMap:
      name: mcp-requirements
      items:
        - key: requirements.yaml
          path: requirements.yaml
  - name: gh-token
    emptyDir: {}

volumeMounts:
  - name: workspace
    mountPath: /work/src
  - name: controller-agents
    mountPath: /etc/agents
    readOnly: true
  - name: mcp-requirements
    mountPath: /work
    subPath: requirements.yaml
    readOnly: true
  - name: gh-token
    mountPath: /var/run/github
```

## CodeRun CR Generation (unchanged)

```yaml
apiVersion: taskmaster.io/v1
kind: CodeRun
metadata:
  generateName: coderun-
spec:
  repo: "{{steps.context.outputs.parameters.repoFullName}}"
  ref: "{{steps.context.outputs.parameters.ref}}"
  prompts:
    system: "{{steps.validate-prompt.outputs.parameters.systemPromptPath}}"
    user: "task/prompt.md"
  mcpRequirementsFile: "/work/requirements.yaml"
  github:
    appName: "{{inputs.parameters.github-app}}"
    tokenFile: "/var/run/github/token"
  workspace:
    path: "/work/src"
  taskRef: "{{inputs.parameters.taskRef}}"
```

## docsrun-template (unchanged)

```yaml
apiVersion: taskmaster.io/v1
kind: DocsRun
metadata:
  generateName: docsrun-
spec:
  repo: "{{steps.context.outputs.parameters.repoFullName}}"
  ref: "{{steps.context.outputs.parameters.ref}}"
  prompts:
    system: "{{steps.validate-prompt.outputs.parameters.systemPromptPath}}"
    user: "docs/task.md"
  github:
    appName: "{{inputs.parameters.github-app}}"
    tokenFile: "/var/run/github/token"
  workspace:
    path: "/work/src"
  taskRef: "{{inputs.parameters.taskRef}}"
  docs:
    path: "docs/"
    action: "build-preview"
```

## press-play Orchestrator

The press-play workflow enables batch processing of multiple tasks:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: press-play
spec:
  arguments:
    parameters:
      - name: backlog
        value: '[{"githubApp":"clippy","taskRef":"task/format"}]'
      - name: parallelism
        value: "3"
      - name: event
        value: "{}"
  parallelism: "{{workflow.parameters.parallelism}}"
  templates:
    - name: run-backlog
      inputs:
        parameters:
          - name: backlog
      steps:
        - - name: process-item
            template: submit-coderun
            arguments:
              parameters:
                - name: github-app
                  value: "{{item.githubApp}}"
                - name: taskRef
                  value: "{{item.taskRef}}"
            withParam: "{{inputs.parameters.backlog}}"
```

## Usage Patterns

### Single Task Execution

```bash
# Execute clippy on a specific PR
argo submit --from workflowtemplate/coderun-template \
  -p github-app=clippy \
  -p event='{"repository":{"owner":{"login":"myorg"},"name":"myrepo"},"pull_request":{"number":123,"head":{"ref":"feature-branch"}}}'

# Execute with minimal parameters (auto-detection)
argo submit --from workflowtemplate/coderun-template \
  -p github-app=rex
```

### Batch Execution

```bash
# Execute multiple tasks in parallel
argo submit press-play \
  -p backlog='[
    {"githubApp":"clippy","taskRef":"task/format"},
    {"githubApp":"qa","taskRef":"task/verify"},
    {"githubApp":"security","taskRef":"task/scan"}
  ]' \
  -p parallelism=2
```

### Documentation Tasks

```bash
# Generate documentation
argo submit --from workflowtemplate/docsrun-template \
  -p github-app=clippy \
  -p taskRef="docs/api-guide"
```

## Error Handling

### Validation Gates

1. **Context Validation**: Ensures repository information is available
2. **Prompt Validation**: Verifies system prompt file exists
3. **Parameter Validation**: Validates required parameters are provided

### Failure Modes

- **Missing Repository Info**: Fails with clear error when repo cannot be determined
- **Invalid GitHub App**: Fails when unsupported github-app value is provided
- **Missing System Prompt**: Fails when prompt file is not found in ConfigMap
- **Event Parse Errors**: Handles malformed JSON gracefully with defaults

### Error Messages

```bash
# Missing repository information
"Error: Repository information not available from event payload and no git context found"

# Invalid GitHub App
"Error: Invalid github-app 'invalid-app'. Must be one of: rex, clippy, qa, triage, security"

# Missing system prompt
"Error: System prompt not found at /etc/agents/rex_system-prompt.md or /etc/agents/default_system-prompt.md"
```

## Integration Points

### Argo Events Integration

The templates are designed to work seamlessly with Argo Events:

```yaml
triggers:
  - template:
      name: coderun-pr-validation
      k8s:
        group: argoproj.io
        version: v1alpha1
        resource: workflows
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
                    value: clippy
                  - name: event
                    value: "{{events.github.body}}"
```

### GitHub App Token Integration

Integrates with the token generator from Task 2:

```yaml
initContainers:
  - name: gh-token
    image: ghcr.io/myorg/ghapp-token-gen:latest
    env:
      - name: APP_ID
        valueFrom:
          secretKeyRef:
            name: github-app-{{inputs.parameters.github-app}}
            key: appId
      - name: PRIVATE_KEY
        valueFrom:
          secretKeyRef:
            name: github-app-{{inputs.parameters.github-app}}
            key: privateKey
    volumeMounts:
      - name: gh-token
        mountPath: /var/run/github
```

## Performance Considerations

### Resource Optimization
- **Minimal Containers**: Uses lightweight alpine/git images for processing
- **Shared Volumes**: Reuses volumes across workflow steps
- **Efficient Parsing**: Uses jq for fast JSON processing
- **Cached Images**: Pulls container images only once per node

### Concurrency Controls
- **Parallelism Limits**: Configurable parallelism at workflow level  
- **Semaphore Integration**: Supports Argo Workflows semaphores for resource limiting
- **Rate Limiting**: Compatible with Argo Events rate limiting

### Scalability Features
- **Stateless Design**: Templates are completely stateless
- **Resource Templates**: Uses Kubernetes resource templates for efficient CR creation
- **Event-Driven**: Scales automatically with incoming events

## Security Considerations

### Secret Management
- **Token Integration**: Secure integration with GitHub App token generator
- **No PATs**: Does not require or use Personal Access Tokens
- **Least Privilege**: Minimal RBAC permissions for workflow execution
- **Secret Isolation**: Secrets are isolated per GitHub App

### Input Validation
- **Parameter Sanitization**: Validates and sanitizes all input parameters
- **Event Payload Validation**: Safely parses potentially malformed event payloads
- **Path Validation**: Prevents path traversal attacks in file operations

### Network Security
- **Internal Communication**: All communication stays within Kubernetes cluster
- **TLS Encryption**: External communication uses TLS encryption
- **Network Policies**: Compatible with Kubernetes NetworkPolicies

## Monitoring and Observability

### Key Metrics
- **Template Invocation Rate**: Number of template executions per time period
- **Success/Failure Rates**: Success and failure rates by GitHub App
- **Event Processing Time**: Time from event receipt to CodeRun CR creation
- **Auto-Detection Accuracy**: Rate of successful parameter auto-detection

### Logging
- **Structured Logs**: JSON-formatted logs with correlation IDs
- **Event Correlation**: Links between events and generated CRs
- **Parameter Resolution**: Logs showing auto-detected vs provided parameters
- **Error Context**: Detailed error context for troubleshooting

### Dashboards
- **Workflow Execution Dashboard**: Shows active workflows and success rates
- **Event Processing Dashboard**: Tracks event-to-workflow conversion
- **Resource Utilization**: Monitors compute and storage usage
- **Error Analysis**: Categorizes and tracks error patterns

## Troubleshooting Guide

### Common Issues

**Template not found:**
```bash
# Verify template installation
argo template list | grep coderun-template
```

**Event parsing failures:**
```bash
# Test event parsing locally
echo '$EVENT_JSON' | jq -r '.repository.owner.login // empty'
```

**Missing system prompts:**
```bash
# Check ConfigMap contents
kubectl get configmap controller-agents -o yaml
```

**Token generation failures:**
```bash
# Check GitHub App secrets
kubectl get secrets -l app=github-app
```

### Debug Commands

```bash
# Submit with debug output
argo submit --from workflowtemplate/coderun-template \
  -p github-app=rex -p event='{}' --watch

# Check workflow logs  
argo logs workflow-name

# Inspect generated CodeRun CR
kubectl get coderuns -o yaml

# Validate event parsing
kubectl run debug --rm -it --image=alpine/git:latest -- \
  sh -c 'apk add jq && echo "$EVENT" | jq .'
```

## Future Enhancements

### Planned Features
1. **Enhanced Auto-Detection**: More sophisticated event payload parsing
2. **Template Composition**: Ability to compose multiple templates
3. **Conditional Execution**: Skip steps based on event content
4. **Advanced Caching**: Cache frequently used data across executions

### Extensibility Points
1. **Custom Event Processors**: Plugin architecture for custom event types
2. **Parameter Transformers**: Custom logic for parameter transformation
3. **Validation Hooks**: Custom validation logic for specific use cases
4. **Output Processors**: Custom processing of CodeRun CR outputs

## Dependencies

### External Dependencies
- Argo Workflows 3.4+
- Kubernetes 1.20+
- External Secrets Operator (from Task 2)
- GitHub App token generator (from Task 2)

### Internal Dependencies
- controller-agents ConfigMap with system prompts
- mcp-requirements ConfigMap with MCP tool configuration
- GitHub App secrets (managed by External Secrets)
- Appropriate RBAC permissions

## References

- [Argo Workflows Templates](https://argoproj.github.io/argo-workflows/workflow-templates/)
- [Kubernetes Custom Resources](https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/)
- [jq Manual](https://stedolan.github.io/jq/manual/)
- [GitHub Webhooks](https://docs.github.com/en/developers/webhooks-and-events/webhooks/webhook-events-and-payloads)