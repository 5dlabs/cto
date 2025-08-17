# Ready-for-QA Label Logic Implementation

## 🚨 CRITICAL: Argo Events Reference Documentation

**BEFORE implementing ANY Argo Events sensors/triggers, MUST review official examples:**
- **Location:** [docs/references/argo-events/](../../../references/argo-events/)
- **Key Files:**
  - `github.yaml` - GitHub webhook sensor patterns
  - `complete-trigger-parameterization.yaml` - Dynamic parameter extraction  
  - `special-workflow-trigger.yaml` - ArgoWorkflow operations (submit/resume)
  - `trigger-standard-k8s-resource.yaml` - K8s resource creation patterns

**❌ UNSUPPORTED Operations (will cause deployment failures):**
- `operation: delete` ❌
- `operation: patch` ❌  
- `operation: update` ❌
- Template variables in `labelSelector` ❌

**✅ SUPPORTED Operations:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows)
- `dest: metadata.name` (dynamic targeting)

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!


You are implementing the critical handoff mechanism between Cleo's code quality work and Tess's comprehensive testing phase. Create logic for Cleo to add 'ready-for-qa' label to PRs as an explicit signal that triggers Tess workflow resumption.

## Objective

Implement the ready-for-qa labeling system that enables Cleo to signal completion of code quality work and readiness for comprehensive testing, triggering the next phase in multi-agent orchestration.

## Context

The ready-for-qa label serves as the explicit handoff signal in multi-agent workflow:
- **Cleo's Completion Signal**: Indicates all code quality work finished and CI tests passed
- **Tess's Start Trigger**: Event-driven resumption of workflow at testing phase
- **Quality Gate**: Ensures no testing begins until code quality requirements met
- **Audit Trail**: Provides visible evidence of workflow progression

## Implementation Requirements

### 1. Implement Cleo Workflow Sequence

Create complete workflow in container-cleo.sh.hbs:
```bash
# Cleo workflow sequence
1. Run comprehensive code quality checks (Clippy pedantic, rustfmt)
2. Push quality fixes to same feature branch
3. Wait for GitHub Actions CI tests to pass
4. Add 'ready-for-qa' label via GitHub API
5. Complete Cleo workflow successfully
```

### 2. Create CI Test Validation

Implement robust CI status checking:
```bash
#!/bin/bash
# wait-for-ci-success.sh
check_ci_status() {
    # Monitor GitHub Actions status
    # Wait for all CI checks to pass
    # Timeout after reasonable period
    # Return success only when all checks green
}
```

### 3. Implement GitHub API Label Management

Create idempotent label addition:
```bash
#!/bin/bash
# add-ready-for-qa-label.sh
# Check if label already exists (idempotent operation)
# Add ready-for-qa label via GitHub CLI/API
# Verify label was added successfully
# Handle GitHub API errors gracefully
```

### 4. Create Argo Events Integration

Configure sensor to detect ready-for-qa label:
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: ready-for-qa-detection
spec:
  dependencies:
  - name: github-pr-labeled
    eventName: pull-request-labeled
    filters:
      - path: "body.label.name"
        value: ["ready-for-qa"]
      - path: "body.sender.login" 
        value: ["5DLabs-Cleo[bot]"]
```

## Technical Specifications

### Label Addition Logic
- **Idempotent Operations**: Safe to retry, multiple additions don't cause errors
- **CI Prerequisites**: Only add label after all CI tests pass
- **GitHub Authentication**: Use agent-specific GitHub App credentials
- **Error Handling**: Graceful handling of GitHub API failures

### Workflow Integration
- **PR Discovery**: Find PR associated with current task branch
- **Context Management**: Track PR number, task ID, and branch information
- **Event Correlation**: Enable Argo Events to correlate label events with workflows
- **State Validation**: Verify PR is in correct state for labeling

### Tess Prerequisites
- **Label Validation**: Tess checks for ready-for-qa label before starting
- **Workflow Coordination**: Tess waits until label present to begin comprehensive testing
- **Error Recovery**: Handle cases where label is missing or removed

## Workflow Coordination Patterns

### Cleo Container Script Integration
```handlebars
{{#if (eq github_app "5DLabs-Cleo")}}
# Setup PR context and GitHub authentication
source setup-pr-context.sh
export GITHUB_TOKEN=$(generate-github-token.sh)

# Run quality checks and improvements
run-quality-checks.sh
commit-and-push-fixes.sh

# Wait for CI and add handoff label
wait-for-ci-success.sh "$PR_NUMBER"
add-ready-for-qa-label.sh "$PR_NUMBER"

echo "✅ Handoff to Tess complete"
{{/if}}
```

### Event-Driven Workflow Resumption
```yaml
triggers:
- template:
    name: resume-tess-stage
    argoWorkflow:
      operation: resume
      source:
        resource:
          labelSelector: |
            workflow-type=play-orchestration,
            current-stage=waiting-ready-for-qa,
            task-id={{extracted-task-id}}
```

### Tess Integration Points
```handlebars
{{#if (eq github_app "5DLabs-Tess")}}
# Validate prerequisites before starting
if ! validate-ready-for-qa-prerequisite.sh; then
    echo "⏳ Waiting for ready-for-qa signal from Cleo"
    exit 0
fi

echo "✅ Ready-for-qa confirmed, starting comprehensive testing"
{{/if}}
```

## Success Criteria

1. **Quality Gate Enforcement**: Cleo only adds label after all quality checks and CI pass
2. **Event Detection**: Argo Events sensor correctly detects ready-for-qa label additions
3. **Workflow Resumption**: Tess workflow resumes when ready-for-qa label detected
4. **Idempotent Operations**: Label addition safe to retry without side effects
5. **Error Handling**: GitHub API failures handled gracefully with appropriate fallbacks
6. **Audit Trail**: All label operations logged for debugging and monitoring
7. **Task Correlation**: Events correctly correlated to specific task workflows

## Implementation Deliverables

### Cleo Workflow Scripts
- Complete container-cleo.sh.hbs with ready-for-qa workflow
- CI status monitoring script (wait-for-ci-success.sh)
- Label addition script (add-ready-for-qa-label.sh)
- PR context setup and discovery scripts

### Argo Events Configuration
- Sensor configuration for ready-for-qa label detection
- Event filtering to target Cleo-generated label events
- Workflow resumption triggers with proper task correlation

### Tess Integration
- Prerequisites validation for ready-for-qa label
- Workflow coordination logic in container-tess.sh.hbs
- Error handling when prerequisites not met

### Testing and Validation
- Integration tests for complete Cleo → Tess handoff
- GitHub API error simulation and recovery testing
- Concurrent workflow coordination testing

## Testing Requirements

### Label Management Testing
- Test idempotent label addition (multiple calls don't create duplicates)
- Test label addition with various GitHub API response scenarios
- Test CI status monitoring with passing and failing checks
- Test timeout behavior for slow CI processes

### Event Integration Testing
- Test Argo Events sensor detects label additions correctly
- Test event filtering excludes non-Cleo label additions
- Test task ID extraction from PR labels for correlation
- Test workflow resumption at correct stage

### End-to-End Workflow Testing
- Test complete Cleo workflow with quality checks → CI wait → labeling
- Test Tess prerequisite validation and workflow initiation
- Test error scenarios (failed CI, GitHub API errors, missing PRs)
- Test multiple concurrent tasks don't interfere

Focus on creating a reliable, event-driven handoff system that ensures code quality requirements are met before comprehensive testing begins, while providing clear audit trails and robust error handling.