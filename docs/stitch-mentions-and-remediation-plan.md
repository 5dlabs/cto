# Stitch @Mentions & Remediation Buttons

## Overview

This feature enables two interaction patterns for AI agents on GitHub PRs:

1. **@Mention Triggering** - Comment `@5DLabs-Stitch please review` (or any agent) to trigger actions
2. **Remediation Buttons** - Click "Fix with Rex" / "Fix with Blaze" buttons on failed checks

Both mechanisms use the same underlying architecture: GitHub webhook → PM Server → CodeRun CRD.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GitHub Events                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  PR Comment (@mention)              Check Run (remediation button)   │
│         │                                      │                     │
│         ▼                                      ▼                     │
│  issue_comment webhook              check_run webhook                │
│         │                                      │                     │
└─────────┼──────────────────────────────────────┼─────────────────────┘
          │                                      │
          ▼                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    PM Server (pm.5dlabs.ai)                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  POST /webhooks/github/comment      POST /webhooks/github/action     │
│         │                                      │                     │
│         ▼                                      ▼                     │
│  Parse @mention, extract:           Parse action payload:            │
│  - Agent name                       - Agent from button ID           │
│  - Instructions                     - PR context                     │
│  - PR context                       - Failed check info              │
│         │                                      │                     │
│         └──────────────┬───────────────────────┘                     │
│                        ▼                                             │
│              Create CodeRun CRD                                      │
│              - runType: remediation                                  │
│              - agent: detected                                       │
│              - context: PR + instructions                            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Kubernetes (cto namespace)                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  CodeRun CR created                                                  │
│         │                                                            │
│         ▼                                                            │
│  Controller watches, spawns agent pod                                │
│  - Clones repo                                                       │
│  - Checks out PR branch                                              │
│  - Runs agent with prompt                                            │
│  - Agent makes changes, pushes                                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Scope

### Phase 1: GitHub App Webhooks (Foundation)

- [ ] Configure per-app webhooks (instead of org-wide)
  - Each GitHub App (Stitch, Rex, Morgan, etc.) has its own webhook URL
  - Allows granular control and easier debugging
- [ ] Update Cloudflare Tunnel bindings if needed
- [ ] Verify webhook secret rotation/management

**Webhook URLs:**
| App | Webhook URL | Events |
|-----|-------------|--------|
| 5DLabs-Stitch | `https://pm.5dlabs.ai/webhooks/github/stitch` | `issue_comment`, `pull_request_review_comment` |
| 5DLabs-Rex | `https://pm.5dlabs.ai/webhooks/github/rex` | `check_run` (for remediation buttons) |
| 5DLabs-Blaze | `https://pm.5dlabs.ai/webhooks/github/blaze` | `check_run` |
| (org-wide) | `https://github-webhooks.5dlabs.ai` | `*` (existing, for Play workflow) |

### Phase 2: @Mention Sensor & Handler

- [ ] Create `stitch-mention-sensor.yaml` for Argo Events
  - Filter: `issue_comment` + `pull_request_review_comment` events
  - Match: Body contains `@5DLabs-Stitch` (case insensitive)
  - Exclude: Bot authors, own comments
- [ ] Add PM Server endpoint: `POST /webhooks/github/comment`
  - Parse comment for @mention and instructions
  - Extract PR context (repo, number, branch, files changed)
  - Create CodeRun with appropriate prompt
- [ ] Support multiple agents via mention:
  - `@5DLabs-Stitch` → Code review
  - `@5DLabs-Rex` → Rust fixes
  - `@5DLabs-Blaze` → Frontend fixes
  - `@5DLabs-Grizz` → Go fixes

### Phase 3: Remediation Buttons

- [ ] **Language Detection** in check_run annotations
  - Analyze failed files to determine primary language
  - Map language → agent (Rust→Rex, TS/React→Blaze, Go→Grizz)
- [ ] **Button Rendering** (GitHub Check Run Actions)
  - Add custom actions to check_run output
  - Styling: Black/silver, Cursor-inspired aesthetic
  - Button text: "🛠️ Fix with Rex" / "⚡ Fix with Blaze"
- [ ] **Button Click Handler** (`POST /webhooks/github/action`)
  - GitHub sends `check_run` event with `requested_action`
  - Extract action identifier (contains agent + context)
  - Create CodeRun CR

### Phase 4: Local Development & Testing

- [ ] **Local Controller Testing**
  - Determine: Can controller run locally against remote cluster?
  - Option A: Port-forward K8s API, run controller locally
  - Option B: Use `kind` cluster with controller
  - Option C: Use existing launchd setup, mock K8s API
- [ ] **Language Detection Tests**
  - Unit tests for language → agent mapping
  - Integration tests with sample PRs
- [ ] **E2E Testing**
  - Test @mention flow end-to-end
  - Test button click flow end-to-end

---

## Technical Details

### Agent Selection Logic

```rust
fn select_agent_for_files(files: &[ChangedFile]) -> Agent {
    let mut lang_counts: HashMap<Language, usize> = HashMap::new();
    
    for file in files {
        if let Some(lang) = detect_language(&file.filename) {
            *lang_counts.entry(lang).or_default() += 1;
        }
    }
    
    // Priority order if tied: Rust > Go > TypeScript > Other
    match lang_counts.iter().max_by_key(|(_, count)| *count) {
        Some((Language::Rust, _)) => Agent::Rex,
        Some((Language::Go, _)) => Agent::Grizz,
        Some((Language::TypeScript | Language::JavaScript | Language::TSX, _)) => Agent::Blaze,
        Some((Language::Python, _)) => Agent::Nova, // or dedicated Python agent
        _ => Agent::Rex, // Default to Rex for infra/unknown
    }
}
```

### Check Run Actions (Button Definition)

```json
{
  "actions": [
    {
      "label": "🛠️ Fix with Rex",
      "description": "Launch Rex to fix Rust compilation errors",
      "identifier": "fix-rex-pr123-check456"
    },
    {
      "label": "⚡ Fix with Blaze", 
      "description": "Launch Blaze to fix TypeScript/React issues",
      "identifier": "fix-blaze-pr123-check456"
    }
  ]
}
```

### CodeRun CR for Remediation

```yaml
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: remediate-pr-123-rex
  namespace: cto
  labels:
    trigger: remediation-button
    pr-number: "123"
    agent: rex
spec:
  runType: remediation
  service: cto
  repositoryUrl: https://github.com/5dlabs/cto.git
  workingDirectory: "."
  githubApp: "5DLabs-Rex"
  model: "claude-opus-4-5-20251101"
  env:
    PR_NUMBER: "123"
    PR_BRANCH: "feat/my-feature"
    FAILED_CHECK: "lint-rust"
    REMEDIATION_PROMPT: |
      The CI check 'lint-rust' failed on PR #123. 
      Please analyze the failure and fix the issues.
      
      Failed check output:
      <check_output>
      error[E0308]: mismatched types...
      </check_output>
      
      Make the necessary changes and commit with message:
      "fix: resolve lint-rust failures"
```

---

## Files to Create/Modify

### New Files
- `infra/gitops/manifests/argo-workflows/sensors/stitch-mention-sensor.yaml`
- `crates/pm-server/src/webhooks/github_comment.rs`
- `crates/pm-server/src/webhooks/github_action.rs`
- `crates/pm-server/src/language_detection.rs`
- `crates/controller/src/check_run_actions.rs`

### Modified Files
- `crates/pm-server/src/main.rs` - Add new routes
- `crates/pm-server/src/webhooks/mod.rs` - Export new handlers
- `crates/controller/src/tasks/code/status.rs` - Add remediation buttons to check runs
- `infra/charts/cto/values.yaml` - Webhook URLs if needed

---

## Testing Strategy

### Unit Tests
```bash
cargo test -p pm-server language_detection
cargo test -p controller check_run_actions
```

### Integration Tests
```bash
# Test mention parsing
curl -X POST https://pm.5dlabs.ai/webhooks/github/comment \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: issue_comment" \
  -d @test-fixtures/mention-comment.json

# Verify CodeRun created
kubectl get coderun -n cto -l trigger=mention
```

### E2E Tests
1. Create test PR
2. Add comment: "@5DLabs-Stitch please review the error handling"
3. Verify Stitch agent runs and responds
4. Click "Fix with Rex" button on failed check
5. Verify Rex agent runs and pushes fix

---

## Open Questions

1. **Webhook granularity**: Per-app webhooks vs single org webhook with routing?
   - Per-app: Cleaner, but more webhook configs to manage
   - Org-wide: Single point, but more complex routing logic
   - **Decision**: Start with org-wide (already exists), add per-app later if needed

2. **Local controller testing**: 
   - Can we run controller locally against remote K8s?
   - Need to test with `kubectl port-forward` or kubeconfig
   - **Decision**: Test both approaches, document what works

3. **Button styling**: 
   - GitHub limits customization of check_run actions
   - Can use emoji for visual differentiation
   - **Decision**: Use emoji + clear labels, black/silver in PR comments

---

## Timeline

| Phase | Estimated Time | Dependencies |
|-------|---------------|--------------|
| Phase 1: Webhooks | 2-3 hours | None |
| Phase 2: @Mention | 4-6 hours | Phase 1 |
| Phase 3: Buttons | 4-6 hours | Phase 1 |
| Phase 4: Testing | 2-3 hours | Phases 2-3 |

**Total**: ~12-18 hours of work

---

## Success Criteria

- [ ] Can comment `@5DLabs-Stitch review this please` and get a review
- [ ] Can comment `@5DLabs-Rex fix the clippy warnings` and get a fix commit
- [ ] Failed CI shows "Fix with Rex" button when Rust files changed
- [ ] Failed CI shows "Fix with Blaze" button when TS/React files changed
- [ ] Clicking button creates CodeRun and agent pushes fix
- [ ] Language detection correctly identifies primary language from changed files
- [ ] All interactions work both locally (for testing) and in-cluster
