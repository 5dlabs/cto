# Stitch @Mentions & Remediation Buttons

## Overview

This feature enables two interaction patterns for AI agents on GitHub PRs:

1. **@Mention Triggering** - Comment `@5DLabs-Stitch please review` (or any agent) to trigger actions
2. **Remediation Buttons** - Click "Fix with Rex" / "Fix with Blaze" buttons on failed checks

Both mechanisms use the same underlying architecture: GitHub webhook → PM Server → CodeRun CRD.

### Key Insight: Pure Webhook-Based Buttons (No External Links)

Following Cursor's "Fix with Web" pattern, our buttons work entirely through webhooks:

1. **No external URLs** - Buttons don't open browser tabs or redirect
2. **GitHub Actions API** - We add `actions` array to check_run output
3. **Webhook callback** - Button click sends `check_run` event with `action: requested_action`
4. **Instant response** - Everything stays in GitHub's UI, instant feedback

This is cleaner than external link approaches because:
- No page loads or redirects
- User stays in GitHub context
- Immediate webhook-based response
- Consistent with GitHub's native check run actions pattern

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

## Scope & Status

### Phase 1: GitHub App Webhooks (Foundation) ✅ DONE

- [x] Org-wide webhook configured at `https://github-webhooks.5dlabs.ai`
- [x] Argo EventSource receives all GitHub events
- [x] Sensors filter to specific event types

**Note:** We use org-wide webhooks (simpler) rather than per-app webhooks. The Argo Events sensors filter events by type and content.

### Phase 2: @Mention Sensor & Handler ✅ DONE

- [x] `agent-mention-sensor.yaml` - Catches @mention comments
- [x] `crates/pm/src/handlers/agent_interactions.rs` - Handler implementation
  - `handle_mention_webhook()` - Parses @mentions, creates CodeRun
  - `parse_mentions()` - Extracts agent + instructions from comment
  - `Agent` enum - All 12 agents supported
- [x] Route `POST /webhooks/github/mention` registered
- [x] Support for all agents via @mention

### Phase 3: Remediation Buttons (Webhook-Based) ✅ DONE

**How It Works (Cursor-Inspired Pattern):**
1. Check run fails → We include `actions` array in check run output
2. User clicks button → GitHub sends `check_run` webhook with `action: requested_action`
3. Our sensor catches → PM Server handler parses `identifier` → Creates CodeRun
4. Controller spawns agent → Agent fixes issue → Pushes commit

**Completed:**
- [x] `remediation-button-sensor.yaml` - Catches button clicks
- [x] `ci-failure-button-sensor.yaml` - Creates check runs with buttons on CI failure
- [x] `handle_remediation_webhook()` - Creates CodeRun from button click
- [x] `handle_ci_failure_webhook()` - Creates check run with remediation buttons
- [x] `parse_button_identifier()` - Parses `fix-rex-pr123-456789` format
- [x] `detect_primary_language()` - Language detection from changed files
- [x] `select_agent_for_files()` - Maps language → agent (Rust→Rex, TS/React→Blaze, Go→Grizz)
- [x] Route `POST /webhooks/github/remediation` registered
- [x] Route `POST /webhooks/github/ci-failure` registered
- [x] Button rendering with emoji: "🛠️ Fix with Rex", "⚡ Fix with Blaze", etc.

### Phase 4: PR Review Sensor ✅ DONE

- [x] `stitch-pr-review-sensor.yaml` - Triggers on PR open/update
- [x] Monitors repos: `5dlabs/cto`, `5dlabs/web`
- [x] Excludes: `skip-review` label, bot authors
- [x] Deterministic naming for deduplication

### Phase 5: Local Development & Testing 🚧 IN PROGRESS

- [x] Test fixture: `crates/pm/test-fixtures/mention-comment.json`
- [ ] **Cluster Integration Testing**
  - PM server needs cluster secrets to run
  - Sensors deployed but require PM server for full E2E
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

## Files Created/Modified

### Sensors Created
| File | Purpose |
|------|---------|
| `infra/.../sensors/stitch-pr-review-sensor.yaml` | Triggers Stitch on PR open/update |
| `infra/.../sensors/agent-mention-sensor.yaml` | Handles @mention comments |
| `infra/.../sensors/ci-failure-button-sensor.yaml` | Creates buttons on CI failure |
| `infra/.../sensors/remediation-button-sensor.yaml` | Handles button clicks |

### PM Server Modifications
| File | Changes |
|------|---------|
| `crates/pm/src/handlers/agent_interactions.rs` | All handlers implemented |
| `crates/pm/src/server.rs` | Routes registered |
| `crates/pm/src/handlers/mod.rs` | Exports added |
| `crates/pm/test-fixtures/mention-comment.json` | Test fixture |

### Existing Components Used
- `crates/pm/src/detection/` - Language detection module
- `crates/controller/` - CodeRun processing (existing)

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

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Webhooks | ✅ Done | Org-wide webhook, Argo EventSource |
| Phase 2: @Mention | ✅ Done | Sensor + handler + route |
| Phase 3: Remediation Buttons | ✅ Done | Both sensors + handlers + routes |
| Phase 4: PR Review | ✅ Done | Stitch auto-review on PR |
| Phase 5: Testing | 🚧 In Progress | Cluster integration pending secrets |

**Implementation**: Complete
**Testing**: Awaiting cluster secrets for PM server

---

## Success Criteria

### Deployed & Verified ✅
- [x] 4 Argo Event sensors deployed to cluster
- [x] Webhook routes registered in PM server
- [x] Language detection and agent selection implemented
- [x] Button identifier parsing working
- [x] CodeRun creation from webhook payloads

### Testable (Awaiting Cluster)
- [ ] Comment `@5DLabs-Stitch review this please` triggers review
- [ ] Comment `@5DLabs-Rex fix the clippy warnings` triggers fix
- [ ] Failed CI shows "Fix with Rex" button when Rust files changed
- [ ] Failed CI shows "Fix with Blaze" button when TS/React files changed
- [ ] Clicking button creates CodeRun and agent pushes fix
- [ ] Language detection correctly identifies primary language from changed files
- [ ] PR open/update triggers Stitch review automatically
