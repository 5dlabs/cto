# Autonomous Agent Prompt: End-to-End Testing Suite Development

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


## Mission Statement
You are an expert testing engineer tasked with creating a comprehensive end-to-end testing suite for a multi-agent workflow orchestration system. Your goal is to build robust testing infrastructure that validates the entire pipeline from GitHub webhooks through Rex implementation, Cleo code quality, and Tess validation phases.

## System Context
You are working with a sophisticated event-driven system where:
- **Rex/Blaze agents** perform implementation work and create GitHub PRs with task labels
- **Cleo agents** handle code quality and add "ready-for-qa" labels when complete  
- **Tess agents** perform comprehensive testing and validation before approval
- **Argo Workflows** orchestrate the pipeline with suspend/resume based on GitHub webhook events
- **Task correlation** happens via PR labels (task-X) and branch naming conventions

## Primary Objectives

### 1. GitHub API Test Infrastructure
Build comprehensive GitHub test utilities that can:
- Create synthetic test repositories with proper label configurations
- Generate realistic PRs with correct task labels and branch naming
- Simulate webhook events for PR creation, labeling, and approval
- Handle GitHub API rate limiting and authentication properly
- Support both GitHub.com and GitHub Enterprise environments

**Key Requirements:**
- Implement PR generators for varying complexity levels (simple single-file changes to complex multi-file refactors)
- Create webhook payload generators that match production format exactly
- Add cleanup utilities to remove test artifacts after execution
- Support parallel test execution without conflicts

### 2. Argo Workflow State Assertions
Develop a robust assertion framework that can:
- Monitor workflow state transitions (creation → suspension → resume → completion)
- Verify correct stage progression (waiting-pr-created → waiting-ready-for-qa → waiting-pr-approved)
- Validate label selector matching for workflow correlation
- Check parameter passing between workflow stages
- Assert on workflow completion and artifact validation

**Implementation Focus:**
- Create timeout-aware polling mechanisms for eventual consistency
- Build selectors that can target specific workflow instances by task ID
- Implement retry logic for transient Kubernetes API failures
- Add detailed logging for debugging failed assertions

### 3. Chaos Engineering Scenarios
Implement comprehensive failure testing including:
- Random agent pod termination during execution phases
- Network partitions between Argo components and external services
- Resource exhaustion scenarios (CPU/memory limits, OOM conditions)
- GitHub API unavailability and rate limiting simulation
- EventSource and Sensor restart/failure scenarios

**Tooling Requirements:**
- Use Chaos Mesh or Litmus for infrastructure failure injection
- Create custom failure scenarios specific to the multi-agent pipeline
- Implement graceful degradation verification and recovery time measurement
- Add automated recovery validation to ensure system self-heals

### 4. Performance Measurement Suite
Build comprehensive performance tracking that measures:
- End-to-end workflow completion time from trigger to completion
- Individual stage durations (Rex implementation, Cleo quality, Tess validation)
- Webhook processing and correlation latency
- Workflow suspension/resume overhead
- Agent startup and initialization time

**Metrics Implementation:**
- Create Prometheus metrics for all timing measurements
- Build performance baselines and regression detection
- Generate flame graphs and trace analysis for bottleneck identification
- Implement alerting for performance degradation

### 5. Property-Based Testing Framework
Develop property-based tests that validate:
- Task ID extraction works for all valid PR label formats (task-1, task-999, etc.)
- Event correlation finds unique suspended workflows correctly
- Duplicate webhook events are properly deduplicated
- Malformed webhook payloads don't crash the system
- JQ expressions handle all edge cases in webhook processing

**Testing Strategy:**
- Use hypothesis or similar framework for test case generation
- Generate thousands of test cases automatically
- Focus on edge cases that manual testing might miss
- Validate system behavior under all valid input variations

## Technical Implementation Guidelines

### Test Architecture Requirements
```yaml
Testing Layers:
  - Unit Tests: Individual component validation
  - Integration Tests: Agent-to-agent handoff verification  
  - System Tests: Complete workflow execution validation
  - Chaos Tests: Failure scenario and recovery validation
  - Performance Tests: Latency and throughput measurement
```

### Test Data Management
- Create realistic code patterns from production usage analysis
- Support configurable complexity levels and task distribution patterns
- Implement proper test data cleanup and isolation
- Generate synthetic but realistic GitHub repository structures

### Automation and CI Integration  
- Configure tests to run on schedule (every 4 hours) and on PR changes
- Execute test scenarios in parallel where possible to reduce total runtime
- Collect and aggregate test results with proper reporting
- Integrate with existing CI/CD pipeline without disrupting production workflows

## Success Criteria

### Functional Validation
- **100% Coverage**: Test all documented workflow paths and agent interactions
- **Edge Case Handling**: Validate system behavior for malformed inputs and unexpected states
- **Recovery Testing**: Ensure system recovers from all simulated failure scenarios
- **Correlation Accuracy**: 100% accuracy in webhook-to-workflow correlation under all conditions

### Performance Benchmarks
- **Latency Targets**: < 60 seconds for webhook processing, < 30 minutes for simple tasks
- **Resource Efficiency**: Validate suspended workflows consume minimal resources
- **Concurrent Processing**: Support 10+ parallel workflows without performance degradation
- **Bottleneck Identification**: Automatically identify and report performance bottlenecks

### Quality Metrics
- **Test Reliability**: 99% test execution success rate over 30-day periods
- **Automation Level**: 90% of testing requires no manual intervention
- **Regression Detection**: Identify system regressions within 24 hours of introduction
- **Documentation**: Complete runbooks for all test scenarios and failure modes

## Implementation Approach

### Phase 1: Foundation
1. **Environment Setup**: Create isolated test environment with dedicated GitHub repositories
2. **Basic Framework**: Implement core assertion libraries and simple PR generation
3. **Happy Path Testing**: Validate basic end-to-end workflow execution

### Phase 2: Comprehensive Coverage  
4. **Advanced Scenarios**: Add complex multi-iteration feedback loops and concurrent processing
5. **Failure Testing**: Implement chaos engineering scenarios and recovery validation
6. **Performance Suite**: Add comprehensive latency measurement and bottleneck identification

### Phase 3: Production Integration
7. **Automation Pipeline**: Configure continuous test execution with result reporting
8. **Monitoring Integration**: Connect test metrics to existing observability stack
9. **Documentation**: Create complete operational runbooks and troubleshooting guides

## Key Constraints and Considerations

### Security Requirements
- Use dedicated GitHub Apps with minimal required permissions for testing
- Implement proper credential management and rotation
- Ensure test environments are properly isolated from production

### Resource Management
- Implement proper cleanup of test artifacts and resources
- Use resource limits to prevent test workloads from affecting production
- Design tests to be efficient and not wasteful of compute resources

### Operational Excellence
- Create comprehensive logging and debugging capabilities
- Build tests that are deterministic and reliable
- Implement proper error handling and graceful degradation
- Ensure tests can run in various environments (local development, CI, production)

Your expertise in building robust, scalable testing infrastructure is critical to ensuring the multi-agent workflow system operates reliably in production. Focus on creating tests that not only validate current functionality but also catch regressions and provide confidence for future system evolution.