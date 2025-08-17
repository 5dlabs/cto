# Task 1 Discovery Report: CodeRun Controller & Multi-Agent Architecture

**Date:** August 17, 2025  
**Status:** Complete  
**Scope:** Comprehensive analysis of existing CodeRun controller, template system, Argo Events integration, and multi-agent requirements

---

## Executive Summary

The existing CodeRun controller provides a solid foundation for multi-agent orchestration with minimal modifications required. Key findings:

- **✅ CRD Structure**: Fully supports multi-agent use cases with `github_app` field and agent-specific configurations
- **✅ Template System**: Handlebars-based with conditional logic support - ready for agent-specific customization
- **✅ Argo Events**: Infrastructure exists and working (currently rate-limited but functional)
- **✅ Workspace Isolation**: PVC system ready for agent-specific naming (`workspace-{service}-{agent}`)
- **⚠️ Gaps**: Need agent-specific templates, enhanced RBAC, and multi-method task association

---

## 1. CodeRun CRD Architecture Analysis

### 1.1 CRD Specification (`controller/src/crds/coderun.rs`)

**Key Fields for Multi-Agent Support:**

```rust
pub struct CodeRunSpec {
    pub task_id: u32,                    // ✅ Task correlation
    pub service: String,                 // ✅ PVC workspace naming
    pub github_app: Option<String>,      // ✅ Agent identification ("5DLabs-Rex", "5DLabs-Cleo")
    pub model: String,                   // ✅ Agent-specific Claude models
    pub continue_session: bool,          // ✅ Agent session continuity
    pub service_account_name: Option<String>, // ✅ Agent-specific RBAC
    pub env: HashMap<String, String>,    // ✅ Agent-specific environment
    pub env_from_secrets: Vec<SecretEnvVar>, // ✅ Agent-specific secrets
    // ... other fields
}
```

**Status Tracking:**
```rust
pub struct CodeRunStatus {
    pub phase: String,                   // "Running", "Succeeded", "Failed"
    pub work_completed: Option<bool>,    // TTL safety mechanism
    pub job_name: Option<String>,        // K8s Job reference
    pub pull_request_url: Option<String>, // GitHub PR tracking
    pub session_id: Option<String>,     // Session continuity
    // ... other status fields
}
```

**✅ Assessment**: CRD is fully ready for multi-agent use. No schema changes required.

### 1.2 Controller Reconciliation Flow (`controller/src/tasks/code/controller.rs`)

**Primary Reconcile Loop:**
```rust
// 1. Status-first idempotency check
if status.work_completed == Some(true) {
    return Ok(Action::await_change()); // Stop reconciliation
}

// 2. Job state management
match job_state {
    JobState::NotFound => create_resources_optimistically(),
    JobState::Running => monitor_progress(),
    JobState::Completed => mark_work_completed(),
    JobState::Failed => mark_failed(),
}
```

**Resource Creation Pattern:**
1. **PVC Creation**: `workspace-{service}` (needs agent-specific modification)
2. **ConfigMap Generation**: Handlebars template processing
3. **Job Creation**: Idempotent with 409 conflict handling
4. **Status Updates**: Work completion tracking

**✅ Assessment**: Controller follows solid patterns. PVC naming is the main modification needed.

---

## 2. Template System Deep Dive

### 2.1 Template Architecture (`controller/src/tasks/code/templates.rs`)

**Template Generation Flow:**
```rust
pub fn generate_all_templates(code_run: &CodeRun, config: &ControllerConfig) -> Result<BTreeMap<String, String>> {
    let mut templates = BTreeMap::new();
    
    // Core templates
    templates.insert("container.sh", generate_container_script(code_run)?);
    templates.insert("CLAUDE.md", generate_claude_memory(code_run)?);
    templates.insert("settings.json", generate_claude_settings(code_run, config)?);
    templates.insert("mcp.json", generate_mcp_config(code_run, config)?);
    
    // Code-specific templates
    templates.insert("coding-guidelines.md", generate_coding_guidelines(code_run)?);
    templates.insert("github-guidelines.md", generate_github_guidelines(code_run)?);
    
    // Hook scripts with "hooks-" prefix for ConfigMap compliance
    let hook_scripts = generate_hook_scripts(code_run)?;
    for (filename, content) in hook_scripts {
        templates.insert(format!("hooks-{filename}"), content);
    }
}
```

### 2.2 Current Template Structure

```
infra/charts/controller/claude-templates/
├── agents/                              # Generic agent prompts
│   └── system-prompt.md.hbs            # Single generic prompt
├── code/                                # Code task templates
│   ├── claude.md.hbs                   # Agent memory initialization
│   ├── client-config.json.hbs          # MCP tool configuration
│   ├── container.sh.hbs                # Startup script
│   ├── settings.json.hbs               # Claude settings
│   ├── mcp.json.hbs                    # MCP server config
│   ├── coding-guidelines.md.hbs        # Coding standards
│   ├── github-guidelines.md.hbs        # Git workflow
│   └── hooks/                          # Git hooks
│       ├── early-test.sh.hbs
│       ├── stop-code-pr-creation.sh.hbs
│       ├── stop-commit.sh.hbs
│       └── stop-pr-creation.sh.hbs
└── docs/                               # Documentation task templates
    └── ...
```

### 2.3 Handlebars Context Structure

**Current Context Variables:**
```rust
let context = json!({
    "task_id": code_run.spec.task_id,
    "service": code_run.spec.service,
    "repository_url": code_run.spec.repository_url,
    "docs_repository_url": code_run.spec.docs_repository_url,
    "github_app": code_run.spec.github_app.as_deref().unwrap_or(""),
    "model": code_run.spec.model,
    "continue_session": get_continue_session(code_run),
    "working_directory": get_working_directory(code_run),
    // ... other variables
});
```

**✅ Assessment**: Template system is ready for agent-specific conditionals using `{{#if (eq github_app "5DLabs-Cleo")}}` syntax.

### 2.4 Key Template Analysis

#### Container Script (`container.sh.hbs`)
- **Current**: Generic implementation workflow
- **Length**: 1,120+ lines of comprehensive GitHub App authentication, repository handling, task processing
- **Agent Support**: Uses `{{github_app}}` variable throughout
- **Modification Needed**: Add agent-specific setup sections

#### Claude Memory (`claude.md.hbs`)
- **Current**: Generic memory initialization
- **Agent Support**: Ready for `{{#if github_app}}` conditionals
- **Modification Needed**: Agent-specific role descriptions and instructions

#### MCP Configuration (`client-config.json.hbs`)
- **Current**: Standard MCP tools for implementation work
- **Agent Support**: Ready for conditional tool lists
- **Modification Needed**: Agent-specific `remoteTools` arrays

---

## 3. Resource Management Analysis

### 3.1 PVC Management (`controller/src/tasks/code/resources.rs`)

**Current PVC Naming:**
```rust
let pvc_name = format!("workspace-{service_name}");
```

**PVC Creation Logic:**
```rust
async fn ensure_pvc_exists(&self, pvc_name: &str, service_name: &str) -> Result<()> {
    // Check if PVC exists
    match self.pvcs.get(pvc_name).await {
        Ok(_) => {
            info!("✅ PVC already exists: {}", pvc_name);
            Ok(())
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Create PVC with proper labels and storage class
            let pvc = create_workspace_pvc(pvc_name, service_name, self.config);
            self.pvcs.create(&PostParams::default(), &pvc).await?;
            info!("✅ Created PVC: {}", pvc_name);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
```

**Required Modification:**
```rust
// Extract agent name from github_app field
fn extract_agent_name(github_app: &str) -> String {
    github_app.split('-').last().unwrap_or("unknown").to_lowercase()
}

// New PVC naming: workspace-{service}-{agent}
let agent_name = extract_agent_name(&code_run.spec.github_app.unwrap_or_default());
let pvc_name = format!("workspace-{}-{}", service_name, agent_name);
```

### 3.2 Job Creation and Management

**Job Naming Pattern:**
```rust
fn generate_code_job_name(code_run: &CodeRun) -> String {
    format!("coderun-{}-{}-{}", 
        code_run.spec.task_id, 
        code_run.spec.service,
        code_run.name_any()
    )
}
```

**Job Labels:**
```rust
"workflow-name": workflow_name,
"task-id": task_id,
"service": service,
"github-app": github_app,  // ✅ Already supports agent correlation
```

---

## 4. Argo Events Infrastructure Analysis

### 4.1 Current Setup (`infra/gitops/resources/github-webhooks/`)

**EventSource Configuration:**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: EventSource
metadata:
  name: github
  namespace: argo
spec:
  github:
    org:
      events: ["*"]              # Captures all GitHub events
      organizations: ["5dlabs"]
      webhook:
        endpoint: /github/webhook
        port: "12000"
      secret:
        name: github-webhook-secret
        key: secret
```

**Existing Sensor (Test/Demo):**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: github-demo-sensor  # Currently disabled but working
spec:
  dependencies:
    - name: github-any
      eventSourceName: github
      eventName: org
  triggers:
    - template:
        name: log-event
        k8s:
          operation: create
          source:
            resource:
              apiVersion: v1
              kind: Pod
              # Creates busybox pod that logs webhook payload
```

**✅ Assessment**: Infrastructure is functional (rate-limited but working). Ready for workflow correlation sensors.

### 4.2 ngrok Integration

**HTTP Route Configuration:**
```yaml
# infra/gitops/resources/github-webhooks/httproute.yaml
# Routes GitHub webhooks through ngrok Gateway to EventSource
```

**Status**: Working but currently rate-limited. User will resolve billing issue.

---

## 5. Multi-Agent Configuration Analysis

### 5.1 Agent Definitions (`infra/charts/controller/values.yaml`)

**Current Agent Structure:**
```yaml
agents:
  rex:
    name: "Rex"
    githubApp: "5DLabs-Rex"
    appId: "1724452"
    clientId: "Iv23liTnu9e0imdRPhCC"
    role: "Senior Backend Architect & Systems Engineer"
    systemPrompt: |
      # Generic implementation prompt
      
  cleo:
    name: "Cleo"
    githubApp: "5DLabs-Cleo"
    appId: "1794540"
    clientId: "Iv23lieWLAmH0ocG3CUO"
    role: "Formatting & Code Quality Specialist"
    systemPrompt: |
      # Specific Clippy/formatting focus
      
  tess:
    name: "Tess"
    githubApp: "5DLabs-Tess"
    appId: "1794556"
    clientId: "Iv23livHVGK1dDETIcxa"
    role: "Quality Assurance & Testing Specialist"
    systemPrompt: |
      # Testing and QA focus with K8s deployment validation
```

**✅ Assessment**: Agent definitions are comprehensive with role-specific system prompts. Ready for template integration.

### 5.2 External Secrets Configuration

**Pattern Analysis:**
```yaml
# Example: Rex GitHub App secrets
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-app-5dlabs-rex
spec:
  data:
  - secretKey: GITHUB_APP_ID
    remoteRef:
      key: github-app-rex
      property: app_id
  - secretKey: GITHUB_APP_PRIVATE_KEY  
    remoteRef:
      key: github-app-rex
      property: private_key
```

**✅ Assessment**: Pattern established for Cleo and Tess secrets. Ready for controller integration.

---

## 6. Workflow Integration Analysis

### 6.1 Existing CodeRun Template (`infra/charts/controller/templates/coderun-template.yaml`)

**Current Structure:**
```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: coderun-template
spec:
  arguments:
    parameters:
      - name: task-id
      - name: service-id  
      - name: github-app        # ✅ Agent parameterization ready
      - name: model
      # ... other parameters
  entrypoint: coderun-main
  templates:
    - name: coderun-main
      steps:
      - - name: create-coderun-resource
          template: create-coderun-resource
      - - name: wait-for-completion
          template: wait-coderun-completion
```

**✅ Assessment**: Template ready for multi-agent workflows. Parameterized agent selection already implemented.

---

## 7. Critical Integration Points

### 7.1 GitHub App Authentication Flow

**Container Script Authentication (`container.sh.hbs` lines 19-120):**
1. **JWT Generation**: Creates GitHub App JWT using RSA256 signing
2. **Installation Token**: Exchanges JWT for installation access token  
3. **Repository Access**: Configures Git credentials for repository operations
4. **Error Handling**: Comprehensive retry logic and fallback mechanisms

**✅ Assessment**: Authentication is production-ready and supports all agents.

### 7.2 Session Continuity Implementation

**Current Logic:**
```bash
# In container.sh.hbs
if [ "{{continue_session}}" = "true" ]; then
    echo "🔄 Continuing previous session..."
    # Preserves existing CLAUDE.md and workspace state
else
    echo "🆕 Starting fresh session..."
    # Initializes clean workspace
fi
```

**Agent Isolation**: Each agent gets their own PVC, so `continue_session` is agent-specific.

---

## 8. Required Modifications Summary

### 8.1 HIGH PRIORITY - Core Multi-Agent Support

#### PVC Naming Update (`resources.rs`)
```rust
// Current
let pvc_name = format!("workspace-{service_name}");

// Required  
fn extract_agent_name(github_app: &str) -> String {
    github_app.split('-').last().unwrap_or("unknown").to_lowercase()
}
let agent_name = extract_agent_name(&code_run.spec.github_app.unwrap_or_default());
let pvc_name = format!("workspace-{}-{}", service_name, agent_name);
```

#### Agent-Specific Templates (`claude-templates/`)
```
# Required new structure
claude-templates/
├── agents/
│   ├── cleo-system-prompt.md.hbs    # NEW: Cleo-specific prompt
│   ├── tess-system-prompt.md.hbs    # NEW: Tess-specific prompt  
│   └── rex-system-prompt.md.hbs     # NEW: Rex-specific prompt
└── code/
    ├── claude.md.hbs                # MODIFY: Add agent conditionals
    ├── client-config.json.hbs       # MODIFY: Agent-specific MCP tools
    └── container.sh.hbs             # MODIFY: Agent-specific setup
```

#### Template Conditional Logic
```handlebars
{{#if (eq github_app "5DLabs-Cleo")}}
  "remoteTools": ["rustdocs_query_rust_docs", "brave-search_brave_web_search"]
{{else if (eq github_app "5DLabs-Tess")}}
  "remoteTools": ["kubernetes_listResources", "memory_create_entities"]
{{else}}
  <!-- Default Rex/Blaze tools -->
{{/if}}
```

### 8.2 MEDIUM PRIORITY - Workflow Orchestration

#### Multi-Agent Workflow Template
- Create `play-template.yaml` with parameterized agent selection
- Implement suspend/resume points for event-driven transitions  
- Add workflow correlation labels and stage management

#### Argo Events Sensors
- GitHub PR creation → Resume after Rex
- PR labeled "ready-for-qa" → Resume after Cleo
- PR approved → Resume after Tess
- Rex push events → Cancel Cleo/Tess, restart QA pipeline

### 8.3 LOW PRIORITY - Operational Enhancements  

#### Enhanced RBAC for Tess
- Cluster-admin permissions for K8s testing
- Database admin credentials (Postgres, Redis)  
- Argo CD admin access

#### Task Association Validation
- Multi-method validation (PR labels + branch naming + marker file)
- Workflow failure on correlation mismatch

---

## 9. Implementation Readiness Assessment

### ✅ READY - No Changes Needed
- **CRD Structure**: Fully supports multi-agent scenarios
- **Controller Logic**: Reconciliation patterns are solid
- **GitHub App Integration**: Authentication works for all agents
- **Argo Events Infrastructure**: Functional (pending rate limit resolution)
- **External Secrets**: Pattern established for all agents

### ⚡ QUICK WINS - Minimal Code Changes
- **PVC Agent Naming**: Single function modification in `resources.rs`
- **Agent Template Creation**: Copy/customize existing templates
- **Template Conditionals**: Add Handlebars `{{#if}}` blocks to existing files

### 🔧 MODERATE EFFORT - New Components
- **Multi-Agent Workflow**: Create new `play-template.yaml`
- **Event Correlation**: Create new Sensors for webhook processing
- **Task Association**: Implement validation logic in Sensors

### 🏗️ LARGER EFFORT - System Integration
- **End-to-End Testing**: Comprehensive multi-agent workflow validation
- **Operational Monitoring**: Long-running workflow health tracking
- **Production Deployment**: GitOps pipeline for multi-agent system

---

## 10. Discovery Validation

### Architecture Requirements Met ✅
- [x] **Event-Driven Orchestration**: Argo Workflows + Events infrastructure ready
- [x] **Agent Workspace Isolation**: PVC system supports agent-specific naming  
- [x] **Session Continuity**: Each agent maintains independent CLAUDE.md context
- [x] **GitHub Integration**: Full App authentication and webhook processing
- [x] **Template Customization**: Handlebars system ready for agent-specific behavior
- [x] **Quality Gates**: PR creation, labeling, approval events all capturable
- [x] **Rex Remediation**: Event-driven cancellation and restart patterns feasible

### Technical Feasibility Confirmed ✅
- [x] **Suspend/Resume**: Argo Workflows supports indefinite suspension and webhook-triggered resume
- [x] **Event Correlation**: GitHub webhook payloads contain PR labels for task association
- [x] **Resource Management**: Controller handles PVC, Job, ConfigMap lifecycle effectively
- [x] **Template System**: Handlebars conditionals support agent-specific configuration
- [x] **Multi-Method Task Association**: PR labels + branch parsing + marker files all implementable

---

## 11. Next Steps Recommendation

**Immediate Actions:**
1. ✅ **Discovery Complete** - All systems analyzed and documented
2. 🔄 **Start Implementation** - Begin with Task 4 (PVC naming modification)
3. 🔄 **Template Enhancement** - Create agent-specific templates (Task 6)
4. 🔄 **Workflow Design** - Implement multi-agent DAG structure (Task 3)

**Key Success Factors:**
- **Incremental Implementation**: Build on existing patterns rather than rebuilding
- **Backward Compatibility**: Ensure Rex/Blaze workflows continue working  
- **Agent Isolation**: Maintain independent workspaces and session contexts
- **Event Reliability**: Implement robust correlation and error handling

The existing infrastructure provides an excellent foundation for multi-agent orchestration. The required modifications are well-scoped and technically feasible within the current architecture.

---

**Report Status:** ✅ Complete  
**Confidence Level:** High - All critical systems analyzed  
**Ready for Implementation:** Yes - Clear modification path identified
