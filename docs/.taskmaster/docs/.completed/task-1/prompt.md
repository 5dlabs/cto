# Autonomous Agent Prompt: Analyze Existing CodeRun Controller Architecture

## 🚨 CRITICAL: Argo Events Reference Documentation

**BEFORE implementing ANY Argo Events sensors/triggers, MUST review official examples:**
- **Location:** [docs/references/argo-events/](../../../references/argo-events/)
- **Key Files:**
  - `github.yaml` - GitHub webhook sensor patterns
  - `complete-trigger-parameterization.yaml` - Dynamic parameter extraction  
  - `special-workflow-trigger.yaml` - ArgoWorkflow operations (submit/resume)
  - `trigger-standard-k8s-resource.yaml` - K8s resource creation patterns

**❌ UNSUPPORTED Operations (will cause deployment failures):**
- `operation: delete` ❌
- `operation: patch` ❌  
- `operation: update` ❌
- Template variables in `labelSelector` ❌

**✅ SUPPORTED Operations:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows)
- `dest: metadata.name` (dynamic targeting)

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!


## Mission

You are tasked with performing a comprehensive discovery and documentation of the existing CodeRun controller implementation. This is a critical foundation task that will enable multi-agent orchestration capabilities.

## Context

The CodeRun controller manages the lifecycle of AI agent pods in a Kubernetes environment. Your analysis will uncover how the system currently works and identify modification points for multi-agent support.

## Objectives

1. **Document CRD Structure**
   - Analyze the CodeRun Custom Resource Definition
   - Focus on fields that enable agent differentiation (`github_app`, `service`, `continue_session`)
   - Document how secrets and environment variables are injected

2. **Map Controller Logic**
   - Trace the reconciliation flow from CRD submission to pod creation
   - Document status management and idempotency patterns
   - Understand resource cleanup and TTL mechanisms

3. **Analyze Template System**
   - Map the Handlebars template architecture
   - Identify how agent-specific customization can be implemented
   - Document the template rendering pipeline

4. **Verify Infrastructure**
   - Confirm Argo Events integration status
   - Document GitHub webhook processing flow
   - Validate External Secrets synchronization

5. **Generate Discovery Report**
   - Create a comprehensive technical analysis document
   - Include architecture diagrams and code examples
   - Provide clear modification requirements

## Investigation Approach

### Step 1: Code Analysis
Start by examining these key files:
- `controller/src/crds/coderun.rs` - CRD specification
- `controller/src/tasks/code/controller.rs` - Main reconciliation logic
- `controller/src/tasks/code/resources.rs` - Resource creation
- `controller/src/tasks/code/templates.rs` - Template rendering

### Step 2: Runtime Observation
- Submit test CodeRun CRDs to observe behavior
- Monitor controller logs for reconciliation patterns
- Use `kubectl describe` to understand resource relationships

### Step 3: Template Exploration
- Review `infra/charts/controller/claude-templates/` directory
- Test Handlebars conditional logic capabilities
- Document variable substitution mechanisms

### Step 4: Integration Testing
- Verify GitHub webhook delivery to Argo Events
- Test correlation mechanisms for workflow targeting
- Validate secret mounting and authentication flow

## Expected Deliverables

1. **Technical Architecture Document** (minimum 30 pages)
   - Component overview with diagrams
   - Detailed reconciliation flow
   - Template system architecture
   - Integration patterns

2. **Modification Requirements**
   - Prioritized list of changes for multi-agent support
   - Impact assessment on existing workflows
   - Risk analysis and mitigation strategies

3. **Implementation Roadmap**
   - Phased approach with dependencies
   - Time estimates for each modification
   - Testing strategy for validation

## Key Questions to Answer

1. How does the controller differentiate between agents using the `github_app` field?
2. What modifications are needed for agent-specific PVC naming?
3. Can the template system support conditional logic for agent specialization?
4. How does session continuity work with the `continue_session` flag?
5. What is the current webhook correlation mechanism in Argo Events?
6. Are there any breaking changes required for multi-agent support?

## Success Criteria

- Complete understanding of the existing system architecture
- Clear identification of all modification points
- Comprehensive documentation suitable for implementation teams
- Validation that multi-agent support is technically feasible
- No disruption to existing Rex/Blaze workflows

## Tools and Resources

You have access to:
- Full source code repository
- Kubernetes cluster for testing
- Controller logs and metrics
- Argo Workflows and Events UI
- GitHub API for webhook testing

## Important Notes

- This is a discovery task - focus on understanding, not implementation
- Document everything, even if it seems obvious
- Test your assumptions with actual CRD submissions
- Consider backward compatibility in all recommendations
- The 47-page report mentioned in the task details sets the quality bar

Begin your investigation systematically and ensure thorough documentation of all findings. Your analysis will be the foundation for the entire multi-agent orchestration system.