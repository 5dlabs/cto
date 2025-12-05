# Monitor Alerts System

## Overview

The Monitor uses a **hybrid architecture**:
- **Rust code** detects conditions/anomalies (fast, deterministic)
- **Factory (AI)** analyzes detected issues and produces remediation guidance (intelligent, contextual)

The monitor has **two distinct flows**:

### Flow 1: Alerts (Reactive)
Detects anomalies **during** execution. When triggered:
1. Rust detects condition (A1-A8)
2. Factory spawned to analyze
3. Factory writes issue to PVC
4. Remediation Agent spawned

### Flow 2: Completion Analysis (Proactive)
Runs **after** each agent pod succeeds:
1. Pod completes successfully
2. Full logs archived to PVC
3. Factory spawned with "expected behavior" prompts
4. Factory verifies logs contain expected patterns
5. If issues found → Remediation Agent spawned

**Key Insight**: Multiple remediation agents can run concurrently as issues are detected from different sources.

---

## Log Retention

| Workflow Type | TTL After Completion | Action Required |
|---------------|---------------------|-----------------|
| **Play Workflow** | 24 hours | Archive immediately on completion |
| **CodeRun/DocsRun** | 5 minutes | **CRITICAL**: Must archive before TTL |

The monitor **must** archive logs to `/workspace/watch/logs/` before pod garbage collection.

---

## Alert Definitions

| ID | Alert Name | Detection Logic (Rust) | Trigger Condition | Factory Prompt Template |
|----|------------|------------------------|-------------------|-------------------------|
| A1 | Agent Comment Order Mismatch | Compare GitHub PR comments vs workflow step execution | Agent N is running but no GitHub comment exists from Agent N-1 | [A1 Prompt](#a1-agent-comment-order-mismatch) |
| A2 | Silent Agent Failure | Container terminated while pod still "Running" | `containerStatuses[].state.terminated` with non-zero exit code | [A2 Prompt](#a2-silent-agent-failure) |
| A3 | Stale Progress (No Commits) | Track last commit to `feature/task-<id>-implementation` branch | No new commits in >15 minutes while agent pod is "Running" | [A3 Prompt](#a3-stale-progress) |
| A4 | Repeated Approval Loop | Count approval keywords from same agent | Same agent posts "approved"/"LGTM"/"passing" >2 times | [A4 Prompt](#a4-repeated-approval-loop) |
| A5 | Post-Tess CI/Merge Failure | Poll `gh pr checks` + `mergeable` status after Tess approval | Tess approved but CI checks failing OR `mergeable: false` | [A5 Prompt](#a5-post-tess-cimerge-failure) |
| A7 | Pod Failure (Any CTO Pod) | Watch all pods in `cto` namespace | Any pod enters Failed/Error/CrashLoopBackOff | [A7 Prompt](#a7-pod-failure) |
| A8 | Workflow Step Timeout | Track step duration vs expected | Step running longer than threshold (e.g., >30 min) | [A8 Prompt](#a8-workflow-step-timeout) |

> **Note:** A6 (Merge Conflict) has been consolidated into A5. Both CI failures and merge conflicts after Tess approval are now handled by a single alert, since Atlas is responsible for ensuring the PR can merge cleanly.

---

## Expected Agent Order

For each task, agents should comment on the PR in this order:

```
1. Blaze/Rex (Implementation) → Creates PR, posts initial comment
2. Cleo (Quality)             → Posts code review findings
3. Tess (Testing)             → Posts test results
4. Cipher (Security)          → Posts security scan results
5. Atlas (Integration)        → Posts merge status / final approval
```

### Detection Data Sources

| Data Source | How to Query | What It Tells Us |
|-------------|--------------|------------------|
| GitHub PR Comments | `gh pr view <PR> --json comments` | Which agents have commented, timestamps, content |
| GitHub PR Labels | `gh pr view <PR> --json labels` | `task-<id>`, `run-<workflow>`, agent labels |
| Workflow Steps | `argo get <workflow> -o json` | Current step, phase, which agent is running |
| Pod Status | `kubectl get pod -o json` | Container states, exit codes, restart counts |
| CodeRun Status | `kubectl get coderun -o json` | Agent assignment, phase, timestamps |

---

## Alert Details & Prompts

### A1: Agent Comment Order Mismatch

**Detection Logic (Rust):**
```rust
struct AgentCommentState {
    blaze_rex_commented: bool,
    cleo_commented: bool,
    tess_commented: bool,
    cipher_commented: bool,
    atlas_commented: bool,
}

fn check_comment_order(pr_comments: &[Comment], current_agent: &str) -> Option<Alert> {
    let state = parse_agent_comments(pr_comments);
    
    match current_agent {
        "5DLabs-Cleo" if !state.blaze_rex_commented => Some(Alert::A1("Cleo running but no Rex/Blaze comment")),
        "5DLabs-Tess" if !state.cleo_commented => Some(Alert::A1("Tess running but no Cleo comment")),
        "5DLabs-Cipher" if !state.tess_commented => Some(Alert::A1("Cipher running but no Tess comment")),
        "5DLabs-Atlas" if !state.cipher_commented => Some(Alert::A1("Atlas running but no Cipher comment")),
        _ => None,
    }
}
```

**Factory Prompt:**
```markdown
# Alert A1: Agent Comment Order Mismatch

## Detected Condition
- **Current Agent Running**: {{current_agent}}
- **Expected Previous Agent**: {{expected_previous_agent}}
- **PR Number**: {{pr_number}}
- **Task ID**: {{task_id}}

## GitHub PR Comments Found
{{#each pr_comments}}
- **{{this.author}}** ({{this.created_at}}): {{this.body_preview}}
{{/each}}

## Workflow State
- **Workflow**: {{workflow_name}}
- **Current Step**: {{current_step}}
- **Phase**: {{phase}}

## Your Task
1. Analyze why the previous agent ({{expected_previous_agent}}) did not post a comment
2. Check if the agent failed silently, was skipped, or encountered an error
3. Review the workflow logs for the previous agent's step
4. Write a detailed analysis to `/workspace/watch/alerts/A1-{{timestamp}}.md`

Include:
- Root cause hypothesis
- Relevant log snippets
- Recommended remediation action
```

---

### A2: Silent Agent Failure

**Detection Logic (Rust):**
```rust
fn check_silent_failures(pod_status: &PodStatus) -> Option<Alert> {
    for container in &pod_status.container_statuses {
        if let Some(terminated) = &container.state.terminated {
            if terminated.exit_code != 0 && pod_status.phase == "Running" {
                return Some(Alert::A2(format!(
                    "Container {} exited with code {} but pod still Running",
                    container.name, terminated.exit_code
                )));
            }
        }
    }
    None
}
```

**Factory Prompt:**
```markdown
# Alert A2: Silent Agent Failure

## Detected Condition
- **Pod**: {{pod_name}}
- **Failed Container**: {{container_name}}
- **Exit Code**: {{exit_code}}
- **Exit Reason**: {{exit_reason}}
- **Pod Phase**: Still "Running" (due to sidecar containers)

## Container Logs (Last 500 lines)
```
{{container_logs}}
```

## Your Task
1. Analyze the container logs to identify the root cause of failure
2. Determine if this is a code bug, configuration issue, or infrastructure problem
3. Write a detailed analysis to `/workspace/watch/alerts/A2-{{timestamp}}.md`

Include:
- Exact error message and stack trace
- Root cause analysis
- Files likely affected
- Specific fix recommendation
```

---

### A3: Stale Progress

**Detection Logic (Rust):**
```rust
fn check_stale_progress(
    last_comment_time: DateTime<Utc>,
    workflow_phase: &str,
    threshold_minutes: u64,
) -> Option<Alert> {
    if workflow_phase == "Running" {
        let elapsed = Utc::now() - last_comment_time;
        if elapsed.num_minutes() > threshold_minutes as i64 {
            return Some(Alert::A3(format!(
                "No GitHub activity for {} minutes while workflow running",
                elapsed.num_minutes()
            )));
        }
    }
    None
}
```

**Factory Prompt:**
```markdown
# Alert A3: Stale Progress (No Activity)

## Detected Condition
- **Last GitHub Comment**: {{last_comment_time}} ({{minutes_ago}} minutes ago)
- **Current Agent**: {{current_agent}}
- **Workflow Phase**: Running
- **Threshold**: {{threshold_minutes}} minutes

## Current State
- **PR Number**: {{pr_number}}
- **Workflow Step**: {{current_step}}
- **Pod Status**: {{pod_status}}

## Your Task
1. Check if the agent is stuck, waiting for input, or processing slowly
2. Review current pod logs for activity or errors
3. Determine if intervention is needed
4. Write analysis to `/workspace/watch/alerts/A3-{{timestamp}}.md`
```

---

### A4: Repeated Approval Loop

**Detection Logic (Rust):**
```rust
fn check_approval_loop(pr_comments: &[Comment]) -> Option<Alert> {
    let mut approval_counts: HashMap<String, u32> = HashMap::new();
    
    for comment in pr_comments {
        if comment.body.contains("LGTM") || 
           comment.body.contains("approved") ||
           comment.body.contains("looks good") {
            *approval_counts.entry(comment.author.clone()).or_insert(0) += 1;
        }
    }
    
    for (agent, count) in &approval_counts {
        if *count > 2 {
            return Some(Alert::A4(format!(
                "{} has posted {} approvals - possible loop",
                agent, count
            )));
        }
    }
    None
}
```

**Factory Prompt:**
```markdown
# Alert A4: Repeated Approval Loop

## Detected Condition
- **Agent**: {{agent_name}}
- **Approval Count**: {{approval_count}} (threshold: 2)
- **PR Number**: {{pr_number}}

## Approval Comments Found
{{#each approval_comments}}
- **{{this.created_at}}**: "{{this.body_preview}}"
{{/each}}

## Your Task
1. Analyze why the agent is repeatedly approving without the workflow advancing
2. Check if there's a condition that keeps resetting the agent's state
3. Determine if this is an infinite loop or expected behavior
4. Write analysis to `/workspace/watch/alerts/A4-{{timestamp}}.md`

Consider:
- Is CI failing after approval?
- Is another agent reverting changes?
- Is there a webhook/trigger misconfiguration?
```

---

### A5: CI Check Failure

**Detection Logic (Rust):**
```rust
fn check_ci_failures(pr_checks: &[Check], last_agent_comment: DateTime<Utc>) -> Option<Alert> {
    let failed_checks: Vec<_> = pr_checks
        .iter()
        .filter(|c| c.conclusion == "failure")
        .collect();
    
    if !failed_checks.is_empty() {
        let ci_failed_at = failed_checks.iter()
            .map(|c| c.completed_at)
            .max()
            .unwrap();
        
        // CI failed but no agent comment since
        if ci_failed_at > last_agent_comment {
            return Some(Alert::A5(format!(
                "{} CI checks failed, no agent response",
                failed_checks.len()
            )));
        }
    }
    None
}
```

**Factory Prompt:**
```markdown
# Alert A5: CI Check Failure

## Detected Condition
- **Failed Checks**: {{failed_check_count}}
- **Last Agent Comment**: {{last_agent_comment_time}}
- **CI Failure Time**: {{ci_failure_time}}

## Failed Checks
{{#each failed_checks}}
- **{{this.name}}**: {{this.conclusion}} - {{this.output_summary}}
{{/each}}

## Your Task
1. Analyze the CI failure logs
2. Determine why the current agent hasn't addressed the failure
3. Check if the agent is aware of the failure or stuck
4. Write analysis to `/workspace/watch/alerts/A5-{{timestamp}}.md`
```

---

### A5: Post-Tess CI/Merge Failure

> **Note:** This alert consolidates the former A5 (CI Failure) and A6 (Merge Conflict) into a single check. Both conditions indicate issues that should have been caught before Tess approval.

**Detection Logic (Rust):**
```rust
fn check_post_tess_ci_or_merge_failure(
    pr_comments: &[Comment],
    pr_checks: &[Check],
    pr_status: &PrStatus,
) -> Option<Alert> {
    // Find Tess approval
    let tess_approved = pr_comments.iter().any(|c| {
        c.author.contains("Tess") && 
        (c.body.contains("approved") || c.body.contains("LGTM") || c.body.contains("passing"))
    });
    
    if !tess_approved {
        return None; // Issues before Tess approval are expected during development
    }
    
    // Check for merge conflicts (Atlas should have resolved these)
    if !pr_status.mergeable || pr_status.merge_state_status == "DIRTY" {
        return Some(Alert::A5(format!(
            "Tess approved but PR has merge conflicts - Atlas failed to integrate: {}",
            pr_status.merge_state_status
        )));
    }
    
    // Check if CI is failing AFTER Tess approved
    let tess_approval_time = pr_comments.iter()
        .filter(|c| c.author.contains("Tess") && c.body.contains("approved"))
        .map(|c| c.created_at)
        .max();
    
    let ci_failures_after_tess: Vec<_> = pr_checks.iter()
        .filter(|c| c.conclusion == "failure" && c.completed_at > tess_approval_time.unwrap())
        .collect();
    
    if !ci_failures_after_tess.is_empty() {
        return Some(Alert::A5(format!(
            "Tess approved but {} CI checks failed after approval",
            ci_failures_after_tess.len()
        )));
    }
    None
}
```

**Factory Prompt:**
```markdown
# Alert A5: Post-Tess CI/Merge Failure

## Detected Condition
- **Tess Approval Time**: {{tess_approval_time}}
- **CI Failures After Approval**: {{failed_check_count}}
- **Mergeable**: {{mergeable}}
- **Merge State**: {{merge_state_status}}

This is a critical issue: Tess approved the PR but either:
1. CI checks are still failing, OR
2. The PR has merge conflicts

Either Tess approved incorrectly, Atlas failed to integrate properly,
or something broke after approval.

## Failed Checks (if any)
{{#each failed_checks}}
- **{{this.name}}**: {{this.conclusion}} (completed: {{this.completed_at}})
  - Output: {{this.output_summary}}
{{/each}}

## Merge Conflict Info (if applicable)
- **Base Branch**: {{base_branch}}
- **Head Branch**: {{head_branch}}
{{#if conflicting_files}}
- **Conflicting Files**:
{{#each conflicting_files}}
  - {{this}}
{{/each}}
{{/if}}

## Tess's Approval Comment
{{tess_approval_comment}}

## Your Task
1. Determine the root cause:
   - Did Tess approve despite failing CI?
   - Did new commits after Tess approval break the build?
   - Did a concurrent PR to main create merge conflicts?
   - Did Atlas fail to properly rebase/merge?
2. Check Atlas's logs if merge conflicts exist
3. Determine if Tess or Atlas needs to re-evaluate
4. Write analysis to `/workspace/watch/alerts/A5-{{timestamp}}.md`

Include:
- Root cause analysis
- Which agent is responsible for the failure
- Recommended remediation steps
```

---

### A7: Pod Failure (Any CTO Pod)

**Detection Logic (Rust):**
```rust
fn watch_cto_pod_failures(namespace: &str) -> impl Stream<Item = Alert> {
    // Watch all pods in agent-platform namespace
    let watcher = kubectl_watch_pods(namespace);
    
    watcher.filter_map(|event| {
        match event {
            PodEvent::Modified(pod) | PodEvent::Added(pod) => {
                let phase = pod.status.phase.as_deref().unwrap_or("");
                let restart_count: i32 = pod.status.container_statuses
                    .iter()
                    .map(|c| c.restart_count)
                    .sum();
                
                if phase == "Failed" || phase == "Error" {
                    Some(Alert::A7(format!("Pod {} failed: {}", pod.name, phase)))
                } else if restart_count > 3 {
                    Some(Alert::A7(format!("Pod {} in CrashLoopBackOff ({} restarts)", 
                        pod.name, restart_count)))
                } else {
                    None
                }
            }
            _ => None
        }
    })
}
```

**Factory Prompt:**
```markdown
# Alert A7: CTO Pod Failure

## Detected Condition
- **Pod Name**: {{pod_name}}
- **Namespace**: {{namespace}}
- **Phase**: {{phase}}
- **Restart Count**: {{restart_count}}
- **Labels**: {{labels}}

## Pod Details
- **Task ID**: {{task_id}} (from label)
- **Agent**: {{agent}} (from label)  
- **Workflow**: {{workflow_name}} (from label)

## Container Statuses
{{#each container_statuses}}
- **{{this.name}}**: {{this.state}} (exit code: {{this.exit_code}}, reason: {{this.reason}})
{{/each}}

## Recent Events
{{#each pod_events}}
- {{this.type}}: {{this.reason}} - {{this.message}}
{{/each}}

## Your Task
1. Retrieve and analyze container logs before pod is garbage collected
2. Identify root cause (OOM, code error, config issue, etc.)
3. Archive logs to `/workspace/watch/logs/{{pod_name}}/`
4. Write analysis to `/workspace/watch/alerts/A7-{{timestamp}}.md`
```

---

### A8: Workflow Step Timeout

**Detection Logic (Rust):**
```rust
struct StepTimeouts {
    implementation: Duration,  // Rex/Blaze: 45 min
    quality: Duration,         // Cleo: 15 min
    testing: Duration,         // Tess: 30 min
    security: Duration,        // Cipher: 15 min
    integration: Duration,     // Atlas: 20 min
}

fn check_step_timeout(
    workflow_status: &WorkflowStatus,
    timeouts: &StepTimeouts,
) -> Option<Alert> {
    for node in &workflow_status.nodes {
        if node.phase != "Running" {
            continue;
        }
        
        let duration = Utc::now() - node.started_at;
        let threshold = match node.template_name.as_str() {
            t if t.contains("rex") || t.contains("blaze") => timeouts.implementation,
            t if t.contains("cleo") => timeouts.quality,
            t if t.contains("tess") => timeouts.testing,
            t if t.contains("cipher") => timeouts.security,
            t if t.contains("atlas") => timeouts.integration,
            _ => Duration::minutes(60), // default
        };
        
        if duration > threshold {
            return Some(Alert::A8(format!(
                "Step {} running for {} (threshold: {})",
                node.name,
                humanize_duration(duration),
                humanize_duration(threshold)
            )));
        }
    }
    None
}
```

**Factory Prompt:**
```markdown
# Alert A8: Workflow Step Timeout

## Detected Condition
- **Step Name**: {{step_name}}
- **Agent**: {{agent}}
- **Running For**: {{duration}}
- **Threshold**: {{threshold}}
- **Started At**: {{started_at}}

## Workflow Context
- **Workflow**: {{workflow_name}}
- **Task ID**: {{task_id}}
- **Current Phase**: {{phase}}

## Your Task
1. Check if the agent is stuck, waiting for something, or legitimately working
2. Review recent logs from the step's pod
3. Determine if the step should be terminated or allowed to continue
4. Write analysis to `/workspace/watch/alerts/A8-{{timestamp}}.md`

Consider:
- Is the agent in an infinite loop?
- Is it waiting for external resources (GitHub API, etc.)?
- Is the task genuinely complex and needs more time?
```

---

---

## Completion Analysis (Flow 2)

When an agent pod **succeeds**, the monitor:
1. Archives full logs to PVC
2. Spawns Factory to verify expected behavior
3. If verification fails → spawns Remediation Agent

### Expected Behavior Definitions

| Agent | Expected Log Patterns | Failure Indicators |
|-------|----------------------|-------------------|
| **Rex/Blaze** | `gh pr create`, `PR #\d+ created`, branch pushed | No PR URL in output, "error" in git operations |
| **Cleo** | `cargo fmt`, `cargo clippy`, "0 warnings", "passing" | Non-zero clippy warnings, fmt changes needed |
| **Tess** | `cargo test`, "test result: ok", coverage % | Test failures, "FAILED", panic in test output |
| **Cipher** | Security scan complete, "0 vulnerabilities" | CVE found, "HIGH" or "CRITICAL" severity |
| **Atlas** | `gh pr merge`, "merged", branch deleted | Merge conflict, "cannot merge", CI still failing |

### Completion Check Definitions

| ID | Check Name | Agent | Expected Pattern | Factory Prompt |
|----|------------|-------|------------------|----------------|
| C1 | PR Created | Rex/Blaze | `PR #\d+` or PR URL in logs | [C1 Prompt](#c1-pr-created) |
| C2 | Lint Clean | Cleo | "0 warnings" or "no issues" | [C2 Prompt](#c2-lint-clean) |
| C3 | Tests Passing | Tess | "test result: ok" | [C3 Prompt](#c3-tests-passing) |
| C4 | Security Clear | Cipher | "0 vulnerabilities" | [C4 Prompt](#c4-security-clear) |
| C5 | PR Merged | Atlas | "merged" + branch deleted | [C5 Prompt](#c5-pr-merged) |
| C6 | GitHub Comment Posted | All | Agent posted to PR | [C6 Prompt](#c6-github-comment) |

### Completion Analysis Logic (Rust)

```rust
async fn on_pod_succeeded(pod: &Pod, logs_dir: &str) -> Result<()> {
    let pod_name = &pod.metadata.name;
    let agent = pod.labels.get("agent").unwrap_or(&"unknown".to_string());
    
    // Step 1: Archive full logs immediately (before TTL)
    let logs = kubectl_logs(pod_name, None).await?;
    let log_path = format!("{}/{}-full.log", logs_dir, pod_name);
    tokio::fs::write(&log_path, &logs).await?;
    println!("📁 Archived logs: {}", log_path);
    
    // Step 2: Determine which checks apply to this agent
    let checks = get_checks_for_agent(agent);
    
    // Step 3: Run Rust-based quick checks first
    let mut issues = Vec::new();
    for check in &checks {
        if let Some(issue) = check.quick_verify(&logs) {
            issues.push(issue);
        }
    }
    
    // Step 4: If quick checks found issues OR for deeper analysis, spawn Factory
    if !issues.is_empty() || should_deep_analyze(agent) {
        let prompt = build_completion_prompt(agent, &logs, &issues);
        spawn_factory_analysis(prompt, |analysis| {
            if analysis.issues_found {
                // Spawn remediation agent for each issue
                for issue in analysis.issues {
                    tokio::spawn(create_remediation_coderun(issue));
                }
            }
        }).await?;
    }
    
    Ok(())
}
```

### Completion Prompts

#### C1: PR Created

```markdown
# Completion Check C1: PR Created

## Context
- **Agent**: Rex/Blaze (Implementation)
- **Pod**: {{pod_name}}
- **Task ID**: {{task_id}}
- **Expected**: A pull request should have been created

## Logs
```
{{logs}}
```

## Your Task
1. Search logs for evidence of PR creation:
   - `gh pr create` command execution
   - PR URL (https://github.com/.../pull/...)
   - "Pull request #X created" message
2. If PR was created, verify it targets the correct branch
3. If NO PR found, this is a critical failure

## Output
Write to `/workspace/watch/completion/C1-{{pod_name}}.md`:
- PR_CREATED: true/false
- PR_NUMBER: (if found)
- PR_URL: (if found)
- ISSUE: (if not created, explain why)
```

#### C2: Lint Clean

```markdown
# Completion Check C2: Lint Clean

## Context
- **Agent**: Cleo (Quality)
- **Pod**: {{pod_name}}
- **Expected**: Code should pass all linting with zero warnings

## Logs
```
{{logs}}
```

## Your Task
1. Find `cargo fmt` output - should show "0 files changed" or success
2. Find `cargo clippy` output - should show "0 warnings"
3. Any warnings or errors indicate Cleo didn't fully fix issues

## Output
Write to `/workspace/watch/completion/C2-{{pod_name}}.md`:
- FMT_CLEAN: true/false
- CLIPPY_WARNINGS: (count)
- ISSUES: (list any remaining lint issues)
```

#### C3: Tests Passing

```markdown
# Completion Check C3: Tests Passing

## Context
- **Agent**: Tess (Testing)
- **Pod**: {{pod_name}}
- **Expected**: All tests should pass

## Logs
```
{{logs}}
```

## Your Task
1. Find test execution output (`cargo test` or equivalent)
2. Verify "test result: ok" appears
3. Check for any FAILED tests or panics
4. Note coverage percentage if available

## Output
Write to `/workspace/watch/completion/C3-{{pod_name}}.md`:
- TESTS_PASSING: true/false
- TOTAL_TESTS: (count)
- FAILED_TESTS: (list)
- COVERAGE: (percentage if available)
- ISSUES: (any test failures to investigate)
```

#### C6: GitHub Comment Posted

```markdown
# Completion Check C6: GitHub Comment Posted

## Context
- **Agent**: {{agent}}
- **Pod**: {{pod_name}}
- **Expected**: Agent should have posted a comment to the PR

## Logs
```
{{logs}}
```

## Your Task
1. Search for `gh pr comment` or `gh api` calls to post comments
2. Verify the comment was acknowledged (HTTP 200/201)
3. If no comment posted, the agent didn't report its status

## Output
Write to `/workspace/watch/completion/C6-{{pod_name}}.md`:
- COMMENT_POSTED: true/false
- COMMENT_TYPE: (review/general/approval)
- ISSUES: (if not posted, explain)
```

---

## Concurrent Remediation Agents

When issues are detected (from alerts OR completion checks), remediation agents are spawned **concurrently**:

```rust
async fn spawn_remediation_for_issue(issue: Issue) -> Result<String> {
    let coderun_name = format!("remediation-{}-{}", issue.id, timestamp());
    
    let coderun = CodeRun {
        metadata: ObjectMeta {
            name: coderun_name.clone(),
            labels: btreemap! {
                "issue-id" => issue.id.clone(),
                "issue-type" => issue.issue_type.clone(), // "alert" or "completion"
                "source-agent" => issue.source_agent.clone(),
            },
        },
        spec: CodeRunSpec {
            agent: "5DLabs-Rex".to_string(),
            cli: "factory".to_string(),
            template: "watch/factory".to_string(),
            // Issue context injected via ConfigMap
        },
    };
    
    kubectl_apply(&coderun).await?;
    Ok(coderun_name)
}

// Multiple can run at once
async fn handle_detected_issues(issues: Vec<Issue>) {
    let handles: Vec<_> = issues
        .into_iter()
        .map(|issue| tokio::spawn(spawn_remediation_for_issue(issue)))
        .collect();
    
    // Wait for all to be created (not completed)
    for handle in handles {
        match handle.await {
            Ok(Ok(name)) => println!("✅ Spawned remediation: {}", name),
            Ok(Err(e)) => eprintln!("❌ Failed to spawn remediation: {}", e),
            Err(e) => eprintln!("❌ Task panicked: {}", e),
        }
    }
}
```

### Remediation Agent Isolation

Each remediation agent:
- Has its own CodeRun resource
- Runs in its own Pod
- Works on a specific issue
- Writes to `/workspace/watch/remediation/{issue-id}/`

```
/workspace/watch/
├── alerts/
│   ├── A1-20241130-123456.md
│   └── A7-20241130-123789.md
├── completion/
│   ├── C1-rex-pod-abc.md
│   └── C3-tess-pod-xyz.md
├── logs/
│   ├── rex-pod-abc-full.log
│   ├── cleo-pod-def-full.log
│   └── tess-pod-xyz-full.log
└── remediation/
    ├── A1-20241130-123456/
    │   ├── analysis.md
    │   ├── fix.patch
    │   └── pr-url.txt
    └── C3-tess-pod-xyz/
        ├── analysis.md
        └── fix.patch
```

---

## Data Collection Architecture

### Simplified: Watch + GitHub CLI

| Data | Method | Frequency | Events |
|------|--------|-----------|--------|
| **Pods** | `kubectl get pods -w` | Real-time | Created, Running, Succeeded, Failed |
| **Workflows** | `kubectl get workflows -w` | Real-time | Phase changes |
| **CodeRuns** | `kubectl get coderuns -w` | Real-time | Status changes |
| **GitHub PR** | `gh pr view --json` | Poll 2-5 min | Comments, commits, checks, mergeable |

### Why This Works

1. **K8s Watch** = Real-time, no polling overhead
2. **GitHub CLI** = Single command gets all PR data we need
3. **Pod completion from Watch** = Triggers log archival + completion checks

### Single GitHub Query Gets Everything

```bash
gh pr view <PR> -R <repo> --json comments,commits,statusCheckRollup,mergeable,mergeStateStatus,labels,reviews
```

From this one response we evaluate: A1, A3, A4, A5 (including merge conflicts)

### Implementation

```rust
pub struct MonitorEngine {
    k8s_events: mpsc::Receiver<K8sEvent>,
    github_state: Arc<RwLock<GitHubState>>,
    alert_handlers: Vec<Box<dyn AlertHandler>>,
    completion_handlers: Vec<Box<dyn CompletionHandler>>,
}

impl MonitorEngine {
    pub async fn run(&mut self) -> Result<()> {
        // Spawn K8s watches (real-time)
        self.spawn_k8s_watches().await?;
        
        // Spawn GitHub poller (every 2 min)
        self.spawn_github_poller(Duration::from_secs(120)).await?;
        
        // Main event loop
        while let Some(event) = self.k8s_events.recv().await {
            match event {
                K8sEvent::PodSucceeded(pod) => {
                    // Archive logs immediately
                    self.archive_pod_logs(&pod).await?;
                    // Run completion checks
                    self.run_completion_checks(&pod).await?;
                }
                K8sEvent::PodFailed(pod) => {
                    // Archive logs + trigger A7
                    self.archive_pod_logs(&pod).await?;
                    self.handle_alert(Alert::A7(pod)).await?;
                }
                K8sEvent::WorkflowPhaseChanged(wf) => {
                    // Check for timeouts (A8)
                    self.check_step_timeouts(&wf).await?;
                }
                _ => {}
            }
            
            // Evaluate alerts against current state
            let github = self.github_state.read().await;
            for handler in &self.alert_handlers {
                if let Some(alert) = handler.evaluate(&event, &github) {
                    self.handle_alert(alert).await?;
                }
            }
        }
        Ok(())
    }
}
```

---

## Test Plan: Validating the Monitor

Since some conditions are rare in production, we need a way to **simulate** each alert and completion check to validate the monitor works correctly.

### Test Harness Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        MONITOR TEST HARNESS                              │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                      Test Orchestrator                              │ │
│  │                                                                     │ │
│  │  1. Start monitor in test mode                                     │ │
│  │  2. Inject simulated condition                                     │ │
│  │  3. Verify alert detected                                          │ │
│  │  4. Verify Factory spawned with correct prompt                     │ │
│  │  5. Verify remediation CodeRun created                             │ │
│  │                                                                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ Mock K8s     │  │ Mock GitHub  │  │ Mock Factory │                  │
│  │ Events       │  │ API          │  │ Spawner      │                  │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Alert Test Cases

| ID | Alert | Simulation Method | Verification |
|----|-------|-------------------|--------------|
| A1 | Comment Order Mismatch | Inject K8s event "Cipher running" + GitHub state with no Tess comment | Alert A1 fires, Factory prompt contains "Cipher running but no Tess comment" |
| A2 | Silent Agent Failure | Inject pod with `containerStatuses[0].terminated.exitCode: 1` but `phase: Running` | Alert A2 fires, logs archived |
| A3 | Stale Progress | Inject GitHub state with `commits[-1].date` > 15 min ago + pod Running | Alert A3 fires |
| A4 | Repeated Approval | Inject GitHub comments with 3x "approved" from same author | Alert A4 fires |
| A5 | Post-Tess CI/Merge Failure | Inject Tess approval + (`statusCheckRollup: FAILURE` OR `mergeable: false`) | Alert A5 fires |
| A7 | Pod Failure | Inject K8s pod event with `phase: Failed` | Alert A7 fires, logs archived |
| A8 | Step Timeout | Inject workflow node running > threshold | Alert A8 fires |

### Completion Check Test Cases

| ID | Check | Simulation Method | Verification |
|----|-------|-------------------|--------------|
| C1 | PR Created | Inject pod succeeded + logs WITHOUT `gh pr create` | Check C1 fails, remediation triggered |
| C2 | Lint Clean | Inject pod succeeded + logs with "3 warnings" | Check C2 fails |
| C3 | Tests Passing | Inject pod succeeded + logs with "FAILED" | Check C3 fails |
| C4 | Security Clear | Inject pod succeeded + logs with "HIGH vulnerability" | Check C4 fails |
| C5 | PR Merged | Inject Atlas pod succeeded + logs WITHOUT "merged" | Check C5 fails |
| C6 | Comment Posted | Inject pod succeeded + logs WITHOUT `gh pr comment` | Check C6 fails |

### Test Implementation

```rust
// monitor/src/tests/alert_tests.rs

#[tokio::test]
async fn test_a1_comment_order_mismatch() {
    let mut harness = TestHarness::new();
    
    // Setup: Cipher is running
    harness.inject_k8s_event(K8sEvent::PodRunning(Pod {
        name: "cipher-pod-123".into(),
        labels: btreemap! { "agent" => "5DLabs-Cipher" },
        ..Default::default()
    }));
    
    // Setup: GitHub has Rex comment but NO Cleo or Tess comments
    harness.set_github_state(GitHubState {
        comments: vec![
            Comment { author: "5DLabs-Rex".into(), body: "PR created".into(), .. },
            // Missing: Cleo, Tess
        ],
        ..Default::default()
    });
    
    // Run one iteration
    harness.tick().await;
    
    // Verify
    assert!(harness.alert_fired(AlertId::A1));
    assert!(harness.factory_spawned());
    assert!(harness.factory_prompt_contains("Cipher running but no Tess comment"));
}

#[tokio::test]
async fn test_a2_silent_agent_failure() {
    let mut harness = TestHarness::new();
    
    // Inject pod with terminated container but pod still "Running"
    harness.inject_k8s_event(K8sEvent::PodModified(Pod {
        name: "rex-pod-456".into(),
        phase: "Running".into(), // Pod still running (sidecar alive)
        container_statuses: vec![
            ContainerStatus {
                name: "factory-claude".into(),
                state: ContainerState::Terminated {
                    exit_code: 1,
                    reason: "Error".into(),
                },
            },
            ContainerStatus {
                name: "docker-daemon".into(),
                state: ContainerState::Running,
            },
        ],
        ..Default::default()
    }));
    
    harness.tick().await;
    
    assert!(harness.alert_fired(AlertId::A2));
    assert!(harness.logs_archived("rex-pod-456"));
}

#[tokio::test]
async fn test_c1_pr_not_created() {
    let mut harness = TestHarness::new();
    
    // Rex pod succeeded
    harness.inject_k8s_event(K8sEvent::PodSucceeded(Pod {
        name: "rex-pod-789".into(),
        labels: btreemap! { "agent" => "5DLabs-Rex" },
        ..Default::default()
    }));
    
    // But logs don't contain PR creation
    harness.set_pod_logs("rex-pod-789", r#"
        Starting implementation...
        Making changes...
        git commit -m "feat: implementation"
        git push origin feature/task-1
        # NOTE: Missing gh pr create!
        Done.
    "#);
    
    harness.tick().await;
    
    assert!(harness.completion_check_failed(CheckId::C1));
    assert!(harness.remediation_coderun_created());
}
```

### Live Simulation Commands

For testing in a real cluster without mocks:

```bash
# A1: Force comment order mismatch
# Skip Cleo/Tess by manually advancing workflow
kubectl patch workflow play-123 --type=merge -p '{"status":{"phase":"Running","nodes":{"cipher-step":{"phase":"Running"}}}}'

# A2: Force silent container failure
kubectl exec -it rex-pod-456 -c factory-claude -- kill 1

# A3: Force stale progress (just wait 15+ min with no commits)

# A4: Force repeated approval
for i in {1..4}; do
  gh pr comment 123 -R 5dlabs/repo --body "✅ Approved - iteration $i"
done

# A5: Force post-Tess CI/merge failure
gh pr comment 123 --body "✅ Tess approves"
# Then either:
#   Option 1: Push a commit that breaks CI
#   Option 2: Create a merge conflict:
git checkout main && echo "conflict" >> file.txt && git commit -am "conflict" && git push
# Now the PR has conflicts, triggering A5

# A7: Force pod failure
kubectl delete pod rex-pod-456 --grace-period=0 --force
# Or deploy a pod with bad image

# A8: Force timeout (set low threshold in config)
# monitor.alerts.step_timeout_minutes: 1
# Then any step > 1 min triggers
```

### Test Runner Script

```bash
#!/bin/bash
# scripts/test-monitor-alerts.sh

echo "🧪 Monitor Alert Test Suite"
echo "=========================="

# Start monitor in test mode
heal test-mode --mock-k8s --mock-github &
MONITOR_PID=$!

# Run test scenarios
for test in a1 a2 a3 a4 a5 a7 a8 c1 c2 c3 c4 c5 c6; do
  echo "Testing $test..."
  heal inject-test-scenario --scenario $test
  sleep 2
  
  if heal check-alert-fired --alert $test; then
    echo "✅ $test: PASSED"
  else
    echo "❌ $test: FAILED"
    FAILED=1
  fi
done

kill $MONITOR_PID

if [ -z "$FAILED" ]; then
  echo "✅ All tests passed!"
  exit 0
else
  echo "❌ Some tests failed"
  exit 1
fi
```

### Continuous Validation (Monitor the Monitor)

Run a background watcher that validates the monitor is healthy:

```rust
// Meta-monitor: validates the monitor is working
async fn meta_monitor_loop() {
    loop {
        // Check monitor pod is running
        let monitor_pod = get_pod("play-monitor", "agent-platform").await;
        if monitor_pod.phase != "Running" {
            alert_ops_team("Monitor pod not running!");
        }
        
        // Check monitor has processed events recently
        let last_event = read_file("/workspace/watch/last-event-timestamp").await;
        if Utc::now() - last_event > Duration::minutes(10) {
            alert_ops_team("Monitor hasn't processed events in 10 min");
        }
        
        // Check K8s watches are still alive
        let watch_health = call_monitor_health_endpoint().await;
        if !watch_health.all_watches_active {
            alert_ops_team("Some K8s watches died");
        }
        
        sleep(Duration::from_secs(60)).await;
    }
}
```

---

## Implementation Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         MONITOR POD                                  │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    Rust Alert Engine                          │  │
│  │                                                               │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │  │
│  │  │ GitHub Poll │  │  K8s Watch  │  │ Argo Watch  │          │  │
│  │  │  (60s int)  │  │  (realtime) │  │  (realtime) │          │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘          │  │
│  │         │                │                │                  │  │
│  │         └────────────────┼────────────────┘                  │  │
│  │                          ▼                                    │  │
│  │              ┌───────────────────────┐                       │  │
│  │              │   Alert Evaluator     │                       │  │
│  │              │  (A1, A2, A3, A4...)  │                       │  │
│  │              └───────────┬───────────┘                       │  │
│  │                          │                                    │  │
│  │                    Alert Triggered?                          │  │
│  │                          │                                    │  │
│  │              ┌───────────┴───────────┐                       │  │
│  │              ▼                       ▼                       │  │
│  │         No Alert              Alert Detected                 │  │
│  │        (continue)                   │                        │  │
│  │                                     ▼                        │  │
│  └─────────────────────────────────────┼────────────────────────┘  │
│                                        │                            │
│                                        ▼                            │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                  Spawn Factory (Short-lived)                  │  │
│  │                                                               │  │
│  │  • Load alert-specific prompt template                       │  │
│  │  • Inject context (logs, comments, state)                    │  │
│  │  • Run Factory with prompt                                   │  │
│  │  • Factory writes to /workspace/watch/alerts/                │  │
│  │  • Exit when analysis complete                               │  │
│  │                                                               │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                        │                            │
│                                        ▼                            │
│                          Alert file triggers                        │
│                          Remediation CodeRun                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Adding New Alerts

To add a new alert:

1. **Define the alert** in this table with ID, name, detection logic
2. **Implement Rust detection** in `monitor/src/alerts.rs`
3. **Create prompt template** in `infra/charts/controller/agent-templates/watch/alerts/`
4. **Register in alert evaluator** to include in polling loop

---

## Configuration

```json
{
  "monitor": {
    "alerts": {
      "enabled": true,
      "github_poll_interval_secs": 60,
      "stale_progress_threshold_mins": 15,
      "approval_loop_threshold": 2,
      "factory_model": "claude-sonnet-4-20250514",
      "factory_timeout_secs": 300
    }
  }
}
```
