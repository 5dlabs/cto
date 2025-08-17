# Task 21: Create Basic End-to-End Testing Suite

## Overview

This task focuses on developing a basic end-to-end testing suite for the multi-agent workflow orchestration system. The testing suite will validate the core happy-path pipeline from GitHub webhook triggers through Rex implementation, Cleo code quality, and Tess validation phases, ensuring the basic multi-agent flow works correctly.

## Technical Requirements

### Core Testing Scenarios

1. **Happy Path Multi-Agent Flow**
   - Create test PR with `task-X` label
   - Verify workflow starts and progresses through stages
   - Confirm Rex completes implementation work
   - Verify Cleo adds "ready-for-qa" label
   - Confirm Tess completes testing and approves PR
   - Validate workflow completes successfully

2. **Basic Validation Points**
   - Workflow creation from GitHub webhook
   - Proper task correlation via PR labels
   - Agent stage transitions (Rex → Cleo → Tess)
   - Successful PR progression
   - Resource cleanup after completion

## Implementation Approach

### Simple Test Infrastructure

#### Test Repository Setup
- Use existing test repository or create dedicated E2E repo
- Configure basic labels: `task-1`, `task-2`, `ready-for-qa`
- Enable test branch creation and automated cleanup

#### Basic PR Creation Utility
```python
def create_test_pr(task_id: int):
    """Create simple test PR with task label for E2E testing"""
    branch_name = f"task-{task_id}-e2e-test"
    
    # Create test branch with minimal changes
    create_test_branch_with_changes(branch_name)
    
    # Create PR with required task label
    pr = github.create_pull_request(
        title=f"E2E Test - Task {task_id}",
        head=branch_name,
        base="main",
        body="Automated E2E test PR"
    )
    
    # Add task correlation label
    pr.add_to_labels(f"task-{task_id}")
    return pr
```

### Basic Workflow Validation

#### Simple Assertions
- Verify workflow appears in Argo with correct labels
- Check that workflow progresses through expected stages
- Confirm each agent pod completes successfully
- Validate PR receives expected labels and approval

#### Basic Test Script Structure
```python
def test_basic_multi_agent_flow():
    # 1. Create test PR
    pr = create_test_pr(task_id=99)
    
    # 2. Wait for workflow to start
    workflow = wait_for_workflow_creation(task_id=99)
    
    # 3. Verify Rex stage completion
    wait_for_agent_completion("Rex", workflow_id=workflow.id)
    
    # 4. Verify Cleo stage and "ready-for-qa" label
    wait_for_label_addition(pr, "ready-for-qa")
    wait_for_agent_completion("Cleo", workflow_id=workflow.id)
    
    # 5. Verify Tess stage and PR approval
    wait_for_pr_approval(pr, approver="5DLabs-Tess[bot]")
    wait_for_agent_completion("Tess", workflow_id=workflow.id)
    
    # 6. Verify workflow completion
    wait_for_workflow_completion(workflow_id=workflow.id)
    
    # 7. Cleanup test data
    cleanup_test_pr(pr)
    cleanup_test_workflow(workflow)
```

## Deliverables

### Test Suite Components
1. **Simple test harness** - Basic framework for running E2E tests
2. **PR creation utilities** - Simple functions for creating test PRs
3. **Workflow validation helpers** - Basic assertions for workflow progression
4. **Cleanup utilities** - Simple cleanup of test resources

### Test Coverage
- **Single happy path scenario** - Core multi-agent flow (Rex → Cleo → Tess)
- **Basic error detection** - Simple failure identification
- **Resource cleanup verification** - Ensure no test artifacts remain

## Implementation Notes

- Focus on proving the core multi-agent orchestration works
- Keep assertions simple and reliable
- Prioritize test stability over comprehensive coverage
- Use existing tools and APIs where possible
- Implement basic logging for debugging test failures

## Success Criteria

1. **Automated test execution** - Test can run without manual intervention
2. **Multi-agent flow validation** - Verifies Rex → Cleo → Tess progression
3. **Reliable cleanup** - No test artifacts left after execution
4. **Clear pass/fail results** - Obvious test outcome reporting
5. **Basic debugging info** - Sufficient logging to diagnose failures

This simplified approach focuses on validating the core functionality rather than comprehensive testing scenarios, making it more achievable for initial implementation while still proving the multi-agent system works correctly.