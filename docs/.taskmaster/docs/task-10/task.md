# Task 10: Implement Ready-for-QA Label Logic

## Overview

Add logic for Cleo to add 'ready-for-qa' label to PRs through container-cleo.sh.hbs script as explicit handoff signal to Tess. This implements the critical handoff mechanism between code quality work and comprehensive testing phases in the multi-agent workflow.

## Technical Context

The ready-for-qa label serves as the explicit handoff signal from Cleo to Tess in the multi-agent workflow. Cleo must complete all code quality work, wait for CI tests to pass, then add the label to signal readiness for comprehensive testing. This label triggers Tess to begin code review and deployment testing.

## Implementation Guide

### Phase 1: Design Ready-for-QA Workflow Logic

1. **Cleo Workflow Sequence**
   ```bash
   1. Run code quality checks (Clippy, rustfmt)
   2. Push quality fixes to same feature branch
   3. Wait for GitHub Actions CI tests to pass
   4. Add 'ready-for-qa' label via GitHub API
   5. Complete Cleo workflow successfully
   ```

2. **Label Management Strategy**
   ```bash
   # Idempotent label addition - check before adding
   EXISTING_LABELS=$(gh pr view $PR_NUMBER --json labels --jq '.labels[].name')

   if [[ ! "$EXISTING_LABELS" =~ "ready-for-qa" ]]; then
     gh pr edit $PR_NUMBER --add-label "ready-for-qa"
     echo "✅ Ready-for-QA label added"
   else
     echo "ℹ️  Ready-for-QA label already present"
   fi
   ```

### Phase 2: Implement CI Test Validation

1. **GitHub Actions Status Checking**
   ```bash
   #!/bin/bash
   # wait-for-ci-success.sh - Wait for CI tests to complete successfully

   check_ci_status() {
       local pr_number="$1"
       local max_attempts=30
       local attempt=0

       while [ $attempt -lt $max_attempts ]; do
           echo "🔄 Checking CI status (attempt $((attempt + 1))/$max_attempts)..."

           # Get CI status for PR
           CI_STATUS=$(gh pr checks $pr_number --json state,conclusion \
               --jq '.[] | select(.name | test("CI|Test|Build")) | {state, conclusion}')

           # Check if all CI checks are successful
           PENDING_CHECKS=$(echo "$CI_STATUS" | jq -r 'select(.state == "PENDING" or .state == "IN_PROGRESS")' | wc -l)
           FAILED_CHECKS=$(echo "$CI_STATUS" | jq -r 'select(.conclusion == "FAILURE" or .conclusion == "CANCELLED")' | wc -l)

           if [ "$FAILED_CHECKS" -gt 0 ]; then
               echo "❌ CI checks failed, cannot proceed to ready-for-qa"
               return 1
           elif [ "$PENDING_CHECKS" -eq 0 ]; then
               echo "✅ All CI checks passed"
               return 0
           else
               echo "⏳ $PENDING_CHECKS CI checks still pending, waiting..."
               sleep 60  # Wait 1 minute between checks
           fi

           attempt=$((attempt + 1))
       done

       echo "⏰ Timeout waiting for CI checks to complete"
       return 1
   }
   ```

2. **Integration with Cleo Container Script**
   ```handlebars
   # In container-cleo.sh.hbs
   {{#if (eq github_app "5DLabs-Cleo")}}
   echo "🎯 Cleo: Starting code quality workflow"

   # Run code quality checks
   echo "🔍 Running Clippy pedantic checks..."
   cargo clippy --all-targets --all-features -- -D warnings

   echo "🎨 Running rustfmt checks..."
   cargo fmt --check || cargo fmt

   # Push quality improvements
   if ! git diff --quiet; then
       echo "📝 Committing quality improvements..."
       git add .
       git commit -m "style: apply Cleo code quality improvements"
       git push origin HEAD
   fi

   # Wait for CI success before labeling
   echo "⏳ Waiting for CI tests to pass..."
   if wait-for-ci-success.sh "$PR_NUMBER"; then
       echo "🏷️  Adding ready-for-qa label..."
       add-ready-for-qa-label.sh "$PR_NUMBER"
       echo "✅ Cleo workflow complete - handoff to Tess"
   else
       echo "❌ CI tests failed, cannot proceed to testing phase"
       exit 1
   fi
   {{/if}}
   ```

### Phase 3: Create GitHub API Integration Scripts

1. **Label Addition Script**
   ```bash
   #!/bin/bash
   # add-ready-for-qa-label.sh - Add ready-for-qa label to PR

   set -euo pipefail

   PR_NUMBER="$1"

   if [ -z "$PR_NUMBER" ]; then
       echo "Usage: add-ready-for-qa-label.sh <pr_number>"
       exit 1
   fi

   echo "🏷️  Adding ready-for-qa label to PR #$PR_NUMBER"

   # Check if label already exists (idempotent operation)
   EXISTING_LABELS=$(gh pr view "$PR_NUMBER" --json labels --jq '.labels[].name')

   if echo "$EXISTING_LABELS" | grep -q "ready-for-qa"; then
       echo "ℹ️  Ready-for-qa label already exists on PR #$PR_NUMBER"
   else
       # Add the label
       gh pr edit "$PR_NUMBER" --add-label "ready-for-qa"

       # Verify label was added
       UPDATED_LABELS=$(gh pr view "$PR_NUMBER" --json labels --jq '.labels[].name')
       if echo "$UPDATED_LABELS" | grep -q "ready-for-qa"; then
           echo "✅ Ready-for-qa label successfully added to PR #$PR_NUMBER"
       else
           echo "❌ Failed to add ready-for-qa label to PR #$PR_NUMBER"
           exit 1
       fi
   fi

   echo "🚀 Ready for comprehensive testing by Tess"
   ```

2. **PR Discovery and Context Setup**
   ```bash
   #!/bin/bash
   # setup-pr-context.sh - Discover PR and set context for Cleo

   set -euo pipefail

   # Get current branch and extract task ID
   CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
   TASK_ID=$(echo "$CURRENT_BRANCH" | grep -o 'task-[0-9]\+' | cut -d'-' -f2)

   if [ -z "$TASK_ID" ]; then
       echo "❌ Cannot extract task ID from branch: $CURRENT_BRANCH"
       exit 1
   fi

   echo "📋 Task ID: $TASK_ID"
   echo "🌿 Branch: $CURRENT_BRANCH"

   # Find PR for this task/branch
   PR_INFO=$(gh pr list --head "$CURRENT_BRANCH" --json number,title,labels --limit 1)
   PR_NUMBER=$(echo "$PR_INFO" | jq -r '.[0].number // empty')

   if [ -z "$PR_NUMBER" ] || [ "$PR_NUMBER" == "null" ]; then
       echo "❌ No PR found for branch: $CURRENT_BRANCH"
       exit 1
   fi

   echo "🔗 Found PR #$PR_NUMBER for task $TASK_ID"

   # Export context for use by other scripts
   export PR_NUMBER
   export TASK_ID
   export CURRENT_BRANCH

   # Save context to file for script coordination
   cat > /tmp/cleo-context.env <<EOF
   PR_NUMBER=$PR_NUMBER
   TASK_ID=$TASK_ID
   CURRENT_BRANCH=$CURRENT_BRANCH
   EOF

   echo "✅ PR context established"
   ```

### Phase 4: Implement Argo Events Sensor Integration

1. **Ready-for-QA Label Sensor**
   ```yaml
   # Argo Events sensor to detect ready-for-qa label
   apiVersion: argoproj.io/v1alpha1
   kind: Sensor
   metadata:
     name: ready-for-qa-detection
     namespace: argo-events
   spec:
     dependencies:
     - name: github-pr-labeled
       eventSourceName: github-webhook
       eventName: pull-request-labeled
       filters:
         dataFilters:
         - path: "body.action"
           type: string
           value: ["labeled"]
         - path: "body.label.name"
           type: string
           value: ["ready-for-qa"]
         - path: "body.sender.login"
           type: string
           value: ["5DLabs-Cleo[bot]"]

     triggers:
     - template:
         name: resume-tess-stage
         conditions: "github-pr-labeled"
         argoWorkflow:
           operation: resume
           source:
             resource:
               labelSelector: |
                 workflow-type=play-orchestration,
                 current-stage=waiting-ready-for-qa,
                 task-id={{task-id-from-pr-labels}}
           parameters:
           - src:
               dependencyName: github-pr-labeled
               dataTemplate: |
                 {{.body.pull_request.labels | map(select(.name | startswith("task-"))) | .[0].name | split("-")[1]}}
             dest: spec.arguments.parameters.extracted-task-id
   ```

2. **Task ID Extraction from PR Labels**
   ```bash
   # Extract task ID from PR labels in sensor
   TASK_ID=$(echo "$WEBHOOK_PAYLOAD" | jq -r \
     '.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]')

   if [ -n "$TASK_ID" ] && [[ "$TASK_ID" =~ ^[0-9]+$ ]]; then
     echo "📋 Extracted task ID: $TASK_ID"
   else
     echo "❌ Invalid or missing task ID in PR labels"
     exit 1
   fi
   ```

### Phase 5: Create Tess Integration Logic

1. **Tess Prerequisites Check**
   ```handlebars
   # In container-tess.sh.hbs - check for ready-for-qa prerequisite
   {{#if (eq github_app "5DLabs-Tess")}}
   echo "🧪 Tess: Quality Assurance Agent Starting"

   # Check if PR has ready-for-qa label before proceeding
   echo "🔍 Verifying ready-for-qa prerequisite..."

   if [ -f "/tmp/pr-context.json" ]; then
       PR_NUMBER=$(jq -r '.number' /tmp/pr-context.json)
       LABELS=$(jq -r '.labels[].name' /tmp/pr-context.json | tr '\n' ' ')

       if [[ ! "$LABELS" =~ "ready-for-qa" ]]; then
           echo "⏳ PR #$PR_NUMBER does not have ready-for-qa label"
           echo "🛑 Tess cannot start until Cleo completes and adds ready-for-qa"
           exit 0  # Exit successfully but don't start work
       fi

       echo "✅ Ready-for-qa label confirmed, starting comprehensive testing"
   else
       echo "⚠️  No PR context available, proceeding with caution"
   fi

   # Proceed with Tess testing workflow
   echo "🚀 Starting 120% satisfaction testing protocol"
   {{/if}}
   ```

2. **Label Validation and Workflow Coordination**
   ```bash
   #!/bin/bash
   # validate-tess-prerequisites.sh - Ensure Tess can start safely

   set -euo pipefail

   # Get PR information for current task
   TASK_ID=$(cat /workspace/docs/.taskmaster/current-task.json | jq -r '.taskId')

   # Find PR with task label
   PR_INFO=$(gh pr list --label "task-${TASK_ID}" --json number,labels,state --limit 1)

   if [ "$(echo "$PR_INFO" | jq length)" -eq 0 ]; then
       echo "❌ No PR found for task $TASK_ID"
       exit 1
   fi

   PR_NUMBER=$(echo "$PR_INFO" | jq -r '.[0].number')
   LABELS=$(echo "$PR_INFO" | jq -r '.[0].labels[].name')
   PR_STATE=$(echo "$PR_INFO" | jq -r '.[0].state')

   # Validate PR is open and has ready-for-qa label
   if [ "$PR_STATE" != "OPEN" ]; then
       echo "❌ PR #$PR_NUMBER is not open (state: $PR_STATE)"
       exit 1
   fi

   if ! echo "$LABELS" | grep -q "ready-for-qa"; then
       echo "❌ PR #$PR_NUMBER does not have ready-for-qa label"
       echo "Available labels: $(echo "$LABELS" | tr '\n' ', ')"
       exit 1
   fi

   echo "✅ Prerequisites validated for PR #$PR_NUMBER"
   echo "✅ Ready to begin comprehensive testing"

   # Save validated PR context for Tess workflow
   echo "$PR_INFO" > /tmp/validated-pr-context.json
   ```

## Code Examples

### Complete Cleo Ready-for-QA Implementation
```handlebars
#!/bin/bash
# container-cleo.sh.hbs - Complete Cleo workflow with ready-for-qa handoff

set -euo pipefail

echo "🎯 Cleo: Code Quality & Formatting Agent"
echo "Repository: {{service}}"
echo "Mission: Zero tolerance for quality issues + ready-for-qa handoff"

# GitHub API authentication setup
export GITHUB_TOKEN=$(generate-github-token.sh)

# Discover PR context
source setup-pr-context.sh

# Load PR context
source /tmp/cleo-context.env

echo "📋 Working on PR #$PR_NUMBER for task $TASK_ID"

# Phase 1: Code Quality Checks
echo "🔍 Phase 1: Running comprehensive code quality checks"

# Clippy pedantic mode - zero tolerance
echo "  📎 Running Clippy pedantic analysis..."
if ! cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic; then
    echo "  🔧 Clippy issues found, applying fixes..."
    cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings -D clippy::pedantic
fi

# Formatting compliance
echo "  🎨 Checking formatting compliance..."
if ! cargo fmt --check; then
    echo "  🔧 Formatting issues found, applying fixes..."
    cargo fmt
fi

# Additional quality checks
echo "  📚 Running documentation checks..."
cargo doc --no-deps --quiet

echo "✅ Phase 1 complete: All quality checks passed"

# Phase 2: Commit and Push Quality Improvements
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "📝 Phase 2: Committing quality improvements"

    git add .
    git commit -m "style(cleo): apply comprehensive code quality improvements

- Fix all Clippy pedantic warnings
- Apply consistent formatting
- Update documentation as needed

Quality assurance by Cleo agent."

    echo "🚀 Pushing quality improvements to branch $CURRENT_BRANCH"
    git push origin HEAD

    echo "✅ Phase 2 complete: Quality improvements committed and pushed"
else
    echo "ℹ️  Phase 2: No quality improvements needed"
fi

# Phase 3: Wait for CI Success
echo "⏳ Phase 3: Waiting for CI tests to pass"

if wait-for-ci-success.sh "$PR_NUMBER"; then
    echo "✅ Phase 3 complete: All CI tests passed"
else
    echo "❌ Phase 3 failed: CI tests did not pass"
    echo "🛑 Cannot proceed to ready-for-qa until CI is green"
    exit 1
fi

# Phase 4: Add Ready-for-QA Label
echo "🏷️  Phase 4: Adding ready-for-qa label for Tess handoff"

if add-ready-for-qa-label.sh "$PR_NUMBER"; then
    echo "✅ Phase 4 complete: Ready-for-qa label added"
else
    echo "❌ Phase 4 failed: Could not add ready-for-qa label"
    exit 1
fi

# Phase 5: Workflow Completion
echo "🎉 Cleo workflow complete!"
echo "📋 Summary:"
echo "  - Code quality: ✅ 100% compliant"
echo "  - CI tests: ✅ All passing"
echo "  - Ready-for-qa: ✅ Label added"
echo "  - Handoff to Tess: ✅ Ready"

echo "🚀 Tess can now begin comprehensive testing and deployment validation"

# Start Claude with quality-focused context
export CLEO_WORKFLOW_COMPLETE="true"
export READY_FOR_QA_ADDED="true"

exec /app/claude-desktop \
  --config /etc/claude/client-config.json \
  --memory /workspace/CLAUDE.md \
  --continue-session={{continue_session}}
```

### GitHub API Helper Scripts
```bash
#!/bin/bash
# wait-for-ci-success.sh - Robust CI status checking

set -euo pipefail

PR_NUMBER="$1"
MAX_WAIT_MINUTES=30
CHECK_INTERVAL=60

if [ -z "$PR_NUMBER" ]; then
    echo "Usage: wait-for-ci-success.sh <pr_number>"
    exit 1
fi

echo "⏳ Monitoring CI status for PR #$PR_NUMBER"
echo "⚙️  Max wait time: $MAX_WAIT_MINUTES minutes"
echo "🔄 Check interval: $CHECK_INTERVAL seconds"

start_time=$(date +%s)
max_wait_seconds=$((MAX_WAIT_MINUTES * 60))

while true; do
    current_time=$(date +%s)
    elapsed=$((current_time - start_time))

    if [ $elapsed -gt $max_wait_seconds ]; then
        echo "⏰ Timeout: Waited $MAX_WAIT_MINUTES minutes for CI completion"
        exit 1
    fi

    echo "🔍 Checking CI status... (elapsed: $((elapsed / 60))m $((elapsed % 60))s)"

    # Get detailed CI status
    CI_CHECKS=$(gh pr checks "$PR_NUMBER" --json name,state,conclusion)

    if [ "$(echo "$CI_CHECKS" | jq length)" -eq 0 ]; then
        echo "ℹ️  No CI checks found, waiting for checks to appear..."
        sleep $CHECK_INTERVAL
        continue
    fi

    # Analyze check states
    PENDING_COUNT=$(echo "$CI_CHECKS" | jq '[.[] | select(.state == "PENDING" or .state == "IN_PROGRESS")] | length')
    FAILED_COUNT=$(echo "$CI_CHECKS" | jq '[.[] | select(.conclusion == "FAILURE" or .conclusion == "CANCELLED")] | length')
    SUCCESS_COUNT=$(echo "$CI_CHECKS" | jq '[.[] | select(.conclusion == "SUCCESS")] | length')
    TOTAL_COUNT=$(echo "$CI_CHECKS" | jq length)

    echo "📊 CI Status: $SUCCESS_COUNT/$TOTAL_COUNT passed, $PENDING_COUNT pending, $FAILED_COUNT failed"

    if [ "$FAILED_COUNT" -gt 0 ]; then
        echo "❌ CI checks failed:"
        echo "$CI_CHECKS" | jq -r '.[] | select(.conclusion == "FAILURE" or .conclusion == "CANCELLED") | "  - \(.name): \(.conclusion)"'
        exit 1
    fi

    if [ "$PENDING_COUNT" -eq 0 ] && [ "$SUCCESS_COUNT" -eq "$TOTAL_COUNT" ]; then
        echo "✅ All CI checks passed successfully!"
        return 0
    fi

    echo "⏳ $PENDING_COUNT checks still running, waiting $CHECK_INTERVAL seconds..."
    sleep $CHECK_INTERVAL
done
```

## Architecture Patterns

### Cleo → Tess Handoff Flow
```
Cleo Quality Work → CI Success → Ready-for-QA Label → Webhook Event → Tess Resume
```

### Idempotent Label Management
All label operations are idempotent to handle:
- Multiple Cleo runs on same PR
- Webhook delivery retries
- Manual label additions
- Race conditions between agents

### Event-Driven Coordination
The ready-for-qa label serves as:
- **Explicit handoff signal** from Cleo to Tess
- **Event correlation key** for Argo Events sensors
- **Prerequisite check** for Tess workflow initiation
- **Audit trail** for workflow progression

## Testing Strategy

### Label Addition Testing
1. **Successful Label Addition**
   ```bash
   # Test successful ready-for-qa label addition
   PR_NUMBER=123

   # Remove label if present (test setup)
   gh pr edit "$PR_NUMBER" --remove-label "ready-for-qa" || true

   # Test label addition
   add-ready-for-qa-label.sh "$PR_NUMBER"

   # Verify label present
   LABELS=$(gh pr view "$PR_NUMBER" --json labels --jq '.labels[].name')
   if echo "$LABELS" | grep -q "ready-for-qa"; then
       echo "✅ Label addition test passed"
   else
       echo "❌ Label addition test failed"
   fi
   ```

2. **Idempotent Operation Testing**
   ```bash
   # Test that repeated label addition is safe
   add-ready-for-qa-label.sh "$PR_NUMBER"
   add-ready-for-qa-label.sh "$PR_NUMBER"  # Should not fail

   # Verify only one ready-for-qa label exists
   LABEL_COUNT=$(gh pr view "$PR_NUMBER" --json labels \
       --jq '.labels[] | select(.name == "ready-for-qa") | .name' | wc -l)
   [ "$LABEL_COUNT" -eq 1 ] || echo "❌ Multiple ready-for-qa labels found"
   ```

### CI Integration Testing
1. **CI Success Detection**
   - Create PR with passing CI checks
   - Test wait-for-ci-success.sh completes successfully
   - Test timeout behavior with slow CI

2. **CI Failure Handling**
   - Create PR with failing CI checks
   - Test wait-for-ci-success.sh exits with error
   - Test no ready-for-qa label added when CI fails

### Event Integration Testing
1. **Sensor Trigger Testing**
   - Add ready-for-qa label manually
   - Verify Argo Events sensor detects label addition
   - Test workflow resumption at correct stage

2. **Task Correlation Testing**
   - Test task ID extraction from PR labels
   - Verify correct workflow targeted for resumption
   - Test multiple concurrent PRs don't interfere

## Key Design Decisions

1. **Explicit Handoff Signal**: Ready-for-qa label provides clear, visible handoff
2. **CI Dependency**: Label only added after CI success ensures quality gate
3. **Idempotent Operations**: All label operations safe to retry and duplicate
4. **GitHub API Integration**: Direct API calls ensure reliable label management
5. **Event-Driven Architecture**: Label triggers Tess workflow through Argo Events

## References

- [GitHub CLI Labels Documentation](https://cli.github.com/manual/gh_pr_edit)
- [GitHub API Pull Request Labels](https://docs.github.com/en/rest/pulls/pulls#update-a-pull-request)
- [Argo Events Pull Request Webhooks](https://argoproj.github.io/argo-events/eventsources/setup/github/)
- [Multi-Agent Architecture](/.taskmaster/docs/architecture.md)
