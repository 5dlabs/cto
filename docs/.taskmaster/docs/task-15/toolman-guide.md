# Toolman Guide: Create Workflow Monitoring Dashboard

## Overview

This guide provides comprehensive instructions for creating a Grafana dashboard to monitor the multi-agent workflow orchestration system. The dashboard provides real-time visibility into workflow progress, agent performance, and system health using Victoria Metrics as the data source.

## Tool Recommendations

### Primary Tools for Dashboard Development

#### 1. Research and Design
- **brave-search_brave_web_search**: Research Grafana dashboard design patterns, PromQL query optimization, and monitoring best practices
- **memory_create_entities**: Store dashboard configuration patterns, successful PromQL queries, and design decisions
- **memory_query_entities**: Retrieve stored monitoring knowledge and troubleshooting solutions

#### 2. Implementation and Testing
- **mcp-prometheus**: Query Victoria Metrics directly for testing PromQL queries and validating data
- **mcp-grafana**: Interact with Grafana API for dashboard creation, management, and testing

### Tool Usage Patterns

#### Phase 1: Research and Architecture

```bash
# Use brave-search_brave_web_search for research
Search: "grafana dashboard design best practices multi-agent monitoring"
Search: "prometheus promql queries workflow monitoring"
Search: "victoria metrics grafana integration performance"
Search: "grafana alerting rules workflow automation"
Search: "dashboard usability design principles operations teams"
```

#### Phase 2: Dashboard Development

```bash
# Use memory_create_entities to store patterns
Store: "Dashboard panel configurations for workflow monitoring"
Store: "PromQL queries for agent performance metrics"
Store: "Alert rule templates for workflow issues"
Store: "Color schemes and visualization patterns for operations dashboards"

# Use memory_query_entities to retrieve knowledge
Query: "workflow monitoring dashboard panels"
Query: "agent performance comparison queries"
Query: "alerting rules for stuck workflows"
```

#### Phase 3: Testing and Validation

```bash
# Use mcp-prometheus for query testing
Query: "count(workflow_status_gauge{workflow_type='play-orchestration'})"
Query: "rate(agent_success_total[5m]) by (github_app)"
Query: "histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket[5m])) by (le, stage))"

# Use mcp-grafana for dashboard management
Create dashboard: workflow-monitoring
Add panels: workflow-progress, agent-performance, system-health
Test variables: task_id, agent_type, time_range
```

## Best Practices

### 1. Dashboard Architecture Patterns

**Hierarchical Dashboard Structure**:
```json
{
  "dashboard": {
    "title": "Multi-Agent Workflow Monitoring",
    "panels": [
      {
        "title": "System Overview",
        "type": "row",
        "collapsed": false,
        "panels": [
          "active-workflows-stat",
          "completion-rate-stat", 
          "system-health-stat"
        ]
      },
      {
        "title": "Workflow Progress",
        "type": "row", 
        "panels": [
          "stage-duration-timeseries",
          "suspension-wait-bargauge",
          "task-queue-depth-timeseries"
        ]
      },
      {
        "title": "Agent Performance",
        "type": "row",
        "panels": [
          "agent-comparison-table",
          "cleo-quality-metrics",
          "tess-testing-performance"
        ]
      },
      {
        "title": "System Health",
        "type": "row",
        "panels": [
          "resource-utilization",
          "webhook-processing",
          "resume-operations"
        ]
      }
    ]
  }
}
```

### 2. Effective PromQL Query Patterns

**Workflow Progress Queries**:
```promql
# Active workflows by stage
count(workflow_status_gauge{workflow_type="play-orchestration", status="Running"}) by (current_stage)

# Stage duration percentiles
histogram_quantile(0.95, 
  sum(rate(workflow_stage_duration_seconds_bucket{workflow_type="play-orchestration"}[5m])) 
  by (le, stage)
)

# Task completion rate with smoothing
rate(workflow_status_gauge{workflow_type="play-orchestration", status="Succeeded"}[1h]) * 3600

# Suspension wait times by point
avg(workflow_suspension_duration_seconds{workflow_type="play-orchestration"}) by (suspension_point)

# Queue depth over time
count(task_status{status="pending"}) or vector(0)
```

**Agent Performance Queries**:
```promql
# Agent execution time comparison
avg(agent_execution_duration_seconds{github_app=~"$agent_type"}) by (github_app)

# Success rate calculation with error handling
(
  rate(agent_success_total{github_app=~"$agent_type"}[5m]) / 
  (rate(agent_success_total{github_app=~"$agent_type"}[5m]) + 
   rate(agent_failure_total{github_app=~"$agent_type"}[5m]))
) * 100 or vector(0)

# Top failure reasons
topk(5, sum(rate(agent_failure_total[5m])) by (failure_reason))

# Agent resource usage
sum(rate(container_cpu_usage_seconds_total{pod=~".*-$agent_type-.*"}[5m])) by (github_app)
```

**System Health Queries**:
```promql
# GitHub webhook processing performance
histogram_quantile(0.95, 
  sum(rate(github_webhook_processing_duration_seconds_bucket[5m])) 
  by (le, event_type)
)

# Resume operation success rate
rate(resume_successful_total[5m]) / rate(resume_total_attempts[5m]) * 100

# Circuit breaker state monitoring
sum(circuit_breaker_state_gauge{state="open"}) by (circuit_breaker_name)

# Error rate by component
sum(rate(error_total[5m])) by (component)
```

### 3. Panel Configuration Best Practices

**Stat Panel Configuration**:
```json
{
  "type": "stat",
  "title": "Active Workflows",
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
          {"color": "green", "value": null},
          {"color": "yellow", "value": 10},
          {"color": "red", "value": 25}
        ]
      },
      "unit": "short",
      "displayName": "Workflows"
    }
  },
  "options": {
    "colorMode": "background",
    "graphMode": "area",
    "justifyMode": "center",
    "orientation": "horizontal"
  }
}
```

**Time Series Panel Configuration**:
```json
{
  "type": "timeseries",
  "title": "Workflow Stage Durations",
  "targets": [
    {
      "expr": "histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket[5m])) by (le, stage))",
      "legendFormat": "{{stage}} (95th %ile)"
    }
  ],
  "fieldConfig": {
    "defaults": {
      "unit": "s",
      "custom": {
        "drawStyle": "line",
        "lineInterpolation": "smooth",
        "lineWidth": 2,
        "fillOpacity": 10,
        "spanNulls": "1h"
      }
    }
  },
  "options": {
    "tooltip": {
      "mode": "multi",
      "sort": "desc"
    },
    "legend": {
      "calcs": ["lastNotNull", "max", "mean"],
      "displayMode": "table",
      "placement": "bottom"
    }
  }
}
```

**Table Panel Configuration**:
```json
{
  "type": "table",
  "title": "Agent Performance Comparison",
  "targets": [
    {
      "expr": "avg(agent_execution_duration_seconds) by (github_app)",
      "format": "table",
      "instant": true
    },
    {
      "expr": "rate(agent_success_total[5m]) / (rate(agent_success_total[5m]) + rate(agent_failure_total[5m])) * 100",
      "format": "table", 
      "instant": true
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
          "Value #B": "Success Rate (%)"
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
            "id": "custom.cellOptions",
            "value": {"type": "color-background"}
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

### 4. Variable Templating Patterns

**Dashboard Variables Configuration**:
```json
{
  "templating": {
    "list": [
      {
        "name": "datasource",
        "type": "datasource",
        "query": "prometheus",
        "hide": 2,
        "current": {"value": "Victoria Metrics", "text": "Victoria Metrics"}
      },
      {
        "name": "task_id",
        "type": "query",
        "datasource": "${datasource}",
        "query": "label_values(workflow_info{workflow_type=\"play-orchestration\"}, task_id)",
        "regex": "",
        "sort": 3,
        "multi": true,
        "includeAll": true,
        "current": {"value": "$__all", "text": "All"},
        "refresh": "on_dashboard_load"
      },
      {
        "name": "agent_type",
        "type": "query",
        "datasource": "${datasource}",
        "query": "label_values(agent_execution_duration_seconds, github_app)",
        "multi": true,
        "includeAll": true,
        "current": {"value": "$__all", "text": "All"},
        "refresh": "on_time_range_change"
      },
      {
        "name": "time_range",
        "type": "interval",
        "query": "1m,5m,15m,30m,1h,6h,12h",
        "current": {"value": "5m", "text": "5m"}
      }
    ]
  }
}
```

## Common Workflows

### Workflow 1: Complete Dashboard Development

1. **Research and Planning Phase**
   ```bash
   # Use brave-search_brave_web_search
   Search: "grafana dashboard design patterns devops monitoring"
   Search: "prometheus metrics workflow orchestration"
   Search: "grafana alert rules configuration best practices"
   
   # Store research findings
   memory_create_entities("Dashboard Design Patterns", {
     "topic": "workflow-monitoring-dashboard-architecture",
     "patterns": [
       "Hierarchical row-based organization",
       "Overview-to-detail drill-down structure",
       "Consistent color schemes and thresholds",
       "Variable templating for flexible filtering"
     ],
     "best_practices": [
       "Start with high-level overview panels",
       "Group related metrics in logical sections",
       "Use consistent time ranges across panels",
       "Implement meaningful alert thresholds"
     ]
   })
   ```

2. **Dashboard Structure Implementation**
   ```json
   // Create main dashboard JSON file
   // infra/telemetry/telemetry-dashboards/workflow-monitoring/main-dashboard.json
   {
     "dashboard": {
       "id": null,
       "uid": "workflow-monitoring",
       "title": "Multi-Agent Workflow Monitoring",
       "tags": ["taskmaster", "workflows", "multi-agent"],
       "timezone": "UTC",
       "refresh": "30s",
       "time": {"from": "now-24h", "to": "now"},
       "panels": [
         // Panel definitions
       ]
     }
   }
   ```

3. **Panel Development and Testing**
   ```bash
   # Use mcp-prometheus to test queries
   # Test active workflows query
   mcp-prometheus query "count(workflow_status_gauge{workflow_type='play-orchestration', status='Running'})"
   
   # Test agent performance comparison
   mcp-prometheus query "avg(agent_execution_duration_seconds) by (github_app)"
   
   # Test stage duration percentiles
   mcp-prometheus query "histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket[5m])) by (le, stage))"
   
   # Store successful queries
   memory_create_entities("Dashboard Queries", {
     "active_workflows": "count(workflow_status_gauge{workflow_type='play-orchestration', status='Running'})",
     "agent_performance": "avg(agent_execution_duration_seconds) by (github_app)",
     "stage_durations": "histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket[5m])) by (le, stage))"
   })
   ```

4. **Dashboard Deployment and Validation**
   ```bash
   # Use mcp-grafana to deploy dashboard
   mcp-grafana create-dashboard workflow-monitoring main-dashboard.json
   
   # Test dashboard functionality
   mcp-grafana test-dashboard workflow-monitoring
   
   # Verify variable templating
   mcp-grafana test-variables workflow-monitoring task_id
   
   # Check panel data loading
   mcp-grafana validate-panels workflow-monitoring
   ```

### Workflow 2: Alert Configuration and Testing

1. **Alert Rule Development**
   ```yaml
   # Create alert rules configuration
   # alerts/workflow-monitoring.yaml
   apiVersion: monitoring.coreos.com/v1
   kind: PrometheusRule
   metadata:
     name: workflow-monitoring-alerts
     namespace: monitoring
   spec:
     groups:
     - name: workflow.critical
       rules:
       - alert: WorkflowStuckOver24Hours
         expr: (time() - workflow_start_time_seconds{workflow_type="play-orchestration"}) > 86400
         for: 5m
         labels:
           severity: warning
           component: workflow
         annotations:
           summary: "Workflow {{ $labels.workflow_name }} stuck for over 24 hours"
           dashboard_url: "https://grafana.com/d/workflow-monitoring"
   ```

2. **Alert Testing and Validation**
   ```bash
   # Deploy alert rules
   kubectl apply -f alerts/workflow-monitoring.yaml
   
   # Create test scenario for stuck workflow
   kubectl apply -f - <<EOF
   apiVersion: argoproj.io/v1alpha1
   kind: Workflow
   metadata:
     name: test-stuck-workflow
     labels:
       workflow-type: play-orchestration
       task-id: "999"
   spec:
     entrypoint: long-running
     templates:
     - name: long-running
       script:
         image: alpine
         command: [sh]
         source: sleep 90000
   EOF
   
   # Monitor alert firing
   curl http://alertmanager:9093/api/v1/alerts | jq '.data[] | select(.labels.alertname=="WorkflowStuckOver24Hours")'
   ```

### Workflow 3: Performance Optimization and Monitoring

1. **Query Performance Analysis**
   ```bash
   # Use mcp-prometheus to analyze query performance
   # Test query execution times
   start_time=$(date +%s%3N)
   mcp-prometheus query "count(workflow_status_gauge{workflow_type='play-orchestration'})"
   end_time=$(date +%s%3N)
   echo "Query time: $((end_time - start_time))ms"
   
   # Identify slow queries
   mcp-prometheus slow-queries --threshold 5s
   
   # Optimize high-cardinality queries
   memory_query_entities("query optimization strategies")
   ```

2. **Dashboard Load Testing**
   ```bash
   # Simulate concurrent users
   for i in {1..10}; do
     curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
       http://grafana:3000/d/workflow-monitoring &
   done
   wait
   
   # Monitor Victoria Metrics performance
   curl http://victoria-metrics:8428/api/v1/query \
     --data-urlencode 'query=rate(vm_http_requests_total[5m])'
   
   # Check Grafana resource usage
   kubectl top pods -l app=grafana -n monitoring
   ```

3. **Optimization Implementation**
   ```bash
   # Store optimization patterns
   memory_create_entities("Performance Optimizations", {
     "query_optimizations": [
       "Use recording rules for complex calculations",
       "Implement query result caching",
       "Optimize label selection with regex",
       "Use appropriate time ranges for aggregations"
     ],
     "dashboard_optimizations": [
       "Limit concurrent panel updates",
       "Use efficient panel types",
       "Implement progressive loading",
       "Cache dashboard configurations"
     ]
   })
   ```

## Troubleshooting Guide

### Issue 1: Dashboard Panels Not Loading Data
**Symptoms**: Empty panels or "No data" messages

**Diagnosis**:
```bash
# Use mcp-prometheus to test queries directly
mcp-prometheus query "count(workflow_status_gauge{workflow_type='play-orchestration'})"

# Check Victoria Metrics connectivity
curl http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=up{job="victoria-metrics"}'

# Verify data source configuration in Grafana
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/datasources | jq '.[] | select(.type=="prometheus")'
```

**Resolution**:
1. Verify Victoria Metrics is collecting workflow metrics
2. Check PromQL query syntax and fix any errors
3. Ensure data source is correctly configured in Grafana
4. Verify time range includes periods with data

### Issue 2: Variable Templating Not Working
**Symptoms**: Variables show no options or don't filter data

**Diagnosis**:
```bash
# Test variable queries directly
curl -G http://victoria-metrics:8428/api/v1/label/task_id/values

# Check variable configuration
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.templating.list[] | select(.name=="task_id")'

# Test variable query execution
mcp-prometheus query "label_values(workflow_info{workflow_type='play-orchestration'}, task_id)"
```

**Resolution**:
1. Verify variable query returns values
2. Check label existence in metrics
3. Update variable query syntax if needed
4. Ensure panel queries use variables correctly

### Issue 3: Alert Rules Not Firing
**Symptoms**: Expected alerts not appearing in AlertManager

**Diagnosis**:
```bash
# Check alert rule status
kubectl get prometheusrules -n monitoring

# Verify alert rule syntax
kubectl get prometheusrule workflow-monitoring-alerts -n monitoring -o yaml | yq '.spec.groups[].rules[].expr'

# Test alert queries
curl http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=(time() - workflow_start_time_seconds{workflow_type="play-orchestration"}) > 86400'

# Check alert evaluation
curl http://prometheus:9090/api/v1/rules | jq '.data.groups[] | select(.name=="workflow.critical")'
```

**Resolution**:
1. Verify PrometheusRule resource is created correctly
2. Check alert query syntax and logic
3. Ensure metrics referenced in alerts exist
4. Verify alert evaluation interval and conditions

### Issue 4: Poor Dashboard Performance
**Symptoms**: Slow loading times, timeouts, high resource usage

**Diagnosis**:
```bash
# Monitor query performance
curl http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(vm_slow_queries_total[5m])'

# Check dashboard query complexity
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.panels[].targets[].expr' | grep -c 'histogram_quantile\|rate\|sum'

# Monitor resource usage
kubectl top pods -l app=victoria-metrics -n monitoring
kubectl top pods -l app=grafana -n monitoring
```

**Resolution**:
1. Optimize complex PromQL queries
2. Implement recording rules for expensive calculations
3. Adjust panel refresh intervals
4. Use more efficient visualization types

## Tool-Specific Tips

### brave-search_brave_web_search
- Search for "grafana dashboard json examples prometheus"
- Look for "promql query optimization victoria metrics"
- Research "grafana alerting best practices devops"
- Find "dashboard design patterns monitoring operations"

### memory_create_entities / memory_query_entities
- Store successful PromQL query patterns and optimizations
- Document dashboard configuration templates and standards
- Keep track of alert thresholds and their business rationale
- Record troubleshooting procedures and common solutions

### mcp-prometheus / mcp-grafana
- Use for direct query testing and validation
- Implement automated dashboard deployment
- Test variable functionality and panel interactions
- Monitor query performance and resource usage

## Quality Checks

### Pre-Implementation Checklist
- [ ] Research completed on dashboard design patterns
- [ ] Metrics requirements identified and documented
- [ ] Alert thresholds defined based on operational needs
- [ ] Dashboard structure planned with user workflows

### Implementation Checklist
- [ ] All required panels implemented and tested
- [ ] Variable templating provides effective filtering
- [ ] Alert rules configured with proper thresholds
- [ ] Performance acceptable with realistic data volumes
- [ ] Mobile compatibility and export functionality verified

### Post-Implementation Checklist
- [ ] Operations team trained on dashboard usage
- [ ] Runbooks created for common alert scenarios
- [ ] Performance monitoring shows acceptable load
- [ ] User feedback collected and incorporated
- [ ] Documentation updated for maintenance procedures

## Success Indicators

1. **Operational Visibility**:
   - Real-time workflow progress monitoring
   - Agent performance comparison and analysis
   - System health status always visible

2. **Performance Excellence**:
   - Dashboard loads in <5 seconds
   - Queries execute in <10 seconds
   - Auto-refresh doesn't impact user experience

3. **Alert Effectiveness**:
   - Stuck workflows detected within alert thresholds
   - Performance degradation alerts provide early warning
   - False positive rate <5%

4. **User Adoption**:
   - Operations team actively uses dashboard
   - Incident response time improved
   - Data-driven optimization decisions

This guide provides the foundation for creating comprehensive workflow monitoring that enables proactive system management, rapid incident response, and continuous performance optimization.