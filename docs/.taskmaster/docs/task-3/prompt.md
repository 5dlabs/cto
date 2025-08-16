# Autonomous Implementation Prompt: Simplified CodeRun API with Auto-Detection

## Mission Statement

You are implementing a simplified API wrapper for CodeRun and DocsRun Custom Resources that reduces required parameters from many to just 1-2, while automatically detecting all other parameters from event payloads. Your goal is to create production-ready WorkflowTemplates that make it trivial to invoke AI agents with minimal configuration.

## Context

The current system requires numerous parameters to create CodeRun/DocsRun CRs, making it complex for users and event-driven automation. This task simplifies the API to require only a `github-app` parameter, with everything else auto-detected from GitHub event payloads or sensible defaults.

## Technical Requirements

### Must Implement

1. **coderun-template WorkflowTemplate**
   - Single required parameter: `github-app` (values: rex, clippy, qa, triage, security)
   - Optional parameters: `event` (JSON, default "{}"), `taskRef` (string, default "")
   - Auto-detect: repo, owner, ref, prNumber, issueNumber, workflowRunId, sha
   - Create CodeRun CR with all auto-detected parameters

2. **Event Payload Auto-Detection**
   - Parse GitHub webhook payloads (pull_request, issues, workflow_run, push, etc.)
   - Extract repository information, PR/issue numbers, commit details
   - Fall back to git commands in workspace when event data missing
   - Provide sensible defaults (ref=main) when nothing available

3. **docsrun-template WorkflowTemplate** 
   - Same API as coderun-template but creates DocsRun CRs
   - Tailored for documentation and deployment tasks
   - Uses "docs/task.md" as user prompt path

4. **press-play Orchestrator Workflow**
   - Accept backlog array of items with githubApp and taskRef
   - Fan out to multiple coderun-template invocations
   - Configurable parallelism limits
   - Concurrency control via semaphores

5. **Infrastructure Integration**
   - Mount controller-agents ConfigMap for system prompts
   - Mount mcp-requirements ConfigMap for MCP tools
   - Integrate with GitHub App token generator (Task 2)
   - Proper workspace and volume setup

### Auto-Detection Schema

**Event Payload Processing (jq patterns):**
```bash
OWNER=$(jq -r '.repository.owner.login // .organization.login // empty')
REPO=$(jq -r '.repository.name // empty')
REF=$(jq -r '.pull_request.head.ref // .ref // .workflow_run.head_branch // empty')
PRNR=$(jq -r '.pull_request.number // empty')
ISSNR=$(jq -r '.issue.number // empty')
SHA=$(jq -r '.pull_request.head.sha // .after // .workflow_run.head_sha // empty')
```

**Fallback Logic:**
```bash
# Git fallback when event data missing
[ -z "$REF" ] && [ -d /work/src/.git ] && REF=$(cd /work/src && git rev-parse --abbrev-ref HEAD || true)
[ -z "$FULL" ] && [ -n "$OWNER" ] && [ -n "$REPO" ] && FULL="$OWNER/$REPO"
[ -z "$REF" ] && REF=main
```

### Template Structure Requirements

**coderun-template.yaml:**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: coderun-template
spec:
  entrypoint: coderun-main
  arguments:
    parameters:
      - name: github-app
      - name: event
        value: "{}"
      - name: taskRef
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

**System Prompt Resolution:**
```bash
SYSTEM="/etc/agents/${GITHUB_APP}_system-prompt.md"
[ -f "$SYSTEM" ] || SYSTEM="/etc/agents/default_system-prompt.md" 
[ -f "$SYSTEM" ] || { echo "missing prompt" >&2; exit 2; }
```

**CodeRun CR Manifest:**
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

## Implementation Approach

### Phase 1: Context Processing Template

Create `coderun-context` template that:

```yaml
- name: coderun-context
  inputs:
    parameters:
      - name: event
  script:
    image: alpine:3.19
    command: [sh]
    source: |
      set -euo pipefail
      apk add --no-cache jq git
      
      echo '{{inputs.parameters.event}}' > /tmp/event.json
      
      # Extract fields with jq
      OWNER=$(jq -r '.repository.owner.login // .organization.login // empty' /tmp/event.json)
      REPO=$(jq -r '.repository.name // empty' /tmp/event.json)
      FULL=$(jq -r '.repository.full_name // empty' /tmp/event.json)
      REF=$(jq -r '.pull_request.head.ref // .ref // .workflow_run.head_branch // empty' /tmp/event.json)
      PRNR=$(jq -r '.pull_request.number // empty' /tmp/event.json)
      ISSNR=$(jq -r '.issue.number // empty' /tmp/event.json)
      WRID=$(jq -r '.workflow_run.id // empty' /tmp/event.json)
      SHA=$(jq -r '.pull_request.head.sha // .after // .workflow_run.head_sha // empty' /tmp/event.json)
      
      # Git fallbacks
      [ -z "$REF" ] && [ -d /work/src/.git ] && REF=$(cd /work/src && git rev-parse --abbrev-ref HEAD || true)
      [ -z "$FULL" ] && [ -n "$OWNER" ] && [ -n "$REPO" ] && FULL="$OWNER/$REPO"
      [ -z "$REF" ] && REF=main
      
      # Validate required fields
      [ -z "$FULL" ] && { echo "Error: Repository information not available"; exit 78; }
      
      # Output parameters
      echo -n "$FULL" > /tmp/repoFullName
      echo -n "$OWNER" > /tmp/owner  
      echo -n "$REPO" > /tmp/repo
      echo -n "$REF" > /tmp/ref
      echo -n "$PRNR" > /tmp/prNumber
      echo -n "$ISSNR" > /tmp/issueNumber
      echo -n "$WRID" > /tmp/workflowRunId
      echo -n "$SHA" > /tmp/sha
      echo -n "$([ -n "$PRNR" ] && echo 1 || echo 0)" > /tmp/isPR
      
      # Debug output (no secrets)
      echo "Resolved context: owner=$OWNER repo=$REPO ref=$REF pr=$PRNR issue=$ISSNR sha=${SHA:0:8}"
  outputs:
    parameters:
      - name: repoFullName
        valueFrom: {path: /tmp/repoFullName}
      - name: owner
        valueFrom: {path: /tmp/owner}
      - name: repo
        valueFrom: {path: /tmp/repo}
      - name: ref
        valueFrom: {path: /tmp/ref}
      - name: prNumber
        valueFrom: {path: /tmp/prNumber}
      - name: issueNumber
        valueFrom: {path: /tmp/issueNumber}
      - name: workflowRunId
        valueFrom: {path: /tmp/workflowRunId}
      - name: sha
        valueFrom: {path: /tmp/sha}
      - name: isPR
        valueFrom: {path: /tmp/isPR}
```

### Phase 2: System Prompt Validation

Create `validate-prompt` template:

```yaml
- name: validate-prompt
  inputs:
    parameters:
      - name: github-app
  script:
    image: alpine:3.19
    command: [sh]
    source: |
      set -euo pipefail
      GITHUB_APP={{inputs.parameters.github-app}}
      
      # Validate github-app parameter
      case "$GITHUB_APP" in
        rex|clippy|qa|triage|security) ;;
        *) echo "Error: Invalid github-app '$GITHUB_APP'. Must be one of: rex, clippy, qa, triage, security"; exit 1 ;;
      esac
      
      # Find system prompt
      SYSTEM="/etc/agents/${GITHUB_APP}_system-prompt.md"
      [ -f "$SYSTEM" ] || SYSTEM="/etc/agents/default_system-prompt.md"
      [ -f "$SYSTEM" ] || { echo "Error: System prompt not found at $SYSTEM or /etc/agents/default_system-prompt.md"; exit 2; }
      
      echo -n "$SYSTEM" > /tmp/system.txt
    volumeMounts:
      - name: controller-agents
        mountPath: /etc/agents
        readOnly: true
  outputs:
    parameters:
      - name: systemPromptPath
        valueFrom: {path: /tmp/system.txt}
```

### Phase 3: Workspace and Volume Setup

**Volume Configuration:**
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

### Phase 4: GitHub App Token Integration

**InitContainer Pattern:**
```yaml
initContainers:
  - name: gh-token
    image: ghcr.io/YOUR_ORG/ghapp-token-gen:latest
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
      - name: OUTPUT_PATH
        value: /var/run/github/token
    volumeMounts:
      - name: gh-token
        mountPath: /var/run/github
    securityContext:
      runAsNonRoot: true
      runAsUser: 65532
```

### Phase 5: DocsRun Template Implementation

Create `docsrun-template` with same structure but different CR:

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

### Phase 6: Press-Play Orchestrator

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: press-play
spec:
  arguments:
    parameters:
      - name: backlog
        value: '[]'
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
            templateRef:
              name: coderun-template
              template: coderun-main
            arguments:
              parameters:
                - name: github-app
                  value: "{{item.githubApp}}"
                - name: taskRef
                  value: "{{item.taskRef}}"
                - name: event
                  value: "{{workflow.parameters.event}}"
            withParam: "{{inputs.parameters.backlog}}"
```

## Error Handling Requirements

### Validation Gates
1. **Repository Validation**: Fail fast when repo info unavailable
2. **GitHub App Validation**: Reject invalid github-app values  
3. **System Prompt Validation**: Ensure prompt file exists
4. **Event Parsing**: Handle malformed JSON gracefully

### Error Messages
```bash
# Repository validation
"Error: Repository information not available from event payload and no git context found"

# GitHub App validation  
"Error: Invalid github-app 'badapp'. Must be one of: rex, clippy, qa, triage, security"

# System prompt validation
"Error: System prompt not found at /etc/agents/rex_system-prompt.md or /etc/agents/default_system-prompt.md"

# Event parsing
"Warning: Malformed event JSON, using defaults"
```

### Robust Defaults
- `ref` defaults to "main" when not determinable
- Empty strings for optional fields when not available
- Clear logging of resolved vs default values

## Testing Requirements

### Unit Tests
1. **Event Parsing**: Test all GitHub event types (PR, issues, push, workflow_run)
2. **Auto-Detection**: Verify correct field extraction from various payloads
3. **Fallback Logic**: Test git-based fallbacks and defaults
4. **Validation**: Test invalid inputs and error cases

### Integration Tests  
1. **Template Execution**: Submit templates with various event payloads
2. **CR Creation**: Verify CodeRun/DocsRun CRs are created correctly
3. **End-to-End**: Test complete workflow from event to CR creation
4. **Press-Play**: Test batch execution with multiple items

### Test Data Examples

**Pull Request Event:**
```json
{
  "repository": {
    "owner": {"login": "myorg"},
    "name": "myrepo", 
    "full_name": "myorg/myrepo"
  },
  "pull_request": {
    "number": 123,
    "head": {"ref": "feature-branch", "sha": "abc123"}
  }
}
```

**Issue Event:**
```json
{
  "repository": {
    "owner": {"login": "myorg"},
    "name": "myrepo"
  },
  "issue": {"number": 456}
}
```

### Validation Commands

```bash
# Test coderun-template
argo submit --from workflowtemplate/coderun-template \
  -p github-app=clippy \
  -p event='{"repository":{"owner":{"login":"test"},"name":"repo"},"pull_request":{"number":1,"head":{"ref":"main"}}}' \
  --watch

# Test docsrun-template
argo submit --from workflowtemplate/docsrun-template \
  -p github-app=clippy --watch

# Test press-play
argo submit press-play \
  -p backlog='[{"githubApp":"clippy","taskRef":"task/format"}]' \
  -p parallelism=1 --watch

# Validate templates
argo template lint coderun-template.yaml
argo template lint docsrun-template.yaml
```

## Performance Requirements

### Resource Optimization
- Use minimal container images (alpine:3.19)
- Efficient jq parsing of JSON payloads
- Shared volumes to minimize I/O
- Fast startup times (<5 seconds per step)

### Scalability Targets
- Handle 100+ concurrent template invocations
- Support event payloads up to 1MB
- Process 1000+ backlog items in press-play
- Sub-second auto-detection processing

## Security Requirements

### Input Sanitization
- Validate all event payload fields
- Prevent shell injection in jq processing
- Sanitize file paths and parameters
- Validate GitHub App parameter against whitelist

### Secret Handling
- Never log event payloads (may contain secrets)
- Secure GitHub App token integration
- Proper RBAC for template execution
- No secrets in container environment

## Success Criteria

Your implementation is complete when:

1. **Simplified API**: Only `github-app` parameter required for basic usage
2. **Auto-Detection**: Correctly infers all parameters from GitHub events
3. **Template Creation**: Both CodeRun and DocsRun CRs are created successfully
4. **Error Handling**: Graceful failure with clear error messages
5. **Integration**: Works with Argo Events and GitHub App tokens
6. **Testing**: Comprehensive test suite covers all scenarios
7. **Documentation**: Complete usage examples and troubleshooting

## Delivery Artifacts

Create these files:
- `coderun-template.yaml` - Main CodeRun WorkflowTemplate
- `docsrun-template.yaml` - DocsRun WorkflowTemplate  
- `press-play.yaml` - Orchestrator Workflow
- Test workflows with sample event payloads
- Integration test scripts
- Usage examples and documentation

Remember: Simplicity is key. The goal is to make it trivial for users to invoke AI agents with minimal configuration while maintaining full functionality through auto-detection.