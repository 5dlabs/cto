# Atlas PR Guardian - Automated PR Shepherding

## Overview

Atlas PR Guardian is an automated system that watches pull requests in the
`5dlabs/cto` repository from creation to merge, ensuring they stay clean,
pass CI, and have no outstanding issues before automatically merging them.

**Key Concept**: One Atlas instance per PR, staying active until the PR is
merged or closed.

## Features

### 1. Cursor Bugbot Resolution

- Monitors for comments from [Cursor Bugbot](https://github.com/apps/cursor)
- Analyzes Bugbot feedback and applies fixes
- Pushes fixes to PR branch
- Verifies Bugbot is satisfied (no open threads)

### 2. CI Failure Recovery

- Watches for failing status checks and CI runs
- Analyzes failure logs (clippy, tests, lints, builds)
- Applies minimal fixes to get CI green
- Re-runs checks automatically

### 3. Merge Conflict Resolution

- Detects when PR becomes unmergeable
- Rebases or merges main into PR branch
- Resolves conflicts intelligently
- Ensures clean merge state

### 4. Auto-Merge When Ready

**Merge Criteria** (ALL required):

- ✅ No open Bugbot comment threads
- ✅ All CI checks passing
- ✅ No merge conflicts
- ✅ PR is mergeable

**Merge Strategy**: Always squash merge

### 5. Blocked State Handling

If Atlas cannot resolve issues after 3 attempts:

- Adds `blocked` label to PR
- Posts detailed comment explaining blockers
- Tags PR author for human intervention
- Suspends Atlas session until new activity

## Architecture

### Event-Driven Activation

Atlas uses Argo Events sensors to react to GitHub webhooks:

```text
GitHub Webhook → Argo Events Sensor → CodeRun Creation → Atlas Activation
```

**Triggering Events**:

- `pull_request` (opened, reopened, synchronize, ready_for_review)
- `issue_comment` (created) - for Bugbot feedback
- `status` / `check_run` / `check_suite` - for CI updates
- `workflow_run` (completed/requested) - to react on GitHub Actions finishing for PR commits

### Session Continuity

- `continueSession: true` ensures Atlas remembers context across events
- Each PR gets its own Atlas workspace
- Session persists until PR is merged or closed

### Workflow Loop

```text
PR Event → Atlas Activated
  ↓
Check PR State:
  ├─ Bugbot comments? → Resolve → Push fixes
  ├─ CI failing? → Analyze logs → Fix → Push
  └─ Merge conflicts? → Rebase → Resolve → Push
  ↓
Wait for CI to complete
  ↓
All checks pass? → Verify merge criteria
  ↓
  ├─ Ready? → Squash merge → Post summary → Close session ✅
  └─ Blocked? → Add label → Post comment → Suspend ⏸️
```

## Configuration

### Values.yaml Configuration

Located in `infra/charts/controller/values.yaml`:

```yaml
atlas:
  name: "Atlas"
  githubApp: "5DLabs-Atlas"
  cli: "Claude"
  model: "claude-sonnet-4-20250514"
  maxTokens: 8192
  temperature: 0.3
  role: "PR Guardian & Integration Specialist"
  
  guardianMode:
    enabled: true
    targetRepository: "5dlabs/cto"
    mergeStrategy: "squash"
    maxAttempts: 3
    autoMerge: true
    watchBugbot: true
    watchCI: true
    watchConflicts: true

**CLI Selection:** Atlas now uses the same controller-driven CLI configuration as other agents. Set `.Values.agents.atlas.cli` (and related model/maxTokens/temperature) to switch between Claude, Cursor, Codex, Factory, or OpenCode without touching the sensor. The sensor no longer hard-codes `cliConfig`, so whatever you configure in Helm values is what the controller injects into the CodeRun.
```

### Sensor Configuration

Located in `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`:

- **Dependencies**: PR events, comment events, status events
- **Trigger**: Creates CodeRun with guardian mode enabled
- **Labels**: PR number, repository name for correlation
- **Environment**: PR_NUMBER, PR_URL, REPOSITORY_FULL_NAME, GUARDIAN_MODE, MERGE_STRATEGY

## Deployment

### Prerequisites

1. **GitHub App Permissions** (5DLabs-Atlas):
   - `contents: write` - Push fixes to PR branches
   - `pull_requests: write` - Comment, label, and merge PRs
   - `workflows: read` - Read CI status checks
   - `statuses: read` - Read commit statuses

2. **Argo Events Infrastructure**:
   - GitHub webhook EventSource deployed
   - Argo Events service account with RBAC

3. **Controller Configuration**:
   - Atlas agent configured in values.yaml
   - GitHub App credentials in Vault/External Secrets

### Installation

The Atlas PR Guardian sensor is automatically deployed via ArgoCD app-of-apps:

```bash
# Verify deployment
kubectl get sensors -n argo | grep atlas-pr-guardian

# Check sensor status
kubectl describe sensor atlas-pr-guardian -n argo

# View sensor logs
kubectl logs -n argo -l sensor-name=atlas-pr-guardian
```

### Manual Deployment

If needed, deploy manually:

```bash
# Apply sensor
kubectl apply -f infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml

# Apply ArgoCD application
kubectl apply -f infra/gitops/applications/atlas-pr-guardian-sensor.yaml
```

## Usage

### For Developers

**No action required!** Atlas automatically activates when you:

1. Open a PR
2. Push new commits
3. Receive Bugbot comments
4. Encounter CI failures

### Monitoring Atlas Activity

**View Atlas CodeRuns**:

```bash
# List active Atlas guardians
kubectl get coderuns -n agent-platform -l agent=atlas,role=pr-guardian

# View specific guardian
kubectl get coderun <coderun-name> -n agent-platform -o yaml

# Check logs
kubectl logs -n agent-platform <pod-name>
```

**GitHub PR Comments**:
Atlas posts comments on PRs to explain its actions:

- Fix summaries
- Merge notifications
- Blocked state explanations

### Handling Blocked PRs

If Atlas adds the `blocked` label:

1. **Read Atlas comment** - explains what it couldn't fix
2. **Fix the issue** - apply necessary changes
3. **Push to PR** - Atlas will reactivate automatically
4. **Remove `blocked` label** - optional, Atlas will remove it on success

## Example Scenarios

### Scenario A: Bugbot Feedback Loop

```text
1. Developer opens PR
2. Atlas activates, checks PR
3. Bugbot comments: "Missing error handling in api.rs:42"
4. Atlas analyzes, adds error handling
5. Atlas pushes fix with commit: "fix(api): add error handling per Bugbot"
6. Bugbot satisfied, no more comments
7. CI passes
8. Atlas squash-merges PR ✅
```

### Scenario B: CI Failure Recovery

```text
1. PR opened, CI fails (clippy error)
2. Atlas activates, sees red CI
3. Atlas reads clippy logs
4. Atlas fixes clippy warnings
5. Atlas pushes fix with commit: "fix(lint): resolve clippy warnings"
6. CI re-runs, passes ✅
7. Atlas squash-merges PR ✅
```

### Scenario C: Merge Conflict Resolution

```text
1. PR open, another PR merges to main
2. Original PR now has conflicts
3. Atlas detects unmergeable state
4. Atlas rebases PR on latest main
5. Atlas resolves conflicts
6. Atlas pushes resolution
7. CI passes
8. Atlas squash-merges PR ✅
```

### Scenario D: Blocked State

```text
1. PR has complex issue Atlas can't fix
2. Atlas tries 3 times, fails
3. Atlas adds `blocked` label
4. Atlas comments: "⚠️ Unable to resolve X, Y, Z. @author please review."
5. Atlas suspends session ⏸️
6. Human fixes issue, pushes
7. New push triggers Atlas
8. Atlas sees clean state, merges ✅
```

## Integration with Existing Agents

- **Independent of Rex/Cleo/Tess**: Atlas does not interfere with multi-agent play workflows
- **Watches All PRs**: Regardless of origin (human, Rex, Cursor, etc.)
- **Only Acts When Needed**: Bugbot comments, CI failures, or merge conflicts

## Troubleshooting

### Atlas Not Activating

**Check sensor status**:

```bash
kubectl get sensor atlas-pr-guardian -n argo
kubectl describe sensor atlas-pr-guardian -n argo
```

**Verify webhook delivery**:

```bash
# Check GitHub webhook deliveries in repo settings
# Look for recent PR events
```

**Check EventSource**:

```bash
kubectl get eventsource github -n argo
kubectl logs -n argo -l eventsource-name=github
```

### Atlas Not Merging

**Check merge criteria**:

- Are there open Bugbot comments?
- Is CI passing (all checks green)?
- Are there merge conflicts?
- Is PR in mergeable state?

**Check Atlas logs**:

```bash
kubectl logs -n agent-platform <atlas-coderun-pod>
```

**Check GitHub App permissions**:

- Verify 5DLabs-Atlas has `pull_requests: write`
- Verify 5DLabs-Atlas has `contents: write`

### Atlas Stuck in Loop

**Check attempt count**:

- Atlas should suspend after 3 attempts
- Look for `blocked` label on PR

**Manual intervention**:

```bash
# Delete stuck CodeRun
kubectl delete coderun <coderun-name> -n agent-platform

# Fix issue manually
# Push new commit to trigger fresh Atlas activation
```

## Metrics & Success Criteria

Track Atlas performance:

- **Auto-merge rate**: % of PRs merged without human intervention
- **Time to merge**: Average time from PR open to merge
- **Bugbot resolution rate**: % of Bugbot comments resolved automatically
- **CI recovery rate**: % of CI failures fixed automatically
- **Blocked rate**: % of PRs requiring human intervention

## Security Considerations

### GitHub App Permissions

Atlas requires write access to:

- PR branches (to push fixes)
- PR metadata (to comment, label, merge)

**Mitigation**:

- Atlas only operates on `5dlabs/cto` repository
- Atlas uses squash merge (no force-push to main)
- Atlas never modifies main branch directly

### Rate Limiting

Atlas respects GitHub API rate limits:

- Backs off on 429 responses
- Uses conditional requests when possible
- Caches PR state to minimize API calls

### Secrets Management

GitHub App credentials stored securely:

- Vault backend (production)
- External Secrets Operator
- Kubernetes secrets (encrypted at rest)

## Future Enhancements

### Planned Features

1. **Multi-Repository Support**: Extend beyond `5dlabs/cto`
2. **Custom Merge Strategies**: Support merge commits, rebase
3. **Approval Requirements**: Wait for human approval before merge
4. **Scheduled Merges**: Merge during specific time windows
5. **Dependency Updates**: Auto-merge Dependabot PRs

### Metrics Dashboard

Build Grafana dashboard showing:

- Active Atlas guardians
- Merge success rate
- Average time to merge
- Blocked PR trends

## References

- [Cursor Bugbot](https://github.com/apps/cursor)
- [Argo Events Documentation](https://argoproj.github.io/argo-events/)
- [GitHub Apps Permissions](https://docs.github.com/en/apps/creating-github-apps/setting-up-a-github-app/choosing-permissions-for-a-github-app)
- [Atlas Workflow Design](../atlas-workflow-design.md)

## Support

For issues or questions:

- Check sensor logs: `kubectl logs -n argo -l sensor-name=atlas-pr-guardian`
- Check CodeRun logs: `kubectl logs -n agent-platform <atlas-pod>`
- Review PR comments from Atlas
- Contact platform team

---

**Atlas PR Guardian**: Keeping the cto repo flowing smoothly, one PR at a time. 🔗
