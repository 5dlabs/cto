# Remediation Buttons Implementation Plan

## Overview

When a CI check fails on a PR, show "Fix with {Agent}" buttons that trigger the appropriate agent to fix the issue.

```
PR with failed CI → Language Detection → Show Button → Click → CodeRun → Agent Fixes
```

---

## Part 1: Language & Framework Detection

### File Pattern → Language Mapping

```rust
pub enum Language {
    Rust,
    Go,
    TypeScript,
    JavaScript,
    CSharp,
    Cpp,
    Python,
}

pub enum Framework {
    // Rust
    Axum, Tokio, Actix,
    
    // Go
    Chi, Gin, Echo,
    
    // TypeScript/JavaScript
    NextJs,      // web-blaze
    Expo,        // mobile-tap
    Electron,    // desktop-spark
    Elysia,      // node-nova (backend)
    Express,     // node-nova (backend)
    
    // C#
    Unity,       // unity-vex
    
    // C++
    Unreal,      // unreal-forge
}
```

### Detection Strategy

**Step 1: Analyze changed files in PR**
```rust
async fn detect_from_changed_files(files: &[ChangedFile]) -> DetectionResult {
    let mut signals = Vec::new();
    
    for file in files {
        // Check file extension
        match file.extension() {
            "rs" => signals.push(Signal::Language(Rust)),
            "go" => signals.push(Signal::Language(Go)),
            "ts" | "tsx" => signals.push(Signal::Language(TypeScript)),
            "cs" => signals.push(Signal::Language(CSharp)),
            "cpp" | "h" => signals.push(Signal::Language(Cpp)),
            _ => {}
        }
        
        // Check path patterns
        if file.path.contains("app/(tabs)") || file.path.contains("expo") {
            signals.push(Signal::Framework(Expo));
        }
        if file.path.contains("src/main") && file.path.contains("electron") {
            signals.push(Signal::Framework(Electron));
        }
        if file.path.contains("Assets/Scripts") {
            signals.push(Signal::Framework(Unity));
        }
        if file.path.contains("Source/") && file.extension() == "cpp" {
            signals.push(Signal::Framework(Unreal));
        }
    }
    
    signals
}
```

**Step 2: Analyze package files (if present)**
```rust
async fn detect_from_package_files(repo: &str, branch: &str) -> DetectionResult {
    // Check for Cargo.toml → Rust
    // Check for go.mod → Go
    // Check for package.json → analyze dependencies:
    //   - "next" → NextJs (Blaze)
    //   - "expo" → Expo (Tap)
    //   - "electron" → Electron (Spark)
    //   - "elysia" or "effect" → Backend Node (Nova)
    //   - "react" without above → generic React (Blaze)
}
```

**Step 3: Distinguish TypeScript Variants**

This is the tricky part - all use TypeScript but need different agents:

| Indicator | Framework | Agent |
|-----------|-----------|-------|
| `next` in package.json | Next.js Web | Blaze |
| `expo` in package.json | Expo Mobile | Tap |
| `electron` in package.json | Electron Desktop | Spark |
| `elysia`/`effect`/`express` + server paths | Node Backend | Nova |
| `react` + `components/` paths | React Web | Blaze |

```rust
fn classify_typescript_project(package_json: &PackageJson, paths: &[String]) -> Agent {
    let deps = &package_json.dependencies;
    
    // Mobile (Tap)
    if deps.contains_key("expo") || deps.contains_key("react-native") {
        return Agent::Tap;
    }
    
    // Desktop (Spark)  
    if deps.contains_key("electron") {
        return Agent::Spark;
    }
    
    // Backend Node (Nova)
    if deps.contains_key("elysia") || deps.contains_key("effect") || 
       deps.contains_key("express") || deps.contains_key("fastify") {
        // Also check if changes are in server/ or api/ paths
        if paths.iter().any(|p| p.contains("server/") || p.contains("api/")) {
            return Agent::Nova;
        }
    }
    
    // Web Frontend (Blaze) - default for React/Next
    if deps.contains_key("next") || deps.contains_key("react") {
        return Agent::Blaze;
    }
    
    // Fallback
    Agent::Blaze
}
```

---

## Part 2: Agent Selection

### Final Mapping

```rust
pub fn select_agent(detection: &DetectionResult) -> Agent {
    // Priority: Framework > Language
    
    // Check framework first (more specific)
    if let Some(framework) = detection.framework {
        return match framework {
            Framework::Axum | Framework::Actix => Agent::Rex,
            Framework::Chi | Framework::Gin => Agent::Grizz,
            Framework::NextJs => Agent::Blaze,
            Framework::Expo => Agent::Tap,
            Framework::Electron => Agent::Spark,
            Framework::Elysia | Framework::Express => Agent::Nova,
            Framework::Unity => Agent::Vex,
            Framework::Unreal => Agent::Forge,
        };
    }
    
    // Fall back to language
    match detection.primary_language {
        Language::Rust => Agent::Rex,
        Language::Go => Agent::Grizz,
        Language::TypeScript | Language::JavaScript => Agent::Blaze, // default TS to web
        Language::CSharp => Agent::Vex,
        Language::Cpp => Agent::Forge,
        Language::Python => Agent::Nova, // Python goes to Nova for now
    }
}
```

---

## Part 3: Button Rendering

### Where Buttons Appear

Buttons are added to **GitHub Check Run** output when the check **fails**.

### Check Run Actions API

```rust
// When reporting check run status, include actions for failed checks
async fn report_check_status(
    github: &GitHubClient,
    repo: &str,
    head_sha: &str,
    check_name: &str,
    conclusion: CheckConclusion,
    pr_context: &PrContext,
) -> Result<()> {
    let mut payload = json!({
        "name": check_name,
        "head_sha": head_sha,
        "status": "completed",
        "conclusion": conclusion.to_string(),
        "output": {
            "title": format!("{} {}", check_name, if conclusion.is_failure() { "failed" } else { "passed" }),
            "summary": "...",
        }
    });
    
    // Add remediation buttons ONLY on failure
    if conclusion.is_failure() {
        let agent = detect_agent_for_pr(pr_context).await?;
        
        payload["actions"] = json!([
            {
                "label": format!("🛠️ Fix with {}", agent.display_name()),
                "description": format!("Launch {} to fix {}", agent.display_name(), check_name),
                "identifier": format!("fix-{}-pr{}-{}", agent.id(), pr_context.number, check_name)
            }
        ]);
    }
    
    github.create_check_run(repo, &payload).await
}
```

### Button Identifier Format

```
fix-{agent}-pr{number}-{check_name}

Examples:
- fix-rex-pr4113-lint-rust
- fix-blaze-pr4116-lint-frontend
- fix-grizz-pr4114-test-go
```

---

## Part 4: Button Click Handler

### Webhook Flow

```
User clicks button
    → GitHub sends check_run event with requested_action
    → PM Server receives at /webhooks/github/action
    → Parse identifier to get agent, PR, check
    → Fetch PR context (files, branch, error output)
    → Create CodeRun CR with remediation prompt
    → Agent runs, fixes, pushes
```

### CodeRun Creation

```rust
async fn handle_remediation_button(
    event: &CheckRunEvent,
    action: &RequestedAction,
) -> Result<()> {
    let (agent, pr_number, check_name) = parse_identifier(&action.identifier)?;
    
    // Get PR details
    let pr = github.get_pull_request(repo, pr_number).await?;
    let check_output = github.get_check_run_output(event.check_run.id).await?;
    
    // Create CodeRun
    let code_run = CodeRun {
        spec: CodeRunSpec {
            run_type: "remediation".to_string(),
            service: "cto".to_string(),
            repository_url: pr.head.repo.clone_url,
            github_app: agent.github_app(),
            cli_config: Some(agent.default_cli_config()),
            prompt_modification: Some(format!(
                "The CI check '{}' failed on PR #{}.

## Error Output
```
{}
```

## Changed Files
{}

Please analyze the failure and fix the issues. 
Commit with message: \"fix: resolve {} failures\"",
                check_name,
                pr_number,
                check_output.text,
                pr.changed_files.join("\n"),
                check_name
            )),
            env: [
                ("PR_NUMBER", pr_number.to_string()),
                ("PR_BRANCH", pr.head.ref_name.clone()),
                ("CHECK_NAME", check_name.clone()),
            ].into(),
            ..Default::default()
        },
        ..Default::default()
    };
    
    kube_client.create(&code_run).await?;
    
    // Post comment that remediation started
    github.create_issue_comment(
        repo,
        pr_number,
        &format!("🤖 {} is working on fixing the {} failure...", agent.display_name(), check_name)
    ).await?;
    
    Ok(())
}
```

---

## Part 5: Files to Create/Modify

### New Files

| File | Purpose |
|------|---------|
| `crates/pm/src/detection/mod.rs` | Language/framework detection module |
| `crates/pm/src/detection/language.rs` | File extension → language mapping |
| `crates/pm/src/detection/framework.rs` | Package.json/path analysis |
| `crates/pm/src/detection/agent.rs` | Detection → Agent selection |
| `crates/controller/src/tasks/code/check_actions.rs` | Button rendering on check runs |

### Modified Files

| File | Changes |
|------|---------|
| `crates/pm/src/handlers/agent_interactions.rs` | Wire up detection in remediation handler |
| `crates/pm/src/handlers/github.rs` | Add check_run event routing |
| `crates/controller/src/tasks/code/status.rs` | Add actions to failed check runs |

---

## Part 6: Implementation Order

### Phase A: Detection (2-3 hours)
1. Create `detection/` module with Language, Framework, Agent enums
2. Implement file extension detection
3. Implement package.json parsing
4. Implement path pattern matching
5. Add TypeScript variant classification
6. Unit tests for each detection method

### Phase B: Button Rendering (2-3 hours)
1. Modify controller's check run reporting
2. Add `actions` field to failed checks
3. Test with intentionally failing CI
4. Verify buttons appear in GitHub UI

### Phase C: Click Handler (2-3 hours)
1. Wire up check_run webhook routing
2. Parse button identifier
3. Fetch PR context and error output
4. Create CodeRun with remediation prompt
5. Post status comment

### Phase D: Integration Testing (1-2 hours)
1. Test with rust-rex PR (Rust → Rex)
2. Test with web-blaze PR (Next.js → Blaze)
3. Test with mobile-tap PR (Expo → Tap)
4. Verify correct agent runs and pushes fix

---

## Test Matrix

| PR | Changed Files | Expected Detection | Expected Button |
|----|---------------|-------------------|-----------------|
| #4113 | `*.rs`, `Cargo.toml` | Rust | Fix with Rex |
| #4114 | `*.go`, `go.mod` | Go | Fix with Grizz |
| #4115 | `server/*.ts`, `elysia` | Node Backend | Fix with Nova |
| #4116 | `components/*.tsx`, `next` | Next.js Web | Fix with Blaze |
| #4117 | `app/(tabs)/*.tsx`, `expo` | Expo Mobile | Fix with Tap |
| #4118 | `src/main/*.ts`, `electron` | Electron | Fix with Spark |
| #4119 | `Assets/Scripts/*.cs` | Unity | Fix with Vex |
| #4120 | `Source/*.cpp`, `*.h` | Unreal | Fix with Forge |

---

## Open Questions

1. **Multiple languages in one PR** - Show multiple buttons or pick primary?
   - *Recommendation*: Show button for primary language (most changed files)

2. **Monorepo detection** - PR touches both frontend and backend?
   - *Recommendation*: Analyze which check failed, match to relevant code

3. **Custom check names** - Different repos have different CI check names
   - *Recommendation*: Store mapping in cto-config.json or detect from check output

4. **Rate limiting** - User spams button clicks?
   - *Recommendation*: Debounce, only allow one active remediation per PR/check
