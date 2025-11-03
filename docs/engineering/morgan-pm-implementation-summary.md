# Morgan PM Integration - Implementation Summary

**Status**: ✅ Complete - Ready for Testing  
**Date**: November 3, 2025  
**Branch**: feat/add-atlas-bolt-agents

---

## What Was Built

### **1. Core PM Templates** (Handlebars → ConfigMap)

**Location**: `infra/charts/controller/agent-templates/pm/`

- `morgan-pm.sh.hbs` - Main PM daemon script
  - Initializes GitHub Project
  - Creates issues for all tasks
  - Runs continuous monitoring loop
  - Updates Project fields in real-time

- `github-projects-helpers.sh.hbs` - GraphQL API helpers
  - Project creation/lookup
  - Custom field setup
  - Issue-to-project linking
  - Field value updates
  - Retry logic with exponential backoff

- `process-issue-comment.sh.hbs` - Human feedback processor
  - Handles scope changes
  - Processes clarifications
  - Updates priority
  - Commits changes to docs repo

### **2. Workflow Integration**

**File**: `infra/charts/controller/templates/workflowtemplates/play-project-workflow-template.yaml`

**Changes**:
- Added `volumeClaimTemplates` for Morgan PM workspace
- Added Step 0: `launch-morgan-pm` (daemon template)
- Morgan runs throughout entire workflow lifecycle
- Uses `daemon: true` - no extra compute cost

### **3. Event-Driven Feedback**

**File**: `infra/gitops/resources/argo-events/sensors/morgan-issue-comment-sensor.yaml`

**Capabilities**:
- Triggers on GitHub issue comments
- Filters for `taskmaster-task` labeled issues
- Ignores bot comments (prevents loops)
- Analyzes comment intent (scope/priority/clarification)
- Launches Morgan workflow to process feedback

### **4. Agent Configuration**

**File**: `infra/charts/controller/values.yaml`

**Updates to Morgan**:
- Added expertise: `project-tracking`, `github-projects`
- Enhanced system prompt for PM mode
- Documented PM-specific behaviors

### **5. ConfigMap Template**

**File**: `infra/charts/controller/templates/agent-templates-pm.yaml`

- Packages PM templates into Kubernetes ConfigMap
- Follows existing agent template patterns
- Auto-deployed via Helm/ArgoCD

### **6. Documentation**

- `docs/engineering/morgan-pm-github-projects-integration.md` - Architecture & design
- `docs/engineering/morgan-pm-how-it-works.md` - Complete flow explanation
- `docs/engineering/morgan-pm-implementation-summary.md` - This file

---

## How It Works (Executive Summary)

```
User calls play({ task_id: 1 })
  ↓
Play Project Workflow starts
  ↓
┌─────────────────────────────────────┐
│ Step 0: Morgan PM Daemon (launches) │
│ - Creates GitHub Project            │
│ - Creates Issues (1 per task)       │
│ - Starts 30s monitoring loop        │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│ Step 1-3: Task Workflows Execute    │
│ Rex → Cleo → Cipher → Tess          │
└─────────────────────────────────────┘
  ↓
Morgan monitors via kubectl:
  - Gets workflow.labels.current-stage
  - Maps to agent (Rex/Cleo/Tess)
  - Updates GitHub Project fields
  - Updates issue labels
  - Repeats every 30s

Result: GitHub Project shows real-time status
```

---

## What You Get

### **GitHub Projects Board**
- Column per status (Todo/In Progress/In Review/Done)
- Custom fields showing current agent, stage, priority
- Each task = 1 GitHub Issue
- Updates automatically every 30 seconds

### **GitHub Issues**
- Rich task details from TaskMaster
- Comments for human feedback
- Labels for filtering
- Linked to Project for dashboard view

### **Human Feedback Loop**
- Comment `@morgan Add feature X` → Morgan updates docs
- Comment `Priority: high` → Morgan updates priority
- Comment `Clarify: what about Y?` → Morgan documents question

---

## Deployment Checklist

### **Automatic** (via ArgoCD on merge to main)
- [x] ConfigMap `controller-agent-templates-pm`
- [x] Updated `play-project-workflow-template`
- [x] Updated Morgan agent config

### **Manual** (after merge)
```bash
# 1. Deploy issue comment sensor
kubectl apply -f infra/gitops/resources/argo-events/sensors/morgan-issue-comment-sensor.yaml

# 2. Verify GitHub webhook EventSource exists
kubectl get eventsource github-webhook -n argo

# 3. Ensure Morgan GitHub App has permissions:
#    - Projects: Read/Write
#    - Issues: Read/Write
#    - Contents: Read
```

### **Verification**
```bash
# Check ConfigMap
kubectl get cm controller-agent-templates-pm -n agent-platform
kubectl describe cm controller-agent-templates-pm -n agent-platform | head -20

# Check sensor
kubectl get sensor morgan-issue-comment -n agent-platform

# Check workflow template
kubectl get workflowtemplate play-project-workflow-template -n agent-platform -o yaml | grep -A20 morgan-project-manager
```

---

## Testing Plan

### **Phase 1: Smoke Test**
```bash
# 1. Deploy everything
git add .
git commit -m "feat: add Morgan PM GitHub Projects integration"
git push origin feat/add-atlas-bolt-agents

# After ArgoCD syncs...

# 2. Deploy sensor manually
kubectl apply -f infra/gitops/resources/argo-events/sensors/morgan-issue-comment-sensor.yaml

# 3. Run simple play workflow
play({ task_id: 1 })

# 4. Monitor Morgan logs
kubectl logs -f -l agent=morgan -n agent-platform

# 5. Check GitHub
#    - Verify project created
#    - Verify issues created
#    - Watch fields update
```

### **Phase 2: Integration Test**
- Let workflow run through all agents
- Verify status updates at each stage
- Test human comment processing
- Verify bidirectional sync works

### **Phase 3: Load Test**
- Run with 10+ tasks
- Verify rate limiting works
- Check performance of 30s polling
- Validate memory usage stays stable

---

## Potential Issues & Solutions

### **Issue: Project creation fails**
```bash
# Check Morgan GitHub App permissions
gh api /app/installations/{id} | jq '.permissions'

# Verify auth works
gh auth status

# Check for existing project
gh api graphql -f query='...' | jq '.data.repositoryOwner.projectsV2'
```

### **Issue: Field updates not visible**
```bash
# Check Morgan monitoring loop
kubectl logs -l agent=morgan | grep "Iteration"

# Verify workflow labels exist
kubectl get workflow play-task-1-xyz -o json | jq '.metadata.labels'

# Check GraphQL errors
kubectl logs -l agent=morgan | grep "GraphQL\|error\|failed"
```

### **Issue: Comment processing not triggering**
```bash
# Check sensor status
kubectl describe sensor morgan-issue-comment -n agent-platform

# Check EventSource
kubectl get eventsource github-webhook -n argo -o yaml

# Verify webhook delivery
# GitHub repo → Settings → Webhooks → Recent Deliveries
```

---

## Files Modified/Created

### **New Files**
```
infra/charts/controller/
├── agent-templates/pm/
│   ├── morgan-pm.sh.hbs
│   ├── github-projects-helpers.sh.hbs
│   └── process-issue-comment.sh.hbs
└── templates/
    └── agent-templates-pm.yaml

infra/gitops/resources/argo-events/sensors/
└── morgan-issue-comment-sensor.yaml

docs/engineering/
├── morgan-pm-github-projects-integration.md
├── morgan-pm-how-it-works.md
└── morgan-pm-implementation-summary.md
```

### **Modified Files**
```
infra/charts/controller/
├── templates/workflowtemplates/
│   └── play-project-workflow-template.yaml
│       - Added volumeClaimTemplates
│       - Added Step 0: launch-morgan-pm
│       - Added morgan-project-manager template
└── values.yaml
    └── agents.morgan
        - Added project-tracking expertise
        - Enhanced system prompt for PM mode
```

---

## Next Steps

1. **Merge to main** (creates PR per workflow)
2. **Wait for ArgoCD sync** (~2 minutes)
3. **Deploy sensor manually** (`kubectl apply -f ...`)
4. **Run test workflow** (`play({ task_id: 1 })`)
5. **Verify in GitHub** (check Project and Issues created)
6. **Test human feedback** (comment on issue, verify Morgan responds)

---

## Success Criteria

- [ ] Morgan PM daemon starts when Play workflow begins
- [ ] GitHub Project created automatically
- [ ] Issues created for all tasks
- [ ] Custom fields setup (Current Agent, Stage, Task ID, Priority)
- [ ] Status updates every 30 seconds
- [ ] Agent field changes as workflow progresses
- [ ] Issue comments trigger Morgan processing
- [ ] Scope changes update task docs
- [ ] Morgan replies to comments confirming updates

---

*Implementation complete! Ready for deployment and testing.*

