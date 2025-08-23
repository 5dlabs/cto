# Task 21 Tool Usage Guide: End-to-End Testing Suite

## Tool Categories and Usage

### Kubernetes Management Tools
**Primary Purpose**: Monitor and manipulate Kubernetes resources for workflow testing

#### kubernetes_listResources



```bash


# List all workflows with specific labels
kubernetes_listResources --api-version=argoproj.io/v1alpha1 --kind=Workflow --label-selector="workflow-type=play-orchestration,task-id=21"

# List CodeRun CRDs for specific agents
kubernetes_listResources --api-version=agents.platform/v1 --kind=CodeRun --label-selector="github-app=5DLabs-Rex"






```

**Best Practices**:


- Use label selectors to filter resources efficiently


- Always include namespace when working across multiple environments


- Combine with grep for specific field filtering

#### kubernetes_describeResource



```bash
# Get detailed workflow status and events
kubernetes_describeResource --api-version=argoproj.io/v1alpha1 --kind=Workflow --name=play-workflow-abc123 --namespace=argo

# Check agent pod status and recent events
kubernetes_describeResource --kind=Pod --name=coderun-rex-xyz789 --namespace=agent-platform






```

**Usage Patterns**:


- Check workflow suspension status and stage transitions


- Investigate agent pod failures and resource constraints


- Analyze event history for debugging timing issues

#### kubernetes_createResource



```bash
# Create chaos testing resources
kubernetes_createResource --file=./test-configs/pod-chaos.yaml

# Deploy test workflow instances
kubernetes_createResource --file=./test-workflows/e2e-test-workflow.yaml






```

**Safety Guidelines**:


- Always use test namespaces (never production)


- Include resource limits in test configurations


- Use generateName for test resources to avoid conflicts

#### kubernetes_deleteResource



```bash
# Clean up test workflows after completion
kubernetes_deleteResource --api-version=argoproj.io/v1alpha1 --kind=Workflow --label-selector="test-run-id=abc123"

# Remove stuck agent pods during testing
kubernetes_deleteResource --kind=Pod --label-selector="github-app=5DLabs-Rex,test-scenario=chaos"






```

**Cleanup Best Practices**:


- Always clean up test resources after scenarios complete


- Use label selectors for bulk cleanup operations


- Verify deletions complete before starting new tests

#### kubernetes_listPods & kubernetes_getPodLogs



```bash
# Monitor agent pod status during testing
kubernetes_listPods --namespace=agent-platform --label-selector="github-app=5DLabs-Tess"

# Collect logs for performance analysis
kubernetes_getPodLogs --pod-name=coderun-tess-xyz789 --namespace=agent-platform --follow=true






```

**Log Analysis Tips**:


- Stream logs during active testing for real-time monitoring


- Collect logs from all agents involved in failed test scenarios


- Use timestamps to correlate events across different pods

### Search and Research Tools



#### brave-search_brave_web_search



```bash
# Research testing frameworks and best practices
brave-search_brave_web_search --query="Argo Workflows end-to-end testing patterns"

# Find solutions for specific testing challenges
brave-search_brave_web_search --query="Kubernetes chaos engineering tools comparison 2024"






```

**Research Focus Areas**:


- Testing framework documentation and examples


- Chaos engineering implementation patterns


- Performance monitoring and observability best practices


- GitHub API testing utilities and libraries

### Memory and Context Management

#### memory_create_entities & memory_query_entities



```bash
# Store test execution results and patterns
memory_create_entities --entities='[{"type":"test-result","name":"workflow-latency-baseline","properties":{"avg_duration":1800,"success_rate":0.95}}]'

# Query historical test data for trend analysis
memory_query_entities --query="test results for workflow latency over past month"






```

**Memory Organization**:


- Store performance baselines and regression thresholds


- Track test scenario success patterns and failure modes


- Maintain repository of useful test configurations and snippets

## Local Server Integration

### GitHub Testing Server
**Purpose**: Automated GitHub repository and PR management for testing




```javascript
// Example usage patterns
const testRepo = await github.createTestRepository({
  name: 'test-taskmaster-e2e-21',
  labels: [
    { name: 'task-21', color: 'ff0000' },
    { name: 'ready-for-qa', color: '00ff00' }
  ]
});

const testPR = await github.createSyntheticPR({
  repository: testRepo.name,
  taskId: 21,
  complexity: 'complex',
  files: generateRustTestFiles()
});






```

**Configuration Requirements**:


- GITHUB_TOKEN with repo creation and management permissions


- GITHUB_ORG pointing to dedicated test organization


- Proper cleanup procedures to avoid resource accumulation

### Chaos Mesh Integration
**Purpose**: Infrastructure failure injection and resilience testing




```python
# Example chaos scenario execution
chaos_config = {
    "pod_kill": {
        "namespace": "agent-platform",
        "label_selector": "github-app=5DLabs-Rex",
        "duration": "60s"
    },
    "network_partition": {
        "targets": ["api.github.com"],
        "duration": "30s"
    }
}

await chaos_mesh.execute_scenario(chaos_config)






```

**Safety Protocols**:


- Only target test namespaces and resources


- Set appropriate duration limits for chaos scenarios


- Implement automatic cleanup and recovery verification


- Monitor system health throughout chaos testing

### Performance Monitor
**Purpose**: Collect and analyze performance metrics across test runs




```python
# Performance data collection patterns
metrics = performance_monitor.collect_workflow_metrics(
    workflow_name="play-workflow-test-21",
    start_time=test_start_timestamp,
    end_time=test_complete_timestamp
)

baseline = performance_monitor.get_baseline_metrics(
    task_complexity="complex",
    agent_combination=["rex", "cleo", "tess"]
)






```

**Metrics Focus Areas**:


- End-to-end workflow completion times


- Individual agent execution durations


- Webhook processing and correlation latency


- Resource utilization patterns during testing

## Tool Combination Strategies

### End-to-End Test Orchestration



```bash
# 1. Setup test environment
kubernetes_createResource --file=test-namespace.yaml
kubernetes_createResource --file=test-rbac.yaml



# 2. Deploy test workflows
kubernetes_createResource --file=e2e-test-workflow.yaml

# 3. Monitor execution
kubernetes_listPods --watch=true --label-selector="test-run-id=e2e-21"

# 4. Analyze results
kubernetes_getPodLogs --pod-name=test-coordinator --follow=false
memory_create_entities --entities="[test_results]"

# 5. Cleanup
kubernetes_deleteResource --label-selector="test-run-id=e2e-21"






```

### Chaos Testing Workflow



```bash
# 1. Establish baseline
performance-monitor --baseline --duration=300s

# 2. Inject failures
chaos-mesh --scenario=pod-kill --target-agent=rex

# 3. Monitor recovery
kubernetes_listPods --watch=true --label-selector="github-app=5DLabs-Rex"
kubernetes_getPodLogs --pod-name=new-rex-pod --follow=true



# 4. Measure impact
performance-monitor --compare-baseline --scenario=pod-kill

# 5. Document results
memory_create_entities --type=chaos-result --scenario=rex-pod-kill






```

### Performance Regression Testing



```bash
# 1. Execute performance test suite
kubernetes_createResource --file=performance-test-workflows.yaml

# 2. Collect comprehensive metrics
performance-monitor --full-metrics --duration=7200s

# 3. Compare against historical data
memory_query_entities --query="performance baselines for task complexity complex"

# 4. Generate regression report
performance-monitor --regression-analysis --threshold=10%

# 5. Update baselines if needed
memory_create_entities --update-baseline --metrics="[current_results]"






```



## Best Practices Summary

### Resource Management


- Always use test-specific namespaces and labels


- Implement comprehensive cleanup procedures


- Set appropriate resource limits on test workloads


- Monitor resource usage to prevent cluster impact



### Test Reliability


- Use deterministic test data and configurations


- Implement proper timeout and retry logic


- Validate preconditions before starting test scenarios


- Collect comprehensive logs and metrics for debugging

### Security and Safety


- Restrict test permissions to minimum required scope


- Use dedicated test credentials and GitHub organizations


- Implement safeguards against accidental production impact


- Regular rotation of test credentials and cleanup of test data

### Operational Excellence


- Document all test scenarios and expected outcomes


- Maintain runbooks for common test failure modes


- Implement automated result reporting and alerting


- Regular review and update of test configurations and baselines

This tool guide provides the foundation for building and maintaining a comprehensive end-to-end testing suite that validates the multi-agent workflow system across all operational scenarios and failure modes.
