# Task 21: Create End-to-End Testing Suite

## Overview

This task focuses on developing a comprehensive end-to-end testing suite for the multi-agent workflow orchestration system. The testing suite will validate the entire pipeline from GitHub webhook triggers through Rex implementation, Cleo code quality, and Tess validation phases, ensuring robust operation across various scenarios including failure modes and performance characteristics.

## Technical Implementation

### 1. Test Architecture Design

The E2E testing framework will consist of multiple test layers:

- **Synthetic Event Generation**: Create realistic GitHub webhook payloads and PR events
- **Workflow State Verification**: Assert on Argo Workflow state transitions and suspension points  
- **Agent Behavior Validation**: Verify each agent (Rex, Cleo, Tess) performs expected actions
- **Performance Measurement**: Track latency and identify bottlenecks across the pipeline
- **Chaos Engineering**: Test system resilience under various failure conditions

### 2. GitHub API Test Infrastructure

#### Test Repository Management
```yaml
# Test repository configuration
test_repos:
  - name: "test-taskmaster-e2e-primary"
    visibility: private
    labels:
      - name: "task-1"
        color: "ff0000"
        description: "Task 1 implementation" 
      - name: "ready-for-qa"
        color: "00ff00"
        description: "Ready for QA review"
    branch_protection: false  # Allow force pushes for testing
```

#### Synthetic PR Generation
```python
# Python test utility for PR creation
class SyntheticPRGenerator:
    def __init__(self, github_client, repo_name):
        self.github = github_client
        self.repo = repo_name
    
    def create_test_pr(self, task_id: int, complexity: str = "simple"):
        """Create PR with proper labels and branch naming"""
        branch_name = f"task-{task_id}-test-{complexity}-{uuid4().hex[:8]}"
        
        # Create branch with test changes
        changes = self.generate_changes_by_complexity(complexity)
        self.create_branch_with_changes(branch_name, changes)
        
        # Create PR with required labels
        pr = self.github.create_pull_request(
            title=f"Test PR for Task {task_id} - {complexity}",
            head=branch_name,
            base="main",
            body=f"Synthetic PR for E2E testing - Task {task_id}"
        )
        
        # Add task label for workflow correlation
        pr.add_to_labels(f"task-{task_id}")
        
        return pr
```

### 3. Argo Workflow Assertion Framework

#### State Verification Engine
```yaml
# Workflow assertion configuration
workflow_assertions:
  initialization:
    - check: workflow_created
      timeout: "30s"
      selector: "workflow-type=play-orchestration,task-id={{task_id}}"
    - check: stage_label_present
      expected: "current-stage=waiting-pr-created"
  
  rex_completion:
    - check: workflow_suspended
      node_name: "wait-pr-created"
      timeout: "300s"
    - check: agent_pod_completed
      selector: "github-app=5DLabs-Rex,task-id={{task_id}}"
  
  cleo_handoff:
    - check: workflow_resumed
      trigger_event: "pr-created"
      timeout: "60s"
    - check: stage_transition
      from: "waiting-pr-created"
      to: "waiting-ready-for-qa"
```

#### Workflow Monitoring Library
```go
// Go library for workflow state monitoring
type WorkflowAssertion struct {
    ArgoClient    argoclientset.Interface
    TaskID        string
    TimeoutPeriod time.Duration
}

func (wa *WorkflowAssertion) WaitForSuspension(nodeName string) error {
    return wait.Poll(5*time.Second, wa.TimeoutPeriod, func() (bool, error) {
        wf, err := wa.getWorkflowByTaskID()
        if err != nil {
            return false, err
        }
        
        node := wf.Status.Nodes[nodeName]
        return node.Phase == wfv1.NodeSuspended, nil
    })
}

func (wa *WorkflowAssertion) VerifyStageTransition(fromStage, toStage string) error {
    wf, err := wa.getWorkflowByTaskID()
    if err != nil {
        return err
    }
    
    currentStage := wf.Labels["current-stage"]
    if currentStage != toStage {
        return fmt.Errorf("expected stage %s, got %s", toStage, currentStage)
    }
    
    return nil
}
```

### 4. Synthetic Task Generation

#### Complexity-Based Task Generator
```python
class TaskComplexityGenerator:
    SIMPLE_TASKS = {
        "file_count": 1,
        "lines_per_file": 50,
        "dependency_count": 0,
        "test_required": False
    }
    
    COMPLEX_TASKS = {
        "file_count": 8,
        "lines_per_file": 200,
        "dependency_count": 3,
        "test_required": True,
        "integration_points": 2
    }
    
    def generate_task_files(self, complexity: str, language: str = "rust"):
        """Generate synthetic code changes for testing"""
        if complexity == "simple":
            return self.generate_simple_rust_changes()
        elif complexity == "complex":
            return self.generate_complex_rust_refactor()
        
    def generate_simple_rust_changes(self):
        return {
            "src/lib.rs": """
                // Simple test implementation
                pub fn test_function() -> Result<String, Error> {
                    Ok("test implementation".to_string())
                }
                
                #[cfg(test)]
                mod tests {
                    use super::*;
                    
                    #[test]
                    fn test_basic_functionality() {
                        assert!(test_function().is_ok());
                    }
                }
            """
        }
```

### 5. Chaos Testing Implementation

#### Infrastructure Failure Scenarios
```yaml
# Chaos Mesh configuration for testing
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: agent-pod-failure
  namespace: agent-platform
spec:
  action: pod-kill
  mode: one
  duration: "30s"
  selector:
    namespaces:
      - agent-platform
    labelSelectors:
      "app": "coderun"
      "github-app": "5DLabs-Rex"
  scheduler:
    cron: "@every 5m"
```

#### Network Partition Testing
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: github-api-partition
spec:
  action: partition
  mode: all
  selector:
    namespaces: ["agent-platform"]
  direction: to
  externalTargets: ["api.github.com"]
  duration: "60s"
```

### 6. Performance Measurement Suite

#### Latency Tracking Implementation
```python
class PerformanceMetrics:
    def __init__(self):
        self.metrics = {}
    
    def track_stage_duration(self, stage_name: str, start_time: datetime, end_time: datetime):
        duration = (end_time - start_time).total_seconds()
        self.metrics[f"{stage_name}_duration"] = duration
        
        # Send to Prometheus
        stage_duration_histogram.labels(stage=stage_name).observe(duration)
    
    def measure_workflow_end_to_end(self, task_id: int):
        """Measure complete workflow execution time"""
        workflow_start = self.get_workflow_creation_time(task_id)
        workflow_end = self.get_workflow_completion_time(task_id)
        
        total_duration = (workflow_end - workflow_start).total_seconds()
        
        return {
            "total_duration_seconds": total_duration,
            "stages": self.get_stage_breakdown(task_id)
        }
```

### 7. Property-Based Testing

#### Event Correlation Validation
```python
from hypothesis import given, strategies as st

class EventCorrelationTests:
    @given(
        task_id=st.integers(min_value=1, max_value=999),
        branch_format=st.sampled_from(["task-{id}-feature", "task-{id}", "task-{id}-bugfix"]),
        label_variations=st.lists(st.text(min_size=1), min_size=1, max_size=3)
    )
    def test_task_id_extraction_consistency(self, task_id, branch_format, label_variations):
        """Test that task ID extraction works for all valid formats"""
        
        # Create synthetic webhook payload
        webhook_payload = {
            "pull_request": {
                "head": {"ref": branch_format.format(id=task_id)},
                "labels": [{"name": f"task-{task_id}"}] + 
                         [{"name": label} for label in label_variations]
            }
        }
        
        extracted_id = extract_task_id_from_webhook(webhook_payload)
        assert extracted_id == str(task_id), f"Expected {task_id}, got {extracted_id}"
    
    @given(st.text().filter(lambda x: not x.startswith("task-")))
    def test_invalid_labels_rejected(self, invalid_label):
        """Ensure non-task labels don't cause false matches"""
        webhook_payload = {
            "pull_request": {
                "labels": [{"name": invalid_label}]
            }
        }
        
        extracted_id = extract_task_id_from_webhook(webhook_payload)
        assert extracted_id is None, f"Invalid label {invalid_label} should not extract task ID"
```

### 8. Test Execution Pipeline

#### Continuous Testing Configuration
```yaml
# GitHub Actions workflow for E2E testing
name: End-to-End Multi-Agent Testing
on:
  schedule:
    - cron: '0 */4 * * *'  # Every 4 hours
  pull_request:
    paths: 
      - 'infra/**'
      - '.argo/**'
      - 'controller/**'

jobs:
  e2e-happy-path:
    runs-on: ubuntu-latest
    timeout-minutes: 180  # 3 hours for complete workflow
    steps:
      - name: Setup Test Environment
        run: |
          # Deploy test cluster configuration
          kubectl apply -f tests/e2e/test-cluster-setup.yaml
          
      - name: Execute Simple Task Workflow
        run: |
          python tests/e2e/run_simple_task_test.py --task-id=1001
          
      - name: Validate Results
        run: |
          python tests/e2e/validate_workflow_completion.py --task-id=1001
  
  e2e-chaos-testing:
    runs-on: ubuntu-latest
    timeout-minutes: 240  # 4 hours for chaos scenarios
    steps:
      - name: Deploy Chaos Mesh
        run: |
          helm install chaos-mesh chaos-mesh/chaos-mesh
          
      - name: Execute Chaos Scenarios
        run: |
          python tests/e2e/run_chaos_tests.py --scenarios=pod-failures,network-partition
```

## Implementation Steps

### Phase 1: Foundation (Week 1-2)
1. **Test Infrastructure Setup**
   - Create dedicated test GitHub repository
   - Deploy test-specific Kubernetes namespace
   - Configure test GitHub Apps and webhooks
   
2. **Basic Assertion Framework**
   - Implement workflow state monitoring
   - Create simple PR generation utilities
   - Develop basic performance measurement

### Phase 2: Core Testing (Week 3-4)
3. **Workflow Validation**
   - Implement end-to-end happy path testing
   - Add stage transition assertions  
   - Create agent behavior verification

4. **Synthetic Task Generation**
   - Build complexity-based task generators
   - Implement multi-language code generation
   - Add realistic change patterns

### Phase 3: Advanced Testing (Week 5-6)
5. **Chaos Engineering**
   - Deploy Chaos Mesh for infrastructure testing
   - Implement failure scenario generators
   - Add recovery time measurement

6. **Property-Based Testing**
   - Create event correlation test generators
   - Implement webhook payload validation
   - Add edge case coverage

### Phase 4: Integration (Week 7-8)
7. **Performance Suite**
   - Implement comprehensive latency tracking
   - Add bottleneck identification tools
   - Create performance regression detection

8. **Continuous Execution**
   - Configure automated test pipeline
   - Add result reporting and alerting
   - Implement test history tracking

## Success Metrics

### Functional Validation
- **100% Happy Path Success**: All standard workflows complete successfully
- **Failure Recovery**: System recovers gracefully from 95% of simulated failures
- **Event Correlation**: 100% accuracy in webhook-to-workflow correlation
- **Agent Behavior**: Each agent performs expected actions in 100% of scenarios

### Performance Benchmarks  
- **End-to-End Latency**: < 30 minutes for simple tasks, < 4 hours for complex tasks
- **Stage Transition Time**: < 60 seconds for webhook processing and workflow resume
- **Resource Utilization**: Suspended workflows consume < 10MB memory each
- **Concurrent Processing**: Support 10+ parallel workflows without degradation

### Quality Assurance
- **Test Coverage**: 95% coverage of workflow paths and failure modes  
- **Automation Level**: 90% of testing runs without manual intervention
- **Reliability**: 99% test execution success rate over 30-day periods
- **Detection Rate**: Identify 100% of system regressions within 24 hours

## Dependencies

### Internal Dependencies
- **Task 17**: Monitoring infrastructure for performance measurement
- **Task 19**: Argo Workflows deployment for state assertions

### External Dependencies
- **GitHub API Access**: Test repository creation and webhook management
- **Kubernetes Cluster**: Test namespace with appropriate RBAC permissions
- **Chaos Engineering Tools**: Chaos Mesh or Litmus for failure injection
- **Monitoring Stack**: Prometheus/Grafana for metrics collection

## Risk Mitigation

### Test Environment Isolation
- Dedicated test namespace prevents interference with production
- Separate GitHub organization for test repositories
- Resource limits prevent test workloads from affecting other systems

### Data Management
- Automatic cleanup of test artifacts after execution
- Retention policies for test metrics and logs  
- Backup/restore procedures for test configurations

### Security Considerations
- Test GitHub Apps with minimal required permissions
- Network policies restricting test environment access
- Secure handling of test credentials and tokens

This comprehensive E2E testing suite ensures the multi-agent workflow system operates reliably across all scenarios, providing confidence in production deployment and ongoing operational stability.