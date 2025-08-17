# Autonomous Agent Prompt: Create Workflow Monitoring Dashboard

## Mission

You are tasked with developing a comprehensive Grafana dashboard to monitor the multi-agent workflow orchestration system. This dashboard must provide real-time visibility into workflow progress, agent performance, and system health using Victoria Metrics as the data source.

## Context

**System Architecture**: Multi-agent Play Workflow with Rex/Blaze (implementation) → Cleo (quality) → Tess (testing) → Human approval

**Your Role**: DevOps/Monitoring engineer creating operational visibility for a complex distributed system

**Critical Need**: The multi-agent orchestration system processes tasks through multiple stages with various suspension points, agent handoffs, and potential failure modes. Without proper monitoring, operators cannot:
- Track workflow progress and identify bottlenecks
- Compare agent performance (Rex vs Blaze effectiveness)
- Detect stuck workflows or system degradation
- Understand resource utilization and capacity needs
- Respond quickly to incidents and failures

## Primary Objectives

### 1. Comprehensive Workflow Monitoring
Create visibility into workflow progress, stage durations, suspension wait times, and task completion rates.

### 2. Agent Performance Analytics
Implement detailed monitoring of agent-specific metrics including Rex vs Blaze comparison, Cleo quality metrics, and Tess testing performance.

### 3. System Health Monitoring
Track resource utilization, error rates, GitHub webhook processing, and resume operation metrics.

### 4. Proactive Alerting
Implement alerts for stuck workflows (>24h), high failure rates, and system performance degradation.

## Technical Implementation

### Phase 1: Metrics Collection and Enhancement

**Workflow Metrics Enhancement**:
```go
// Add to existing Argo Workflows metrics collection
var (
    workflowStageDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "workflow_stage_duration_seconds",
            Help: "Duration of each workflow stage in seconds",
            Buckets: []float64{60, 300, 600, 1800, 3600, 7200, 21600, 86400},
        },
        []string{"stage", "task_id", "workflow_type"},
    )
    
    workflowSuspensionDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "workflow_suspension_duration_seconds",
            Help: "Duration of workflow suspensions in seconds",
            Buckets: []float64{300, 1800, 3600, 21600, 86400, 259200, 604800},
        },
        []string{"suspension_point", "task_id"},
    )
    
    workflowStartTime = prometheus.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "workflow_start_time_seconds",
            Help: "Unix timestamp when workflow started",
        },
        []string{"workflow_name", "task_id"},
    )
)

// Record workflow stage completion
func recordStageCompletion(stage, taskId string, duration time.Duration) {
    workflowStageDuration.WithLabelValues(stage, taskId, "play-orchestration").Observe(duration.Seconds())
}

// Record workflow suspension
func recordSuspension(suspensionPoint, taskId string, duration time.Duration) {
    workflowSuspensionDuration.WithLabelValues(suspensionPoint, taskId).Observe(duration.Seconds())
}
```

**Agent-Specific Metrics**:
```go
// Agent performance metrics
var (
    agentExecutionDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "agent_execution_duration_seconds",
            Help: "Agent execution duration in seconds",
            Buckets: []float64{300, 600, 1200, 1800, 3600, 7200, 14400, 28800},
        },
        []string{"github_app", "task_id", "stage"},
    )
    
    agentSuccessTotal = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "agent_success_total",
            Help: "Total successful agent executions",
        },
        []string{"github_app", "stage"},
    )
    
    agentFailureTotal = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "agent_failure_total",
            Help: "Total failed agent executions",
        },
        []string{"github_app", "stage", "failure_reason"},
    )
    
    // Cleo-specific metrics
    cleoClippyWarningsFixed = prometheus.NewCounter(
        prometheus.CounterOpts{
            Name: "cleo_clippy_warnings_fixed_total",
            Help: "Total Clippy warnings fixed by Cleo",
        },
    )
    
    cleoReadyForQALabels = prometheus.NewCounter(
        prometheus.CounterOpts{
            Name: "cleo_ready_for_qa_labels_added_total",
            Help: "Total ready-for-qa labels added by Cleo",
        },
    )
    
    // Tess-specific metrics
    tessCodeReviewApprovals = prometheus.NewCounter(
        prometheus.CounterOpts{
            Name: "tess_code_review_approvals_total",
            Help: "Total code review approvals by Tess",
        },
    )
    
    tessTestCoverageImprovements = prometheus.NewCounter(
        prometheus.CounterOpts{
            Name: "tess_test_coverage_improvements_total",
            Help: "Total test coverage improvements by Tess",
        },
    )
)
```

### Phase 2: Dashboard Architecture and Design

**Dashboard Structure Configuration**:
```json
{
  "dashboard": {
    "id": null,
    "title": "Multi-Agent Workflow Monitoring",
    "tags": ["taskmaster", "workflows", "multi-agent", "devops"],
    "timezone": "UTC",
    "refresh": "30s",
    "schemaVersion": 39,
    "version": 1,
    "time": {
      "from": "now-24h",
      "to": "now"
    },
    "templating": {
      "list": [
        {
          "name": "datasource",
          "type": "datasource",
          "query": "prometheus",
          "current": {
            "value": "Victoria Metrics",
            "text": "Victoria Metrics"
          }
        },
        {
          "name": "task_id",
          "type": "query",
          "datasource": "${datasource}",
          "query": "label_values(workflow_info{workflow_type=\"play-orchestration\"}, task_id)",
          "regex": "",
          "multi": true,
          "includeAll": true,
          "current": {
            "value": "$__all",
            "text": "All"
          },
          "refresh": "on_dashboard_load"
        },
        {
          "name": "agent_type",
          "type": "query",
          "datasource": "${datasource}",
          "query": "label_values(agent_execution_duration_seconds, github_app)",
          "multi": true,
          "includeAll": true,
          "current": {
            "value": "$__all",
            "text": "All"
          }
        },
        {
          "name": "time_range",
          "type": "interval",
          "query": "1m,5m,15m,30m,1h,6h,12h,1d,7d",
          "current": {
            "value": "5m",
            "text": "5m"
          }
        }
      ]
    },
    "panels": [
      // Panel configurations defined below
    ]
  }
}
```

### Phase 3: Key Dashboard Panels Implementation

**Overview Section - System Status Panel**:
```json
{
  "id": 1,
  "title": "System Overview",
  "type": "row",
  "collapsed": false,
  "panels": [
    {
      "id": 2,
      "title": "Active Workflows",
      "type": "stat",
      "targets": [
        {
          "expr": "count(workflow_status_gauge{workflow_type=\"play-orchestration\", status=\"Running\", task_id=~\"$task_id\"})",
          "legendFormat": "Active Workflows",
          "refId": "A"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "thresholds"
          },
          "thresholds": {
            "steps": [
              {"color": "green", "value": null},
              {"color": "yellow", "value": 10},
              {"color": "red", "value": 25}
            ]
          },
          "unit": "short",
          "displayName": "Active"
        }
      },
      "options": {
        "colorMode": "background",
        "graphMode": "area",
        "justifyMode": "auto"
      }
    },
    {
      "id": 3,
      "title": "Task Completion Rate",
      "type": "stat",
      "targets": [
        {
          "expr": "rate(workflow_status_gauge{workflow_type=\"play-orchestration\", status=\"Succeeded\", task_id=~\"$task_id\"}[24h]) * 86400",
          "legendFormat": "Tasks/Day",
          "refId": "A"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "color": {"mode": "palette-classic"},
          "unit": "short",
          "displayName": "Completed/Day"
        }
      }
    },
    {
      "id": 4,
      "title": "System Health Score",
      "type": "stat",
      "targets": [
        {
          "expr": "(1 - (rate(workflow_status_gauge{status=\"Failed\", task_id=~\"$task_id\"}[$time_range]) / rate(workflow_status_gauge{task_id=~\"$task_id\"}[$time_range]))) * 100",
          "legendFormat": "Health %",
          "refId": "A"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "percent",
          "thresholds": {
            "steps": [
              {"color": "red", "value": null},
              {"color": "yellow", "value": 95},
              {"color": "green", "value": 99}
            ]
          }
        }
      }
    }
  ]
}
```

**Workflow Progress Panel**:
```json
{
  "id": 10,
  "title": "Workflow Stage Durations",
  "type": "timeseries",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket{workflow_type=\"play-orchestration\", task_id=~\"$task_id\"}[$time_range])) by (le, stage))",
      "legendFormat": "{{stage}} (95th %ile)",
      "refId": "A"
    },
    {
      "expr": "histogram_quantile(0.50, sum(rate(workflow_stage_duration_seconds_bucket{workflow_type=\"play-orchestration\", task_id=~\"$task_id\"}[$time_range])) by (le, stage))",
      "legendFormat": "{{stage}} (median)",
      "refId": "B"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "s",
      "custom": {
        "drawStyle": "line",
        "lineInterpolation": "smooth",
        "lineWidth": 2,
        "fillOpacity": 10
      }
    }
  },
  "options": {
    "tooltip": {
      "mode": "multi",
      "sort": "desc"
    },
    "legend": {
      "calcs": ["lastNotNull", "max"],
      "displayMode": "table",
      "placement": "bottom"
    }
  }
}
```

**Agent Performance Comparison Table**:
```json
{
  "id": 20,
  "title": "Agent Performance Comparison",
  "type": "table",
  "targets": [
    {
      "expr": "avg(agent_execution_duration_seconds{github_app=~\"$agent_type\", task_id=~\"$task_id\"}) by (github_app)",
      "format": "table",
      "instant": true,
      "legendFormat": "",
      "refId": "A"
    },
    {
      "expr": "(rate(agent_success_total{github_app=~\"$agent_type\", task_id=~\"$task_id\"}[$time_range]) / (rate(agent_success_total{github_app=~\"$agent_type\", task_id=~\"$task_id\"}[$time_range]) + rate(agent_failure_total{github_app=~\"$agent_type\", task_id=~\"$task_id\"}[$time_range]))) * 100",
      "format": "table",
      "instant": true,
      "refId": "B"
    },
    {
      "expr": "rate(agent_success_total{github_app=~\"$agent_type\", task_id=~\"$task_id\"}[$time_range]) + rate(agent_failure_total{github_app=~\"$agent_type\", task_id=~\"$task_id\"}[$time_range])",
      "format": "table",
      "instant": true,
      "refId": "C"
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
          "Value #B": "Success Rate (%)",
          "Value #C": "Execution Rate (ops/sec)"
        }
      }
    }
  ],
  "fieldConfig": {
    "overrides": [
      {
        "matcher": {"id": "byName", "options": "Success Rate (%)"},
        "properties": [
          {
            "id": "unit",
            "value": "percent"
          },
          {
            "id": "custom.cellOptions",
            "value": {
              "type": "color-background",
              "mode": "gradient"
            }
          },
          {
            "id": "thresholds",
            "value": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 90},
                {"color": "green", "value": 95}
              ]
            }
          }
        ]
      }
    ]
  }
}
```

### Phase 4: Alert Configuration

**Critical Alert Rules**:
```yaml
# alerts/workflow-monitoring.yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: workflow-monitoring-alerts
  namespace: monitoring
  labels:
    app: workflow-monitoring
spec:
  groups:
  - name: workflow.critical
    rules:
    - alert: WorkflowStuckOver24Hours
      expr: |
        (time() - workflow_start_time_seconds{workflow_type="play-orchestration"}) > 86400
      for: 5m
      labels:
        severity: warning
        component: workflow
        team: devops
      annotations:
        summary: "Workflow {{ $labels.workflow_name }} stuck for over 24 hours"
        description: |
          Workflow {{ $labels.workflow_name }} (Task {{ $labels.task_id }}) has been running 
          for {{ $value | humanizeDuration }} which exceeds the 24-hour threshold.
          Current stage: {{ $labels.current_stage }}
        runbook_url: "https://docs.taskmaster.com/runbooks/stuck-workflows"
        dashboard_url: "https://grafana.com/d/workflow-monitoring"
        
    - alert: HighWorkflowFailureRate
      expr: |
        (
          rate(workflow_status_gauge{status="Failed"}[1h]) / 
          rate(workflow_status_gauge[1h])
        ) > 0.15
      for: 10m
      labels:
        severity: critical
        component: workflow
        team: devops
      annotations:
        summary: "High workflow failure rate: {{ $value | humanizePercentage }}"
        description: |
          Workflow failure rate has been {{ $value | humanizePercentage }} for the last hour,
          which exceeds the 15% threshold. This indicates potential system issues.
          
    - alert: AgentPerformanceDegradation
      expr: |
        (
          avg_over_time(agent_execution_duration_seconds[1h]) > 
          avg_over_time(agent_execution_duration_seconds[24h]) * 1.5
        ) and (
          avg_over_time(agent_execution_duration_seconds[24h]) > 300
        )
      for: 15m
      labels:
        severity: warning
        component: agent
        team: devops
      annotations:
        summary: "Agent {{ $labels.github_app }} performance degraded by {{ $value | humanizePercentage }}"
        description: |
          Agent {{ $labels.github_app }} execution time has increased significantly:
          Current 1h average: {{ $value }}s
          24h baseline average: {{ with query "avg_over_time(agent_execution_duration_seconds{github_app=\"" }}{{ $labels.github_app }}{{ \"}[24h])" }}{{ . | first | value }}s{{ end }}
```

## Critical Success Criteria

### 1. Dashboard Functionality
- [ ] All panels load data correctly from Victoria Metrics
- [ ] Variable templating filters data appropriately
- [ ] Time range selection affects all relevant panels
- [ ] Dashboard auto-refreshes every 30 seconds
- [ ] Export functionality works for sharing and reports

### 2. Workflow Monitoring
- [ ] Active workflow count displays current running workflows
- [ ] Stage duration metrics show performance by workflow phase
- [ ] Suspension wait times tracked for each suspension point
- [ ] Task completion rate calculated accurately
- [ ] Queue depth visualization shows pending/in-progress/completed tasks

### 3. Agent Performance Tracking
- [ ] Rex vs Blaze comparison table shows execution time and success rate differences
- [ ] Cleo quality metrics track Clippy fixes and ready-for-qa labels
- [ ] Tess testing metrics show approval rates and test improvements
- [ ] Agent failure analysis categorizes and trends failure reasons

### 4. System Health Monitoring
- [ ] Resource utilization panels show CPU/memory usage for agent containers
- [ ] GitHub webhook processing latency and success rates tracked
- [ ] Resume operation metrics display retry patterns and success rates
- [ ] Circuit breaker states monitored across all components

### 5. Alert Integration
- [ ] Stuck workflow alert fires for workflows running >24 hours
- [ ] High failure rate alert triggers at 15% failure threshold
- [ ] Agent performance degradation alert detects 50% slowdowns
- [ ] Alert annotations include links to dashboard and runbooks

## Implementation Strategy

### Step 1: Metrics Enhancement
```bash
# Add metrics collection to existing components
# 1. Enhance Argo Workflows integration
cat >> infra/telemetry/workflow-metrics.yaml << 'EOF'
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-metrics-config
data:
  metrics.yaml: |
    - name: workflow_stage_duration_seconds
      help: Duration of workflow stages
      type: histogram
    - name: workflow_suspension_duration_seconds  
      help: Duration of workflow suspensions
      type: histogram
EOF

# 2. Deploy agent metrics collectors
kubectl apply -f agent-metrics-config.yaml
```

### Step 2: Dashboard Development
```bash
# Create dashboard directory structure
mkdir -p infra/telemetry/telemetry-dashboards/workflow-monitoring

# Create main dashboard JSON
cat > infra/telemetry/telemetry-dashboards/workflow-monitoring/main-dashboard.json << 'EOF'
{
  "dashboard": {
    // Complete dashboard configuration
  }
}
EOF

# Deploy to Grafana
curl -X POST http://grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  -d @main-dashboard.json
```

### Step 3: Alert Configuration
```bash
# Deploy alert rules
kubectl apply -f alerts/workflow-monitoring.yaml

# Verify alerts are loaded
kubectl get prometheusrules -n monitoring

# Test alert firing with test workflow
kubectl apply -f test/long-running-workflow.yaml
```

### Step 4: Testing and Validation
```bash
# Generate test data
for i in {1..5}; do
  argo submit play-workflow-template.yaml -p task-id=$i &
done

# Monitor dashboard updates
open http://grafana:3000/d/workflow-monitoring

# Verify metrics collection
curl http://victoria-metrics:8428/api/v1/query?query=workflow_status_gauge

# Test alert functionality
kubectl apply -f test/failure-scenario.yaml
```

## Key Files to Create/Modify

### New Dashboard Files
- `infra/telemetry/telemetry-dashboards/workflow-monitoring/main-dashboard.json`
- `infra/telemetry/telemetry-dashboards/workflow-monitoring/agent-details-dashboard.json`
- `infra/telemetry/telemetry-dashboards/workflow-monitoring/system-health-dashboard.json`

### Alert Configuration
- `alerts/workflow-monitoring.yaml`
- `alerts/agent-performance.yaml`
- `alerts/system-health.yaml`

### Metrics Enhancement
- `infra/telemetry/workflow-metrics-config.yaml`
- `infra/telemetry/agent-metrics-config.yaml`

### Documentation
- `docs/monitoring/dashboard-user-guide.md`
- `docs/runbooks/stuck-workflows.md`
- `docs/runbooks/agent-performance-issues.md`

## Testing Scenarios

### Dashboard Functionality Testing
```bash
# Test variable templating
# 1. Select specific task IDs and verify filtering
# 2. Choose individual agents and confirm data isolation
# 3. Adjust time ranges and validate historical data

# Test panel interactions
# 1. Hover over metrics for detailed tooltips
# 2. Click on legend items to toggle series
# 3. Zoom into time series for detailed views

# Test responsive design
# 1. View dashboard on different screen sizes
# 2. Verify mobile compatibility
# 3. Test export functionality (PDF/PNG)
```

### Alert Testing
```bash
# Test stuck workflow alert
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-stuck-workflow
  labels:
    task-id: "999"
spec:
  entrypoint: long-sleep
  templates:
  - name: long-sleep
    script:
      image: alpine
      command: [sh]
      source: sleep 90000  # Long enough to trigger alert
EOF

# Manually set start time to trigger 24h alert
kubectl annotate workflow test-stuck-workflow \
  'start-time'='$(date -d '25 hours ago' -u +%s)'

# Verify alert fires
curl http://alertmanager:9093/api/v1/alerts | jq '.data[] | select(.labels.alertname=="WorkflowStuckOver24Hours")'
```

## Expected Deliverables

1. **Complete Grafana Dashboard**: Multi-section dashboard with workflow, agent, and system monitoring
2. **Alert Rules Configuration**: Comprehensive alerting for workflow and system issues
3. **Metrics Enhancement**: Additional metrics collection for detailed monitoring
4. **Documentation**: User guide and operational runbooks
5. **Testing Suite**: Validation scripts and test scenarios
6. **Integration Verification**: Confirmed integration with existing Victoria Metrics stack

## Dependencies & Prerequisites

- **Task 3**: Multi-agent orchestration system operational
- **Victoria Metrics**: Telemetry stack deployed and functional
- **Grafana**: Installation with dashboard creation permissions
- **Prometheus**: Metrics collection from workflows and agents
- **Alert Manager**: Notification routing configured

## Constraints

- **Data Source**: Must use existing Victoria Metrics (no additional data sources)
- **Performance**: Dashboard must load within 5 seconds
- **Compatibility**: Support Grafana v8.0+ features
- **Resource Usage**: Minimal impact on cluster performance

## Quality Gates

Before marking complete:
- [ ] Dashboard displays all required metrics correctly
- [ ] Variable templating filters data appropriately
- [ ] Alerts fire correctly for defined thresholds
- [ ] Performance acceptable with real data volumes
- [ ] Documentation complete and accurate
- [ ] Integration with existing monitoring stack verified
- [ ] Mobile and export functionality tested
- [ ] Runbooks created for common alert scenarios

This dashboard implementation provides comprehensive visibility into the multi-agent orchestration system, enabling proactive monitoring, quick incident response, and data-driven optimization decisions.