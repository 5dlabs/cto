# Oracle Research Query: Argo Events Task-Specific Targeting Solutions

## Context & Problem Statement

We're implementing a multi-agent orchestration system using Argo Events + Argo Workflows where GitHub webhooks need to trigger task-specific workflow resumptions. However, we've discovered that Argo Events has significant limitations that break our assumed architecture.

### Current Architecture Goal




```yaml


# Desired workflow: PR approved for task-3 → Resume ONLY task-3 workflow
GitHub PR #123 approved (labeled: task-3)
  ↓ GitHub webhook
  ↓ Argo Events sensor
  ↓ Resume play-task-3-workflow (ONLY this one)








```

### Discovered Limitations




```yaml
# What we learned Argo Events CANNOT do:
- labelSelector with template variables: labelSelector: "task-id={{webhook.taskId}}" ❌
- Dynamic resource targeting: metadata.name: "{{dynamic-name}}" ❌ (for resume ops)
- Delete/patch operations: operation: delete/patch ❌


- Complex resource filtering with webhook data ❌








```

### Current Broken Behavior




```yaml
# What happens now with static targeting:
GitHub PR #123 approved (task-3)
  ↓ Sensor: labelSelector: "workflow-type=play-orchestration,current-stage=waiting-pr-approved"
  ↓ Resumes: task-1, task-3, task-5, task-7 workflows (ALL OF THEM!) ❌








```

## Research Questions for Oracle Investigation

### 1. Advanced Argo Events Patterns Research
**Query:** "Are there undocumented or advanced Argo Events patterns for dynamic resource targeting that we've missed?"

**Investigation Areas:**


- Argo Events source code for advanced labelSelector capabilities


- Custom template functions or filters for resource targeting


- Multi-step trigger configurations that enable task correlation


- EventBus or inter-trigger communication patterns


- Advanced dataTemplate/contextTemplate usage for resource selection

### 2. Alternative Argo Events Architectures
**Query:** "What are alternative Argo Events architectural patterns for achieving task-specific targeting?"

**Exploration Areas:**


- Task-specific EventSources (one per task vs. global)


- Hierarchical sensor configurations with task discrimination


- Conditional trigger logic based on extracted webhook data


- Sensor composition patterns for selective triggering


- Custom webhook routing mechanisms within Argo Events

### 3. Argo Events + Kubernetes Resource Discovery
**Query:** "Can Argo Events sensors use Kubernetes resource queries to dynamically discover target workflows?"

**Research Focus:**


- Using kubectl queries within sensor triggers to find specific workflows
- Multi-step workflows: sensor → discovery → targeting


- Custom resource discovery patterns for task correlation


- Integration with Kubernetes watch APIs for dynamic resource detection

### 4. Workflow Name/Label Strategy Analysis
**Query:** "What's the optimal workflow naming and labeling strategy for Argo Events correlation?"

**Analysis Points:**


- Deterministic vs. generated workflow names


- Label hierarchy strategies for multi-dimensional correlation (task-id, stage, agent)


- Workflow metadata patterns that enable precise sensor targeting


- Best practices for workflow discovery and state management

### 5. Alternative Event-Driven Patterns
**Query:** "Are there alternative event-driven orchestration patterns within the Argo ecosystem?"

**Investigation Areas:**


- Argo Workflows native event handling (without Argo Events)


- Workflow templates with embedded event logic


- Custom controllers or operators for event handling


- Event-driven DAG patterns using workflow-level logic


- Integration with other CNCF event systems (CloudEvents, etc.)

## Architecture Decision Framework

### Evaluation Criteria
For each solution discovered, evaluate against:

1. **Task Isolation**: Can it achieve perfect task-specific targeting?
2. **Complexity**: How much architectural change required?
3. **Maintainability**: How many sensors/resources needed per task?
4. **Scalability**: How does it handle 100+ concurrent tasks?
5. **Reliability**: What failure modes exist?
6. **Performance**: Resource overhead and latency implications

### Critical Constraints
- **Must maintain task isolation**: No cross-task interference allowed
- **GitHub webhook integration**: Must work with existing webhook setup
- **Multi-agent support**: Rex, Blaze, Morgan implementation agents
- **Event-driven**: Cannot use polling-based solutions
- **Existing infrastructure**: Prefer solutions that work with current Argo Events setup

## Specific Technical Investigation

### Current Argo Events Configuration Analysis
**Examine our current setup:**
- EventSource configuration: `infra/gitops/resources/github-webhooks/`


- Webhook routing and processing patterns


- Label correlation mechanisms already in place


- GitHub App integration and authentication flow

### Workflow Creation Pattern Analysis
**Investigate:**


- How workflows are currently created (via `coderun-template.yaml`)


- Naming patterns and label assignment


- Workflow lifecycle and state management


- Integration points where task correlation could be enhanced



### GitHub Webhook Payload Deep Dive
**Analyze:**


- Complete webhook payload structure for task correlation data


- Label extraction and validation mechanisms


- Branch name parsing vs. label-based correlation


- Event timing and ordering considerations

## Deliverable Requirements



### Research Report Structure
1. **Executive Summary**: Is task-specific targeting achievable with Argo Events?
2. **Technical Analysis**: Detailed findings on each investigation area
3. **Solution Options**: Concrete implementation approaches with pros/cons
4. **Architecture Recommendation**: Preferred approach with implementation roadmap
5. **Risk Assessment**: Failure modes and mitigation strategies



### Specific Outputs Needed


- **Working code examples** of any advanced patterns discovered


- **Configuration templates** for recommended approach


- **Migration plan** from current broad targeting to precise targeting


- **Testing strategy** for validating task isolation


- **Documentation updates** required for implementation teams

## Context Files for Review

### Current Implementation


- `infra/gitops/resources/github-webhooks/*.yaml` - Our current sensor configurations


- `infra/charts/controller/templates/coderun-template.yaml` - Workflow creation pattern


- `docs/.taskmaster/docs/prd.txt` - Complete system requirements

### Reference Documentation


- `docs/references/argo-events/` - Official Argo Events examples we've gathered


- `docs/.taskmaster/docs/architecture.md` - Current architectural assumptions

### Task Definition


- `docs/.taskmaster/docs/task-3/task.md` - Workflow DAG requirements


- `docs/.taskmaster/docs/task-*` - Related event-driven task definitions

## Research Urgency

This is a **critical architectural decision point**. If Argo Events cannot support task-specific targeting, we need to either:



1. **Restructure our entire workflow approach** (major effort)


2. **Find advanced Argo Events patterns we missed** (preferred)


3. **Accept architectural limitations and design around them** (compromised solution)

The research should prioritize finding **any way to achieve task-specific targeting within Argo Events** before recommending architectural changes.



---

**Oracle Task**: Please conduct comprehensive research on these areas and provide concrete recommendations for achieving perfect task isolation in our multi-agent orchestration system using Argo Events, or definitively determine if alternative approaches are required.
