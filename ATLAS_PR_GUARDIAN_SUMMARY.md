# Atlas PR Guardian - Implementation Summary

## Overview

Implemented a comprehensive **Atlas PR Guardian** system that automatically
shepherds pull requests in the `5dlabs/cto` repository from creation to merge,
resolving Bugbot comments, fixing CI failures, handling merge conflicts, and
auto-merging when ready.

## What Was Built

### 1. Event-Driven Sensor

**File**: `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`

- Triggers on PR events (opened, reopened, synchronize, ready_for_review)
- Triggers on issue comments (for Bugbot feedback)
- Triggers on CI status updates (status, check_run, check_suite)
- Creates one Atlas CodeRun per PR with session continuity
- Passes PR metadata via environment variables

### 2. Enhanced Atlas Configuration

**File**: `infra/charts/controller/values.yaml`

- Added `guardianMode` configuration section
- Updated system prompt for PR guardian behavior
- Increased token limit to 8192 for complex operations
- Lowered temperature to 0.3 for more deterministic behavior
- Added expertise tags: pr-guardian, bugbot-resolution, ci-recovery

### 3. Detailed System Prompt

**File**: `infra/charts/controller/templates/prompts/atlas-pr-guardian-system-prompt.md`

- Comprehensive instructions for Atlas PR guardian role
- Bugbot comment resolution strategies
- CI failure recovery procedures
- Merge conflict resolution techniques
- Auto-merge criteria and process
- Blocked state escalation procedures
- Example scenarios and best practices

### 4. ArgoCD Application

**File**: `infra/gitops/applications/atlas-pr-guardian-sensor.yaml`

- Deploys sensor via app-of-apps pattern
- Automated sync and self-heal enabled
- Targets argo namespace

### 5. Comprehensive Documentation

**File**: `docs/engineering/atlas-pr-guardian.md`

- Architecture overview
- Configuration details
- Deployment instructions
- Usage guide for developers
- Troubleshooting procedures
- Example scenarios
- Security considerations

### 6. Validation Script

**File**: `scripts/test-atlas-pr-guardian.sh`

- Validates YAML syntax
- Checks sensor structure
- Verifies configuration
- Tests cluster deployment (optional)
- Provides next steps

## Key Features

### Cursor Bugbot Resolution

- Monitors for comments from [Cursor Bugbot](https://github.com/apps/cursor)
- Analyzes feedback and applies fixes
- Iterates until Bugbot is satisfied

### CI Failure Recovery

- Watches for failing checks (clippy, tests, lints, builds)
- Analyzes failure logs
- Applies minimal fixes to get CI green
- Re-runs checks automatically

### Merge Conflict Resolution

- Detects unmergeable PRs
- Rebases on latest main
- Resolves conflicts intelligently
- Ensures clean merge state

### Auto-Merge

**Merge Criteria** (ALL required):

- ✅ No open Bugbot comment threads
- ✅ All CI checks passing
- ✅ No merge conflicts
- ✅ PR mergeable

**Merge Strategy**: Always squash merge

### Blocked State Handling

After 3 failed attempts:

- Adds `blocked` label
- Posts detailed explanation
- Tags PR author
- Suspends session until new activity

## Architecture Highlights

### Event-Driven Activation

```text
GitHub Webhook → Argo Events Sensor → CodeRun Creation → Atlas Activation
```

### Session Continuity

- `continueSession: true` maintains context across events
- One Atlas instance per PR
- Persists until PR merged or closed

### Workflow Loop

```text
PR Event → Check State → Fix Issues → Push → Wait for CI → Verify → Merge
```

## Deployment

### Automatic via ArgoCD

Once merged to main, ArgoCD automatically deploys:

1. Atlas PR Guardian sensor to `argo` namespace
2. ConfigMap with guardian strategy documentation
3. RBAC permissions for CodeRun creation

### Verification

```bash
# Check sensor deployment
kubectl get sensor atlas-pr-guardian -n argo

# Check ArgoCD application
kubectl get application atlas-pr-guardian-sensor -n argocd

# View sensor logs
kubectl logs -n argo -l sensor-name=atlas-pr-guardian
```

## Integration with Existing System

### GitHub App Permissions

Atlas uses the existing **5DLabs-Atlas** GitHub App with:

- `contents: write` - Push fixes to PR branches
- `pull_requests: write` - Comment, label, and merge PRs
- `workflows: read` - Read CI status checks

### Webhook Infrastructure

Leverages existing GitHub webhook EventSource:

- Already configured for org-level webhooks
- No additional webhook setup required
- Reuses existing event bus and RBAC

### Controller Integration

Works seamlessly with existing controller:

- Uses standard CodeRun CRD
- Follows agent template patterns
- Integrates with workspace isolation

### Independent of Multi-Agent Workflows

- Does not interfere with Rex/Cleo/Tess workflows
- Watches all PRs regardless of origin
- Only acts when needed (Bugbot, CI, conflicts)

## Files Created/Modified

### New Files

1. `infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml`
2. `infra/gitops/applications/atlas-pr-guardian-sensor.yaml`
3. `infra/charts/controller/templates/prompts/atlas-pr-guardian-system-prompt.md`
4. `docs/engineering/atlas-pr-guardian.md`
5. `scripts/test-atlas-pr-guardian.sh`

### Modified Files

1. `infra/charts/controller/values.yaml` - Added guardianMode configuration

## Testing & Validation

### Pre-Deployment Validation

```bash
# Run validation script
./scripts/test-atlas-pr-guardian.sh

# Check YAML syntax
yamllint infra/gitops/resources/sensors/atlas-pr-guardian-sensor.yaml
yamllint infra/gitops/applications/atlas-pr-guardian-sensor.yaml

# Validate values.yaml
yamllint infra/charts/controller/values.yaml
```

### Post-Deployment Testing

1. **Create Test PR**: Open a PR with intentional issues
2. **Verify Activation**: Check Atlas CodeRun created
3. **Monitor Behavior**: Watch Atlas resolve issues
4. **Verify Merge**: Confirm squash merge when ready

## Success Metrics

Track Atlas performance:

- **Auto-merge rate**: % of PRs merged without human intervention
- **Time to merge**: Average time from PR open to merge
- **Bugbot resolution rate**: % of Bugbot comments resolved automatically
- **CI recovery rate**: % of CI failures fixed automatically
- **Blocked rate**: % of PRs requiring human intervention

## Security Considerations

### Permissions

- Atlas only operates on `5dlabs/cto` repository
- Uses squash merge (no force-push to main)
- Never modifies main branch directly

### Secrets Management

- GitHub App credentials in Vault
- External Secrets Operator integration
- Kubernetes secrets encrypted at rest

### Rate Limiting

- Respects GitHub API rate limits
- Backs off on 429 responses
- Caches PR state to minimize API calls

## Next Steps

### Immediate

1. **Create Feature Branch**:

   ```bash
   git checkout -b feature/atlas-pr-guardian
   ```

2. **Commit Changes**:

   ```bash
   git add .
   git commit -m "feat(atlas): add PR guardian automation

   - Add event-driven sensor for PR lifecycle events
   - Configure guardian mode in values.yaml
   - Add comprehensive system prompt for PR shepherding
   - Create ArgoCD application for sensor deployment
   - Add documentation and validation scripts

   Atlas PR Guardian automatically watches PRs from creation to merge,
   resolving Bugbot comments, fixing CI failures, handling merge conflicts,
   and auto-merging when ready using squash strategy.

   Closes #<issue-number>"
   ```

3. **Push and Create PR**:

   ```bash
   git push -u origin feature/atlas-pr-guardian
   gh pr create --title "feat(atlas): Add PR Guardian Automation" \
     --body "$(cat ATLAS_PR_GUARDIAN_SUMMARY.md)"
   ```

4. **Merge to Main**: Once approved, merge PR

5. **Verify Deployment**: ArgoCD will automatically deploy sensor

### Future Enhancements

1. **Multi-Repository Support**: Extend beyond `5dlabs/cto`
2. **Custom Merge Strategies**: Support merge commits, rebase
3. **Approval Requirements**: Wait for human approval before merge
4. **Scheduled Merges**: Merge during specific time windows
5. **Metrics Dashboard**: Grafana dashboard for Atlas performance

## Troubleshooting

### Atlas Not Activating

- Check sensor status: `kubectl get sensor atlas-pr-guardian -n argo`
- Verify webhook delivery in GitHub repo settings
- Check EventSource logs: `kubectl logs -n argo -l eventsource-name=github`

### Atlas Not Merging

- Verify merge criteria (Bugbot comments, CI, conflicts)
- Check Atlas logs: `kubectl logs -n agent-platform <atlas-pod>`
- Verify GitHub App permissions

### Atlas Stuck in Loop

- Check for `blocked` label on PR
- Delete stuck CodeRun: `kubectl delete coderun <name> -n agent-platform`
- Push new commit to trigger fresh activation

## References

- [Cursor Bugbot](https://github.com/apps/cursor)
- [Argo Events Documentation](https://argoproj.github.io/argo-events/)
- [GitHub Apps Permissions](https://docs.github.com/en/apps)
- [Atlas Workflow Design](docs/atlas-workflow-design.md)

## Support

For issues or questions:

- Check sensor logs
- Review PR comments from Atlas
- Consult documentation: `docs/engineering/atlas-pr-guardian.md`
- Contact platform team

---

**Atlas PR Guardian**: Keeping the cto repo flowing smoothly, one PR at a
time. 🔗

