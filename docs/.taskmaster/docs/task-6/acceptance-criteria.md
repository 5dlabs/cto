# Acceptance Criteria: Agent-Specific Handlebars Templates

## Core Functionality Requirements

### ✅ Template Creation
- [ ] **Rex Container Template**: `container-rex.sh.hbs` created with implementation workflow
- [ ] **Cleo Container Template**: `container-cleo.sh.hbs` created with quality workflow  
- [ ] **Tess Container Template**: `container-tess.sh.hbs` created with testing workflow
- [ ] **Template Selection Logic**: Controller selects correct template based on `github_app` field
- [ ] **Backward Compatibility**: Default `container.sh.hbs` continues working for existing workflows

### ✅ Agent-Specific Functionality

#### Rex/Blaze Template Requirements
- [ ] Documentation-first approach with MCP server integration
- [ ] Task file copying from `.taskmaster/docs/task-{id}/` directory
- [ ] Environment variables: `AGENT_ROLE=implementation`, `WORKFLOW_STAGE=implementation`
- [ ] PR creation workflow with `task-{id}` labeling
- [ ] Branch naming pattern: `task-{id}-{description}`

#### Cleo Template Requirements  
- [ ] Code quality tools setup (Clippy pedantic, rustfmt)
- [ ] GitHub API authentication for label management
- [ ] Environment variables: `AGENT_ROLE=quality`, `WORKFLOW_STAGE=quality-work`
- [ ] CI test validation before completion
- [ ] Ready-for-QA label addition capability

#### Tess Template Requirements
- [ ] Kubernetes admin access configuration (`KUBECONFIG` setup)
- [ ] Database admin credentials (Postgres, Redis) mounting
- [ ] Environment variables: `AGENT_ROLE=testing`, `WORKFLOW_STAGE=testing-work`
- [ ] 120% satisfaction requirement setup
- [ ] Ready-for-QA label prerequisite check

## Controller Integration Requirements

### ✅ Template Selection Logic
- [ ] **Template Resolution Function**: `get_container_template(github_app)` implemented
- [ ] **Agent Mapping**: Correct template selected for each GitHub App:
  - `5DLabs-Rex` → `container-rex.sh.hbs`
  - `5DLabs-Blaze` → `container-rex.sh.hbs` 
  - `5DLabs-Cleo` → `container-cleo.sh.hbs`
  - `5DLabs-Tess` → `container-tess.sh.hbs`
  - Default → `container.sh.hbs`

### ✅ Template Loading Enhancement
- [ ] **Render Function Update**: `render_container_script()` uses agent-specific templates
- [ ] **ConfigMap Integration**: Agent-specific container script included in pod ConfigMap
- [ ] **Context Variables**: All required Handlebars variables available to templates
- [ ] **Error Handling**: Graceful fallback when agent-specific template missing

## File Structure Requirements

### ✅ Template Organization
- [ ] **Template Directory**: All templates in `infra/charts/controller/claude-templates/`
- [ ] **Naming Convention**: `container-{agent}.sh.hbs` pattern followed
- [ ] **Agent Prompts**: Agent-specific system prompts in `agents/` subdirectory
- [ ] **Template Hierarchy**: Clear inheritance and override patterns

### ✅ Controller Code Changes
- [ ] **Template Loading**: Modified `templates.rs` with agent-specific logic
- [ ] **Resource Creation**: Updated ConfigMap creation in `resources.rs`
- [ ] **Type Safety**: Proper error handling for template resolution
- [ ] **Documentation**: Code comments explaining agent template selection

## Testing Requirements

### ✅ Unit Testing
- [ ] **Template Selection Tests**: Verify correct template chosen for each agent
- [ ] **Context Building Tests**: Ensure all variables available to templates
- [ ] **Rendering Tests**: Templates render without Handlebars errors
- [ ] **Fallback Tests**: Default template used when agent template missing

### ✅ Integration Testing
- [ ] **Rex Workflow Test**: Create CodeRun with `github_app: 5DLabs-Rex`, verify behavior
- [ ] **Cleo Workflow Test**: Create CodeRun with `github_app: 5DLabs-Cleo`, verify behavior
- [ ] **Tess Workflow Test**: Create CodeRun with `github_app: 5DLabs-Tess`, verify behavior
- [ ] **Backward Compatibility Test**: Existing Rex/Blaze workflows continue working

### ✅ End-to-End Testing  
- [ ] **Template Deployment**: Templates deployed to cluster successfully
- [ ] **Pod Creation**: Pods created with correct container scripts
- [ ] **Environment Setup**: Agent-specific environment variables set correctly
- [ ] **Script Execution**: Container scripts execute without errors

## Workflow Integration Requirements

### ✅ Agent Handoff Signals
- [ ] **Rex to Cleo**: PR creation triggers Cleo workflow
- [ ] **Cleo to Tess**: Ready-for-QA label triggers Tess workflow  
- [ ] **Label Correlation**: Task ID extracted from PR labels correctly
- [ ] **Event Processing**: Argo Events sensors detect agent transitions

### ✅ Session Management
- [ ] **Agent Isolation**: Each agent uses separate PVC workspace
- [ ] **Context Continuity**: Agent sessions continue from previous work
- [ ] **Clean Handoffs**: No context contamination between agents
- [ ] **Resource Cleanup**: Cancelled agents cleaned up properly

## Performance Requirements

### ✅ Template Loading Performance
- [ ] **Load Time**: Template selection adds <100ms to pod creation
- [ ] **Memory Usage**: Template caching doesn't cause memory leaks
- [ ] **Concurrent Access**: Multiple CodeRuns don't cause template conflicts
- [ ] **Error Recovery**: Template loading failures don't crash controller

### ✅ Workflow Performance  
- [ ] **Agent Startup**: Each agent starts within 60 seconds
- [ ] **Handoff Latency**: Agent transitions happen within 5 minutes
- [ ] **Resource Usage**: Templates don't significantly increase pod resource usage
- [ ] **Scalability**: System handles multiple concurrent tasks

## Security Requirements

### ✅ Template Security
- [ ] **Code Injection**: Templates sanitized against injection attacks
- [ ] **Secret Exposure**: No sensitive data leaked in template rendering
- [ ] **Permission Isolation**: Each agent gets only required permissions
- [ ] **Access Control**: Template files protected from unauthorized modification

### ✅ Agent Isolation
- [ ] **Workspace Separation**: Agent PVCs completely isolated
- [ ] **Secret Segregation**: Each agent accesses only its secrets
- [ ] **Network Policies**: Agent pods follow network isolation rules
- [ ] **RBAC Compliance**: Agent permissions follow least-privilege principle

## Documentation Requirements

### ✅ Technical Documentation
- [ ] **Implementation Guide**: Step-by-step template creation process
- [ ] **Architecture Overview**: Agent template system architecture documented
- [ ] **Configuration Reference**: All template variables and options documented
- [ ] **Troubleshooting Guide**: Common issues and solutions documented

### ✅ Operational Documentation
- [ ] **Deployment Instructions**: How to deploy new templates
- [ ] **Monitoring Guidelines**: How to monitor agent template performance
- [ ] **Maintenance Procedures**: How to update and maintain templates
- [ ] **Rollback Procedures**: How to rollback template changes if needed

## Validation Checklist

Before marking this task complete, verify:

1. **All Templates Created**: Rex, Cleo, and Tess templates exist and render correctly
2. **Controller Integration**: Template selection logic integrated into controller
3. **Testing Passed**: All unit, integration, and end-to-end tests pass
4. **Workflow Validation**: Multi-agent workflow progression works correctly  
5. **Backward Compatibility**: Existing workflows unaffected
6. **Documentation Complete**: All technical and operational docs updated
7. **Security Review**: Security requirements met and validated
8. **Performance Verified**: Performance requirements met under load

## Success Metrics

- **Template Rendering**: 100% success rate for all agent templates
- **Agent Startup**: <60 seconds average agent startup time
- **Workflow Handoffs**: <5 minutes average handoff between agents  
- **Error Rate**: <1% template-related errors in production
- **Backward Compatibility**: 0 regressions in existing Rex/Blaze workflows