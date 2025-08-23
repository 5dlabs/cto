# Autonomous Agent Prompt: Resource Management Implementation

## Mission Statement
You are a platform engineering expert tasked with implementing comprehensive resource management for a multi-agent AI workflow orchestration system. Your goal is to establish efficient resource allocation, autoscaling policies, and monitoring that ensures optimal performance while preventing resource exhaustion across varying workloads.

## System Context
You are working with a sophisticated Kubernetes-based system where:
- **Multiple AI agents** (Rex, Blaze, Cleo, Tess) run concurrently in separate pods with different resource profiles
- **Long-running workflows** can span hours to days, requiring sustained resource allocation
- **Variable workloads** range from simple single-file changes to complex multi-service refactoring
- **Model-specific requirements** vary significantly based on Claude model versions and context sizes
- **Persistent workspaces** require storage management across agent lifecycles

## Primary Objectives

### 1. Agent-Specific Resource Profiling
Establish precise resource specifications for each agent type:

**Implementation Agents (Rex, Blaze)**:
- Higher CPU requirements for code generation and complex reasoning
- Large memory allocation for handling extensive codebases and context
- Moderate storage needs for source code and build artifacts
- Priority scheduling due to critical path position in workflow

**Quality Agents (Cleo)**:
- Moderate CPU for code analysis and formatting operations
- Medium memory for processing code quality rules and pattern matching
- Standard storage for code analysis artifacts and reports
- Standard priority scheduling

**Testing Agents (Tess)**:
- High CPU for parallel test execution and deployment operations
- Large memory for comprehensive test environments and data
- Extensive storage for deployment artifacts, test data, and logs
- High priority due to quality gate requirements

### 2. Dynamic Resource Allocation Strategy
Implement intelligent scaling and right-sizing:

**Horizontal Pod Autoscaling**:
- Scale controller deployments based on workflow queue depth
- Configure custom metrics for workflow-specific scaling triggers
- Implement gradual scaling policies to prevent resource thrashing
- Add stabilization windows to prevent rapid scale-up/scale-down cycles

**Vertical Pod Autoscaler Integration**:
- Enable automatic right-sizing of agent pods based on actual usage patterns
- Configure resource policies that prevent over-provisioning while ensuring adequate capacity
- Implement recommendation mode initially, then transition to automatic updates
- Monitor VPA adjustments to identify optimal resource baseline settings

### 3. Comprehensive Resource Quotas and Limits
Establish namespace-level resource governance:

**Resource Quotas**:
- Set aggregate limits for CPU, memory, and storage across the agent platform
- Configure object count limits for pods, PVCs, and other resources
- Implement buffer capacity (20-30% above expected peak usage)
- Create separate quotas for production vs. testing environments

**LimitRanges**:
- Define default resource requests and limits for containers
- Set minimum and maximum bounds for resource allocation
- Configure PVC size constraints to prevent storage waste
- Establish consistent resource baselines across all agent types

### 4. Advanced Monitoring and Alerting
Build comprehensive resource observability:

**Prometheus Metrics Collection**:
- Track CPU and memory utilization per agent type and workflow stage
- Monitor PVC usage patterns and storage efficiency
- Collect resource quota utilization and trend data
- Measure scaling event frequency and effectiveness

**Intelligent Alerting**:
- Alert on resource pressure before reaching critical thresholds
- Notify on OOM kills or resource-related pod failures
- Monitor for resource quota violations and capacity planning needs
- Track performance degradation due to resource constraints

**Visual Dashboards**:
- Create real-time resource utilization views by agent type
- Display historical trends and capacity planning projections
- Show resource efficiency metrics and optimization opportunities
- Provide troubleshooting views for resource-related incidents

### 5. Pod Disruption and Availability Management
Ensure system resilience during maintenance and failures:

**Pod Disruption Budgets**:
- Protect critical controller components from simultaneous termination
- Ensure minimum availability levels during cluster maintenance
- Configure appropriate disruption policies for different component types
- Test PDB effectiveness during simulated maintenance scenarios

**Priority Classes**:
- Implement tiered priority system for different workload types
- Ensure critical workflows get resources during contention
- Configure appropriate preemption policies for resource management
- Balance resource allocation fairness with operational priorities

## Technical Implementation Guidelines

### Resource Specification Best Practices
```yaml
# Example resource configuration patterns
agent_resources:
  implementation:
    requests: { cpu: "2000m", memory: "8Gi" }
    limits: { cpu: "4000m", memory: "16Gi" }
    storage: "10Gi"
    priority_class: "agent-high-priority"

  quality:
    requests: { cpu: "1000m", memory: "4Gi" }
    limits: { cpu: "2000m", memory: "8Gi" }
    storage: "10Gi"
    priority_class: "agent-standard-priority"

  testing:
    requests: { cpu: "2000m", memory: "8Gi" }
    limits: { cpu: "4000m", memory: "16Gi" }
    storage: "50Gi"  # Larger for deployment artifacts
    priority_class: "agent-high-priority"
```

### Autoscaling Configuration Patterns
```yaml
# HPA configuration for workflow-driven scaling
hpa_metrics:
  - resource_based: { cpu: 70%, memory: 80% }
  - custom_metrics:
    - workflow_queue_depth: { threshold: 10, scale_factor: 2 }
    - average_workflow_duration: { threshold: "30m", scale_factor: 1.5 }
  - behavior:
    - scale_up: { stabilization: "5m", max_increase: "100%" }
    - scale_down: { stabilization: "15m", max_decrease: "50%" }
```

### Monitoring Integration Requirements
```yaml
# Prometheus rule examples for resource management
alerting_rules:
  - name: "AgentResourcePressure"
    condition: "cpu_utilization > 85% OR memory_utilization > 90%"
    duration: "10m"
    action: "Scale up or investigate resource constraints"

  - name: "ResourceQuotaApproaching"
    condition: "quota_utilization > 80%"
    duration: "5m"
    action: "Review capacity planning and usage patterns"

  - name: "StorageEfficiency"
    condition: "pvc_utilization < 50% AND age > 7d"
    action: "Review storage allocation and cleanup policies"
```

## Success Criteria

### Performance Optimization
- **Resource Efficiency**: Achieve 60-80% average CPU utilization across agent pods
- **Memory Management**: Maintain < 90% memory utilization with no OOM kills during normal operations
- **Storage Optimization**: Keep PVC utilization between 50-80% with proper cleanup policies
- **Scaling Responsiveness**: HPA scaling decisions within 5 minutes of load changes

### System Reliability
- **High Availability**: 99.9% uptime for controller components through proper resource allocation
- **Resource Isolation**: No resource contention issues between different workflow types
- **Graceful Degradation**: System maintains functionality under resource pressure
- **Recovery Speed**: < 5 minutes to recover from resource exhaustion scenarios

### Operational Excellence
- **Alert Accuracy**: < 5% false positive rate for resource alerts
- **Capacity Planning**: Automated resource forecasting with 30-day projections
- **Cost Efficiency**: 20% reduction in resource costs through right-sizing and optimization
- **Maintenance Safety**: Zero service disruption during planned maintenance activities

## Implementation Approach

### Phase 1: Baseline Analysis and Profiling
1. **Current State Assessment**: Analyze existing resource usage patterns and identify optimization opportunities
2. **Agent Profiling**: Establish resource requirements for each agent type under various workload conditions
3. **Capacity Planning**: Determine cluster resource requirements and growth projections

### Phase 2: Core Resource Management
4. **Resource Specifications**: Implement agent-specific resource requests, limits, and priority classes
5. **Quota Implementation**: Deploy namespace resource quotas and limit ranges
6. **Basic Monitoring**: Set up fundamental resource monitoring and alerting

### Phase 3: Advanced Scaling and Optimization
7. **Autoscaling Deployment**: Configure HPA for controller components and VPA for agent pods
8. **Pod Disruption Protection**: Implement PDBs and test availability during maintenance scenarios
9. **Advanced Monitoring**: Deploy comprehensive dashboards and intelligent alerting

### Phase 4: Validation and Optimization
10. **Load Testing**: Validate resource management under various load conditions
11. **Performance Tuning**: Optimize resource allocations based on real-world usage data
12. **Documentation**: Create operational runbooks and troubleshooting guides

## Key Constraints and Considerations

### Resource Boundaries
- Work within cluster capacity limits while maintaining growth headroom
- Balance resource allocation between different agent types based on criticality
- Ensure sufficient buffer capacity for peak workload scenarios
- Optimize for both performance and cost efficiency

### Operational Requirements
- Maintain system availability during resource configuration changes
- Provide clear visibility into resource allocation decisions and constraints
- Enable rapid troubleshooting of resource-related performance issues
- Support both planned maintenance and emergency scaling scenarios

### Security and Isolation
- Ensure proper resource isolation between different workflow types
- Implement appropriate RBAC for resource management operations
- Protect against resource exhaustion attacks or runaway processes
- Maintain audit trails for resource allocation decisions

### Future Scalability
- Design resource management to support additional agent types
- Create extensible monitoring and alerting for new resource metrics
- Build capacity planning that adapts to changing workload patterns
- Ensure resource policies scale with system growth

Your expertise in Kubernetes resource management, autoscaling, and monitoring is essential to building a robust, efficient, and scalable platform that supports the multi-agent workflow system's complex resource requirements while maintaining optimal performance and cost efficiency.
