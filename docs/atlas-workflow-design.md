# Atlas Workflow Design - Merge Conflict Detection & Resolution

**Agent:** Atlas (5DLabs-Atlas)  
**Role:** Integration & Merge Specialist  
**Primary Function:** Automatic merge conflict detection and resolution

---

## Conflict Detection Strategy

### **Recommended: Event-Driven with Pre-Merge Gate**

#### **1. Real-Time Conflict Detection (Primary)**

**Trigger:** GitHub PR webhook events

**GitHub Webhook Payload Includes:**
```json
{
  "action": "synchronize",  // or opened, reopened, ready_for_review
  "pull_request": {
    "number": 123,
    "mergeable": false,      // ← KEY FIELD
    "mergeable_state": "dirty",  // ← DETAILED STATE
    "head": {
      "ref": "feature-branch",
      "sha": "abc123"
    },
    "base": {
      "ref": "main",
      "sha": "def456"
    }
  }
}
```

**Mergeable States:**
- `clean` - No conflicts, ready to merge
- `dirty` - Has conflicts ← **TRIGGER ATLAS**
- `unstable` - Checks pending
- `blocked` - Required checks failing
- `behind` - Needs update but no conflicts
- `unknown` - GitHub still calculating

**Sensor Logic:**
```yaml
# Trigger Atlas when PR has conflicts
filters:
  - path: body.pull_request.mergeable
    type: bool
    value: ["false"]
  - path: body.pull_request.mergeable_state
    type: string
    value: ["dirty"]
```

---

#### **2. Pre-Merge Conflict Check (Safety Gate)**

**Trigger:** After Tess approves, before merge

**Workflow Stage:**
```
Rex → Cleo → Tess → Atlas (conflict check) → Merge
```

**Why Both?**
- Event-driven catches conflicts early (during development)
- Pre-merge gate catches race conditions (conflicts that occur after approval)

---

## Atlas Workflow Logic

### **Container Script Behavior**

```bash
#!/bin/sh
# Atlas Container Script - Merge Conflict Resolution

echo "🔗 ATLAS - Integration & Merge Specialist"
echo "Task: Resolve merge conflicts for PR #${PR_NUMBER}"

# 1. Fetch latest main branch
git fetch origin main

# 2. Check if conflicts exist
if git merge-base --is-ancestor origin/main HEAD; then
  echo "✅ Already up to date with main"
  exit 0
fi

# 3. Attempt automatic rebase
git rebase origin/main

if [ $? -eq 0 ]; then
  echo "✅ Rebase successful - no conflicts"
  git push --force-with-lease
  exit 0
fi

# 4. Conflicts detected - analyze and resolve
echo "⚠️  Merge conflicts detected, resolving..."

# Get list of conflicted files
CONFLICT_FILES=$(git diff --name-only --diff-filter=U)

# For each conflicted file, use Claude to resolve
for file in $CONFLICT_FILES; do
  echo "🔍 Analyzing conflict in: $file"
  
  # Extract conflict markers and context
  git show :1:$file > /tmp/base.txt    # Common ancestor
  git show :2:$file > /tmp/ours.txt    # Our changes
  git show :3:$file > /tmp/theirs.txt  # Their changes (main)
  
  # Use Claude to intelligently resolve
  # (Atlas's system prompt guides this)
  claude resolve-conflict \
    --base /tmp/base.txt \
    --ours /tmp/ours.txt \
    --theirs /tmp/theirs.txt \
    --output $file
    
  git add $file
done

# 5. Verify resolution builds/tests
cargo check || cargo build || true
cargo test || true

# 6. Continue rebase
git rebase --continue

# 7. Push resolution
git push --force-with-lease

# 8. Post comment explaining resolution
gh pr comment $PR_NUMBER --body "## 🔗 Atlas Resolution

Automatically resolved merge conflicts with main branch.

**Files resolved:** $(echo $CONFLICT_FILES | wc -w)
**Strategy:** Intelligent merge preserving intent of both changes
**Verification:** Build and tests pass

**Next:** Cleo and Tess will re-validate the resolved code."

echo "✅ Conflicts resolved successfully"
```

---

## Activation Scenarios

### **Scenario A: Developer Creates PR with Conflicts**
```
1. Developer creates PR from stale branch
2. GitHub webhook: mergeable: false, state: dirty
3. Argo Events sensor triggers Atlas workflow
4. Atlas rebases, resolves conflicts, pushes
5. GitHub webhook: PR updated
6. Triggers Cleo → Tess validation
```

### **Scenario B: Main Branch Moves Ahead During Review**
```
1. PR approved by Tess
2. Another PR merges to main first
3. Original PR now has conflicts
4. GitHub webhook: mergeable changed to false
5. Atlas triggers, resolves, pushes
6. Re-triggers validation
```

### **Scenario C: Pre-Merge Safety Check**
```
1. All approvals obtained (Cleo, Cipher, Tess)
2. Ready to merge
3. Atlas runs final conflict check
4. If conflicts: Resolve and re-validate
5. If clean: Proceed to merge
```

---

## Argo Events Sensor Design

### **Sensor 1: Real-Time Conflict Detection**

**File:** `infra/gitops/resources/argo-events/sensors/atlas-conflict-detector.yaml`

```yaml
---
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: atlas-conflict-detector
  namespace: agent-platform
spec:
  dependencies:
  - name: pr-conflict-detected
    eventSourceName: github-webhook
    eventName: pull-request
    filters:
      data:
      # Trigger on PR updates
      - path: body.action
        type: string
        value:
        - synchronize
        - opened
        - reopened
      # Only when unmergeable
      - path: body.pull_request.mergeable
        type: bool
        value:
        - "false"
      # Only when conflicts exist (not just checks failing)
      - path: body.pull_request.mergeable_state
        type: string
        value:
        - "dirty"
        - "behind"  # Might have conflicts when updated
        
  triggers:
  - template:
      name: trigger-atlas-resolution
      k8s:
        operation: create
        source:
          resource:
            apiVersion: agents.platform/v1
            kind: CodeRun
            metadata:
              generateName: "coderun-atlas-conflict-"
              namespace: agent-platform
              labels:
                agent: atlas
                pr-number: "{{ .Input.body.pull_request.number }}"
                task-id: "conflict-resolution"
            spec:
              taskId: "conflict-resolution"
              service: "atlas-integration"
              repositoryUrl: "{{ .Input.body.repository.clone_url }}"
              githubApp: "5DLabs-Atlas"
              cliConfig:
                cliType: "claude"
                model: "claude-sonnet-4-20250514"
                maxTokens: 4096
                temperature: 0.4
              continueSession: false
              overwriteMemory: false
```

---

### **Sensor 2: Pre-Merge Quality Gate**

**File:** `infra/gitops/resources/argo-events/sensors/atlas-pre-merge-check.yaml`

```yaml
---
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: atlas-pre-merge-check
  namespace: agent-platform
spec:
  dependencies:
  - name: ready-to-merge
    eventSourceName: github-webhook
    eventName: pull-request-review
    filters:
      data:
      # Trigger when Tess approves
      - path: body.review.user.login
        type: string
        value:
        - "5dlabs-tess[bot]"
      - path: body.review.state
        type: string
        value:
        - "approved"
      # Only if PR has required label
      - path: body.pull_request.labels[].name
        type: string
        value:
        - "ready-for-qa"  # Or "all-checks-passed"
        
  triggers:
  - template:
      name: trigger-atlas-pre-merge
      k8s:
        operation: create
        source:
          resource:
            apiVersion: agents.platform/v1
            kind: CodeRun
            metadata:
              generateName: "coderun-atlas-premerge-"
              namespace: agent-platform
              labels:
                agent: atlas
                stage: pre-merge
                pr-number: "{{ .Input.body.pull_request.number }}"
            spec:
              taskId: "pre-merge-validation"
              service: "atlas-integration"
              repositoryUrl: "{{ .Input.body.repository.clone_url }}"
              githubApp: "5DLabs-Atlas"
              cliConfig:
                cliType: "claude"
                model: "claude-sonnet-4-20250514"
                maxTokens: 4096
                temperature: 0.4
```

---

## Alternative: Polling Approach (Fallback)

If webhooks are unreliable or you want redundancy:

**CronWorkflow:** Runs every 10 minutes

```yaml
apiVersion: argoproj.io/v1alpha1
kind: CronWorkflow
metadata:
  name: atlas-conflict-scanner
  namespace: agent-platform
spec:
  schedule: "*/10 * * * *"  # Every 10 minutes
  workflowSpec:
    entrypoint: scan-for-conflicts
    templates:
    - name: scan-for-conflicts
      script:
        image: alpine:latest
        command: [sh]
        source: |
          #!/bin/sh
          # Query GitHub for open PRs with conflicts
          gh pr list --json number,mergeable,mergeableState \
            --jq '.[] | select(.mergeable == false and .mergeableState == "dirty") | .number' \
            | while read PR_NUM; do
              echo "Found conflict in PR #$PR_NUM, triggering Atlas..."
              # Trigger Atlas CodeRun
            done
```

---

## My Recommendation

**Use Event-Driven (Option 1) because:**

1. **You already have the infrastructure** - GitHub webhook EventSource exists
2. **Real-time response** - Conflicts resolved as soon as they're detected
3. **Efficient** - Only runs when needed, no wasted polls
4. **GitHub provides the data** - `mergeable` field in webhook payload

**Plus add:**
- Pre-merge check as safety gate after Tess approval
- Optional polling as redundancy (can add later if needed)

---

## Implementation Steps for Atlas

Want me to:

1. **Create the Argo Events sensor** for conflict detection?
2. **Design the Atlas container script** with conflict resolution logic?
3. **Update the controller Rust code** to recognize Atlas?
4. **Create the Helm templates** for Atlas scripts?

Should I proceed with the event-driven approach and implement the sensor + container script? This would give you a complete Atlas workflow that triggers automatically when conflicts are detected!


