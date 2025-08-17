# Task 15: Create Workflow Monitoring Dashboard

## Overview

Develop a comprehensive Grafana dashboard to monitor multi-agent workflow progress, agent performance, and system health using the existing telemetry stack (Victoria Metrics). This dashboard provides real-time visibility into the entire multi-agent orchestration system performance and operational health.

## Context

The multi-agent Play Workflow system processes tasks through complex pipelines involving Rex/Blaze (implementation), Cleo (quality), and Tess (testing) agents. Monitoring this system requires tracking:

- **Workflow Progress**: Task completion rates, stage durations, suspension wait times
- **Agent Performance**: Success rates, execution times, failure patterns
- **System Health**: Resource usage, error rates, event processing
- **Operational Metrics**: GitHub webhook processing, resume operations, task progression

## Technical Architecture

### Dashboard Structure

```yaml
# Dashboard Organization
Workflow Monitoring Dashboard:
  - Overview Section:
    - Active workflows count
    - Task completion rate (24h)
    - Current system health status
    - Alert summary
    
  - Workflow Progress Section:
    - Workflow duration by stage
    - Suspension wait times
    - Task completion trends
    - Queue depth over time
    
  - Agent Performance Section:
    - Rex vs Blaze comparison
    - Cleo code quality metrics
    - Tess testing success rates
    - Agent-specific failure analysis
    
  - System Health Section:
    - Resource utilization
    - Error rates by component
    - GitHub webhook processing
    - Resume operation metrics
    
  - Alerting Section:
    - Stuck workflows (>24h)
    - High failure rates
    - Resource exhaustion warnings
    - Event processing delays
```

### Metrics Architecture

```yaml
# Victoria Metrics Integration
Data Sources:
  - Argo Workflows Metrics:
    - workflow_duration_seconds
    - workflow_status_gauge
    - workflow_info
    
  - Agent-Specific Metrics:
    - agent_execution_duration
    - agent_success_rate
    - agent_failure_count
    
  - System Metrics:
    - github_webhook_processing_time
    - resume_operation_duration
    - task_progression_success_rate
    
  - Resource Metrics:
    - container_cpu_usage
    - container_memory_usage
    - kubernetes_pod_status
```

## Implementation Requirements

### 1. Grafana Dashboard Configuration

**Main Dashboard JSON Structure**:
```json
{
  "dashboard": {
    "id": null,
    "title": "Multi-Agent Workflow Monitoring",
    "tags": ["taskmaster", "workflows", "agents"],
    "timezone": "UTC",
    "refresh": "30s",
    "time": {
      "from": "now-24h",
      "to": "now"
    },
    "templating": {
      "list": [
        {
          "name": "task_id",
          "type": "query",
          "query": "label_values(workflow_info{workflow_type=\"play-orchestration\"}, task_id)",
          "multi": true,
          "includeAll": true
        },
        {
          "name": "agent_type",
          "type": "query",
          "query": "label_values(agent_execution_duration, github_app)",
          "multi": true,
          "includeAll": true
        }
      ]
    },
    "panels": [
      // Panel definitions below
    ]
  }
}
```

### 2. Overview Section Panels

**Active Workflows Counter**:
```json
{
  "title": "Active Workflows",
  "type": "stat",
  "targets": [
    {
      "expr": "count(workflow_status_gauge{workflow_type=\"play-orchestration\", status=\"Running\"})",
      "legendFormat": "Active"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "color": {
        "mode": "thresholds"
      },
      "thresholds": {
        "steps": [
          {"color": "green", "value": 0},
          {"color": "yellow", "value": 10},
          {"color": "red", "value": 25}
        ]
      }
    }
  }
}
```

**Task Completion Rate**:
```json
{
  "title": "Task Completion Rate (24h)",
  "type": "stat",
  "targets": [
    {
      "expr": "rate(workflow_status_gauge{workflow_type=\"play-orchestration\", status=\"Succeeded\"}[24h]) * 86400",
      "legendFormat": "Tasks/Day"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "short",
      "color": {"mode": "palette-classic"}
    }
  }
}
```

**System Health Status**:
```json
{
  "title": "System Health",
  "type": "stat",
  "targets": [
    {
      "expr": "(
        (1 - rate(workflow_status_gauge{status=\"Failed\"}[1h]) / rate(workflow_status_gauge[1h])) * 100
      )",
      "legendFormat": "Success Rate %"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "percent",
      "thresholds": {
        "steps": [
          {"color": "red", "value": 0},
          {"color": "yellow", "value": 95},
          {"color": "green", "value": 99}
        ]
      }
    }
  }
}
```

### 3. Workflow Progress Section

**Workflow Duration by Stage**:
```json
{
  "title": "Workflow Duration by Stage",
  "type": "timeseries",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket{workflow_type=\"play-orchestration\"}[5m])) by (le, stage))",
      "legendFormat": "{{stage}} (95th percentile)"
    },
    {
      "expr": "histogram_quantile(0.50, sum(rate(workflow_stage_duration_seconds_bucket{workflow_type=\"play-orchestration\"}[5m])) by (le, stage))",
      "legendFormat": "{{stage}} (median)"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "s",
      "custom": {
        "drawStyle": "line",
        "lineInterpolation": "smooth"
      }
    }
  }
}
```

**Suspension Wait Times**:
```json
{
  "title": "Suspension Wait Times",
  "type": "bargauge",
  "targets": [
    {
      "expr": "avg(workflow_suspension_duration_seconds{workflow_type=\"play-orchestration\"}) by (suspension_point)",
      "legendFormat": "{{suspension_point}}"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "s",
      "thresholds": {
        "steps": [
          {"color": "green", "value": 0},
          {"color": "yellow", "value": 3600},
          {"color": "red", "value": 86400}
        ]
      }
    }
  }
}
```

**Task Queue Depth**:
```json
{
  "title": "Task Queue Depth",
  "type": "timeseries",
  "targets": [
    {
      "expr": "count(task_status{status=\"pending\"})",
      "legendFormat": "Pending Tasks"
    },
    {
      "expr": "count(task_status{status=\"in_progress\"})",
      "legendFormat": "In Progress"
    },
    {
      "expr": "count(task_status{status=\"completed\"})",
      "legendFormat": "Completed"
    }
  ]
}
```

### 4. Agent Performance Section

**Rex vs Blaze Comparison**:
```json
{
  "title": "Implementation Agent Comparison",
  "type": "table",
  "targets": [
    {
      "expr": "avg(agent_execution_duration_seconds{github_app=~\"5DLabs-Rex|5DLabs-Blaze\"}) by (github_app)",
      "format": "table",
      "legendFormat": ""
    },
    {
      "expr": "rate(agent_success_total{github_app=~\"5DLabs-Rex|5DLabs-Blaze\"}[1h]) / rate(agent_total{github_app=~\"5DLabs-Rex|5DLabs-Blaze\"}[1h])",
      "format": "table",
      "legendFormat": ""
    }
  ],
  "transformations": [
    {
      "id": "merge",
      "options": {}
    },
    {
      "id": "organize",
      "options": {
        "excludeByName": {"Time": true},
        "renameByName": {
          "github_app": "Agent",
          "Value #A": "Avg Duration (s)",
          "Value #B": "Success Rate"
        }
      }
    }
  ]
}
```

**Cleo Code Quality Metrics**:
```json
{
  "title": "Cleo Quality Metrics",
  "type": "timeseries",
  "targets": [
    {
      "expr": "rate(cleo_clippy_warnings_fixed_total[5m])",
      "legendFormat": "Clippy Warnings Fixed/min"
    },
    {
      "expr": "rate(cleo_formatting_issues_fixed_total[5m])",
      "legendFormat": "Format Issues Fixed/min"
    },
    {
      "expr": "rate(cleo_ready_for_qa_labels_added_total[5m])",
      "legendFormat": "Ready-for-QA Labels Added/min"
    }
  ]
}
```

**Tess Testing Success Rates**:
```json
{
  "title": "Tess Testing Performance",
  "type": "timeseries",
  "targets": [
    {
      "expr": "rate(tess_code_review_approvals_total[5m])",
      "legendFormat": "Code Reviews Approved/min"
    },
    {
      "expr": "rate(tess_deployment_tests_passed_total[5m])",
      "legendFormat": "Deployment Tests Passed/min"
    },
    {
      "expr": "rate(tess_test_coverage_improvements_total[5m])",
      "legendFormat": "Test Coverage Improvements/min"
    },
    {
      "expr": "histogram_quantile(0.95, sum(rate(tess_testing_duration_seconds_bucket[5m])) by (le))",
      "legendFormat": "Testing Duration (95th %ile)"
    }
  ]
}
```

### 5. System Health Section

**Resource Utilization**:
```json
{
  "title": "Agent Resource Usage",
  "type": "timeseries",
  "targets": [
    {
      "expr": "avg(container_cpu_usage_seconds_total{pod=~\".*-rex-.*|.*-blaze-.*|.*-cleo-.*|.*-tess-.*\"}) by (pod)",
      "legendFormat": "{{pod}} CPU"
    },
    {
      "expr": "avg(container_memory_working_set_bytes{pod=~\".*-rex-.*|.*-blaze-.*|.*-cleo-.*|.*-tess-.*\"}) by (pod) / 1024 / 1024 / 1024",
      "legendFormat": "{{pod}} Memory (GB)"
    }
  ]
}
```

**GitHub Webhook Processing**:
```json
{
  "title": "GitHub Webhook Processing",
  "type": "timeseries",
  "targets": [
    {
      "expr": "rate(github_webhook_received_total[5m])",
      "legendFormat": "Webhooks Received/min"
    },
    {
      "expr": "rate(github_webhook_processed_total[5m])",
      "legendFormat": "Webhooks Processed/min"
    },
    {
      "expr": "histogram_quantile(0.95, sum(rate(github_webhook_processing_duration_seconds_bucket[5m])) by (le))",
      "legendFormat": "Processing Duration (95th %ile)"
    }
  ]
}
```

### 6. Alert Configuration

**Stuck Workflows Alert**:
```yaml
# alerts/stuck-workflows.yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: workflow-monitoring-alerts
  namespace: monitoring
spec:
  groups:
  - name: workflow.stuck
    rules:
    - alert: WorkflowStuckTooLong
      expr: |
        (
          time() - workflow_start_time_seconds{workflow_type="play-orchestration"}
        ) > 86400  # 24 hours
      for: 5m
      labels:
        severity: warning
        component: workflow
      annotations:
        summary: "Workflow {{ $labels.workflow_name }} stuck for >24h"
        description: |
          Workflow {{ $labels.workflow_name }} for task {{ $labels.task_id }} 
          has been running for more than 24 hours. Current stage: {{ $labels.current_stage }}
        runbook_url: "https://docs.taskmaster.com/runbooks/stuck-workflows"
```

**High Failure Rate Alert**:
```yaml
- alert: HighWorkflowFailureRate
  expr: |
    (
      rate(workflow_status_gauge{status="Failed"}[1h]) / 
      rate(workflow_status_gauge[1h])
    ) > 0.1  # 10% failure rate
  for: 10m
  labels:
    severity: critical
    component: workflow
  annotations:
    summary: "High workflow failure rate detected"
    description: |
      Workflow failure rate is {{ $value | humanizePercentage }} over the last hour.
      This indicates potential system issues requiring investigation.
```

**Agent Performance Degradation Alert**:
```yaml
- alert: AgentPerformanceDegradation
  expr: |
    (
      avg_over_time(agent_execution_duration_seconds[1h]) > 
      avg_over_time(agent_execution_duration_seconds[24h]) * 1.5
    )
  for: 15m
  labels:
    severity: warning
    component: agent
  annotations:
    summary: "Agent {{ $labels.github_app }} performance degradation"
    description: |
      Agent {{ $labels.github_app }} execution time has increased by >50% 
      compared to 24h average. Current: {{ $value }}s
```

## PromQL Queries for Key Metrics

### Workflow Progress Queries

```promql
# Active workflows by stage
count(workflow_status_gauge{workflow_type="play-orchestration", status="Running"}) by (current_stage)

# Average workflow duration by task
avg(workflow_duration_seconds{workflow_type="play-orchestration"}) by (task_id)

# Task completion rate (tasks per hour)
rate(workflow_status_gauge{workflow_type="play-orchestration", status="Succeeded"}[1h]) * 3600

# Suspension wait times by point
histogram_quantile(0.95, sum(rate(workflow_suspension_duration_seconds_bucket[5m])) by (le, suspension_point))

# Workflow success rate
rate(workflow_status_gauge{status="Succeeded"}[1h]) / rate(workflow_status_gauge[1h]) * 100
```

### Agent Performance Queries

```promql
# Agent execution time comparison
avg(agent_execution_duration_seconds) by (github_app)

# Agent success rates
rate(agent_success_total[1h]) / rate(agent_total[1h]) by (github_app) * 100

# Top agent failure reasons
topk(5, sum(rate(agent_failure_total[1h])) by (failure_reason))

# Agent resource usage
sum(container_cpu_usage_seconds_total{pod=~".*-rex-.*|.*-cleo-.*|.*-tess-.*"}) by (github_app)

# Rex vs Blaze performance comparison
avg(agent_execution_duration_seconds{github_app=~"5DLabs-Rex|5DLabs-Blaze"}) by (github_app)
```

### System Health Queries

```promql
# GitHub webhook processing latency
histogram_quantile(0.95, sum(rate(github_webhook_processing_duration_seconds_bucket[5m])) by (le))

# Resume operation success rate
rate(resume_successful_total[1h]) / rate(resume_total_attempts[1h]) * 100

# Event correlation accuracy
rate(event_correlation_success_total[1h]) / rate(event_correlation_attempts_total[1h]) * 100

# Task progression success rate
rate(task_progression_success_total[1h]) / rate(task_progression_attempts_total[1h]) * 100

# Circuit breaker states
sum(circuit_breaker_state_gauge) by (circuit_breaker_name, state)
```

## Implementation Steps

### Phase 1: Metrics Collection Setup

1. **Enhance Existing Metrics**:
   ```yaml
   # Add to existing Argo Workflows metrics
   - workflow_stage_duration_seconds
   - workflow_suspension_duration_seconds
   - workflow_start_time_seconds
   ```

2. **Implement Agent-Specific Metrics**:
   ```go
   // In agent containers
   var (
       agentExecutionDuration = prometheus.NewHistogramVec(
           prometheus.HistogramOpts{
               Name: "agent_execution_duration_seconds",
               Help: "Agent execution duration in seconds",
           },
           []string{"github_app", "task_id"},
       )
       
       agentSuccessTotal = prometheus.NewCounterVec(
           prometheus.CounterOpts{
               Name: "agent_success_total",
               Help: "Total successful agent executions",
           },
           []string{"github_app"},
       )
   )
   ```

3. **Add System Health Metrics**:
   ```go
   // GitHub webhook processing
   var (
       webhookProcessingDuration = prometheus.NewHistogramVec(
           prometheus.HistogramOpts{
               Name: "github_webhook_processing_duration_seconds",
               Help: "GitHub webhook processing duration",
           },
           []string{"event_type"},
       )
   )
   ```

### Phase 2: Dashboard Development

1. **Create Base Dashboard**:
   ```bash
   # Create dashboard directory
   mkdir -p infra/telemetry/telemetry-dashboards/workflow-monitoring/
   
   # Create main dashboard JSON
   cat > infra/telemetry/telemetry-dashboards/workflow-monitoring/main-dashboard.json << 'EOF'
   {
     "dashboard": {
       // Dashboard configuration
     }
   }
   EOF
   ```

2. **Implement Panel Components**:
   ```json
   // Create reusable panel templates
   {
     "panels": [
       // Overview section panels
       // Workflow progress panels
       // Agent performance panels
       // System health panels
     ]
   }
   ```

3. **Configure Variables and Templating**:
   ```json
   {
     "templating": {
       "list": [
         {
           "name": "task_filter",
           "type": "query",
           "query": "label_values(workflow_info, task_id)"
         }
       ]
     }
   }
   ```

### Phase 3: Alert Integration

1. **Deploy Alert Rules**:
   ```yaml
   # alerts/workflow-alerts.yaml
   apiVersion: monitoring.coreos.com/v1
   kind: PrometheusRule
   metadata:
     name: workflow-monitoring
     namespace: monitoring
   spec:
     groups:
     - name: workflow.health
       rules:
       # Alert rule definitions
   ```

2. **Configure Alert Manager**:
   ```yaml
   # Configure notification channels
   route:
     group_by: ['alertname', 'severity']
     routes:
     - match:
         component: workflow
       receiver: 'workflow-alerts'
   ```

### Phase 4: Testing and Validation

1. **Test Dashboard Functionality**:
   ```bash
   # Deploy dashboard to Grafana
   curl -X POST http://grafana:3000/api/dashboards/db \
     -H "Content-Type: application/json" \
     -d @main-dashboard.json
   
   # Verify panels load correctly
   # Test variable templating
   # Validate alert thresholds
   ```

2. **Load Test with Real Data**:
   ```bash
   # Generate test workflows
   for i in {1..10}; do
     argo submit play-workflow-template.yaml -p task-id=$i
   done
   
   # Monitor dashboard updates
   # Verify metrics accuracy
   # Test alert triggering
   ```

## Dashboard Features

### Interactive Elements
- **Task ID Filter**: Filter all panels by specific task IDs
- **Time Range Selector**: Adjust monitoring time window
- **Agent Type Filter**: Focus on specific agents (Rex, Blaze, Cleo, Tess)
- **Drill-Down Links**: Navigate from high-level metrics to detailed views

### Visual Components
- **Status Indicators**: Green/yellow/red status for system health
- **Trend Lines**: Historical performance trends
- **Heat Maps**: Resource usage patterns
- **Comparison Charts**: Agent performance comparisons
- **Alert Annotations**: Mark significant events on timelines

### Responsive Design
- **Mobile Compatibility**: Dashboard works on mobile devices
- **Auto-Refresh**: Configurable refresh intervals
- **Export Options**: PDF/PNG export for reports
- **Sharing Capabilities**: Dashboard links and snapshots

## Operational Use Cases

### Daily Operations
1. **Morning Health Check**: Review overnight workflow activity
2. **Performance Monitoring**: Track agent execution times and success rates
3. **Capacity Planning**: Monitor resource usage trends
4. **Issue Investigation**: Drill down into specific failures

### Incident Response
1. **Alert Triage**: Quickly identify root causes
2. **Impact Assessment**: Understand scope of issues
3. **Recovery Monitoring**: Track system recovery progress
4. **Post-Incident Analysis**: Review performance during incidents

### Planning and Optimization
1. **Performance Trends**: Identify optimization opportunities
2. **Resource Planning**: Forecast scaling needs
3. **Agent Comparison**: Evaluate agent effectiveness
4. **System Evolution**: Track improvements over time

## Dependencies

- **Task 3**: Multi-agent orchestration system foundation
- Existing Victoria Metrics telemetry stack
- Grafana installation with appropriate permissions
- Prometheus metrics collection from agents and workflows
- Alert Manager for notification routing

## Expected Outcomes

### Operational Visibility
1. **Real-time Monitoring**: Current system state always visible
2. **Historical Analysis**: Trends and patterns easily identifiable
3. **Performance Tracking**: Agent and workflow performance quantified
4. **Proactive Alerting**: Issues detected before user impact

### Improved Operations
1. **Faster Incident Response**: Reduced time to identify and resolve issues
2. **Better Capacity Planning**: Data-driven scaling decisions
3. **Performance Optimization**: Identify and address bottlenecks
4. **System Reliability**: Proactive issue prevention

### Enhanced Team Productivity
1. **Reduced Manual Monitoring**: Automated visibility into system health
2. **Clear Performance Metrics**: Objective measurement of system performance
3. **Actionable Alerts**: Focused notifications on issues requiring attention
4. **Historical Context**: Long-term trends inform strategic decisions

## Future Enhancements

- **Machine Learning Integration**: Predictive analytics for performance forecasting
- **Custom Alert Correlation**: Intelligent alert grouping and root cause analysis
- **Mobile App**: Dedicated mobile application for monitoring
- **Integration APIs**: Programmatic access to dashboard data
- **Advanced Analytics**: Statistical analysis and anomaly detection
- **Multi-Environment Support**: Compare performance across different environments