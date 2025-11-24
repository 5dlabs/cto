# Atlas PR Guardian Event Flow

## Event Processing Flow (Before Fix - BROKEN)

```
┌─────────────────────────────────────────────────────────────────┐
│                    GitHub Webhook Events                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              Argo Events - GitHub EventSource                   │
│                  (Receives ALL events)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│            Atlas PR Guardian Sensor - Data Filters              │
│  ✅ Repository: 5dlabs/cto                                      │
│  ✅ Event Types: pull_request, issue_comment, pull_request_review│
│  ✅ Actions: opened, reopened, synchronize, ready_for_review... │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│         Atlas PR Guardian Sensor - Expression Filter            │
│  ❌ BROKEN EXPRESSION:                                          │
│     body.X-GitHub-Event != "issue_comment" ||                   │
│     body.issue.pull_request != null                             │
│                                                                 │
│  Problem: Tries to access body.issue.pull_request on ALL events│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────┴─────────┐
                    │                   │
                    ▼                   ▼
        ┌───────────────────┐   ┌──────────────────┐
        │  pull_request     │   │  issue_comment   │
        │  Event            │   │  Event           │
        │                   │   │                  │
        │  body.pull_request│   │  body.issue.     │
        │  EXISTS ✅        │   │  pull_request    │
        │                   │   │  EXISTS ✅       │
        │  body.issue.      │   │                  │
        │  pull_request     │   │  ✅ PASSES       │
        │  MISSING ❌       │   │  FILTER          │
        │                   │   │                  │
        │  ❌ FILTERING     │   └──────────────────┘
        │  ERROR!           │            │
        │                   │            ▼
        │  Event DISCARDED  │   ┌──────────────────┐
        └───────────────────┘   │ CodeRun Created  │
                                │ (Only for        │
                                │  comments on PRs)│
                                └──────────────────┘

Result: NO Atlas CodeRuns created for PR events!
        Only issue_comment events pass through.
        Atlas never monitors PRs.
```

## Event Processing Flow (After Fix - WORKING)

```
┌─────────────────────────────────────────────────────────────────┐
│                    GitHub Webhook Events                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              Argo Events - GitHub EventSource                   │
│                  (Receives ALL events)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│            Atlas PR Guardian Sensor - Data Filters              │
│  ✅ Repository: 5dlabs/cto                                      │
│  ✅ Event Types: pull_request, issue_comment, pull_request_review│
│  ✅ Actions: opened, reopened, synchronize, ready_for_review... │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│         Atlas PR Guardian Sensor - Expression Filter            │
│  ✅ FIXED EXPRESSION:                                           │
│     body.X-GitHub-Event == "pull_request" ||                    │
│     body.X-GitHub-Event == "pull_request_review" ||             │
│     (body.X-GitHub-Event == "issue_comment" &&                  │
│      has(body.issue.pull_request))                              │
│                                                                 │
│  Solution: Explicitly handles each event type                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                ┌─────────────┴─────────────┐
                │                           │
                ▼                           ▼
    ┌───────────────────┐       ┌──────────────────────┐
    │  pull_request     │       │  pull_request_review │
    │  Event            │       │  Event               │
    │                   │       │                      │
    │  X-GitHub-Event   │       │  X-GitHub-Event      │
    │  == "pull_request"│       │  == "pull_request_   │
    │  ✅ TRUE          │       │      review"         │
    │                   │       │  ✅ TRUE             │
    │  ✅ PASSES        │       │                      │
    │  FILTER           │       │  ✅ PASSES           │
    └───────────────────┘       │  FILTER              │
                │               └──────────────────────┘
                │                           │
                │               ┌───────────┘
                │               │
                ▼               ▼
        ┌──────────────────────────────┐
        │  issue_comment Event         │
        │                              │
        │  X-GitHub-Event ==           │
        │  "issue_comment" ✅ TRUE     │
        │                              │
        │  has(body.issue.pull_request)│
        │  ✅ TRUE (if on PR)          │
        │  ❌ FALSE (if on issue)      │
        │                              │
        │  ✅ PASSES FILTER            │
        │  (only if on PR)             │
        └──────────────────────────────┘
                │
                ▼
        ┌──────────────────┐
        │  ALL PR Events   │
        │  Pass Through!   │
        └──────────────────┘
                │
                ▼
        ┌──────────────────┐
        │  CodeRun Created │
        │                  │
        │  coderun-atlas-  │
        │  pr-xxxxx        │
        │                  │
        │  ✅ Atlas Active │
        └──────────────────┘
                │
                ▼
        ┌──────────────────┐
        │  Atlas Monitors  │
        │  PR:             │
        │  - Bugbot        │
        │  - CI Status     │
        │  - Conflicts     │
        │  - Auto-merge    │
        └──────────────────┘

Result: Atlas CodeRuns created for ALL PR events!
        Full PR monitoring and auto-merge functionality.
```

## GitHub Webhook Event Structures

### pull_request Event
```json
{
  "X-GitHub-Event": "pull_request",
  "action": "opened",
  "pull_request": {
    "number": 123,
    "html_url": "https://github.com/5dlabs/cto/pull/123",
    "mergeable": true,
    "mergeable_state": "clean"
  },
  "repository": {
    "full_name": "5dlabs/cto",
    "clone_url": "https://github.com/5dlabs/cto.git"
  }
}
```
**Note**: No `issue` field exists!

### issue_comment Event (on PR)
```json
{
  "X-GitHub-Event": "issue_comment",
  "action": "created",
  "issue": {
    "number": 123,
    "pull_request": {
      "url": "https://api.github.com/repos/5dlabs/cto/pulls/123"
    }
  },
  "comment": {
    "body": "Please fix the linting errors"
  },
  "repository": {
    "full_name": "5dlabs/cto"
  }
}
```
**Note**: `issue.pull_request` only exists if comment is on a PR!

### issue_comment Event (on Issue)
```json
{
  "X-GitHub-Event": "issue_comment",
  "action": "created",
  "issue": {
    "number": 456
    // NO pull_request field!
  },
  "comment": {
    "body": "This is a comment on an issue"
  },
  "repository": {
    "full_name": "5dlabs/cto"
  }
}
```
**Note**: `issue.pull_request` does NOT exist for regular issues!

### pull_request_review Event
```json
{
  "X-GitHub-Event": "pull_request_review",
  "action": "submitted",
  "review": {
    "state": "approved",
    "body": "Looks good!"
  },
  "pull_request": {
    "number": 123,
    "html_url": "https://github.com/5dlabs/cto/pull/123"
  },
  "repository": {
    "full_name": "5dlabs/cto"
  }
}
```
**Note**: No `issue` field exists!

## Filter Expression Comparison

### Before Fix (BROKEN)
```javascript
body.X-GitHub-Event != "issue_comment" || body.issue.pull_request != null
```

**Logic**:
- If event is NOT issue_comment → Check `body.issue.pull_request != null`
- If event IS issue_comment → Check `body.issue.pull_request != null`

**Problem**:
- `pull_request` events: Tries to access `body.issue.pull_request` → ERROR
- `pull_request_review` events: Tries to access `body.issue.pull_request` → ERROR
- `issue_comment` events: Accesses `body.issue.pull_request` → OK

### After Fix (WORKING)
```javascript
body.X-GitHub-Event == "pull_request" || 
body.X-GitHub-Event == "pull_request_review" || 
(body.X-GitHub-Event == "issue_comment" && has(body.issue.pull_request))
```

**Logic**:
- If event IS pull_request → PASS (no field access needed)
- If event IS pull_request_review → PASS (no field access needed)
- If event IS issue_comment → Check if `body.issue.pull_request` exists using `has()`

**Result**:
- `pull_request` events: PASS ✅
- `pull_request_review` events: PASS ✅
- `issue_comment` events on PRs: PASS ✅
- `issue_comment` events on issues: FAIL ❌ (as intended)

## Key Differences

| Aspect | Before Fix | After Fix |
|--------|-----------|-----------|
| **pull_request events** | ❌ Filtering error | ✅ Pass |
| **pull_request_review events** | ❌ Filtering error | ✅ Pass |
| **issue_comment on PR** | ✅ Pass | ✅ Pass |
| **issue_comment on issue** | ❌ Pass (bug) | ❌ Fail (correct) |
| **Atlas CodeRuns created** | 0 | All PR events |
| **Auto-merge functionality** | Broken | Working |

## Testing the Fix

### Before Merge
```bash
# Check current expression
kubectl get sensor atlas-pr-guardian -n argo -o jsonpath='{.spec.dependencies[0].filters.exprs[0].expr}'

# Should show BROKEN expression:
# body.X-GitHub-Event != "issue_comment" || body.issue.pull_request != null

# Check for filtering errors
kubectl logs $(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) -n argo --tail=100 | grep "filtering error"

# Should show many errors
```

### After Merge
```bash
# Check updated expression
kubectl get sensor atlas-pr-guardian -n argo -o jsonpath='{.spec.dependencies[0].filters.exprs[0].expr}'

# Should show FIXED expression:
# body.X-GitHub-Event == "pull_request" || body.X-GitHub-Event == "pull_request_review" || (body.X-GitHub-Event == "issue_comment" && has(body.issue.pull_request))

# Check for filtering errors
kubectl logs $(kubectl get pods -n argo -l sensor-name=atlas-pr-guardian -o name | head -1) -n argo --tail=100 | grep "filtering error"

# Should show NO errors

# Create test PR and verify CodeRun
kubectl get coderun -n agent-platform -l agent=atlas,role=pr-guardian

# Should show: coderun-atlas-pr-xxxxx
```

