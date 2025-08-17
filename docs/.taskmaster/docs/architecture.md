# Play Workflow - Multi-Agent Event-Driven Requirements

## Overview

The play workflow is an event-driven, multi-agent orchestration that processes tasks through quality gates with real GitHub PR interactions and live Kubernetes deployment testing.

## Workflow Stages

### Stage 1: Implementation (Rex or Blaze)

**Trigger:** Manual start of play workflow

**Process:**
1. Implementation agent (Rex or Blaze) starts on current task (lowest numbered task directory)
2. **Documentation-First Approach:** Agent checks MCP documentation server for:
   - Relevant Rust crate documentation
   - Best practices for the specific implementation
   - API documentation for any integrations
   - Existing implementation patterns
3. Agent completes implementation work following documented patterns
4. Agent pushes pull request
5. **Agent's task is NOT complete yet** - still needs to pass quality gates

**Output:** Pull request created

---

### Stage 2: Code Quality (Cleo)  

**Trigger:** GitHub webhook detects PR creation (via Argo Events)

**Process:**
1. Cleo detects that Rex has submitted a PR for the current task
2. Cleo performs relentless code quality work:
   - All Clippy pedantic warnings must be 100% resolved
   - Full formatting compliance required
   - Zero tolerance for quality issues
3. Cleo pushes changes to the **same feature branch** (not a new PR)
4. **Cleo waits for GitHub Actions tests to pass** - this is Cleo's measure of success
   - Must see green checkmarks on all CI/CD tests
   - Any test failures require Cleo to investigate and fix
5. **Cleo adds "ready-for-qa" label to PR** - explicit completion signal
   - Only when fully satisfied with code quality AND tests passing
   - Clear handoff to Tess for comprehensive testing

**Output:** PR labeled with "ready-for-qa"

---

### Stage 3: Quality Assurance (Tess)

**Trigger:** GitHub webhook detects PR labeled with "ready-for-qa"

#### 3A: Code Review Phase

**Process:**
1. Tess performs comprehensive code review against acceptance criteria
2. **Before attempting to run any code**
3. If issues found: Tess adds comments to the PR
4. Comments trigger Rex to restart (separate process, not in main workflow)

#### 3B: Live Deployment Testing Phase

**Prerequisites:** 
- Helm charts available
- Container image builds successfully  
- All required infrastructure components available

**Process:**
1. Tess deploys application in live Kubernetes environment
2. Tess performs comprehensive regression testing:
   - Endpoint testing for correct responses
   - Data consistency validation
   - Pod health and resource utilization
   - Integration testing across services
3. Tess has full admin access to verify functionality:
   - Postgres Admin access
   - Redis Admin access  
   - Kubernetes Admin access
   - Argo CD Admin access
   - GitHub Actions access

#### 3C: Test Coverage Enhancement Phase

**Process:**
1. **After confirming functionality works**, Tess analyzes test coverage
2. Identifies missing unit tests and integration tests
3. Implements additional tests to achieve near 100% coverage:
   - Unit tests for all new functions/methods
   - Integration tests for API endpoints
   - Edge case testing for error conditions
   - Database interaction testing
4. Pushes test additions to the same feature branch
5. Ensures all new tests pass in CI/CD pipeline

**Requirements:**
- Must be 120% satisfied before approval
- Must test against real acceptance criteria
- Must verify in actual live environment (not mocks/stubs)
- **Must achieve comprehensive test coverage before approval**

**Output:** PR approval (but NOT merge)

---

### Stage 4: Human Approval & Task Completion

**Trigger:** Tess approves PR

**Process:**
1. CTO (human) reviews and merges the PR
2. Task is marked as completed  
3. Task directory moved to `.completed/`
4. Workflow proceeds to next task (if available)

---

## Event-Driven Architecture

### GitHub Events Integration
- **PR Created:** Triggers Cleo stage
- **Push to Feature Branch:** Triggers Tess stage  
- **PR Comments:** Triggers Rex restart (separate from main workflow)
- **PR Merged:** Triggers task completion and next task start

### Argo Events Configuration
- Existing Argo Events → Argo Workflows integration already configured
- Need to identify task association from PR/webhook data
- Workflow steps wait for specific GitHub webhook events before proceeding

---

## Implementation Agent Remediation & QA Pipeline Restart

### Implementation Agent Comment Response (Separate Process)
**Trigger:** Any new comment added to PR (by Tess, CTO, or other agents)

**Process:**
1. Implementation agent (Rex, Blaze, or Morgan) detects new PR comments
2. Agent starts with session resume  
3. Agent pulls down **only unresolved/relevant comments**
4. Agent addresses issues and pushes fixes
5. **Implementation agent push triggers QA pipeline restart** (see below)

### QA Pipeline Restart Logic
**Trigger:** Any implementation agent pushes to feature branch (commit by 5DLabs-Rex, 5DLabs-Blaze, or 5DLabs-Morgan)

**The Problem:** When implementation agents fix issues, any running Cleo/Tess work becomes obsolete

**Cancellation Process:**
1. **GitHub webhook detects push by implementation agent** to feature branch
2. **Cancel running Cleo/Tess CRDs** for this task immediately:
   ```bash
   # Cancels ALL quality/testing agent CodeRuns (not task-specific due to sensor limitations)
   kubectl delete coderun -l github-app=5DLabs-Cleo
   kubectl delete coderun -l github-app=5DLabs-Tess
   ```
3. **Remove "ready-for-qa" label** (if present) to reset state
4. **Update workflow stage** back to waiting-pr-created  
5. **Resume workflow from Cleo stage** with fresh code

### Event-Based Coordination
```yaml
# Multi-agent sensor for implementation agent remediation
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: implementation-agent-remediation
  labels:
    sensor-type: multi-agent-orchestration
    agent-class: implementation
spec:
  dependencies:
  - name: implementation-push-event
    eventSourceName: github
    eventName: org
    filters:
      data:
        # Match any 5DLabs implementation agent (Rex, Blaze, Morgan)
        - path: body.sender.login
          type: string
          comparator: "~"
          value: "5DLabs-(Rex|Blaze|Morgan)\\[bot\\]|5DLabs-(Rex|Blaze|Morgan)"
        # Ensure push is to a task branch
        - path: body.ref
          type: string
          comparator: "~"
          value: "refs/heads/task-.*"
        
  triggers:
  - template:
      name: cancel-running-quality-agents
      k8s:
        # Cancel running quality/testing agents
        operation: delete
        source:
          resource:
            apiVersion: agents.platform/v1
            kind: CodeRun
        labelSelector:
          matchExpressions:
            - key: github-app
              operator: NotIn
              values: ["5DLabs-Rex", "5DLabs-Blaze", "5DLabs-Morgan"]
        # Additional logging and workflow resume triggers follow
```

### Adding New Implementation Agents

The implementation agent remediation sensor supports multiple agents via regex matching. To add a new implementation agent (e.g., "Nova"):

1. **Update Sensor Regex**: Add "Nova" to the regex pattern in `implementation-agent-remediation-sensor.yaml`
   ```yaml
   value: "5DLabs-(Rex|Blaze|Morgan|Nova)\\[bot\\]|5DLabs-(Rex|Blaze|Morgan|Nova)"
   ```

2. **Update Agent Exclusion**: Add "5DLabs-Nova" to the exclusion list
   ```yaml
   values: ["5DLabs-Rex", "5DLabs-Blaze", "5DLabs-Morgan", "5DLabs-Nova"]
   ```

3. **Create GitHub App**: Set up 5DLabs-Nova GitHub App with appropriate permissions

4. **Update Agent Configuration**: Add Nova to `values.yaml` and secret management

The sensor will automatically detect pushes from the new agent and trigger QA pipeline restarts.

---

## Quality Gates Summary

1. **Implementation Gate:** Rex completes feature development
2. **Quality Gate:** Cleo ensures code quality and formatting  
3. **Review Gate:** Tess performs code review against acceptance criteria
4. **Deployment Gate:** Tess verifies functionality in live Kubernetes environment
5. **Approval Gate:** Human (CTO) merges PR after full validation

---

## Realistic Timeline Expectations

**Why This Takes Days/Weeks (Not Hours):**
- **Complex Implementation:** Rex may need 1-3 days for sophisticated features
- **Documentation Research:** MCP server queries and best practice verification takes time
- **Multiple Review Cycles:** Tess may find issues requiring multiple Rex iterations  
- **Real Deployment Testing:** Actual Kubernetes deployment, database testing, integration validation
- **Human Intervention:** CTO review and approval happens on business schedule
- **Infrastructure Issues:** Deployment failures, resource constraints, environment problems
- **Quality Standards:** 120% satisfaction requirement means thorough, not rushed, validation
- **Session Context:** Agents need time to build understanding and make thoughtful decisions

**Legitimate Long-Running Process:** Software development cycles naturally span days to weeks, not minutes or hours.

---

## Key Workflow Characteristics

- **Event-Driven:** Each stage triggered by GitHub events, not sequential timing
- **Real-World Testing:** Actual Kubernetes deployment and validation required
- **Quality-Focused:** Multiple agents ensure different aspects of quality  
- **Human Oversight:** CTO maintains control over final PR merges
- **Feedback Loops:** Rex can be recalled based on review feedback
- **Single Workflow Visibility:** Entire process visible in Argo Workflows UI

---

## Technical Requirements

### Workflow Engine
- Argo Workflows with event-based step triggers
- Integration with existing Argo Events setup
- Ability to pause/wait for external events (GitHub webhooks)

### Agent Capabilities  
- **Rex:** Implementation, comment resolution, session resume
- **Blaze:** Implementation, coding tasks (similar to Rex)
- **Cleo:** Pedantic code quality, formatting, zero-tolerance standards
- **Tess:** Code review, live deployment, comprehensive testing, test coverage enhancement, admin access

### Documentation Access (Rex & Blaze)
- **MCP Documentation Server:** Access to implementation-specific documentation via MCP server
- **Rust Crates Documentation:** Complete documentation for all Rust crates in use
- **Rust Best Practices:** Current Rust coding standards and best practices
- **API Documentation:** All relevant API documentation for integrations
- **Implementation Methodology:** Always check documentation BEFORE implementing to avoid rework
- **Documentation-First Approach:** Verify implementation patterns against available docs to reduce iteration cycles

### Infrastructure Access
- Full Kubernetes admin access for Tess
- Database admin access (Postgres, Redis)
- CI/CD system access (GitHub Actions, Argo CD)
- Ability to deploy and test in live environments

---

## Task Association Implementation

### Multi-Method Approach with Validation
- **Primary:** PR Labels (`task-3`) - main detection method for webhooks
- **Secondary:** Branch name parsing (`task-3-*`) - human readability and backup
- **Fallback:** Marker file (`docs/.taskmaster/current-task.json`) - workflow-controlled reference
- **Validation:** All methods must agree, **workflow fails if mismatch** (no false positives)

### Implementation Requirements
```yaml
# In workflow template - before agent execution
- name: create-task-marker
  script:
    image: alpine:latest
    command: [sh]
    source: |
      echo '{"taskId":"{{workflow.parameters.current-task}}","workflowName":"{{workflow.name}}","startTime":"{{workflow.createdAt}}"}' > /work/src/docs/.taskmaster/current-task.json
      git add docs/.taskmaster/current-task.json
      git commit -m "chore: mark task {{workflow.parameters.current-task}} in progress"

# Agent System Prompt Requirements (Rex & Blaze):
# "CRITICAL: When creating PR, you MUST:
#  1. Add GitHub label 'task-{current-task-id}' to the PR
#  2. Use branch name format: 'task-{current-task-id}-{description}'
#  3. Verify current task ID from docs/.taskmaster/current-task.json
#  Workflow will FAIL if these don't match - no exceptions."
```

### Webhook Detection Logic
```bash
# Argo Events webhook processing
TASK_FROM_LABEL=$(jq -r '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]' webhook.json)
TASK_FROM_BRANCH=$(jq -r '.pull_request.head.ref' webhook.json | grep -o '^task-[0-9]*' | cut -d'-' -f2)

# Validation: both must match or workflow fails
if [ "$TASK_FROM_LABEL" != "$TASK_FROM_BRANCH" ]; then
  echo "ERROR: Task association mismatch - Label: $TASK_FROM_LABEL, Branch: $TASK_FROM_BRANCH"
  exit 1
fi
```

## ✅ Event-Driven Architecture Feasibility (Confirmed)

### Core Implementation: Suspend/Resume Pattern
**✅ FEASIBLE:** Argo Workflows supports mid-workflow pausing via `suspend` templates and resumption through Argo Events integration.

### Technical Implementation Structure
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: play-task-{{workflow.parameters.task-id}}-workflow
  labels:
    workflow-type: play-orchestration
spec:
  entrypoint: main
  activeDeadlineSeconds: 1209600  # 14 days - realistic for multi-agent development cycle
  templates:
  - name: main
    dag:
      tasks:
      - name: rex-implementation
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Rex"
      
      - name: wait-pr-created
        dependencies: [rex-implementation]  
        template: suspend-for-webhook
        
      - name: cleo-quality
        dependencies: [wait-pr-created]
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Cleo"
            
      - name: wait-ready-for-qa
        dependencies: [cleo-quality]
        template: suspend-for-webhook
        
      - name: tess-testing
        dependencies: [wait-ready-for-qa] 
        template: agent-coderun
        arguments:
          parameters:
          - name: github-app
            value: "5DLabs-Tess"
            
      - name: wait-pr-approved
        dependencies: [tess-testing]
        template: suspend-for-webhook
        
      - name: complete-task
        dependencies: [wait-pr-approved]
        template: mark-task-complete
        
  - name: suspend-for-webhook
    suspend: {}  # Indefinite suspend until external resume
```

### Argo Events Integration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: play-workflow-github-events
spec:
  dependencies:
  - name: github-pr-created
    eventSourceName: github-webhook
    eventName: pull-request-opened
  - name: github-push-event  
    eventSourceName: github-webhook
    eventName: push-to-branch
  - name: github-pr-approved
    eventSourceName: github-webhook  
    eventName: pull-request-approved
    
  triggers:
  - template:
      name: resume-workflow-on-event
      argoWorkflow:
        operation: resume
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            # Correlation via task labels from webhook payload
        parameters:
        - src:
            dependencyName: github-pr-created
            dataKey: body.pull_request.labels[?(@.name | startswith('task-'))].name
          dest: spec.arguments.parameters.taskId
```

### Event Correlation Strategy
- **Workflow Identification:** Use task labels (`task-3`) from PR to identify which workflow to resume
- **Event Payload Processing:** Extract task ID from GitHub webhook payload labels
- **Selective Resume:** Only resume workflows waiting for the specific event type + task ID combination

### Resource & Timeout Management  
- **Extended Timeout:** `activeDeadlineSeconds: 604800` (7 days) or longer - legitimately long-running process
- **Timeout Considerations:** 
  - Rex implementation: Could take 1-2 days for complex features
  - Code review iterations: Multiple back-and-forth cycles with Tess
  - Real-world testing: Deployment and comprehensive testing takes time  
  - Human review: CTO approval may not be immediate
  - **Realistic Duration:** Weeks, not hours
- **Resource Efficiency:** Suspended workflows consume no pod resources (only etcd space)
- **Monitoring:** Track suspended workflow count to prevent etcd bloat

### Timeout Strategy Options
```yaml
# Option 1: Extended but bounded (recommended)
activeDeadlineSeconds: 1209600  # 14 days

# Option 2: Very long timeout  
activeDeadlineSeconds: 2592000  # 30 days

# Option 3: Configurable per workflow
activeDeadlineSeconds: {{workflow.parameters.timeout-days | default(14) | multiply(86400)}}

# Option 4: No timeout (risky - manual cleanup required)
# activeDeadlineSeconds: null  # Not recommended
```

## CRD Modifications for Multi-Agent Support

### Decision: Use CodeRun CRD for All Agents (Rex, Cleo, Tess)

**Benefits of Consistent CRD Approach:**
- **Live Input Capability:** Can send inputs to struggling agents via MCP input tool
- **Rich Status Tracking:** Detailed phase/condition reporting for each agent
- **Session Continuity:** Sophisticated memory management with `continue_session`
- **Consistent Interface:** Same monitoring, debugging, and intervention patterns
- **Template System:** Agent-specific configuration via Handlebars templates

### Current CodeRun CRD Analysis

**Existing CRD Capabilities (Good for all agents):**
- ✅ `continue_session`: Each agent continues their OWN previous sessions (not shared)
- ✅ `github_app`: Agent-specific authentication (5DLabs-Cleo, 5DLabs-Tess) 
- ✅ `model`: Agent-specific Claude models
- ✅ Template system: Agent-specific prompts and configurations
- ✅ Live input: MCP input tool for real-time agent interaction
- ✅ PVC management: Already handles workspace persistence per agent

### Required Template Modifications

#### 1. Agent-Specific System Prompts
**Current:** Only Rex/Blaze implementation prompts exist

**Needed:** Add Cleo and Tess system prompts to template system:
```
infra/charts/controller/claude-templates/agents/
├── cleo-system-prompt.md.hbs
└── tess-system-prompt.md.hbs
```

#### 2. Agent-Specific Client Configuration  
**Current:** `client-config.json.hbs` optimized for implementation work

**Needed:** Agent-specific MCP tool configurations:
```handlebars
{{#if (eq github_app "5DLabs-Cleo")}}
  "remoteTools": [
    "rustdocs_query_rust_docs",
    "brave-search_brave_web_search", 
    "memory_create_entities"
  ]
{{else if (eq github_app "5DLabs-Tess")}}
  "remoteTools": [
    "kubernetes_listResources",
    "kubernetes_describeResource", 
    "memory_create_entities",
    "brave-search_brave_web_search"
  ]
{{else}}
  <!-- Rex/Blaze default config -->
{{/if}}
```

#### 3. Agent-Specific Container Scripts
**Current:** `container.sh.hbs` handles implementation workflow

**Needed:** Conditional logic for different agent behaviors:
```handlebars
{{#if (eq github_app "5DLabs-Cleo")}}
  # Cleo-specific setup: focus on code quality
  echo "🎯 Cleo: Code Quality & Formatting Agent"
  echo "Mission: Relentless pursuit of 100% code quality"
{{else if (eq github_app "5DLabs-Tess")}}  
  # Tess-specific setup: testing and deployment
  echo "🧪 Tess: Quality Assurance & Testing Agent"
  echo "Mission: 120% satisfaction through comprehensive testing"
  
  # Additional K8s admin setup for Tess
  export KUBECONFIG=/etc/kube/config
{{else}}
  # Rex/Blaze implementation setup
{{/if}}
```

#### 4. Agent-Specific CLAUDE.md Memory
**Current:** Generic implementation-focused memory template

**Needed:** Agent-specific memory initialization:
```handlebars
# Agent: {{github_app}}
{{#if (eq github_app "5DLabs-Cleo")}}
## Role: Code Quality & Formatting Specialist
- Focus on Clippy pedantic compliance
- Zero tolerance for formatting issues  
- Ensure 100% code quality standards
- CRITICAL: Add "ready-for-qa" label when work complete and tests pass
{{else if (eq github_app "5DLabs-Tess")}}
## Role: Quality Assurance & Testing Specialist  
- Comprehensive code review against acceptance criteria
- Live Kubernetes deployment testing
- Test coverage enhancement - add missing unit/integration tests
- Goal: Near 100% test coverage before approval
- 120% satisfaction requirement before approval
- Only start work when PR has "ready-for-qa" label
{{/if}}
```

### Infrastructure Requirements for Tess

**Additional RBAC for Testing Agent:**
```yaml
# Tess needs cluster-admin for deployment testing
- apiGroups: ["*"]
  resources: ["*"] 
  verbs: ["*"]
```

**Additional Secret Mounts for Tess:**
```yaml
# Database admin credentials
- name: postgres-admin-secret
- name: redis-admin-secret
# Argo CD admin credentials  
- name: argocd-admin-secret
```

### Template Structure Changes

```
infra/charts/controller/claude-templates/
├── agents/                          # NEW: Agent-specific prompts
│   ├── cleo-system-prompt.md.hbs
│   ├── tess-system-prompt.md.hbs
│   └── rex-system-prompt.md.hbs
├── code/
│   ├── claude.md.hbs               # MODIFY: Agent-specific memory
│   ├── client-config.json.hbs      # MODIFY: Agent-specific tools
│   ├── container.sh.hbs            # MODIFY: Agent-specific setup
│   └── settings.json.hbs           # MODIFY: Agent-specific settings
```

### Workspace Isolation Strategy

**Agent-Specific PVC Workspaces (Critical Requirement):**
- **Rex PVC**: `workspace-{service}-rex` - Implementation agent workspace
- **Cleo PVC**: `workspace-{service}-cleo` - Code quality agent workspace  
- **Tess PVC**: `workspace-{service}-tess` - Testing agent workspace

**Benefits of Separate Workspaces:**
- **Clean Cancellation**: When Rex pushes fixes, can cleanly cancel outdated Cleo/Tess work
- **Independent Context**: Each agent builds their own expertise and history over time
- **Fresh Perspectives**: Agents approach problems from their own accumulated experience
- **Isolation**: File system operations don't interfere between agents
- **Restart Efficiency**: Can restart individual agents without affecting others' accumulated knowledge

**PVC Naming Convention:**
```yaml
# Current naming: workspace-{service}
# New naming: workspace-{service}-{agent}

# Examples:
workspace-cto-rex      # Rex's persistent workspace
workspace-cto-cleo     # Cleo's persistent workspace  
workspace-cto-tess     # Tess's persistent workspace
```

**Implementation in CRD Controller:**
```rust
// In controller code - generate agent-specific PVC name
let pvc_name = format!("workspace-{}-{}", 
    code_run.spec.service,
    extract_agent_name(&code_run.spec.github_app)  // "rex", "cleo", "tess"
);
```

### Session Continuity Clarification

**How `continue_session: true` Works:**
- **NOT shared context**: Cleo doesn't continue from Rex's workspace
- **Agent-specific continuity**: Cleo continues from Cleo's previous session  
- **Independent histories**: Each agent maintains their own CLAUDE.md and context
- **Concurrent safety**: Rex can restart without affecting other agents' state

### Controller Code Modifications

**Required Changes:**
1. **PVC Name Generation**: Extract agent name from `github_app` field for unique PVC naming
2. **Template Resolution**: Modify template loading to check for agent-specific templates first
3. **RBAC Assignment**: Conditional RBAC based on `github_app` field  
4. **Secret Mounting**: Additional secrets for Tess testing capabilities

**No CRD Schema Changes:** Existing `CodeRun` fields support all requirements

### Implementation Priority
1. **Phase 1**: Add Cleo/Tess system prompt templates
2. **Phase 2**: Modify existing templates with agent-specific conditionals
3. **Phase 3**: Add Tess infrastructure access (RBAC, secrets)
4. **Phase 4**: Test multi-agent workflow with suspend/resume

## Cleo Completion Signaling

### Explicit Handoff Requirement  
**Problem**: Need clear signal when Cleo's work is complete and Tess can begin
**Solution**: Cleo adds "ready-for-qa" label to PR when fully satisfied

### Completion Signaling Options Considered

#### ✅ Selected: GitHub Label Approach
```yaml
Process:
1. Cleo completes code quality work
2. Cleo pushes changes and waits for CI success  
3. Cleo adds "ready-for-qa" label to PR
4. Webhook triggers Tess stage

Benefits:
- Clear, explicit handoff signal
- Visible in GitHub UI
- Programmatically detectable
- Fits existing label-based correlation
```

#### Alternative Options Evaluated
- **PR Comment**: Requires parsing comment text, more complex
- **Custom Status Check**: More complex implementation, less visible  
- **File Marker**: Creates file clutter in PR
- **PR Review**: Could conflict with human reviews

## Event Correlation Solution

### ✅ GitHub Webhook Payload Confirmation
**Research Confirmed:** GitHub PR webhooks include full `labels` array in payload:
```json
{
  "action": "opened",
  "pull_request": {
    "number": 123,
    "labels": [
      {
        "id": 12345,
        "name": "task-3",        # ← Our correlation key!
        "color": "ff0000", 
        "description": "Task 3 implementation"
      }
    ],
    "head": {
      "ref": "task-3-implement-auth"  # ← Secondary correlation
    }
  }
}
```

### Workflow Correlation Strategy

#### 1. Workflow Creation with Correlation Labels
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow  
metadata:
  name: play-task-{{workflow.parameters.task-id}}-workflow
  labels:
    workflow-type: play-orchestration
    task-id: "{{workflow.parameters.task-id}}"
    current-stage: waiting-pr-created
    repository: "5dlabs-cto"
spec:
  arguments:
    parameters:
    - name: task-id
      value: "3"
```

#### 2. Argo Events Sensor with Precise Targeting
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
spec:
  triggers:
  - template:
      name: resume-after-pr-created
      argoWorkflow:
        operation: resume
        args: []
        parameters:
        - src:
            # Extract task ID from PR labels and construct deterministic workflow name
            dataTemplate: |
              play-task-{{ range $i, $l := .Input.body.pull_request.labels }}{{ if hasPrefix "task-" $l.name }}{{ $p := splitList "-" $l.name }}{{ if gt (len $p) 1 }}{{ index $p 1 }}{{ end }}{{ end }}{{ end }}-workflow
          dest: args.0
```

#### 3. Multi-Stage Progression Logic
```yaml
# After resuming, workflow updates its stage label for next correlation
- name: update-stage-to-waiting-push  
  resource:
    action: patch
    manifest: |
      apiVersion: argoproj.io/v1alpha1
      kind: Workflow
      metadata:
        name: "{{workflow.name}}"
        labels:
          current-stage: waiting-push-event  # ← Update for next stage
```

### Event Correlation Patterns

#### Task ID Extraction Logic
```bash
# From webhook payload, extract task ID:
.pull_request.labels[?(@.name | startswith("task-"))].name | split("-")[1]

# Examples:
# PR with label "task-3" → extracts "3"  
# PR with label "task-15" → extracts "15"
```

#### Multi-Stage Workflow Targeting
```yaml
# Stage 1: After Rex creates PR
labelSelector: "task-id=3,current-stage=waiting-pr-created"

# Stage 2: After Cleo adds ready-for-qa label  
labelSelector: "task-id=3,current-stage=waiting-ready-for-qa"

# Stage 3: After Tess approves PR
labelSelector: "task-id=3,current-stage=waiting-pr-approved"
```

### Event Discrimination by Type and Author

#### Different Events Target Different Stages
```yaml
# PR Created (any author) → Resume after Rex
- eventName: pull-request-opened
  targetStage: waiting-pr-created

# PR Labeled "ready-for-qa" → Resume after Cleo  
- eventName: pull-request-labeled
  targetStage: waiting-ready-for-qa
  conditions:
    - label.name: "ready-for-qa"
    - PR has task label
    - labeled by 5DLabs-Cleo

# PR Review Approved (by Tess) → Resume after Tess
- eventName: pull-request-review
  targetStage: waiting-pr-approved  
  conditions:
    - state: approved
    - user: 5DLabs-Tess
```

## Final Review: Gaps, Questions, and Concerns

### 🚨 Critical Implementation Gaps

#### 1. Missing Argo Events Configuration
**Gap**: We have correlation theory but no actual sensor/eventsource YAML
**Need**: Complete Argo Events configuration for GitHub webhook → workflow resume
**Risk**: Theory might not work in practice without proper configuration

#### 2. Template Implementation Specifics  
**Gap**: We know what templates need but not the actual code changes
**Need**: Specific Handlebars template modifications with exact conditionals
**Risk**: Template complexity could introduce bugs or maintenance overhead

#### 3. Controller Code Changes
**Gap**: Conceptual understanding but no implementation details
**Need**: Specific Rust code changes for agent-specific PVC naming, RBAC, secrets
**Risk**: More complex than anticipated, could require significant controller modifications

### ⚠️ Operational Concerns

#### 4. Long-Running Workflow Management
**Questions**:
- How do we monitor workflow health over days/weeks?
- What alerts fire if workflows get stuck?
- How do we handle Kubernetes cluster restarts/upgrades?
- What's our backup/recovery strategy for corrupted workflows?

#### 5. Resource Management & Limits
**Unknowns**:
- How many suspended workflows can Kubernetes/etcd handle?
- What's our cleanup strategy for completed workflows?
- Are there memory leaks in long-running workflows?
- Do we need resource quotas or limits?

#### 6. Security Deep Dive
**Concerns**:
- **Tess cluster-admin access**: Extremely broad permissions, security team approval needed
- **Webhook authentication**: How do we verify webhooks are actually from GitHub?
- **Agent isolation**: Can agents access each other's secrets/workspaces?
- **Network policies**: Should agents have restricted network access?

### 🔧 Technical Edge Cases

#### 7. Webhook Reliability & Failure Modes
**Scenarios not addressed**:
- GitHub webhook fails/times out → workflow stuck suspended forever
- Duplicate webhooks → multiple resume attempts
- Webhooks arrive out of order → wrong stage resumed  
- GitHub API rate limiting → agent failures
- Argo Events down when webhook arrives → lost events

#### 8. Agent Failure Scenarios
**What happens if**:
- Agent crashes mid-execution → workflow state unclear
- Agent violates labeling conventions → correlation breaks
- Agent can't access required resources → deployment testing fails
- Agent exceeds timeout → partial work lost
- Multiple agents try to work on same PR simultaneously

#### 9. Concurrency & Scaling Questions
**Unclear scenarios**:
- Multiple tasks running simultaneously → resource conflicts?
- Multiple repositories → agent confusion?
- High-frequency PR activity → agent overload?
- Large codebase changes → long agent execution times?

### 🔍 Missing Operational Details

#### 10. User Experience & Monitoring
**Gaps**:
- How does user track progress across multiple suspended workflows?
- What's the interface for manual intervention/debugging?
- How do we report overall project completion status?
- What notifications/alerts go to the user?

#### 11. Integration Dependencies
**External failure points**:
- MCP documentation server unavailable → agents can't research
- GitHub App token refresh failures → authentication breaks
- Kubernetes resource constraints → pod scheduling fails
- Argo CD/Workflows version incompatibilities

#### 12. Data Consistency & State Management
**Potential issues**:
- Task marker files conflicting between concurrent workflows
- Agent workspace corruption → lost context
- GitHub PR state changes outside workflow control
- Task directory modifications during workflow execution

### 📊 Validation & Testing Strategy

#### 13. Missing Testing Plan
**Critical need**: How do we test/validate this complex system?
- End-to-end workflow testing across days/weeks
- Failure scenario testing (webhook failures, agent crashes, etc.)
- Performance testing with multiple concurrent workflows
- Security testing for Tess's broad permissions

### 🎯 Actual Implementation Requirements (Lab Environment)

**Simplified Priority List:**

1. **Argo Events Proof of Concept** - Build minimal sensor/eventsource to validate correlation works
2. **Template Implementation** - Add actual Handlebars conditionals for agent-specific behavior  
3. **Controller PVC Naming** - Extract agent name from `github_app` for unique workspace PVCs
4. **Agent System Prompts** - Create Cleo and Tess specific prompt templates
5. **Rex Remediation Logic** - Event-based cancellation of running agents when Rex pushes fixes

**Assumptions for Lab Environment:**
- ✅ Webhook reliability - Assume GitHub webhooks work (handle failures later)
- ✅ Security model - Cluster-admin fine for isolated lab testing  
- ✅ Operational concerns - Learn by doing, iterate as needed
- ✅ Resource limits - Start small, scale as needed

**Out of Scope for Now:**
- ~~Comprehensive error handling~~ (Handle in production later)
- ~~Advanced security~~ (Lab environment)  
- ~~Monitoring/alerting~~ (Your responsibility later)
- ~~Resource optimization~~ (Learn by running)

## Discovery Phase Requirements

### 🔍 Critical Infrastructure Analysis Needed

#### 1. Current CodeRun Controller Deep Dive
**Understanding Required:**
- How does the controller process CRD specs into running pods?
- Template loading and Handlebars rendering pipeline
- PVC creation and naming logic (`workspace-{service}`)
- ConfigMap generation for agent files (CLAUDE.md, settings.json, etc.)
- Secret mounting patterns for GitHub Apps
- Pod lifecycle management and status reporting
- Session continuity implementation (`continue_session` flag behavior)

**Key Files to Study:**
- `controller/src/tasks/code/controller.rs` - Main CRD processing logic
- `controller/src/tasks/code/templates.rs` - Template generation system
- `controller/src/tasks/code/resources.rs` - K8s resource creation
- `controller/src/crds/coderun.rs` - CRD specification and validation

#### 2. Existing Argo Events Integration
**Understanding Required:**
- Current GitHub webhook → Argo Workflows setup
- EventSource and Sensor configurations already deployed
- How GitHub App events are currently processed
- Webhook payload parsing and parameter extraction
- Correlation mechanisms (if any) currently in use

**Key Areas to Investigate:**
- `infra/resources/github-webhooks/` - Current webhook setup
- Deployed EventSources and Sensors in cluster
- Integration with existing `coderun-template.yaml`

#### 3. Template System Architecture
**Understanding Required:**
- Template file organization and loading
- Handlebars context data structure
- How agent-specific data flows to templates
- ConfigMap mounting and file structure in pods
- Hook script system and execution

**Template Files to Analyze:**
- `infra/charts/controller/claude-templates/code/` - All template files
- Template inheritance and override patterns
- Variable substitution mechanisms

#### 4. GitHub App Integration Patterns  
**Understanding Required:**
- How GitHub App credentials flow from secrets to agents
- App ID, Client ID, Private Key usage patterns
- Authentication flow in agent containers
- GitHub API interaction patterns

**Secret Analysis Required:**
- `github-app-5dlabs-rex`, `github-app-5dlabs-morgan` secret structures
- External Secrets integration and sync patterns

#### 5. Agent Container Environment
**Understanding Required:**
- Container startup sequence in `container.sh.hbs`
- How CLAUDE.md persistence works with PVCs
- MCP server integration and tool availability
- Claude API authentication and model selection
- Workspace directory structure and file management

#### 6. Workflow Orchestration Patterns
**Understanding Required:**
- How `coderun-template.yaml` creates and monitors CRDs
- Workflow timeout and cleanup mechanisms
- Parameter passing between workflow steps
- Status checking and completion detection

### 📋 Discovery Tasks Checklist

#### Phase 1A: Controller Analysis
- [ ] Trace CodeRun CRD processing from submission to pod creation
- [ ] Map template system data flow and context building
- [ ] Understand PVC naming and management logic
- [ ] Document secret mounting and environment setup
- [ ] Analyze session continuity implementation

#### Phase 1B: Event System Analysis  
- [ ] Document current Argo Events setup and configurations
- [ ] Test GitHub webhook delivery to existing sensors
- [ ] Understand correlation mechanisms in use
- [ ] Map event payload to workflow parameter flow

#### Phase 1C: Template System Analysis
- [ ] Catalog all existing template files and their purposes
- [ ] Document Handlebars context structure and available variables
- [ ] Understand ConfigMap generation and mounting
- [ ] Test template modification and reload procedures

#### Phase 1D: Integration Pattern Analysis
- [ ] Document GitHub App authentication flow end-to-end  
- [ ] Test secret management and External Secrets sync
- [ ] Understand agent container startup and environment setup
- [ ] Map MCP tool availability and configuration

#### Phase 1E: Gap Analysis
- [ ] Identify exactly what needs modification vs. new implementation
- [ ] Document breaking changes and compatibility requirements
- [ ] Estimate complexity of each required change
- [ ] Plan incremental implementation strategy

### 🎯 Discovery Deliverables

1. **Architecture Documentation** - How existing system works
2. **Modification Requirements** - Exact changes needed for multi-agent support
3. **Compatibility Assessment** - Impact on existing Rex/Blaze workflows  
4. **Implementation Roadmap** - Detailed technical tasks with dependencies
5. **Test Strategy** - How to validate changes without breaking existing functionality

### 📝 Implementation Notes

**Lab Environment Approach:**
- Complete discovery phase before any implementation
- Start with basic implementation and iterate based on real-world learnings
- Focus on proving core concepts work before optimizing for production
- Security hardening and operational concerns addressed in later phases
- Webhook reliability and error handling to be implemented as operational needs arise

**Key Implementation Phases:**
1. **Discovery Phase**: Deep dive into existing infrastructure and dependencies
2. **Proof of Concept**: Validate Argo Events correlation with minimal test
3. **Template System**: Add agent-specific conditionals and prompts  
4. **Controller Updates**: Agent-specific PVC naming and permissions
5. **End-to-End Testing**: Full Rex → Cleo → Tess workflow validation
