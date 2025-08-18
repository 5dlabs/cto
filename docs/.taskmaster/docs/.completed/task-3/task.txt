# Task ID: 3
# Title: Design Multi-Agent Workflow DAG Structure
# Status: pending
# Dependencies: 1
# Priority: high
# Description: Create the core Argo Workflow template with parameterized agent selection, DAG task dependencies, and suspend points for event-driven transitions
# Details:
Design play-workflow template with configurable parameters: implementation-agent, quality-agent, testing-agent (no hardcoded names). Structure DAG with tasks: implementation-work → wait-pr-created (suspend) → quality-work → wait-ready-for-qa (suspend) → testing-work → wait-pr-approved (suspend) → complete-task. Set activeDeadlineSeconds: 1209600 (14 days). Add workflow labels for correlation: workflow-type=play-orchestration, task-id={{task-id}}, current-stage={{stage}}. Use Argo Workflows v3.5+ for enhanced suspend/resume capabilities.

# Test Strategy:
Deploy workflow template and validate DAG visualization in Argo UI. Test parameter propagation through workflow steps. Verify suspend points pause execution correctly. Confirm workflow can run for extended periods without timing out.

# Subtasks:
## 1. Create Base Workflow Template Structure [pending]
### Dependencies: None
### Description: Design and implement the foundational Argo Workflow template with parameterized agent selection and core DAG structure
### Details:
Create play-workflow.yaml template with ConfigMap or WorkflowTemplate resource. Define workflow parameters: implementation-agent, quality-agent, testing-agent as strings. Set workflow metadata including generateName, labels (workflow-type=play-orchestration), and annotations. Configure spec.activeDeadlineSeconds: 1209600. Define entrypoint pointing to main DAG template. Add volumeClaimTemplates for shared workspace between tasks.

## 2. Implement Implementation-Work Task [pending]
### Dependencies: 3.1
### Description: Create the first DAG task that submits CodeRun CRD for implementation agent execution
### Details:
Define implementation-work task using resource template type. Create CodeRun CRD manifest with agent parameter {{workflow.parameters.implementation-agent}}. Set task-id label from workflow parameter. Configure resource action: create. Add outputs to capture CodeRun name and status. Set retry strategy with backoff. Include failure handling with onExit hooks.

## 3. Add Wait-PR-Created Suspend Point [pending]
### Dependencies: 3.2
### Description: Implement the first suspension point that waits for PR creation event
### Details:
Create suspend template with name wait-pr-created. Add correlation labels: task-id={{workflow.parameters.task-id}}, current-stage=pr-creation. Configure suspend.duration as optional timeout (e.g., 4h). Add parameters to receive PR URL and number on resume. Store resume data in workflow parameters for next tasks. Include status message for UI visibility.

## 4. Create Quality-Work Task [pending]
### Dependencies: 3.3
### Description: Implement quality assurance task that invokes the QA agent after PR creation
### Details:
Define quality-work task dependent on wait-pr-created completion. Submit CodeRun CRD with quality-agent parameter. Pass PR information from previous suspend point as environment variables. Configure task to use PR URL from workflow parameters. Add output parameters for QA results. Set appropriate resource limits and timeout.

## 5. Add Wait-Ready-for-QA Suspend Point [pending]
### Dependencies: 3.4
### Description: Implement suspension point for QA readiness confirmation
### Details:
Create wait-ready-for-qa suspend template after quality-work. Update current-stage label to qa-ready. Configure correlation with same task-id. Add parameters for QA approval status and comments. Implement validation logic for resume data. Store QA feedback in workflow parameters for audit trail.

## 6. Implement Testing-Work Task [pending]
### Dependencies: 3.5
### Description: Create testing task that executes test agent after QA approval
### Details:
Define testing-work task using testing-agent parameter. Submit CodeRun CRD with test configuration. Pass PR and QA information from previous stages. Configure test-specific environment variables and secrets. Add test result outputs including pass/fail status and reports. Implement conditional logic based on QA feedback.

## 7. Add Wait-PR-Approved Suspend Point [pending]
### Dependencies: 3.6
### Description: Implement final suspension point for PR approval
### Details:
Create wait-pr-approved suspend template as final gate. Update current-stage label to pr-approval. Add parameters for approval status and merge commit SHA. Configure longer timeout (e.g., 48h) for human review. Store approval metadata including approver and timestamp. Add validation to ensure PR is mergeable.

## 8. Design Complete-Task with Cleanup [pending]
### Dependencies: 3.7
### Description: Implement workflow completion task with resource cleanup and notifications
### Details:
Create complete-task as final DAG node. Implement cleanup logic: delete temporary CodeRun CRDs, clean workspace volumes. Send completion notifications via webhook or events. Update task status in external system if configured. Archive workflow artifacts to S3/MinIO. Generate workflow summary report with all stage outcomes.

## 9. Configure Workflow Labels and Correlation [pending]
### Dependencies: 3.8
### Description: Set up comprehensive labeling system for event correlation and monitoring
### Details:
Apply workflow-level labels: workflow-type=play-orchestration, task-id={{workflow.parameters.task-id}}. Implement dynamic stage label updates using Argo expressions. Add correlation-id for distributed tracing. Configure label selectors for resume operations. Set up label-based monitoring queries. Ensure labels propagate to child resources (CodeRuns, PVCs).

