# Workflow Stage Transitions Implementation

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

You are ENHANCING existing workflow stage management for multi-agent orchestration. The basic stage labeling and event correlation are already implemented - you need to add explicit stage transition steps and atomic label update mechanisms.



## Objective

ENHANCE existing workflow template by adding explicit stage transition steps after each agent completes. Current implementation only sets labels at suspend points - add dedicated `update-workflow-stage` templates that atomically patch workflow labels via Kubernetes API after Rex, Cleo, and Tess complete. Integrate with existing suspend/resume pattern and task 5's correlation sensor.

## Context

Multi-agent orchestration requires precise state management where:


- **Workflows track their current stage** through metadata labels


- **Stage transitions trigger event-driven agent handoffs**


- **Atomic label updates prevent race conditions**


- **Event sensors target workflows based on stage + task ID**

The stage progression flow is:






```
waiting-pr-created → waiting-ready-for-qa → waiting-pr-approved → completed






```

## Implementation Requirements

### 1. Design Atomic Label Update System

Create Kubernetes resource templates for atomic label updates:



```yaml
- name: update-workflow-stage
  resource:
    action: patch
    manifest: |
      apiVersion: argoproj.io/v1alpha1
      kind: Workflow
      metadata:
        name: "{{workflow.name}}"
        labels:
          current-stage: "{{inputs.parameters.new-stage}}"
          task-id: "{{workflow.parameters.task-id}}"






```

### 2. Implement Stage Transition Logic

Add stage transition steps after each agent completes:
- After Rex: Update to `waiting-pr-created`
- After Cleo: Update to `waiting-ready-for-qa`
- After Tess: Update to `waiting-pr-approved`

### 3. Create Suspend/Resume Pattern with Stages

Structure workflow with explicit stage management:



```yaml
dag:
  tasks:
  - name: rex-implementation
    template: agent-coderun
  - name: update-to-waiting-pr
    dependencies: [rex-implementation]
    template: update-workflow-stage
  - name: wait-pr-created
    dependencies: [update-to-waiting-pr]
    template: suspend-for-webhook






```

### 4. Integrate with Argo Events Sensors

Enable stage-aware event correlation:



```yaml
argoWorkflow:
  operation: resume
  source:
    resource:
      labelSelector: |
        workflow-type=play-orchestration,
        current-stage=waiting-pr-created,
        task-id={{extracted-task-id}}






```

## Technical Specifications



### Label Structure
- `current-stage`: Current workflow stage for event targeting
- `task-id`: Task identifier for correlation
- `repository`: Repository context for multi-repo support
- `updated-at`: Timestamp for debugging and monitoring

### Race Condition Prevention
Use JSON merge patch strategy with resource versioning:



```bash
kubectl patch workflow "$WORKFLOW_NAME" \


  --type='merge' \
  --patch='{"metadata":{"labels":{"current-stage":"new-stage"}}}'






```

### Idempotent Operations
All label updates must be safe to retry and duplicate:


- Check current state before updating


- Verify update succeeded after patch


- Handle concurrent update scenarios gracefully

### Error Handling


- Retry failed label updates with exponential backoff


- Fail workflow if critical stage transitions fail


- Log all stage transitions for debugging


- Provide rollback mechanisms for failed transitions

## Workflow Integration Points

### Agent Completion Detection
Each agent stage should update workflow state upon completion:


- Rex completion → `waiting-pr-created`


- Cleo completion → `waiting-ready-for-qa`


- Tess completion → `waiting-pr-approved`

### Event-Driven Resumption
Argo Events sensors resume workflows based on:


- GitHub PR events (creation, labeling, approval)


- Workflow stage labels (current-stage)


- Task correlation (task-id from PR labels)

### Multi-Workflow Coordination
Support multiple concurrent workflows:


- Each workflow maintains independent stage state


- Event correlation prevents cross-workflow interference


- Task ID ensures events target correct workflow instance



## Success Criteria

1. **Atomic Updates**: Stage transitions are atomic and race-condition free
2. **Event Correlation**: Sensors correctly target workflows by stage + task ID
3. **State Consistency**: Workflow stage always reflects actual progress
4. **Concurrent Safety**: Multiple workflows don't interfere with each other
5. **Error Recovery**: Failed stage transitions are handled gracefully
6. **Monitoring**: All stage transitions are logged and monitorable

## Implementation Deliverables



### Workflow Template Updates


- Add `update-workflow-stage` template for atomic label updates


- Integrate stage transitions into main DAG flow


- Add suspend points between agent stages


- Include proper error handling and verification

### Argo Events Integration


- Update sensors with stage-aware label selectors


- Add event correlation logic for task ID extraction


- Test event targeting with multiple concurrent workflows


- Implement proper webhook validation and filtering

### Testing Infrastructure


- Create test workflows for stage transition validation


- Implement automated tests for concurrent update scenarios


- Add monitoring for stage transition success/failure rates


- Create troubleshooting documentation for common issues

Focus on creating reliable, atomic state management that enables precise event-driven coordination between agents while preventing race conditions and ensuring workflow state consistency.
