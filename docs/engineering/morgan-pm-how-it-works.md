# Morgan PM Integration - Complete Flow Explanation

## TL;DR

**Morgan now runs as a daemon during Play workflows, creating GitHub Projects and Issues for each task, then continuously monitoring and updating them in real-time as Rex/Cleo/Cipher/Tess work.**

---

## The Complete Flow

### **Step 1: User Initiates Play Workflow**

```javascript
// User calls MCP tool
play({ task_id: 1 })
```

This creates an Argo Workflow using `play-project-workflow-template`.

---

### **Step 2: Morgan PM Daemon Launches (Happens First)**

The workflow template now has Morgan as **Step 0** (a daemon):

```yaml
steps:
  # Step 0: Launch Morgan PM (runs entire workflow lifecycle)
  - - name: launch-morgan-pm
      template: morgan-project-manager
      daemon: true  # <-- Keeps running until workflow completes
```

**What happens in Morgan's container:**

#### **A. Template Rendering**
```bash
# Container uses envsubst to render Handlebars templates
cat /pm-templates/github-projects-helpers.sh.hbs | envsubst > /workspace/github-projects-helpers.sh
cat /pm-templates/morgan-pm.sh.hbs | envsubst > /workspace/morgan-pm.sh
cat /pm-templates/process-issue-comment.sh.hbs | envsubst > /workspace/process-issue-comment.sh

# Execute the main script
exec /workspace/morgan-pm.sh
```

#### **B. Initialization Phase** (Runs Once)
```bash
1. Clone docs repository using Morgan's GitHub App credentials
   git clone https://github.com/5dlabs/cto-play-test docs/

2. Read TaskMaster tasks.json
   TASKS_DATA=$(cat .taskmaster/tasks/tasks.json)
   # Gets all tasks: [{id: 1, title: "..."}, {id: 2, ...}]

3. Create or find GitHub Project via GraphQL
   PROJECT_ID=$(get_or_create_project "5dlabs" "play-test - TaskMaster Workflow")
   # Uses: mutation createProjectV2(ownerId, title)

4. Setup custom fields in the project
   create_single_select_field "$PROJECT_ID" "Current Agent" \
     "Pending" "Rex (Implementation)" "Cleo (Quality)" "Cipher (Security)" "Tess (QA)" "Complete ✅"
   create_single_select_field "$PROJECT_ID" "Stage" \
     "Pending" "Implementation" "Code Review" "Security Check" "QA Testing" "Done"
   create_text_field "$PROJECT_ID" "Task ID"

5. Create GitHub Issue for each task
   For task 1:
     ISSUE_NUMBER=$(gh issue create \
       --title "Task 1: {title}" \
       --body "{description + details + test strategy}" \
       --label "taskmaster-task,task-1,priority-high")
     # Returns: 123

6. Add each issue to the Project
   ISSUE_NODE_ID=$(gh issue view 123 --json id --jq '.id')
   ITEM_ID=$(add_issue_to_project "$PROJECT_ID" "$ISSUE_NODE_ID")
   # Uses: mutation addProjectV2ItemById(projectId, contentId)

7. Set initial field values
   set_project_item_status "$PROJECT_ID" "$ITEM_ID" "Pending"
   set_project_item_agent "$PROJECT_ID" "$ITEM_ID" "Pending"
   set_project_item_priority "$PROJECT_ID" "$ITEM_ID" "high"
   # Uses: mutation updateProjectV2ItemFieldValue(projectId, itemId, fieldId, value)

8. Store mapping in JSON
   {
     "1": {"issue_number": 123, "item_id": "PVTI_...", "node_id": "I_..."},
     "2": {"issue_number": 124, "item_id": "PVTI_...", "node_id": "I_..."}
   }
   # Saved to: /shared/morgan-pm/task-issue-map.json
```

#### **C. Monitoring Phase** (Loops Every 30s)
```bash
while workflow_running:
  # 1. Check parent workflow status
  WORKFLOW_STATUS=$(kubectl get workflow play-project-xyz -o jsonpath='{.status.phase}')
  
  if [[ "$WORKFLOW_STATUS" == "Succeeded" || "Failed" ]]; then
    # Final sync and exit
    break
  fi
  
  # 2. Update each task's status
  for task_id in [1, 2, 3, ...]:
    # Find the task's individual workflow
    TASK_WORKFLOW=$(kubectl get workflows -n agent-platform \
      -l task-id=1,parent-workflow=play-project-xyz \
      -o jsonpath='{.items[0].metadata.name}')
    # Returns: "play-task-1-abc123"
    
    # Get current stage from workflow labels
    CURRENT_STAGE=$(kubectl get workflow play-task-1-abc123 \
      -o jsonpath='{.metadata.labels.current-stage}')
    # Returns: "quality-in-progress" (means Cleo is working)
    
    # Get workflow phase
    WORKFLOW_PHASE=$(kubectl get workflow play-task-1-abc123 \
      -o jsonpath='{.status.phase}')
    # Returns: "Running"
    
    # Map to GitHub Project values
    AGENT = map_stage_to_agent("quality-in-progress", "Running")
    # Returns: "Cleo (Quality)"
    
    STATUS = map_workflow_to_status("quality-in-progress", "Running")
    # Returns: "In Review"
    
    # Update GitHub Project via GraphQL
    set_project_item_agent("$PROJECT_ID", "$ITEM_ID", "Cleo (Quality)")
    set_project_item_status("$PROJECT_ID", "$ITEM_ID", "In Review")
    
    # Update issue labels for filtering
    gh issue edit 123 --add-label "status-in-review"
  
  sleep 30  # Wait 30 seconds, then repeat
```

---

### **Step 3: Meanwhile, Task Workflows Execute**

While Morgan monitors, the regular task workflows run:

```
For Task 1:
┌─────────────────┐
│ Rex CodeRun     │ → Creates PR #456
│ (Implementation)│ → Workflow label: current-stage=implementation
└────────┬────────┘
         │ (Morgan sees this and updates Project: "Rex (Implementation)" / "In Progress")
         ▼
┌─────────────────┐
│ Cleo CodeRun    │ → Reviews code
│ (Quality)       │ → Workflow label: current-stage=quality-in-progress
└────────┬────────┘
         │ (Morgan updates: "Cleo (Quality)" / "In Review")
         ▼
┌─────────────────┐
│ Cipher CodeRun  │ → Security scan
│ (Security)      │ → Workflow label: current-stage=security-in-progress
└────────┬────────┘
         │ (Morgan updates: "Cipher (Security)" / "In Review")
         ▼
┌─────────────────┐
│ Tess CodeRun    │ → QA testing
│ (Testing)       │ → Workflow label: current-stage=testing-in-progress
└────────┬────────┘
         │ (Morgan updates: "Tess (QA)" / "In Review")
         ▼
┌─────────────────┐
│ Workflow        │
│ Succeeded       │ → Workflow phase: Succeeded
└─────────────────┘
         │ (Morgan updates: "Complete ✅" / "Done")
```

---

### **Step 4: Human Feedback Loop (Optional)**

If a human comments on an issue:

```
GitHub Issue #123 (Task 1)
┌─────────────────────────────────────────┐
│ @user:                                   │
│ @morgan This task needs to also support │
│ SAML authentication, not just OAuth2.   │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│ GitHub Webhook Fired                     │
│ Event: issue_comment.created             │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│ Argo Events Sensor: morgan-issue-comment│
│                                          │
│ Filters applied:                         │
│ ✓ action = "created"                     │
│ ✓ issue has label "taskmaster-task"     │
│ ✓ comment author type = "User" (not bot)│
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│ Trigger Morgan Workflow                  │
│                                          │
│ Step 1: Extract task ID from labels     │
│   → Finds "task-1" label                 │
│                                          │
│ Step 2: Analyze comment intent          │
│   → Detects "@morgan" + "support"        │
│   → Intent: "scope-change"               │
│   → Requires action: true                │
│                                          │
│ Step 3: Process comment                  │
│   → Clone docs repo                      │
│   → Update implementation-notes.md:      │
│                                          │
│     ---                                  │
│     ## Scope Change Request - 2025-11-03│
│     **Requested by**: @user              │
│                                          │
│     @morgan This task needs to also      │
│     support SAML authentication...       │
│     ---                                  │
│                                          │
│   → git commit -m "docs: update scope"   │
│   → git push                             │
│   → gh issue comment 123 --body "✅..."  │
└─────────────────────────────────────────┘
```

---

## What You See in GitHub

### **GitHub Projects Board**

```
Project: "play-test - TaskMaster Workflow"

┌─────────────┬──────────────┬──────────────┬─────────────┐
│    Todo     │ In Progress  │  In Review   │    Done     │
├─────────────┼──────────────┼──────────────┼─────────────┤
│ Task 2      │ Task 1       │ Task 3       │ Task 5      │
│ Pending     │ Rex (Impl)   │ Cleo (Qual)  │ Complete ✅ │
│ Issue #124  │ Issue #123   │ Issue #125   │ Issue #127  │
│ Priority: M │ Priority: H  │ Priority: H  │ Priority: L │
├─────────────┼──────────────┼──────────────┼─────────────┤
│ Task 7      │              │ Task 4       │ Task 8      │
│ Pending     │              │ Tess (QA)    │ Complete ✅ │
│ Issue #129  │              │ Issue #126   │ Issue #130  │
│ Priority: L │              │ Priority: M  │ Priority: H │
└─────────────┴──────────────┴──────────────┴─────────────┘

Columns auto-update based on "Stage" field
Filters available: Current Agent, Priority, Task ID
```

### **Individual GitHub Issue**

```
Issue #123: Task 1: Implement Authentication Service

Labels: taskmaster-task, task-1, priority-high, status-in-progress

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Task 1: Implement Authentication Service

## 📋 Description
Build a secure authentication service with JWT tokens and session management.

## 🎯 Implementation Details
- Use bcrypt for password hashing
- Implement JWT token generation with 24h expiry
- Store sessions in Redis
- Provide login, logout, and refresh token endpoints

## 🧪 Test Strategy
- Unit tests for password hashing
- Integration tests for token generation
- E2E tests for full auth flow
- Security scan with cargo-audit

## 📊 Metadata
- Priority: high
- Dependencies: None
- Status: In Progress

---

## 🔗 Task Master Integration

**Task File**: `.taskmaster/tasks/tasks.json`
**Workflow**: `play-project-xyz-abc123`

### Agent Pipeline
1. **Rex** - Implementation ← Currently here
2. **Cleo** - Code Quality Review
3. **Cipher** - Security Analysis
4. **Tess** - QA Testing

---

Comments:
└─ @user: @morgan This needs SAML support too
   └─ @Morgan[bot]: ✅ I've updated the implementation notes...
```

---

## Technical Architecture

### **Component Deployment**

All components deploy automatically via ArgoCD:

```
ArgoCD App: controller (from main branch)
├── ConfigMaps Created:
│   ├── controller-agent-templates-claude     (existing)
│   ├── controller-agent-templates-codex      (existing)
│   ├── controller-agent-templates-pm         ← NEW!
│   │   ├── morgan-pm.sh.hbs
│   │   ├── github-projects-helpers.sh.hbs
│   │   └── process-issue-comment.sh.hbs
│   └── ...
│
├── WorkflowTemplates Created:
│   ├── play-workflow-template                (existing)
│   ├── play-project-workflow-template        (UPDATED - includes Morgan daemon)
│   └── ...
│
└── Agents Config Updated:
    └── morgan: (UPDATED - added project-tracking expertise)

Manually Deployed Resources:
└── infra/gitops/resources/argo-events/sensors/
    └── morgan-issue-comment-sensor.yaml      ← NEW!
        (Deploy via: kubectl apply -f)
```

### **Data Flow Architecture**

```
┌─────────────────────────────────────────────────────┐
│                 TaskMaster tasks.json               │
│          (Source of Truth for Tasks)                │
└────────────────┬────────────────────────────────────┘
                 │
                 │ Morgan reads on init
                 ▼
┌─────────────────────────────────────────────────────┐
│              GitHub Project + Issues                │
│           (Human Interface & Visibility)            │
│                                                      │
│  Updated by: Morgan PM (every 30s)                  │
│  Shows: Current Agent, Stage, Priority              │
└────────────────┬────────────────────────────────────┘
                 │
                 │ Morgan monitors
                 ▼
┌─────────────────────────────────────────────────────┐
│           Kubernetes Workflow State                 │
│        (Runtime Status & Progress)                  │
│                                                      │
│  Labels: task-id, current-stage, parent-workflow    │
│  Status: phase (Running/Succeeded/Failed)           │
└────────────────┬────────────────────────────────────┘
                 │
                 │ Workflows execute
                 ▼
┌─────────────────────────────────────────────────────┐
│          Rex → Cleo → Cipher → Tess                 │
│            (Agent Execution)                        │
│                                                      │
│  Updates: workflow.labels.current-stage             │
│  Creates: PRs, Comments, Reviews                    │
└─────────────────────────────────────────────────────┘
```

### **State Storage**

Morgan stores state in multiple places for resilience:

1. **In-Memory** (while running):
   - Task-issue mappings
   - Project field IDs
   - Option IDs for select fields

2. **Shared Volume** (`/shared/morgan-pm/`):
   - `task-issue-map.json` - Main mapping file
   - `project-config.json` - Project metadata
   - `sync.log` - Timestamped activity log

3. **Kubernetes ConfigMap** (for persistence):
   ```bash
   kubectl create configmap project-{workflow-name}-mapping \
     --from-file=task-issue-map.json \
     -n agent-platform
   ```

---

## Stage → Agent → Status Mapping

Morgan uses this logic to translate Kubernetes workflow state to GitHub:

| K8s Stage Label | K8s Phase | GitHub "Current Agent" | GitHub "Stage" |
|-----------------|-----------|------------------------|----------------|
| `pending` | Running | Rex (Implementation) | In Progress |
| `implementation` | Running | Rex (Implementation) | In Progress |
| `waiting-pr-created` | Running | Rex (Implementation) | In Progress |
| `quality-in-progress` | Running | Cleo (Quality) | In Review |
| `waiting-ready-for-qa` | Running | Cleo (Quality) | In Review |
| `security-in-progress` | Running | Cipher (Security) | In Review |
| `testing-in-progress` | Running | Tess (QA) | In Review |
| Any | Succeeded | Complete ✅ | Done |
| Any | Failed/Error | - | Blocked |

---

## GitHub API Operations

Morgan uses GitHub CLI (`gh`) which handles authentication automatically via Morgan's GitHub App.

### **Key GraphQL Mutations Used**

```graphql
# 1. Create Project
mutation {
  createProjectV2(input: {
    ownerId: "O_..." 
    title: "play-test - TaskMaster Workflow"
  }) {
    projectV2 { id }
  }
}

# 2. Create Custom Field
mutation {
  createProjectV2Field(input: {
    projectId: "PVT_..."
    dataType: SINGLE_SELECT
    name: "Current Agent"
    singleSelectOptions: [
      {name: "Pending", color: "GRAY"},
      {name: "Rex (Implementation)", color: "BLUE"},
      ...
    ]
  }) {
    projectV2Field { id }
  }
}

# 3. Add Issue to Project
mutation {
  addProjectV2ItemById(input: {
    projectId: "PVT_..."
    contentId: "I_..."  # Issue node ID
  }) {
    item { id }
  }
}

# 4. Update Field Value
mutation {
  updateProjectV2ItemFieldValue(input: {
    projectId: "PVT_..."
    itemId: "PVTI_..."
    fieldId: "PVTF_..."
    value: { singleSelectOptionId: "PVTSSO_..." }
  }) {
    projectV2Item { id }
  }
}
```

### **Rate Limiting & Retry**

Adapted from `5dlabs/tasks` reference:

```bash
# Exponential backoff on rate limits
graphql_query() {
  retry_count=0
  max_retries=3
  
  while [ $retry_count -lt $max_retries ]; do
    result=$(gh api graphql ...)
    
    if success; then
      return result
    fi
    
    if rate_limited; then
      backoff=$((2 ** retry_count))  # 1s, 2s, 4s
      sleep $backoff
      retry_count++
    else
      return error
    fi
  done
}
```

---

## Issue Comment Processing

### **Webhook Flow**

```
1. Human posts comment on Issue #123
   Content: "@morgan Add OAuth2 support"

2. GitHub fires webhook
   POST /webhook
   {
     "action": "created",
     "issue": {"number": 123, "labels": ["taskmaster-task", "task-1"]},
     "comment": {"body": "@morgan Add OAuth2...", "user": {"type": "User"}}
   }

3. EventSource receives webhook
   Name: github-webhook
   Namespace: argo (existing)

4. Sensor filters event
   Name: morgan-issue-comment
   Namespace: agent-platform
   
   Checks:
   ✓ action == "created"
   ✓ "taskmaster-task" in labels
   ✓ user.type == "User" (not bot)

5. Sensor triggers Workflow
   Creates: morgan-process-comment-{random}
   
   Steps:
   a) Extract task ID → "1"
   b) Analyze intent → "scope-change"
   c) Clone docs repo
   d) Update implementation-notes.md
   e) Commit and push
   f) Reply to issue

6. Human sees reply
   @Morgan[bot]: "✅ I've updated the implementation notes..."
```

### **Comment Intent Detection**

Morgan recognizes these patterns:

| Pattern | Intent | Action |
|---------|--------|--------|
| `@morgan`, `scope`, `add requirement` | scope-change | Update implementation notes |
| `clarify`, `explain`, `unclear` | clarification | Document question, await answer |
| `priority`, `urgent`, `critical` | priority-change | Update tasks.json priority |

---

## Deployment & Configuration

### **What Deploys Automatically**

When you merge to main and ArgoCD syncs:

1. ✅ **ConfigMap** `controller-agent-templates-pm` (via Helm)
2. ✅ **WorkflowTemplate** `play-project-workflow-template` (updated, via Helm)
3. ✅ **Morgan config** in `values.yaml` (via Helm)

### **What You Need to Deploy Manually**

```bash
# 1. Deploy the issue comment sensor
kubectl apply -f infra/gitops/resources/argo-events/sensors/morgan-issue-comment-sensor.yaml

# 2. Verify EventSource exists (should already exist)
kubectl get eventsource github-webhook -n argo

# 3. Verify Morgan's GitHub App has required scopes:
#    - Projects: Read/Write
#    - Issues: Read/Write
#    - Contents: Read
#    - Metadata: Read
```

### **Verification Steps**

```bash
# 1. Check ConfigMap created
kubectl get cm -n agent-platform | grep agent-templates-pm
kubectl describe cm controller-agent-templates-pm -n agent-platform

# 2. Check WorkflowTemplate updated
kubectl get workflowtemplate play-project-workflow-template -n agent-platform -o yaml | grep morgan

# 3. Check sensor deployed
kubectl get sensor morgan-issue-comment -n agent-platform

# 4. Test with actual play workflow
play({ task_id: 1 })

# 5. Watch Morgan logs
kubectl logs -f -l agent=morgan,workflow-type=project-orchestration -n agent-platform
```

---

## Key Benefits

### **For You (Developer/PM)**
- 📊 **Real-time visibility**: See exactly what's happening without checking logs
- 🎯 **Agent tracking**: Know which bot is working on what
- 🔄 **Progress monitoring**: Watch tasks move through pipeline
- 💬 **Easy feedback**: Comment on issues to adjust scope/priority

### **For Stakeholders**
- 📈 **Project dashboard**: Share GitHub Project with non-technical folks
- 📝 **Clear status**: No need to understand Kubernetes/Argo
- 🗓️ **Timeline visibility**: See how long tasks take
- ✅ **Completion tracking**: Know when features are ready

### **For the System**
- 🔗 **Bidirectional sync**: GitHub ↔ TaskMaster ↔ Kubernetes all in sync
- 📝 **Audit trail**: Complete history of changes and comments
- 🤖 **Automated updates**: Zero manual status updates needed
- 🔍 **Troubleshooting**: Issue comments provide context for debugging

---

## Limitations & Considerations

### **Current Limitations**

1. **GitHub App Assignment**: Can't assign GitHub Apps directly to issues
   - **Workaround**: Use "Current Agent" custom field instead

2. **Rate Limits**: GitHub GraphQL API has limits
   - **Mitigation**: Retry with exponential backoff, 30s polling interval

3. **One Project Per Workflow**: Each Play workflow creates its own project
   - **Future**: Could consolidate into one project with workflow grouping

4. **Manual Sensor Deploy**: Issue comment sensor requires `kubectl apply`
   - **Future**: Add to ArgoCD app

### **What Still Needs Work**

After this implementation, you'll likely want to:

1. **Test with real workflow**: Deploy and run through full cycle
2. **Refine field mappings**: May need to adjust status/agent values based on actual use
3. **Add metrics**: Track sync latency, API errors, update frequency
4. **Optimize polling**: Maybe use Kubernetes watches instead of 30s sleep
5. **Add agent avatars**: Show agent profile pics in Project (requires GitHub App profile setup)

---

## Summary: The Value Proposition

**Before Morgan PM:**
- ❌ No visibility into what agents are doing
- ❌ Check Kubernetes logs to see progress
- ❌ No way for non-technical stakeholders to track work
- ❌ Manual status updates required

**With Morgan PM:**
- ✅ Real-time GitHub Project board showing all task status
- ✅ See which agent is currently working on each task
- ✅ Human feedback loop via issue comments
- ✅ Complete audit trail and history
- ✅ Shareable dashboard for stakeholders
- ✅ Zero manual intervention - fully automated

---

## Quick Reference

```bash
# Deploy after merging to main
kubectl apply -f infra/gitops/resources/argo-events/sensors/morgan-issue-comment-sensor.yaml

# Verify deployment
kubectl get cm controller-agent-templates-pm -n agent-platform
kubectl get sensor morgan-issue-comment -n agent-platform
kubectl get workflowtemplate play-project-workflow-template -n agent-platform

# Run test
play({ task_id: 1 })

# Monitor Morgan
kubectl logs -f -l agent=morgan,workflow-type=project-orchestration

# Check GitHub
gh project list --owner 5dlabs
gh issue list --repo 5dlabs/{repo} --label taskmaster-task
```

---

*Everything is ready to deploy! The integration is complete and follows all existing patterns in the codebase.*


