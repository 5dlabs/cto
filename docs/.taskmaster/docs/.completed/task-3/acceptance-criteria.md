# Acceptance Criteria: Design Multi-Agent Workflow DAG Structure

## Functional Requirements



### 1. Base Workflow Template Structure


- [ ] WorkflowTemplate resource created with name `play-workflow-template`


- [ ] Deployed successfully to `argo` namespace


- [ ] `activeDeadlineSeconds` set to 1209600 (14 days)


- [ ] `entrypoint` configured to point to main DAG template


- [ ] Proper `generateName` pattern for workflow instances


- [ ] No hardcoded agent names in any template component

### 2. Parameterized Agent Selection


- [ ] `implementation-agent` parameter defined with default "5DLabs-Rex"


- [ ] `quality-agent` parameter defined with default "5DLabs-Cleo"


- [ ] `testing-agent` parameter defined with default "5DLabs-Tess"


- [ ] `task-id` parameter defined for workflow correlation


- [ ] `repository` parameter defined with default "5dlabs/cto"


- [ ] All parameters have proper descriptions and types


- [ ] Parameters propagate correctly to all workflow tasks

### 3. DAG Task Structure and Dependencies


- [ ] Main DAG template contains exactly 7 tasks in correct order


- [ ] `implementation-work` task has no dependencies (starts workflow)


- [ ] `wait-pr-created` depends on `implementation-work`


- [ ] `quality-work` depends on `wait-pr-created`


- [ ] `wait-ready-for-qa` depends on `quality-work`


- [ ] `testing-work` depends on `wait-ready-for-qa`


- [ ] `wait-pr-approved` depends on `testing-work`


- [ ] `complete-task` depends on `wait-pr-approved`

### 4. Agent CodeRun Integration


- [ ] `agent-coderun` template creates CodeRun CRD


- [ ] Template accepts `github-app`, `task-id`, and `stage` parameters


- [ ] CodeRun spec includes proper agent-specific github_app field
- [ ] `continue_session: true` enabled for agent memory
- [ ] Proper labels applied: `task-id`, `github-app`, `workflow-name`


- [ ] Resource action set to `create` for CRD submission

### 5. Suspend Point Implementation
- [ ] Three suspend templates implemented: `wait-pr-created`, `wait-ready-for-qa`, `wait-pr-approved`


- [ ] Each suspend point uses indefinite suspend (no duration timeout)


- [ ] Suspend templates update workflow labels with current stage


- [ ] Proper correlation labels for event targeting


- [ ] Resume capability preserves workflow state and parameters

### 6. Workflow Label & Naming Management
- [ ] Workflow-level labels: `workflow-type=play-orchestration`


- [ ] Dynamic `task-id` label from workflow parameter
- [ ] Deterministic instance name: `name: play-task-{{workflow.parameters.task-id}}-workflow`


- [ ] `current-stage` label updates at each workflow phase


- [ ] Repository label for multi-repo support


- [ ] Labels propagate to child resources (CodeRuns, PVCs)

### 7. Task Completion and Cleanup


- [ ] `complete-task` implements proper resource cleanup


- [ ] Task directory moved to `.completed/` folder


- [ ] Workflow summary generated and archived


- [ ] Next-task progression logic implemented


- [ ] Temporary CodeRun CRDs cleaned up appropriately

## Technical Requirements

### Workflow Template Validation


- [ ] Template passes `argo template create --dry-run` validation


- [ ] YAML syntax is valid and follows Kubernetes conventions


- [ ] All template references resolve correctly


- [ ] Resource requests and limits configured appropriately



### Parameter System


- [ ] All parameters have appropriate default values


- [ ] Parameter types and descriptions are accurate


- [ ] Parameter substitution works in all template contexts


- [ ] No parameter conflicts or circular dependencies

### Resource Management


- [ ] Proper resource cleanup on workflow completion


- [ ] PVC management for agent workspaces


- [ ] Memory and CPU limits configured for script templates


- [ ] Appropriate retry and error handling strategies

### Event Correlation


- [ ] Label selectors enable precise workflow targeting


- [ ] Stage labels update dynamically during execution


- [ ] Correlation works with GitHub webhook event payloads


- [ ] Multi-method validation (task-id + stage) prevents false positives



## Test Cases

### Test Case 1: Template Deployment and Validation
**Objective**: Verify WorkflowTemplate deploys and validates correctly

**Steps**:


1. Deploy template to Argo namespace
2. Verify template registration: `argo template list`
3. Validate template syntax: `argo template create --dry-run play-workflow-template.yaml`


4. Check template in Argo UI

**Expected Result**: Template deploys successfully with no validation errors

### Test Case 2: Parameter Propagation Testing
**Objective**: Validate parameters flow correctly to all workflow tasks

**Steps**:


1. Submit workflow with custom agent parameters


2. Monitor task execution and parameter substitution


3. Verify CodeRun CRDs created with correct github_app values


4. Confirm task-id propagates to all workflow components

**Expected Result**: All parameters substitute correctly throughout workflow

### Test Case 3: DAG Structure and Dependencies
**Objective**: Verify task dependencies and execution order

**Steps**:


1. Submit test workflow and monitor in Argo UI


2. Verify DAG visualization shows correct task relationships


3. Confirm tasks execute in proper sequential order


4. Validate suspend points pause execution correctly

**Expected Result**: Tasks execute in defined order with proper dependencies

### Test Case 4: Suspend/Resume Functionality
**Objective**: Test indefinite suspend and manual resume capability

**Steps**:


1. Start workflow and wait for first suspend point


2. Verify workflow is suspended indefinitely (no timeout)
3. Manually resume workflow: `argo resume workflow-name`


4. Confirm workflow continues to next stage


5. Repeat for all suspend points

**Expected Result**: All suspend points work correctly with manual resume

### Test Case 5: Agent Integration and CodeRun Creation
**Objective**: Validate CodeRun CRD creation for different agents

**Steps**:


1. Submit workflow with Rex as implementation agent


2. Verify CodeRun created with proper spec and labels


3. Test with Blaze as implementation agent


4. Confirm different agents work with same template

**Expected Result**: CodeRuns created correctly for all supported agents

### Test Case 6: Label Management and Correlation
**Objective**: Test workflow label updates and event correlation

**Steps**:


1. Start workflow and monitor label changes


2. Verify `current-stage` label updates at each phase


3. Test event correlation using label selectors


4. Confirm labels enable precise workflow targeting

**Expected Result**: Labels update correctly and enable accurate correlation

### Test Case 7: Workflow Completion and Cleanup
**Objective**: Validate task completion, cleanup, and progression

**Steps**:


1. Run complete workflow from start to finish


2. Verify task directory moves to `.completed/`


3. Confirm workflow summary generated


4. Test next-task progression logic


5. Validate resource cleanup

**Expected Result**: Completion process executes correctly with proper cleanup

### Test Case 8: Extended Runtime Testing
**Objective**: Verify workflow can run for extended periods

**Steps**:


1. Start workflow with multiple day suspension


2. Monitor resource usage during suspended state


3. Verify workflow doesn't timeout before 14 days


4. Confirm workflow health throughout extended runtime

**Expected Result**: Workflow runs reliably for multi-day periods

### Test Case 9: Error Handling and Recovery
**Objective**: Test workflow behavior under error conditions

**Steps**:


1. Introduce failures in CodeRun creation


2. Test parameter validation with invalid values


3. Verify retry logic and error recovery


4. Test workflow cleanup on failure scenarios

**Expected Result**: Workflow handles errors gracefully with appropriate recovery

### Test Case 10: Multi-Workflow Concurrency
**Objective**: Validate multiple concurrent workflow instances

**Steps**:


1. Start multiple workflows with different task-ids


2. Verify each workflow maintains independent state


3. Test event correlation doesn't cross workflows


4. Monitor resource usage and performance

**Expected Result**: Multiple workflows run concurrently without interference



## Quality Criteria

### Code Quality Standards


- [ ] YAML follows Kubernetes best practices


- [ ] Proper indentation and formatting throughout


- [ ] Consistent naming conventions for all components


- [ ] Clear comments explaining complex template logic


- [ ] No deprecated API versions or fields

### Documentation Requirements


- [ ] Template components documented with clear descriptions


- [ ] Parameter usage explained with examples


- [ ] Suspend/resume behavior documented


- [ ] Event correlation patterns explained


- [ ] Troubleshooting guide included

### Performance Standards


- [ ] Workflow startup time < 30 seconds


- [ ] Suspend points activate within 10 seconds


- [ ] Resource usage minimal during suspended state


- [ ] Template rendering completes without timeout


- [ ] Label updates propagate within 5 seconds

### Security and Compliance


- [ ] Minimal required RBAC permissions defined


- [ ] No sensitive data in workflow parameters or labels


- [ ] Proper namespace isolation maintained


- [ ] Resource limits prevent resource exhaustion



## Deliverable Checklist



- [ ] `play-workflow-template.yaml` created and validated


- [ ] Template deployed to Argo namespace successfully


- [ ] DAG structure visualized correctly in Argo UI


- [ ] All test cases executed and documented


- [ ] Parameter system tested with multiple agent combinations


- [ ] Suspend/resume functionality validated


- [ ] Event correlation patterns tested and documented


- [ ] Troubleshooting guide created


- [ ] Performance benchmarks documented



## Success Metrics

1. **Template Functionality**: 100% of template features working correctly
2. **Parameter Flexibility**: Support for any combination of available agents
3. **Suspend Reliability**: All suspend points work indefinitely without timeout
4. **Event Correlation**: 100% accuracy in workflow targeting for events
5. **Performance**: Workflow operations complete within defined time limits
6. **Concurrency**: Multiple workflows run simultaneously without conflicts



## Notes



- This template forms the foundation for the entire multi-agent orchestration system


- Focus on parameterization to avoid hardcoded dependencies


- Ensure suspend/resume patterns work reliably for extended periods


- Label management is critical for event correlation accuracy


- Consider operational monitoring and debugging requirements
