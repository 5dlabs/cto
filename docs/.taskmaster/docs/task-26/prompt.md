# Autonomous Agent Prompt: Task Association Validation System

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


## Objective

Implement a comprehensive task association validation system that ensures GitHub workflows are correctly correlated with Task Master tasks using three complementary validation methods. The system must require agreement between all methods before allowing workflow execution.

## Context

You are implementing a critical validation system for the Task Master workflow orchestration platform. This system prevents workflow execution errors by validating that:
1. PR labels contain the correct task ID (`task-{id}`)
2. Branch names follow the expected pattern (`task-{id}-description` or `feature/task-{id}`)
3. Marker files contain matching task IDs

## Implementation Requirements

### Core Validation Logic

Implement three-tier validation in Argo Workflows:

1. **PR Label Extraction**
   - Use JQ pattern: `.pull_request.labels[] | select(.name | startswith("task-")) | .name | split("-")[1]`
   - Handle multiple task labels by taking the first match
   - Return empty string if no task labels found

2. **Branch Name Parsing**
   - Apply regex: `^(?:feature/)?task-(\d+)(?:-.*)?$`
   - Extract task ID from `pull_request.head.ref`
   - Support both `task-26-description` and `feature/task-26` patterns

3. **Marker File Reading**
   - Read `docs/.taskmaster/current-task.json`
   - Extract `task_id` field from JSON structure
   - Handle missing file gracefully

### Validation Enforcement

Create validation logic that:
- Compares all three extracted task IDs
- Requires exact matches between all non-empty IDs
- Fails workflow if any IDs disagree
- Creates detailed error reports for mismatches
- Posts GitHub comments with validation results

### Marker File Management

Implement marker file creation with:
```json
{
  "task_id": "extracted-id",
  "started_at": "ISO-8601-timestamp",
  "agent": "workflow-parameter-agent",
  "workflow_id": "workflow-name",
  "commit_sha": "current-git-sha",
  "pr_number": "pr-number"
}
```

## Technical Specifications

### Argo Workflows Integration

Create WorkflowTemplate with:
- Parameter extraction from webhook payload
- Step-based validation pipeline
- Conditional execution based on validation results
- Error handling with retry mechanisms
- GitHub API integration for comments

### Error Handling Requirements

Implement comprehensive error handling:
- Retry failed validations up to 3 times
- Create GitHub comments for validation failures
- Log all validation attempts with structured data
- Emit Prometheus metrics for monitoring
- Implement graceful degradation for API failures

### Security Requirements

Ensure secure implementation:
- Store GitHub tokens in Kubernetes secrets
- Validate all external inputs
- Use least-privilege service accounts
- Implement audit logging for all operations
- Sanitize branch names and labels before processing

## Expected Deliverables

1. **Argo Events Sensor Configuration**
   - Webhook event handling
   - Parameter extraction templates
   - Workflow trigger logic

2. **WorkflowTemplate Implementation**
   - Multi-step validation pipeline
   - Task ID extraction logic
   - Validation comparison functions
   - Marker file management

3. **Error Handling System**
   - GitHub comment templates
   - Retry strategies
   - Monitoring integration
   - Audit logging

4. **Documentation Updates**
   - Validation flow diagrams
   - Error scenario playbooks
   - Monitoring setup guides

## Acceptance Criteria

- All three validation methods extract task IDs correctly
- System fails fast on validation mismatches
- GitHub comments provide clear guidance for fixes
- Marker files are created and managed properly
- Retry logic handles transient failures
- Monitoring captures all validation events
- Security requirements are met
- Integration tests pass for all scenarios

## Quality Standards

- Follow existing Argo Workflows patterns
- Use structured YAML configurations
- Implement comprehensive error handling
- Include detailed logging and metrics
- Maintain backward compatibility
- Document all integration points
- Test edge cases thoroughly

## Resources

- Argo Workflows documentation for templates and sensors
- GitHub API documentation for webhook payloads
- JQ documentation for JSON processing
- Kubernetes secrets management best practices
- Task Master architecture documentation

Focus on creating a robust, secure, and maintainable validation system that prevents workflow execution errors through comprehensive task association validation.