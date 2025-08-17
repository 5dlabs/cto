# Task 22 Tool Usage Guide: Resource Management Implementation

## Tool Categories and Usage

### Kubernetes Resource Management Tools
**Primary Purpose**: Deploy, monitor, and manage Kubernetes resources for comprehensive resource management

#### kubernetes_createResource & kubernetes_updateResource
```bash
# Deploy resource quota for agent namespace
kubernetes_createResource --file=./manifests/agent-platform-quota.yaml

# Update HPA configuration with new metrics
kubernetes_updateResource --file=./manifests/coderun-controller-hpa.yaml

# Create PriorityClass for agent workloads
kubernetes_createResource --api-version=scheduling.k8s.io/v1 --kind=PriorityClass --manifest='
metadata:
  name: agent-high-priority
value: 1000
description: "High priority for critical agent workflows"'
```

**Best Practices**:
- Always validate YAML manifests before creating resources
- Use labels consistently for resource organization and cleanup
- Test resource changes in non-production environment first
- Include resource version in updates to prevent conflicts

#### kubernetes_listResources & kubernetes_describeResource
```bash
# Monitor resource quota utilization
kubernetes_listResources --api-version=v1 --kind=ResourceQuota --namespace=agent-platform

# Check HPA status and scaling decisions
kubernetes_describeResource --api-version=autoscaling/v2 --kind=HorizontalPodAutoscaler --name=coderun-controller-hpa

# Review VPA recommendations
kubernetes_describeResource --api-version=autoscaling.k8s.io/v1 --kind=VerticalPodAutoscaler --name=agent-pods-vpa

# Examine PodDisruptionBudget policies
kubernetes_listResources --api-version=policy/v1 --kind=PodDisruptionBudget --all-namespaces=true
```

**Monitoring Patterns**:
- Regular quota utilization checks to prevent exhaustion
- HPA metric evaluation to validate scaling triggers
- VPA recommendation analysis for right-sizing opportunities
- PDB status verification during maintenance windows

#### kubernetes_listPods & kubernetes_getPodLogs
```bash
# Monitor agent pod resource consumption
kubernetes_listPods --namespace=agent-platform --show-metrics=true

# Check for OOM-killed pods
kubernetes_listPods --field-selector=status.phase=Failed --all-namespaces=true

# Analyze resource pressure in agent pods
kubernetes_getPodLogs --pod-name=coderun-rex-abc123 --namespace=agent-platform --grep="memory\|CPU\|resource"

# Monitor VPA-triggered pod restarts
kubernetes_getPodLogs --pod-name=vpa-recommender --namespace=kube-system --follow=true
```

**Resource Analysis Focus**:
- Identify pods approaching resource limits
- Monitor for resource-related failures and restarts
- Track VPA and HPA scaling events and decisions
- Analyze performance patterns during resource changes

#### kubernetes_deleteResource
```bash
# Clean up test resources after validation
kubernetes_deleteResource --api-version=v1 --kind=ResourceQuota --name=test-quota

# Remove problematic HPA configurations
kubernetes_deleteResource --api-version=autoscaling/v2 --kind=HorizontalPodAutoscaler --name=problematic-hpa

# Emergency cleanup of resource-intensive workloads
kubernetes_deleteResource --api-version=v1 --kind=Pod --label-selector="test-run=resource-stress"
```

**Safety Guidelines**:
- Always verify resource deletion impact before executing
- Use dry-run mode for testing deletion commands
- Maintain backups of critical resource configurations
- Document deletion rationale for audit purposes

### Research and Documentation Tools

#### brave-search_brave_web_search
```bash
# Research Kubernetes resource management best practices
brave-search_brave_web_search --query="Kubernetes VPA HPA integration best practices 2024"

# Find solutions for specific resource management challenges  
brave-search_brave_web_search --query="Kubernetes resource quota troubleshooting OOMKilled prevention"

# Research autoscaling patterns for AI workloads
brave-search_brave_web_search --query="machine learning workload Kubernetes autoscaling patterns"
```

**Research Focus Areas**:
- Latest Kubernetes resource management features and capabilities
- Best practices for AI/ML workload resource management
- Troubleshooting guides for common resource management issues
- Performance optimization techniques for containerized workloads

### Memory and Context Management

#### memory_create_entities & memory_query_entities
```bash
# Store resource configuration baselines
memory_create_entities --entities='[{"type":"resource-baseline","name":"agent-cpu-memory-baseline","properties":{"rex_cpu":"2000m","rex_memory":"8Gi","cleo_cpu":"1000m","cleo_memory":"4Gi"}}]'

# Track resource optimization results
memory_create_entities --entities='[{"type":"optimization-result","name":"vpa-rightsizing-week1","properties":{"cpu_reduction":"15%","memory_efficiency":"12%","cost_savings":"$240/month"}}]'

# Query historical resource patterns
memory_query_entities --query="resource optimization results from past 30 days"

# Retrieve troubleshooting knowledge
memory_query_entities --query="solutions for resource quota violations and OOM issues"
```

**Knowledge Management**:
- Maintain repository of proven resource configurations
- Track performance benchmarks and optimization results
- Store troubleshooting procedures and solution patterns
- Build knowledge base of resource management best practices

## Local Server Integration

### Prometheus Metrics Server
**Purpose**: Collect and analyze resource utilization metrics for optimization decisions

```python
# Example usage patterns for Prometheus integration
import prometheus_client

# Collect agent resource utilization data
metrics = prometheus_metrics.query_range(
    query='rate(container_cpu_usage_seconds_total{namespace="agent-platform"}[5m])',
    start_time='2024-01-01T00:00:00Z',
    end_time='2024-01-01T23:59:59Z',
    step='1m'
)

# Analyze resource quota utilization
quota_usage = prometheus_metrics.query(
    query='kube_resourcequota{namespace="agent-platform",type="used"} / kube_resourcequota{namespace="agent-platform",type="hard"}'
)

# Monitor HPA scaling decisions
hpa_events = prometheus_metrics.query_range(
    query='kube_hpa_status_current_replicas{namespace="agent-platform"}',
    start_time='-24h',
    step='5m'
)
```

**Metric Analysis Focus**:
- CPU and memory utilization trends across agent types
- Resource quota consumption patterns and growth rates
- Autoscaling effectiveness and response times
- Cost optimization opportunities based on usage data

### Resource Analyzer Tool
**Purpose**: Automated analysis of resource configurations and usage patterns

```python
# Resource configuration analysis
analyzer = ResourceAnalyzer(kubeconfig_path='/path/to/kubeconfig')

# Analyze current resource allocations
allocation_report = analyzer.analyze_resource_allocations(
    namespace='agent-platform',
    include_recommendations=True
)

# Identify optimization opportunities
optimization_recommendations = analyzer.identify_optimizations(
    threshold_cpu_waste=20,  # Flag if CPU usage < 80% of allocation
    threshold_memory_waste=15,  # Flag if memory usage < 85% of allocation
    analysis_period_days=30
)

# Generate capacity planning report
capacity_report = analyzer.generate_capacity_plan(
    growth_rate=0.2,  # 20% monthly growth
    planning_horizon_months=6
)
```

**Analysis Capabilities**:
- Right-sizing recommendations based on actual usage
- Waste identification and cost optimization opportunities
- Capacity planning with growth projections
- Resource efficiency scoring and benchmarking

### Autoscaler Manager
**Purpose**: Configure and manage HPA and VPA autoscaling policies

```python
# Autoscaler configuration management
autoscaler = AutoscalerManager(kubeconfig_path='/path/to/kubeconfig')

# Configure HPA for controller deployment
hpa_config = {
    'target': 'coderun-controller',
    'min_replicas': 2,
    'max_replicas': 10,
    'cpu_threshold': 70,
    'memory_threshold': 80,
    'custom_metrics': [
        {'name': 'workflow_queue_depth', 'threshold': 10}
    ]
}

autoscaler.configure_hpa(namespace='agent-platform', config=hpa_config)

# Set up VPA for agent pods
vpa_config = {
    'target_selector': {'app': 'coderun'},
    'update_mode': 'Auto',
    'resource_policy': {
        'cpu_min': '100m',
        'cpu_max': '8000m', 
        'memory_min': '128Mi',
        'memory_max': '32Gi'
    }
}

autoscaler.configure_vpa(namespace='agent-platform', config=vpa_config)
```

**Autoscaler Management**:
- Dynamic HPA configuration based on workload patterns
- VPA policy optimization for different agent types
- Custom metrics integration for workflow-aware scaling
- Scaling event monitoring and policy effectiveness analysis

## Tool Combination Strategies

### Resource Baseline Establishment
```bash
# 1. Analyze current resource utilization
kubernetes_listPods --namespace=agent-platform --show-metrics=true
prometheus_metrics --query="container_cpu_usage_seconds_total" --range=7d

# 2. Create resource specifications based on analysis
kubernetes_createResource --file=agent-resource-specs.yaml

# 3. Store baseline configurations for future reference
memory_create_entities --type=resource-baseline --data="[current_configs]"

# 4. Monitor baseline performance
resource_analyzer --analyze-allocations --baseline-comparison=true
```

### Autoscaling Implementation and Tuning
```bash
# 1. Deploy initial HPA configurations
kubernetes_createResource --file=hpa-configs.yaml

# 2. Monitor scaling behavior and effectiveness
kubernetes_describeResource --kind=HorizontalPodAutoscaler --name=coderun-controller-hpa
prometheus_metrics --query="kube_hpa_status_current_replicas" --range=24h

# 3. Analyze scaling patterns and adjust policies
autoscaler_manager --tune-hpa --target=coderun-controller --analysis-period=7d

# 4. Implement VPA for right-sizing optimization
autoscaler_manager --configure-vpa --update-mode=Auto --namespace=agent-platform

# 5. Monitor and validate VPA recommendations
kubernetes_describeResource --kind=VerticalPodAutoscaler --name=agent-pods-vpa
```

### Resource Quota and Limit Management
```bash
# 1. Establish namespace resource quotas
kubernetes_createResource --file=namespace-quotas.yaml

# 2. Monitor quota utilization
kubernetes_listResources --kind=ResourceQuota --namespace=agent-platform
prometheus_metrics --query="kube_resourcequota" --namespace=agent-platform

# 3. Analyze quota effectiveness and adjust as needed
resource_analyzer --quota-analysis --namespace=agent-platform

# 4. Implement LimitRanges for default constraints
kubernetes_createResource --file=limit-ranges.yaml

# 5. Validate resource governance effectiveness
kubernetes_listPods --field-selector=status.phase=Failed --grep="quota\|limit"
```

### Performance Optimization Workflow
```bash
# 1. Collect comprehensive resource usage data
prometheus_metrics --export-metrics --namespace=agent-platform --period=30d

# 2. Analyze optimization opportunities
resource_analyzer --optimization-report --include-cost-analysis=true

# 3. Implement recommended optimizations
kubernetes_updateResource --file=optimized-resource-specs.yaml

# 4. Monitor optimization impact
prometheus_metrics --compare-periods --before=30d --after=current

# 5. Document results and update baselines
memory_create_entities --type=optimization-result --improvements="[performance_gains]"
```

## Best Practices Summary

### Resource Configuration Management
- Use infrastructure-as-code for all resource management configurations
- Implement staged rollouts for resource policy changes
- Maintain comprehensive documentation of resource allocation decisions
- Regular review and optimization of resource specifications

### Monitoring and Alerting
- Set up proactive alerts before reaching critical resource thresholds
- Monitor both utilization and efficiency metrics
- Implement capacity planning with automated forecasting
- Track resource optimization improvements over time

### Autoscaling Best Practices
- Start with conservative scaling policies and tune based on observed behavior
- Implement appropriate stabilization windows to prevent thrashing
- Use custom metrics relevant to workload patterns
- Monitor scaling decisions and adjust policies based on effectiveness

### Operational Excellence
- Create runbooks for common resource management scenarios
- Implement automated capacity planning and reporting
- Regular testing of resource management policies under load
- Maintain cost visibility and optimization initiatives

This comprehensive tool guide enables effective implementation and management of the resource management system, ensuring optimal performance, cost efficiency, and operational excellence for the multi-agent workflow orchestration platform.