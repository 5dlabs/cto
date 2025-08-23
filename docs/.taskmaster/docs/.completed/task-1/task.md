# Task 1: Analyze Existing CodeRun Controller Architecture



## Overview

Perform a comprehensive discovery and documentation of the existing CodeRun controller implementation, focusing on understanding the CRD structure, template system, and agent pod creation flow. This foundational analysis is critical for enabling multi-agent orchestration capabilities.

## Technical Context

The CodeRun controller is the core infrastructure component that processes CRD specifications into running agent pods. Understanding its architecture, reconciliation patterns, and template system is essential before implementing multi-agent support.

## Implementation Guide

### Phase 1: CRD Structure Analysis



1. **Examine CRD Specification**


   - Review `controller/src/crds/coderun.rs` for field definitions


   - Document `github_app` field capabilities for agent differentiation


   - Analyze `continue_session` behavior for session persistence


   - Map secret and environment variable injection patterns



2. **Validate Multi-Agent Support**
   ```yaml
   # Example CRD showing multi-agent capabilities
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: coderun-task-analysis
   spec:
     github_app: "5DLabs-Rex"  # Agent-specific authentication
     service: "cto"
     continue_session: true     # Session continuity
     model: "claude-3-5-sonnet-20241022"





```

### Phase 2: Controller Reconciliation Logic



1. **Map Reconciliation Flow**


   - Trace request processing from CRD submission to pod creation


   - Document status-first idempotency patterns


   - Understand TTL safety mechanisms and cleanup logic


   - Analyze finalizer implementation for resource cleanup



2. **Key Files to Analyze**
   ```rust
   // controller/src/tasks/code/controller.rs
   // Main reconciliation logic and state machine

   // controller/src/tasks/code/resources.rs
   // Pod, ConfigMap, and PVC creation logic

   // controller/src/tasks/code/templates.rs
   // Handlebars template rendering pipeline





```



### Phase 3: Template System Architecture



1. **Document Template Loading**


   - Map template file organization in `infra/charts/controller/claude-templates/`


   - Understand Handlebars context data structure


   - Document variable substitution mechanisms



2. **Identify Agent Customization Points**
   ```handlebars
   {{#if (eq github_app "5DLabs-Cleo")}}
     # Cleo-specific configuration
   {{else if (eq github_app "5DLabs-Tess")}}
     # Tess-specific configuration
   {{else}}
     # Default Rex/Blaze configuration
   {{/if}}





```

### Phase 4: Infrastructure Integration



1. **Argo Events Analysis**


   - Document existing GitHub webhook → Argo Workflows setup


   - Verify EventSource and Sensor configurations


   - Test webhook payload processing and correlation



2. **GitHub App Authentication**
   - Map secret structure: app-id, private-key, client-id


   - Document authentication flow in `container.sh.hbs`


   - Verify External Secrets integration

### Phase 5: Generate Discovery Report



1. **Create Comprehensive Documentation**


   - Architecture overview with component diagrams


   - Detailed technical specifications


   - Modification requirements for multi-agent support


   - Compatibility assessment for existing workflows


   - Implementation roadmap with dependencies



## Code Examples

### CRD Processing Flow



```rust
// Simplified reconciliation pattern
async fn reconcile(crd: Arc<CodeRun>, ctx: Arc<Context>) -> Result<Action> {
    // 1. Update status to Processing
    update_status(&crd, Phase::Processing).await?;

    // 2. Create/update resources
    let pvc = create_pvc(&crd, &ctx).await?;
    let configmap = create_configmap(&crd, &ctx).await?;
    let pod = create_pod(&crd, &ctx, &pvc, &configmap).await?;

    // 3. Update status based on pod state
    match pod.status {
        Some(status) if status.phase == "Succeeded" => {
            update_status(&crd, Phase::Completed).await?
        }
        Some(status) if status.phase == "Failed" => {
            update_status(&crd, Phase::Failed).await?
        }
        _ => {}
    }

    // 4. Requeue for status checks
    Ok(Action::requeue(Duration::from_secs(30)))
}






```

### Template Context Building



```rust
// Building Handlebars context for agent-specific rendering
let mut context = Context::new();
context.insert("github_app", &crd.spec.github_app);
context.insert("service", &crd.spec.service);
context.insert("model", &crd.spec.model);
context.insert("continue_session", &crd.spec.continue_session);

// Render templates with agent-specific logic
let claude_md = render_template("claude.md.hbs", &context)?;
let client_config = render_template("client-config.json.hbs", &context)?;






```

## Architecture Patterns

### Status-First Idempotency
The controller uses a status-first approach where:


1. Status updates are persisted before resource creation


2. Reconciliation can safely restart at any point


3. TTL mechanisms prevent zombie resources


4. Finalizers ensure clean resource deletion

### Agent Workspace Isolation
Each agent maintains separate PVC workspaces:
- Pattern: `workspace-{service}-{agent}`
- Benefits: Clean cancellation, independent context, session continuity
- Implementation: Extract agent name from `github_app` field

## Key Findings



1. **CRD Fully Supports Multi-Agent Scenarios**


   - `github_app` field enables agent differentiation


   - Environment variables and secrets are agent-specific


   - No schema changes required



2. **Controller Has Solid Reconciliation Patterns**


   - Status-first idempotency ensures reliability


   - TTL safety prevents resource leaks


   - Finalizer logic handles cleanup properly



3. **Template System Ready for Customization**


   - Handlebars conditionals support agent-specific logic


   - Template loading can check for agent-specific overrides


   - Context data structure is extensible



4. **Infrastructure Components Functional**


   - Argo Events working (with rate limiting considerations)


   - GitHub App authentication is production-ready


   - External Secrets integration is operational

## References



- [CodeRun CRD Specification](controller/src/crds/coderun.rs)


- [Controller Implementation](controller/src/tasks/code/controller.rs)


- [Template System](infra/charts/controller/claude-templates/)


- [Architecture Documentation](.taskmaster/docs/architecture.md)


- [Product Requirements](.taskmaster/docs/prd.txt)
