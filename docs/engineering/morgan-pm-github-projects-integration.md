## Morgan PM - GitHub Projects Integration Design

**Agent**: Morgan (5DLabs-Morgan)  
**Role**: Project Manager & Documentation Specialist  
**New Capability**: Real-time GitHub Projects tracking and issue management

---

## Overview

Morgan now runs as a **continuous daemon** throughout Play workflows, managing GitHub Projects integration to provide real-time visibility into multi-agent development progress.

### Key Features

1. **Automatic Project Creation**: Creates GitHub Project for each Play workflow
2. **Issue-Task Mapping**: Creates GitHub Issues for each TaskMaster task
3. **Real-Time Status Updates**: Monitors workflow progress and updates Project fields
4. **Human Feedback Loop**: Processes issue comments to update task scope/priorities
5. **Agent Tracking**: Shows which agent (Rex/Cleo/Cipher/Tess) is currently working on each task

---

## Architecture

### Component Structure

```
Play Project Workflow
├── Step 0: Launch Morgan PM (daemon)
│   ├── Create GitHub Project
│   ├── Create Issues for all tasks
│   ├── Setup custom fields (Current Agent, Stage, Priority)
│   └── Start monitoring loop
├── Step 1: Discover tasks (Morgan credentials for cloning)
├── Step 2: Build dependency graph
└── Step 3: Process tasks (Rex → Cleo → Cipher → Tess)
    └── Morgan monitors and updates Project in real-time
```

### File Organization

Following existing agent template patterns:

```
infra/charts/controller/
├── agent-templates/
│   └── pm/  # New PM templates directory
│       ├── morgan-pm.sh.hbs                  # Main PM daemon script
│       ├── github-projects-helpers.sh.hbs    # GraphQL API helpers
│       └── process-issue-comment.sh.hbs      # Issue comment processor
├── templates/
│   ├── agent-templates-pm.yaml               # ConfigMap for PM templates
│   └── workflowtemplates/
│       └── play-project-workflow-template.yaml  # Updated with Morgan daemon
└── values.yaml  # Morgan config updated with PM capabilities
```

---

## GitHub Projects V2 Integration

### Custom Fields Created

Morgan automatically creates these custom fields in each project:

| Field Name | Type | Options | Purpose |
|------------|------|---------|---------|
| **Current Agent** | Single Select | Pending, Rex (Implementation), Cleo (Quality), Cipher (Security), Tess (QA), Complete ✅ | Tracks which agent is working |
| **Stage** | Single Select | Pending, Implementation, Code Review, Security Check, QA Testing, Done | Workflow stage |
| **Task ID** | Text | - | TaskMaster task ID |
| **Priority** | Single Select | High, Medium, Low | Task priority |

### GraphQL Operations

Morgan uses the GitHub Projects V2 GraphQL API via `gh` CLI:

**Key Operations:**
```graphql
# Create Project
mutation createProject($ownerId: ID!, $title: String!) {
  createProjectV2(input: {
    ownerId: $ownerId
    title: $title
  }) {
    projectV2 { id }
  }
}

# Add Issue to Project
mutation addIssue($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: {
    projectId: $projectId
    contentId: $contentId
  }) {
    item { id }
  }
}

# Update Field Value
mutation updateField($projectId: ID!, $itemId: ID!, $fieldId: ID!, $value: String!) {
  updateProjectV2ItemFieldValue(input: {
    projectId: $projectId
    itemId: $itemId
    fieldId: $fieldId
    value: { singleSelectOptionId: $value }
  }) {
    projectV2Item { id }
  }
}
```

### Retry Logic & Rate Limiting

Adapted from `5dlabs/tasks` reference implementation:

- **Exponential backoff**: 2^retry_count seconds
- **Max retries**: 3 attempts
- **Rate limit detection**: Automatic retry on rate limit errors
- **Graceful degradation**: Continues on non-critical failures

---

## Status Mapping

Morgan translates Kubernetes workflow states to GitHub Project statuses:

| Workflow Stage | Workflow Phase | Current Agent | Project Status |
|----------------|----------------|---------------|----------------|
| `pending` | Running | Rex (Implementation) | In Progress |
| `implementation` | Running | Rex (Implementation) | In Progress |
| `quality-in-progress` | Running | Cleo (Quality) | In Review |
| `security-in-progress` | Running | Cipher (Security) | In Review |
| `testing-in-progress` | Running | Tess (QA) | In Review |
| Any | Succeeded | Complete ✅ | Done |
| Any | Failed/Error | - | Blocked |

---

## Monitoring Loop

Morgan runs a continuous monitoring loop (30-second interval):

```bash
while workflow_running; do
  for each task:
    # Get task workflow via K8s API
    workflow = kubectl get workflow -l task-id=$TASK_ID
    
    # Extract current stage and phase
    stage = workflow.labels.current-stage
    phase = workflow.status.phase
    
    # Map to agent and status
    agent = map_stage_to_agent(stage, phase)
    status = map_workflow_to_status(stage, phase)
    
    # Update GitHub Project via GraphQL
    update_project_item(project_id, item_id, agent, status)
    
    # Update issue labels for filtering
    update_issue_labels(issue_number, status, agent)
  
  sleep 30
done
```

---

## Human Feedback Processing

### Issue Comment Webhook

Argo Events sensor triggers on TaskMaster issue comments:

**Sensor**: `morgan-issue-comment-sensor.yaml`

```yaml
filters:
  - path: body.action
    value: ["created"]
  - path: body.issue.labels[].name
    value: ["taskmaster-task"]  # Only TaskMaster issues
  - path: body.comment.user.type
    value: ["User"]  # Humans only (no bot loops)
```

### Comment Intent Detection

Morgan analyzes comments for actionable patterns:

| Pattern | Intent Type | Action |
|---------|-------------|--------|
| `@morgan`, `scope`, `add requirement` | scope-change | Update implementation notes, commit changes |
| `clarify`, `explain`, `unclear` | clarification | Document question in notes, await response |
| `priority`, `urgent`, `critical` | priority-change | Update tasks.json priority, update labels |

### Bidirectional Sync

**GitHub Issue → TaskMaster Docs**:
1. Human posts comment on issue
2. Webhook triggers Morgan workflow
3. Morgan analyzes comment intent
4. Updates TaskMaster docs (implementation-notes.md)
5. Commits changes to docs repository
6. Replies to issue confirming update

---

## Configuration

### Morgan Agent Enhancement

Updated in `values.yaml`:

```yaml
agents:
  morgan:
    expertise: 
      - documentation
      - requirements
      - planning
      - task-management
      - project-tracking  # NEW
      - github-projects   # NEW
    systemPrompt: |
      ...
      **Project Management Mode:**
      When running as a project manager daemon:
      - Create and maintain GitHub Projects for workflow visibility
      - Create GitHub Issues for each TaskMaster task
      - Monitor workflow progress via Kubernetes API
      - Update Project status fields in real-time
      - Process human feedback from issue comments
      - Maintain bidirectional sync between TaskMaster and GitHub
```

### Required GitHub App Permissions

Morgan's GitHub App needs these permissions:

- **Projects**: Read/Write (for creating and updating projects)
- **Issues**: Read/Write (for creating issues and processing comments)
- **Metadata**: Read (for repository access)
- **Contents**: Read (for accessing TaskMaster files)

### Webhook Events

Morgan should subscribe to:
- `issues` (for comment processing)
- `issue_comment` (for human feedback)

---

## State Management

### Persistent State

Morgan stores state in `/shared/morgan-pm/`:

```
/shared/morgan-pm/
├── task-issue-map.json      # Maps task IDs to issue/project item IDs
├── project-config.json       # Project metadata and IDs
└── sync.log                  # Timestamped activity log
```

**task-issue-map.json structure**:
```json
{
  "1": {
    "issue_number": 123,
    "item_id": "PVTI_...",
    "node_id": "I_..."
  },
  "2": {
    "issue_number": 124,
    "item_id": "PVTI_...",
    "node_id": "I_..."
  }
}
```

### Kubernetes-Based State

Also stores mapping as ConfigMap for persistence:

```bash
kubectl create configmap project-$WORKFLOW_NAME-mapping \
  --from-file=task-issue-map.json \
  --from-file=project-config.json \
  -n agent-platform
```

---

## Implementation Details

### Daemon Pattern

Morgan uses Argo Workflows `daemon: true` feature:

**Benefits**:
- Runs throughout parent workflow lifecycle
- No additional compute costs (part of workflow)
- Automatic cleanup when workflow completes
- Access to K8s API for workflow monitoring

**Lifecycle**:
1. Starts when workflow begins
2. Runs initialization (create project/issues)
3. Enters monitoring loop
4. Exits when parent workflow completes

### Template Rendering

Following existing agent patterns:

1. **Templates stored in ConfigMap**: `agent-templates-pm.yaml`
2. **Runtime rendering**: Container script uses `envsubst` to inject values
3. **Handlebars placeholders**: `{{repository_url}}`, `{{workflow_name}}`, etc.
4. **Execution**: Rendered scripts executed via `exec`

---

## Use Cases & Benefits

### For Development Teams

- **Real-time visibility**: See exactly what agent is working on what
- **Progress tracking**: Monitor velocity across parallel tasks
- **Bottleneck identification**: Spot stuck workflows quickly
- **Dependency visualization**: Understand task relationships

### For Product Management

- **Stakeholder communication**: Share Project board with non-technical stakeholders
- **Scope management**: Direct interface for requirement changes via issue comments
- **Priority adjustment**: Update task priorities through issue labels
- **Metrics**: Track completion rates and cycle times

### For QA & Operations

- **Testing coordination**: See when tasks enter QA stage
- **Issue correlation**: Link defects back to original tasks
- **Release planning**: Group tasks by milestone
- **Compliance**: Document review and approval stages

---

## Error Handling & Recovery

### Graceful Degradation

If GitHub Projects API fails:
- Morgan logs error but continues monitoring
- Workflow proceeds normally (Projects is visibility layer, not blocking)
- Retries with exponential backoff
- Falls back to basic status updates only

### Common Error Scenarios

| Error | Cause | Mitigation |
|-------|-------|------------|
| Project creation fails | Permissions or API limit | Use existing project if found |
| Issue creation fails | Rate limit | Batch creation with delays |
| Field update fails | Field doesn't exist | Create field on-demand |
| GraphQL timeout | Large project | Implement pagination |

---

## Testing Strategy

### Unit Testing

Test helper functions independently:

```bash
# Test field creation
source github-projects-helpers.sh
PROJECT_ID="PVT_test123"
create_single_select_field "$PROJECT_ID" "Test Field" "Option1" "Option2"
```

### Integration Testing

Test with real GitHub Project:

1. Create test project in GitHub
2. Run Morgan PM container locally
3. Verify issues created
4. Verify field updates
5. Clean up test data

### End-to-End Testing

Full workflow test:

```bash
# Submit play workflow with Morgan PM enabled
kubectl create -f test-play-with-morgan.yaml

# Monitor Morgan logs
kubectl logs -f -l agent=morgan,workflow-type=project-orchestration

# Verify GitHub Project
gh project list --owner 5dlabs
gh project item-list <project-number>
```

---

## Future Enhancements

### Phase 2 Capabilities

- **Automated PR linking**: Link PRs to Project items automatically
- **Milestone integration**: Sync TaskMaster tags to GitHub milestones
- **Sprint planning**: Support iteration fields for agile workflows
- **Metrics dashboard**: Export Project data for analytics

### Phase 3 Capabilities

- **Multi-project support**: Handle multiple related projects
- **Cross-repository tracking**: Track dependencies across repos
- **Advanced automation**: GitHub Actions triggered by Project changes
- **AI-driven insights**: Morgan suggests priorities based on velocity

---

## Deployment

### Installation

Morgan PM templates are automatically deployed with controller Helm chart:

```bash
helm upgrade controller ./infra/charts/controller \
  --namespace agent-platform \
  --install
```

### Verification

Check ConfigMap created:

```bash
kubectl get configmap -n agent-platform | grep agent-templates-pm
kubectl describe configmap controller-agent-templates-pm -n agent-platform
```

### Troubleshooting

**Morgan PM not starting:**
```bash
# Check pod logs
kubectl logs -l agent=morgan,workflow-type=project-orchestration -n agent-platform

# Check ConfigMap exists
kubectl get cm controller-agent-templates-pm -n agent-platform

# Verify Morgan GitHub App secret
kubectl get secret github-app-5dlabs-morgan -n agent-platform
```

**Project updates not appearing:**
```bash
# Check Morgan sync log
kubectl exec -it <morgan-pod> -n agent-platform -- cat /shared/morgan-pm/sync.log

# Verify GitHub permissions
gh auth status
gh api graphql -f query='query { viewer { login } }'

# Check rate limit status
gh api rate_limit
```

---

## Security Considerations

### Least Privilege Access

- Morgan uses its own GitHub App (5DLabs-Morgan)
- Separate permissions from implementation agents
- Read-only access to code repositories
- Write access only to Projects and Issues

### Secret Management

- GitHub App credentials stored as Kubernetes secrets
- Private key never logged or exposed
- JWT tokens generated on-demand with short TTL
- Installation tokens rotated per operation

### Audit Trail

All Morgan operations logged:
- Project creations
- Issue creations
- Field updates
- Comment responses
- Timestamp for each operation

---

## Reference Implementation

### Inspiration

Patterns adapted from `5dlabs/tasks` - a production Rust tool for TaskMaster-GitHub sync:

- GraphQL query structure
- Error handling with retries
- Field mapping patterns
- Authentication via GitHub CLI
- Type-safe API interactions

### Key Learnings

1. **Use GitHub CLI**: Simpler than direct API, handles auth automatically
2. **Retry with backoff**: GitHub API can be flaky, always retry
3. **Cache field IDs**: Querying field structure is expensive
4. **Batch operations**: Group updates to reduce API calls
5. **Idempotent operations**: Safe to replay, important for daemon recovery

---

## Next Steps

1. **Deploy and test** with simple project (3-5 tasks)
2. **Validate issue comment processing** with manual feedback
3. **Optimize monitoring loop** based on performance data
4. **Add metrics** for Project sync latency and accuracy
5. **Document user workflow** for PM-enabled projects

---

*This integration transforms GitHub Projects from a static planning tool into a dynamic dashboard for real-time AI agent orchestration visibility.*





