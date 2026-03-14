# Adding a New CTO Agent: Complete Checklist

This document provides a comprehensive checklist for adding a new agent to the CTO platform. Follow each section carefully to ensure complete integration.

---

## Overview

CTO agents require configuration across multiple systems:

| System | Purpose | Files/Locations |
|--------|---------|-----------------|
| **GitHub App** | Repository access, commit signing | GitHub Developer Settings |
| **Linear OAuth App** | Issue tracking, mentions, webhooks | Linear API Settings |
| **Configuration** | Agent definition, tools, models | `cto-config.json` |
| **Templates** | System prompts, container scripts | `templates/agents/{agent}/` |
| **Secrets** | Credentials storage | OpenBao, ExternalSecrets |
| **Rust Code** | Agent identity, routing | Controller, PM, Config crates |
| **Skills** | Contextual knowledge | `templates/skills/skill-mappings.yaml` |
| **Assets** | Avatar images | `assets/` |
| **Helm Chart** | Kubernetes deployment | `infra/charts/cto/values.yaml` |

---

## Pre-Planning Checklist

Before starting, determine:

- [ ] **Agent name** (lowercase, single word, e.g., "phoenix")
- [ ] **Display name** (e.g., "5DLabs-Phoenix")
- [ ] **Agent category**: Implementation, Support, or Integration
- [ ] **Primary role/specialization** (e.g., "Python ML specialist")
- [ ] **CLI type**: claude, codex, gemini, factory, opencode, cursor
- [ ] **Job types supported**: coder, healer, intake, quality, test, deploy, security, review, integration

---

## 1. GitHub App Setup

### 1.1 Create GitHub App

1. [ ] Navigate to [GitHub Developer Settings](https://github.com/settings/apps/new)
2. [ ] **App Name**: `5DLabs-{AgentName}` (e.g., `5DLabs-Phoenix`)
3. [ ] **Homepage URL**: `https://5dlabs.ai`
4. [ ] **Callback URL**: `https://cto.5dlabs.ai/oauth/callback`
5. [ ] **Webhook URL**: `https://github-webhooks.5dlabs.ai/webhook`
6. [ ] **Webhook Secret**: Generate and save securely

### 1.2 Configure Permissions

Repository permissions:
- [ ] **Contents**: Read & write
- [ ] **Issues**: Read & write
- [ ] **Pull requests**: Read & write
- [ ] **Metadata**: Read-only
- [ ] **Commit statuses**: Read & write

Organization permissions (if needed):
- [ ] **Members**: Read-only

### 1.3 Generate Credentials

1. [ ] Generate **Private Key** (.pem file)
2. [ ] Note the **App ID**
3. [ ] Note the **Client ID**
4. [ ] (Optional) Generate **Client Secret** if OAuth needed

### 1.4 Install App

1. [ ] Install on target organization (5dlabs)
2. [ ] Grant access to required repositories

---

## 2. Linear OAuth App Setup

### 2.1 Create Linear OAuth App

1. [ ] Navigate to [Linear API Settings](https://linear.app/settings/api/applications/new)
2. [ ] **App Name**: `5DLabs-{AgentName}`
3. [ ] **Callback URL**: `https://cto.5dlabs.ai/oauth/callback`
4. [ ] **Webhook URL**: `https://cto.5dlabs.ai/webhooks/linear`

### 2.2 Configure Webhook Events

Enable:
- [ ] Agent session events
- [ ] Permission changes
- [ ] Inbox notifications (optional)

### 2.3 Record Credentials

1. [ ] **Client ID** (32 hex characters)
2. [ ] **Client Secret** (32 hex characters)
3. [ ] **Webhook Secret** (starts with `lin_wh_`)

### 2.4 Install App in Workspace

Use the setup script or manual URL:

```bash
./scripts/setup-linear-agents.sh add {agentname}
./scripts/setup-linear-agents.sh install {agentname}
```

---

## 3. Configuration Files

### 3.1 Update `cto-config.json`

Add agent to the `agents` object:

```json
{
  "agents": {
    "{agentname}": {
      "githubApp": "5DLabs-{AgentName}",
      "cli": "claude",
      "model": "claude-opus-4-5-20251101",
      "tools": {
        "remote": [
          "context7_resolve_library_id",
          "context7_get_library_docs",
          "firecrawl_scrape",
          "firecrawl_crawl",
          "firecrawl_map",
          "firecrawl_search",
          "openmemory_openmemory_query",
          "openmemory_openmemory_store",
          "openmemory_openmemory_list",
          "openmemory_openmemory_get",
          "openmemory_openmemory_reinforce",
          "github_create_pull_request",
          "github_push_files",
          "github_create_branch",
          "github_get_file_contents"
        ],
        "localServers": {}
      }
    }
  }
}
```

### 3.2 Update Play Defaults (if applicable)

If the agent is a new role type, add to `defaults.play`:

```json
{
  "defaults": {
    "play": {
      "{role}Agent": "5DLabs-{AgentName}"
    }
  }
}
```

---

## 4. Templates

### 4.1 Create Agent Template Directory

```bash
mkdir -p templates/agents/{agentname}
```

### 4.2 Create Required Templates

Based on job types the agent supports:

#### Coder Template (for implementation agents)

Create `templates/agents/{agentname}/coder.md.hbs`:

```handlebars
# {AgentName} - Coder Agent

You are **{AgentName}**, an expert **{specialization}** agent focused on writing high-quality, production-ready code.

## Your Role

{{#if subagents.enabled}}
You coordinate and implement features...
{{else}}
You implement features, fix bugs, and write production-ready code...
{{/if}}

## Core Specialization

- **Language**: {primary language}
- **Build**: {build tools}
- **Frameworks**: {frameworks}

## Execution Rules

1. {rule 1}
2. {rule 2}

## Tool Usage Priority

1. **Documentation First** - Query Context7 for library docs before coding
2. **Memory Search** - Check OpenMemory for similar past implementations
3. **Code Search** - Look for existing patterns in the codebase
4. **Implementation** - Write code following discovered patterns
5. **Validation** - Run tests and linters

## Guidelines

- Follow existing code patterns in the repository
- Write comprehensive tests for new functionality
- **Update your Linear issue** with progress and status

{{#if subagents.enabled}}
{{> _shared/partials/coordinator }}
{{/if}}

## Task Context

- Task ID: {{task_id}}
- Service: {{service}}
- Model: {{model}}
- Branch: feature/task-{{task_id}}-coder

Read `task/` directory for full task specification.
```

#### Healer Template (for agents that can remediate)

Create `templates/agents/{agentname}/healer.md.hbs`:

```handlebars
# {AgentName} - Healer Agent

You are **{AgentName}**, responsible for fixing CI failures and platform issues.

## Your Role

Analyze the failure, identify the root cause, and implement a fix.

## Context

- Issue: {{issue_title}}
- Repository: {{repository}}
- Branch: {{branch}}

Read the failure details in `task/` and implement the fix.
```

### 4.3 Template Partials (if needed)

If your agent needs specialized partials, add them to `templates/_shared/partials/`:

```bash
templates/_shared/partials/{agentname}-specific.md.hbs
```

---

## 5. Skills Configuration

### 5.1 Update `templates/skills/skill-mappings.yaml`

Add agent entry:

```yaml
{agentname}:
  description: "{Agent description}"
  default:
    # Context engineering (all agents)
    - context-fundamentals
    - context-degradation
    - context-optimization
    # Tool skills
    - openmemory
    - context7
    - llm-docs
    - github-mcp
    # Agent-specific skills
    - {relevant-skill}
  coder:
    - tool-design
    - firecrawl
  healer:
    - incident-response
    - observability
    - kubernetes-mcp
  optional:
    {optional-skill}:
      triggers: [{keyword1}, {keyword2}]
      description: "When to use this skill"
```

### 5.2 Create New Skills (if needed)

If your agent needs new skills:

```bash
mkdir -p templates/skills/{category}/{skill-name}
```

Create `SKILL.md`:

```markdown
# {Skill Name}

{Skill content that will be loaded into agent context}
```

---

## 6. Secrets Management

### 6.1 Store GitHub App Credentials in OpenBao

```bash
bao kv put github-app-{agentname} \
  app-id="{APP_ID}" \
  client-id="{CLIENT_ID}" \
  private-key="$(cat path/to/{agentname}.pem)"
```

### 6.2 Store Linear App Credentials in OpenBao

```bash
bao kv put linear-app-{agentname} \
  client_id="{CLIENT_ID}" \
  client_secret="{CLIENT_SECRET}" \
  webhook_secret="{WEBHOOK_SECRET}"
```

### 6.3 Update ExternalSecrets Manifest

Edit `infra/gitops/manifests/external-secrets/cto-secrets.yaml`:

```yaml
---
# =============================================================================
# github-app-5dlabs-{agentname} - GitHub App credentials
# =============================================================================
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-{agentname}
  namespace: cto
  labels:
    app.kubernetes.io/managed-by: external-secrets
    app.kubernetes.io/instance: external-secrets-config
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  target:
    name: github-app-5dlabs-{agentname}
    creationPolicy: Owner
  data:
    - secretKey: app-id
      remoteRef:
        key: github-app-{agentname}
        property: app-id
    - secretKey: client-id
      remoteRef:
        key: github-app-{agentname}
        property: client-id
    - secretKey: private-key
      remoteRef:
        key: github-app-{agentname}
        property: private-key

---
# =============================================================================
# linear-app-{agentname} - Linear OAuth App credentials
# =============================================================================
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: linear-app-{agentname}
  namespace: cto
  labels:
    app.kubernetes.io/managed-by: external-secrets
    app.kubernetes.io/component: linear-agent
    app.kubernetes.io/instance: external-secrets-config
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: openbao
    kind: ClusterSecretStore
  target:
    name: linear-app-{agentname}
    creationPolicy: Owner
  data:
    - secretKey: client_id
      remoteRef:
        key: linear-app-{agentname}
        property: client_id
    - secretKey: client_secret
      remoteRef:
        key: linear-app-{agentname}
        property: client_secret
    - secretKey: webhook_secret
      remoteRef:
        key: linear-app-{agentname}
        property: webhook_secret
    - secretKey: access_token
      remoteRef:
        key: linear-app-{agentname}
        property: access_token
```

---

## 7. Helm Chart Updates

### 7.1 Update `infra/charts/cto/values.yaml`

Add to `secrets.githubApps`:

```yaml
secrets:
  githubApps:
    # ... existing agents ...
    - name: {agentname}
      baoKey: github-app-{agentname}
```

---

## 8. Rust Code Updates

### 8.1 Update Agent Names Constant

Edit `crates/pm/src/config.rs`:

```rust
/// All known agent names for the CTO platform.
pub const AGENT_NAMES: &[&str] = &[
    "morgan", "rex", "blaze", "grizz", "nova", "tap", "spark", 
    "cleo", "cipher", "tess", "atlas", "bolt",
    "{agentname}",  // <-- Add new agent
];
```

### 8.2 Update Agent Constants

Edit `crates/config/src/types.rs`:

```rust
/// Hardcoded agent suffixes
pub const AGENT_{AGENTNAME_UPPER}: &str = "{AgentName}";
```

### 8.3 Update Agent Name Extraction (if needed)

Edit `crates/controller/src/tasks/code/templates.rs`:

```rust
fn extract_agent_name_from_github_app(github_app: &str) -> Result<String> {
    let agent_name = match github_app {
        // ... existing mappings ...
        "5DLabs-{AgentName}" => "{agentname}",
        _ => { /* error */ }
    };
    Ok(agent_name.to_string())
}
```

### 8.4 Update Agent Classifier (if implementation agent)

Edit `crates/controller/src/tasks/code/agent.rs`:

```rust
impl AgentClassifier {
    pub fn new() -> Self {
        let mut implementation_agents = HashSet::new();
        implementation_agents.insert("rex".to_string());
        implementation_agents.insert("blaze".to_string());
        // Add if this is an implementation agent that shares workspace
        implementation_agents.insert("{agentname}".to_string());
        
        Self { implementation_agents }
    }
}
```

### 8.5 Update Play Defaults (if new role type)

Edit `crates/config/src/types.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlayDefaults {
    // ... existing fields ...
    
    /// Override {role} agent (defaults to {orgName}-{AgentName}).
    #[serde(rename = "{role}Agent", skip_serializing_if = "Option::is_none")]
    pub {role}_agent: Option<String>,
}

impl PlayDefaults {
    /// Get the {role} agent name.
    #[must_use]
    pub fn get_{role}_agent(&self, org_name: &str) -> String {
        self.{role}_agent
            .clone()
            .unwrap_or_else(|| make_agent_name(org_name, AGENT_{AGENTNAME_UPPER}))
    }
}
```

---

## 9. Assets

### 9.1 Create Avatar Images

Required sizes:
- [ ] `assets/{agentname}-avatar.png` (256x256)
- [ ] `assets/{agentname}-avatar-512.png` (512x512)

Design guidelines:
- Consistent style with existing agent avatars
- Distinctive color scheme for the agent's role
- Clear, simple iconography

### 9.2 Upload to Linear (Optional)

Upload avatar when configuring the Linear OAuth app.

---

## 10. Documentation

### 10.1 Update `AGENTS.md`

Add to the appropriate section:

```markdown
**Implementation Agents:**
- **{AgentName}** ({Language}) - {frameworks, tools}

OR

**Support Agents:**
- **{AgentName}** - {Role description}
```

### 10.2 Update Quick Reference

```markdown
### Agents Overview

**Implementation Agents:**
- **{AgentName}** ({Language}) - {description}
```

---

## 11. Linear Setup Script

### 11.1 Update `scripts/setup-linear-agents.sh`

Add to `AGENTS` array:

```bash
AGENTS=(morgan rex blaze grizz nova tap spark cleo cipher tess atlas bolt {agentname})
```

---

## 12. Verification & Testing

### 12.1 Build Verification

```bash
# Format check
cargo fmt --all --check

# Clippy pedantic
cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# Tests
cargo test --workspace

# Build
cargo build --release
```

### 12.2 Template Verification

```bash
cargo run --bin test_templates
```

### 12.3 Secrets Verification

```bash
# Verify OpenBao secrets exist
bao kv get github-app-{agentname}
bao kv get linear-app-{agentname}

# Verify ExternalSecrets sync
kubectl get externalsecrets -n cto | grep {agentname}
kubectl get secrets -n cto | grep {agentname}
```

### 12.4 Integration Testing

1. [ ] Create a test CodeRun with the new agent
2. [ ] Verify GitHub commits are signed correctly
3. [ ] Verify Linear mentions work
4. [ ] Verify webhook routing works
5. [ ] Test all supported job types (coder, healer, etc.)

---

## 13. Deployment

### 13.1 Commit and Push

```bash
git add .
git commit -m "feat: add {AgentName} agent

- Add GitHub App configuration
- Add Linear OAuth app configuration
- Add agent templates
- Update skills mapping
- Update ExternalSecrets
- Update Helm values
- Update Rust code constants"

git push origin feat/add-{agentname}-agent
```

### 13.2 ArgoCD Sync

```bash
# Sync ExternalSecrets
argocd app sync external-secrets-config

# Sync CTO platform
argocd app sync cto
```

### 13.3 Post-Deployment Verification

1. [ ] Verify secrets are created in Kubernetes
2. [ ] Test a CodeRun with the new agent
3. [ ] Verify Linear status updates work

---

## Quick Reference: File Locations

| Component | Location |
|-----------|----------|
| Agent config | `cto-config.json` → `agents.{agentname}` |
| Templates | `templates/agents/{agentname}/` |
| Skills | `templates/skills/skill-mappings.yaml` |
| External secrets | `infra/gitops/manifests/external-secrets/cto-secrets.yaml` |
| Helm values | `infra/charts/cto/values.yaml` |
| Agent names (PM) | `crates/pm/src/config.rs` → `AGENT_NAMES` |
| Agent constants | `crates/config/src/types.rs` |
| Agent classifier | `crates/controller/src/tasks/code/agent.rs` |
| Template mapping | `crates/controller/src/tasks/code/templates.rs` |
| Linear setup | `scripts/setup-linear-agents.sh` |
| Avatar assets | `assets/{agentname}-avatar*.png` |
| Documentation | `AGENTS.md` |

---

## Troubleshooting

### GitHub App Not Working

1. Verify private key is correctly stored in OpenBao
2. Check ExternalSecret sync status
3. Verify app is installed on the repository
4. Check webhook delivery logs in GitHub

### Linear Mentions Not Working

1. Verify Linear app is installed in workspace
2. Check access token is stored (after OAuth flow)
3. Verify webhook secret matches
4. Check PM server logs for webhook errors

### Agent Not Routed Correctly

1. Verify `AGENT_NAMES` constant includes the agent
2. Check `extract_agent_name_from_github_app` mapping
3. Verify `cto-config.json` has correct `githubApp` value
4. Check controller logs for routing errors

### Templates Not Found

1. Verify template files exist in `templates/agents/{agentname}/`
2. Check template names match job type (e.g., `coder.md.hbs`)
3. Verify Handlebars syntax is valid
4. Run `cargo run --bin test_templates`

---

## Lessons Learned & Best Practices

### Order of Operations

Based on real implementation experience, follow this sequence to minimize rework:

1. **Plan completely first** - Agent name, role, tools, skills all decided
2. **Create GitHub App** - Takes time for approval/setup
3. **Create Linear OAuth App** - Can be done in parallel with GitHub
4. **Store secrets in OpenBao** - Before any code changes
5. **Update Rust code constants** - All at once to pass compilation
6. **Add ExternalSecrets manifests** - Commit with Rust changes
7. **Create templates** - Agent prompts and skills
8. **Update cto-config.json** - Final configuration
9. **Test locally with test_templates**
10. **Deploy and verify**

### Common Pitfalls

| Pitfall | Solution |
|---------|----------|
| Forgot to add agent to `AGENT_NAMES` in PM crate | Linear webhooks silently fail - always update |
| Template file named wrong | Must be `{job}.md.hbs` exactly (e.g., `coder.md.hbs`) |
| ExternalSecret key mismatch | Double-check OpenBao path matches ExternalSecret |
| Agent routing falls back to Rex | Add explicit match in `get_agent_system_prompt_template()` |
| Container environment missing | Update `container.sh.hbs` if agent needs new env setup |

### When to Update Container Template

If your agent needs a **new development environment** (not Node.js, Go, or Rust), you must update `templates/_shared/container.sh.hbs`:

```handlebars
{{!-- Example: Add Unity environment for a VR agent --}}
{{#if (or (eq agent_name "vertex") (eq task_language "csharp"))}}
{{> unity-env }}
{{/if}}
```

And create the corresponding partial in `templates/_shared/partials/`.

### Agent Category Guidelines

| Category | Workspace Sharing | Examples |
|----------|-------------------|----------|
| **Implementation** | Shared PVC | Rex, Blaze, Grizz, Nova, Tap, Spark, Angie |
| **Support** | Isolated PVC | Cleo, Cipher, Tess, Morgan |
| **Integration** | Isolated PVC | Atlas, Bolt, Stitch |

Implementation agents share a workspace PVC so they can see each other's code changes within a Play workflow.

### Choosing MCP Tools

Select tools based on agent role:

| Role | Essential Tools |
|------|-----------------|
| Backend coder | `github_*`, `context7_*`, `openmemory_*` |
| Frontend coder | + `shadcn_*`, `ai_elements_*` |
| Mobile coder | + `xcodebuild_*`, `appstore_*` (for Tap) |
| Infrastructure | + `kubernetes_*`, `argocd_*`, `terraform_*` |
| Quality/Security | + `github_get_pull_request*`, `github_*_scanning_*` |

### Skills Inheritance

All agents should include these base skills:
- `context-fundamentals`
- `context-degradation`  
- `context-optimization`
- `openmemory`
- `context7`
- `llm-docs`
- `github-mcp`

Add domain-specific skills on top of this base.

---

## Example: Adding a VR Agent (Vertex)

This section walks through adding a hypothetical **Vertex** agent for Unity VR development.

### Pre-Planning

| Item | Value |
|------|-------|
| **Name** | vertex |
| **Display Name** | 5DLabs-Vertex |
| **Category** | Implementation |
| **Specialization** | Unity VR development with OpenXR |
| **CLI** | claude |
| **Job Types** | coder, healer |
| **Language** | C# |
| **Frameworks** | Unity, XR Interaction Toolkit, Meta XR SDK, OpenXR |

### Key Decisions

1. **Workspace Sharing**: Yes - Vertex is an implementation agent, so it should share workspace with other implementation agents (Rex, Blaze) in the same Play workflow.

2. **Container Environment**: Needs a new `unity-env.sh.hbs` partial since Unity/C# is not covered by existing Node.js, Go, or Rust environments.

3. **New Skills Required**:
   - `templates/skills/platforms/unity-vr/SKILL.md` - Unity VR patterns
   - `templates/skills/platforms/openxr/SKILL.md` - Cross-platform XR

4. **MCP Tools**: Standard implementation tools plus potentially:
   - No specialized VR MCP tools exist yet (could be future enhancement)

### Template Structure

```
templates/agents/vertex/
├── coder.md.hbs      # Unity VR implementation prompts
└── healer.md.hbs     # CI failure remediation
```

### Coder Template Content Highlights

```handlebars
# Vertex - Unity VR Engineer

You are **Vertex** 🥽, the VR virtuoso. Cross-platform XR experiences using Unity
and OpenXR. Quest, SteamVR, Pico, Vive — one codebase, all headsets.

## Core Specialization

- **Engine**: Unity 2022.3 LTS+ (Universal Render Pipeline)
- **XR**: OpenXR runtime, XR Interaction Toolkit 3.0+
- **Language**: C# (.NET Standard 2.1)
- **Platforms**: Meta Quest, SteamVR, Pico, Windows Mixed Reality
- **Build**: Unity Build Automation, or local builds

## VR SDK Strategy

Use **Unity XR Interaction Toolkit** as the primary interaction framework:
- Cross-platform OpenXR support
- Action-based input (not device-specific)
- Built-in locomotion, grabbing, UI interaction
- Compatible with hand tracking (XR Hands package)

For Meta-specific features (passthrough, hand tracking API):
- Use **Meta XR SDK** as additional layer
- Only when Quest-specific features are required

## Key Packages

| Package | Purpose |
|---------|---------|
| `com.unity.xr.openxr` | OpenXR runtime |
| `com.unity.xr.interaction.toolkit` | Interactions framework |
| `com.unity.xr.hands` | Hand tracking |
| `com.meta.xr.sdk.all` | Meta Quest features (optional) |

## Context7 Library IDs

Query these before implementing:
- **Unity**: `/Unity-Technologies/Unity-Documentation` 
- **XR Interaction Toolkit**: Query via `unity xr interaction toolkit`
- **OpenXR**: Query via `openxr specification`

## Validation Commands

\`\`\`bash
# Build for Android (Quest)
unity -batchmode -quit -projectPath . -executeMethod BuildScript.BuildAndroid

# Run tests
unity -batchmode -quit -projectPath . -runTests -testResults results.xml
\`\`\`
```

### Skills Mapping Entry

```yaml
vertex:
  description: "Unity VR specialist with OpenXR cross-platform development"
  default:
    - context-fundamentals
    - context-degradation
    - context-optimization
    - openmemory
    - context7
    - llm-docs
    - github-mcp
    - unity-vr          # New skill
    - openxr            # New skill
  coder:
    - tool-design
    - firecrawl
  healer:
    - incident-response
    - observability
    - kubernetes-mcp
  optional:
    meta-xr:
      triggers: [quest, meta, passthrough, hand tracking]
      description: "Meta Quest specific features"
```

### Container Environment Update

Add to `templates/_shared/container.sh.hbs`:

```handlebars
{{!-- Unity/C# environment for VR agents (Vertex) --}}
{{#if (or (eq agent_name "vertex") (eq task_language "csharp") (eq task_language "unity"))}}
{{> unity-env }}
{{/if}}
```

Create `templates/_shared/partials/unity-env.sh.hbs`:

```bash
# =========================================================================
# Unity/C# Environment Bootstrap
# =========================================================================

# Unity Hub CLI location (if installed)
export UNITY_HUB="${UNITY_HUB:-/Applications/Unity Hub.app/Contents/MacOS/Unity Hub}"

# Add Unity Editor to PATH (common locations)
if [ -d "/Applications/Unity/Hub/Editor" ]; then
  UNITY_VERSION=$(ls -1 /Applications/Unity/Hub/Editor | sort -V | tail -1)
  export PATH="/Applications/Unity/Hub/Editor/$UNITY_VERSION/Unity.app/Contents/MacOS:$PATH"
fi

# .NET SDK
if command -v dotnet >/dev/null 2>&1; then
  echo ".NET version: $(dotnet --version)"
fi

# Verify Unity is available
if command -v Unity >/dev/null 2>&1; then
  echo "Unity available in PATH"
else
  echo "⚠️ Unity not found in PATH - builds may need explicit path"
fi
```

### Rust Code Updates

**`crates/pm/src/config.rs`**:
```rust
pub const AGENT_NAMES: &[&str] = &[
    "morgan", "rex", "blaze", "grizz", "nova", "tap", "spark", 
    "cleo", "cipher", "tess", "atlas", "bolt", "stitch",
    "vertex",  // VR agent
];
```

**`crates/config/src/types.rs`**:
```rust
pub const AGENT_VERTEX: &str = "Vertex";

// In PlayDefaults:
#[serde(rename = "vrAgent", skip_serializing_if = "Option::is_none")]
pub vr_agent: Option<String>,

pub fn get_vr_agent(&self, org_name: &str) -> String {
    self.vr_agent
        .clone()
        .unwrap_or_else(|| make_agent_name(org_name, AGENT_VERTEX))
}
```

**`crates/controller/src/tasks/code/templates.rs`**:
```rust
let agent = match github_app {
    // ... existing ...
    "5DLabs-Vertex" => "vertex",
    _ => "rex",
};
```

**`crates/controller/src/tasks/code/agent.rs`**:
```rust
// Vertex is an implementation agent (shares workspace)
implementation_agents.insert("vertex".to_string());
```

### cto-config.json Entry

```json
"vertex": {
  "githubApp": "5DLabs-Vertex",
  "cli": "claude",
  "model": "claude-opus-4-5-20251101",
  "tools": {
    "remote": [
      "context7_resolve_library_id",
      "context7_get_library_docs",
      "firecrawl_scrape",
      "firecrawl_crawl",
      "firecrawl_map",
      "firecrawl_search",
      "openmemory_openmemory_query",
      "openmemory_openmemory_store",
      "openmemory_openmemory_list",
      "openmemory_openmemory_get",
      "openmemory_openmemory_reinforce",
      "github_create_pull_request",
      "github_push_files",
      "github_create_branch",
      "github_get_file_contents"
    ],
    "localServers": {}
  }
}
```

This example demonstrates all the touchpoints required for a new implementation agent with a custom development environment.

---

## See Also

- [Play Workflow](play-workflow.md) - Multi-agent orchestration
- [MCP Tools Reference](mcp-tools.md) - Available tools for agents
- [Development Guide](development-guide.md) - Build commands and setup
