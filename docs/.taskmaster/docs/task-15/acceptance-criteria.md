# Acceptance Criteria: Create Workflow Monitoring Dashboard

## Overview

This document defines the specific, testable criteria that must be met to consider Task 15 (Create Workflow Monitoring Dashboard) complete. All criteria must pass before the task can be approved and merged.

## Functional Requirements

### FR-1: Dashboard Structure and Navigation
**Requirement**: Dashboard provides comprehensive monitoring across all system components

**Test Cases**:
- [ ] Dashboard loads successfully in Grafana
- [ ] All panels display data from Victoria Metrics
- [ ] Navigation between dashboard sections works correctly
- [ ] Variable templating filters data appropriately
- [ ] Time range selection affects all relevant panels

**Verification**:
```bash
# Test dashboard accessibility
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | jq '.dashboard.title'

# Verify data source connectivity
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  "http://grafana:3000/api/ds/query" -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "queries": [{
      "expr": "up{job=\"victoria-metrics\"}",
      "refId": "A"
    }]
  }' | jq '.results[].frames[].data.values'

# Test variable templating
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  "http://grafana:3000/api/templating/variables/task_id" | jq '.options[].text'

# Verify panel count and structure
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.panels | length'
```

### FR-2: Workflow Progress Monitoring
**Requirement**: Dashboard tracks workflow progress and stage performance

**Test Cases**:
- [ ] Active workflow count displays current running workflows
- [ ] Stage duration metrics show performance by workflow phase
- [ ] Suspension wait times tracked for each suspension point
- [ ] Task completion rate calculated accurately over time
- [ ] Queue depth visualization shows pending/in-progress/completed tasks

**Verification**:
```bash
# Create test workflows to generate data
for i in {1..5}; do
  kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-workflow-$i
  labels:
    workflow-type: play-orchestration
    task-id: "$((100+i))"
    current-stage: waiting-pr-created
spec:
  entrypoint: test-template
  templates:
  - name: test-template
    script:
      image: alpine
      command: [sh]
      source: echo "Test workflow $i"
EOF
done

# Wait for metrics collection
sleep 30

# Test active workflow count query
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=count(workflow_status_gauge{workflow_type="play-orchestration", status="Running"})' | \
  jq '.data.result[0].value[1]'

# Test stage duration metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket[5m])) by (le, stage))' | \
  jq '.data.result[].metric.stage'

# Test suspension wait time metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=avg(workflow_suspension_duration_seconds) by (suspension_point)' | \
  jq '.data.result[].metric.suspension_point'
```

### FR-3: Agent Performance Analytics
**Requirement**: Dashboard provides detailed agent performance monitoring and comparison

**Test Cases**:
- [ ] Rex vs Blaze comparison table shows execution time differences
- [ ] Agent success rates calculated and displayed correctly
- [ ] Cleo-specific metrics track code quality improvements
- [ ] Tess-specific metrics show testing performance
- [ ] Agent failure analysis categorizes failure reasons

**Verification**:
```bash
# Test agent execution metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=avg(agent_execution_duration_seconds) by (github_app)' | \
  jq '.data.result[] | {agent: .metric.github_app, duration: .value[1]}'

# Test agent success rate calculation
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(agent_success_total[1h]) / (rate(agent_success_total[1h]) + rate(agent_failure_total[1h])) * 100' | \
  jq '.data.result[] | {agent: .metric.github_app, success_rate: .value[1]}'

# Test Cleo-specific metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(cleo_clippy_warnings_fixed_total[1h])' | \
  jq '.data.result[0].value[1]'

curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(cleo_ready_for_qa_labels_added_total[1h])' | \
  jq '.data.result[0].value[1]'

# Test Tess-specific metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(tess_code_review_approvals_total[1h])' | \
  jq '.data.result[0].value[1]'

curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(tess_test_coverage_improvements_total[1h])' | \
  jq '.data.result[0].value[1]'
```

### FR-4: System Health Monitoring
**Requirement**: Dashboard monitors system health and operational metrics

**Test Cases**:
- [ ] Resource utilization panels show CPU/memory usage for agent containers
- [ ] GitHub webhook processing metrics tracked
- [ ] Resume operation success rates displayed
- [ ] Error rates by component visualized
- [ ] Circuit breaker states monitored

**Verification**:
```bash
# Test resource utilization metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=avg(container_cpu_usage_seconds_total{pod=~".*-rex-.*|.*-cleo-.*|.*-tess-.*"}) by (pod)' | \
  jq '.data.result[] | {pod: .metric.pod, cpu: .value[1]}'

curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=avg(container_memory_working_set_bytes{pod=~".*-rex-.*|.*-cleo-.*|.*-tess-.*"}) by (pod)' | \
  jq '.data.result[] | {pod: .metric.pod, memory_gb: (.value[1] | tonumber / 1024 / 1024 / 1024)}'

# Test GitHub webhook processing metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(github_webhook_received_total[5m])' | \
  jq '.data.result[0].value[1]'

curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=histogram_quantile(0.95, sum(rate(github_webhook_processing_duration_seconds_bucket[5m])) by (le))' | \
  jq '.data.result[0].value[1]'

# Test resume operation metrics
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(resume_successful_total[1h]) / rate(resume_total_attempts[1h]) * 100' | \
  jq '.data.result[0].value[1]'

# Test circuit breaker monitoring
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=circuit_breaker_state_gauge' | \
  jq '.data.result[] | {breaker: .metric.circuit_breaker_name, state: .metric.state, value: .value[1]}'
```

## Alert Integration Requirements

### AI-1: Alert Rule Configuration
**Requirement**: Comprehensive alerts configured for workflow and system issues

**Test Cases**:
- [ ] Stuck workflow alert rule exists and is valid
- [ ] High failure rate alert rule configured correctly
- [ ] Agent performance degradation alert implemented
- [ ] Alert rules loaded into Prometheus successfully
- [ ] Alert annotations include dashboard links

**Verification**:
```bash
# Verify alert rules are loaded
kubectl get prometheusrules -n monitoring | grep workflow-monitoring

# Check alert rule syntax
kubectl get prometheusrule workflow-monitoring-alerts -n monitoring -o yaml | \
  yq '.spec.groups[].rules[] | .alert'

# Test alert rule evaluation
curl http://prometheus:9090/api/v1/rules | \
  jq '.data.groups[] | select(.name=="workflow.critical") | .rules[] | {alert: .name, health: .health}'

# Verify alert annotations
kubectl get prometheusrule workflow-monitoring-alerts -n monitoring -o yaml | \
  yq '.spec.groups[].rules[] | select(.alert=="WorkflowStuckOver24Hours") | .annotations'
```

### AI-2: Alert Firing and Notification
**Requirement**: Alerts fire correctly and include proper context

**Test Cases**:
- [ ] Stuck workflow alert fires for workflows running >24 hours
- [ ] High failure rate alert triggers at configured threshold
- [ ] Agent performance alerts detect degradation
- [ ] Alert notifications include dashboard and runbook links
- [ ] Alert resolution works correctly

**Verification**:
```bash
# Create test scenario for stuck workflow alert
kubectl apply -f - <<EOF
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: test-stuck-workflow
  labels:
    workflow-type: play-orchestration
    task-id: "999"
  annotations:
    # Simulate 25-hour old workflow
    start-time: "$(date -d '25 hours ago' -u +%s)"
spec:
  entrypoint: long-running
  templates:
  - name: long-running
    script:
      image: alpine
      command: [sh]
      source: sleep 86400  # 24 hours
EOF

# Wait for alert evaluation
sleep 300  # 5 minutes for alert evaluation

# Check if alert is firing
curl http://alertmanager:9093/api/v1/alerts | \
  jq '.data[] | select(.labels.alertname=="WorkflowStuckOver24Hours")'

# Verify alert contains required annotations
curl http://alertmanager:9093/api/v1/alerts | \
  jq '.data[] | select(.labels.alertname=="WorkflowStuckOver24Hours") | .annotations | keys'

# Test alert resolution by completing workflow
kubectl delete workflow test-stuck-workflow
sleep 300  # Wait for resolution

# Verify alert is resolved
curl http://alertmanager:9093/api/v1/alerts | \
  jq '.data[] | select(.labels.alertname=="WorkflowStuckOver24Hours" and .status.state=="active") | length'
```

## Performance Requirements

### PR-1: Dashboard Load Performance
**Requirement**: Dashboard loads quickly with acceptable performance

**Test Cases**:
- [ ] Dashboard loads within 5 seconds
- [ ] Panel queries execute within 10 seconds
- [ ] Auto-refresh doesn't impact user experience
- [ ] Large time ranges don't cause timeouts
- [ ] Variable changes update panels efficiently

**Verification**:
```bash
# Test dashboard load time
start_time=$(date +%s%3N)
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring > /dev/null
end_time=$(date +%s%3N)
load_time=$((end_time - start_time))
echo "Dashboard load time: ${load_time}ms"
test $load_time -lt 5000  # Should be under 5 seconds

# Test panel query performance
start_time=$(date +%s%3N)
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=count(workflow_status_gauge{workflow_type="play-orchestration"})' > /dev/null
end_time=$(date +%s%3N)
query_time=$((end_time - start_time))
echo "Query execution time: ${query_time}ms"
test $query_time -lt 10000  # Should be under 10 seconds

# Test with large time range (7 days)
start_time=$(date +%s%3N)
curl -G http://victoria-metrics:8428/api/v1/query_range \
  --data-urlencode 'query=rate(workflow_status_gauge[5m])' \
  --data-urlencode 'start='$(date -d '7 days ago' -u +%s) \
  --data-urlencode 'end='$(date -u +%s) \
  --data-urlencode 'step=300' > /dev/null
end_time=$(date +%s%3N)
range_query_time=$((end_time - start_time))
echo "Range query time: ${range_query_time}ms"
test $range_query_time -lt 30000  # Should be under 30 seconds
```

### PR-2: Resource Usage Impact
**Requirement**: Dashboard doesn't significantly impact system performance

**Test Cases**:
- [ ] Victoria Metrics query load remains acceptable
- [ ] Grafana memory usage stays within limits
- [ ] Auto-refresh doesn't overload metrics backend
- [ ] Multiple concurrent users don't degrade performance
- [ ] Historical data queries don't block real-time metrics

**Verification**:
```bash
# Monitor Victoria Metrics query rate
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(vm_http_requests_total[5m])' | \
  jq '.data.result[] | {handler: .metric.handler, rate: .value[1]}'

# Check Grafana resource usage
kubectl top pod -l app=grafana -n monitoring

# Test concurrent user load
for i in {1..10}; do
  curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
    http://grafana:3000/api/dashboards/uid/workflow-monitoring &
done
wait

# Monitor backend performance during load
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=rate(vm_slow_queries_total[1m])' | \
  jq '.data.result[0].value[1] // "0"'
```

## User Experience Requirements

### UX-1: Dashboard Usability
**Requirement**: Dashboard provides intuitive and effective user experience

**Test Cases**:
- [ ] Panel titles and legends are clear and descriptive
- [ ] Color schemes provide good visual distinction
- [ ] Tooltips provide helpful additional information
- [ ] Drill-down links work correctly
- [ ] Mobile view is functional and readable

**Verification**:
```bash
# Test panel configuration
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.panels[] | {id: .id, title: .title, type: .type}'

# Verify color configuration
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.panels[] | select(.fieldConfig.defaults.color) | .fieldConfig.defaults.color'

# Check tooltip configuration
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.panels[] | select(.options.tooltip) | .options.tooltip'

# Test mobile viewport
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  -H "User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)" \
  http://grafana:3000/d/workflow-monitoring
```

### UX-2: Variable Templating and Filtering
**Requirement**: Variable templating provides effective data filtering

**Test Cases**:
- [ ] Task ID variable filters all relevant panels
- [ ] Agent type variable filters agent-specific metrics
- [ ] Time range variable affects historical data queries
- [ ] "All" option works correctly for multi-value variables
- [ ] Variable dependencies work as expected

**Verification**:
```bash
# Test task ID variable filtering
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  "http://grafana:3000/api/templating/variables/task_id/query" | \
  jq '.results[] | .text'

# Test agent type variable
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  "http://grafana:3000/api/templating/variables/agent_type/query" | \
  jq '.results[] | .text'

# Verify variable usage in panel queries
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/dashboards/uid/workflow-monitoring | \
  jq '.dashboard.panels[] | .targets[] | select(.expr | contains("$task_id")) | .expr'

# Test "All" option functionality
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=count(workflow_status_gauge{task_id=~".*"})' | \
  jq '.data.result[0].value[1]'
```

## Integration Requirements

### IR-1: Victoria Metrics Integration
**Requirement**: Dashboard integrates seamlessly with existing Victoria Metrics stack

**Test Cases**:
- [ ] All queries use Victoria Metrics as data source
- [ ] PromQL queries are syntactically correct
- [ ] Historical data retrieval works correctly
- [ ] High cardinality metrics handled efficiently
- [ ] Rate and aggregation functions work as expected

**Verification**:
```bash
# Verify data source configuration
curl -H "Authorization: Bearer $GRAFANA_API_TOKEN" \
  http://grafana:3000/api/datasources | \
  jq '.[] | select(.type=="prometheus") | {name: .name, url: .url}'

# Test PromQL query syntax
echo 'rate(workflow_status_gauge[5m])' | \
  curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode query@- | \
  jq '.status'

# Test historical data retrieval
curl -G http://victoria-metrics:8428/api/v1/query_range \
  --data-urlencode 'query=count(workflow_status_gauge)' \
  --data-urlencode 'start='$(date -d '24 hours ago' -u +%s) \
  --data-urlencode 'end='$(date -u +%s) \
  --data-urlencode 'step=3600' | \
  jq '.data.result[0].values | length'

# Test high cardinality handling
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=count by (task_id) (workflow_info)' | \
  jq '.data.result | length'
```

### IR-2: Alert Manager Integration
**Requirement**: Dashboard alerts integrate with existing Alert Manager setup

**Test Cases**:
- [ ] Alert rules reference correct metrics
- [ ] Alert labels match routing configuration
- [ ] Alert annotations include dashboard URLs
- [ ] Silence and acknowledgment functionality works
- [ ] Notification channels receive alerts correctly

**Verification**:
```bash
# Verify alert routing configuration
curl http://alertmanager:9093/api/v1/status | \
  jq '.data.config.route'

# Check alert rule metrics references
kubectl get prometheusrule workflow-monitoring-alerts -n monitoring -o yaml | \
  yq '.spec.groups[].rules[] | .expr' | \
  grep -E 'workflow_|agent_'

# Test alert label matching
curl http://alertmanager:9093/api/v1/alerts | \
  jq '.data[] | select(.labels.alertname=="WorkflowStuckOver24Hours") | .labels'

# Verify dashboard URL in annotations
kubectl get prometheusrule workflow-monitoring-alerts -n monitoring -o yaml | \
  yq '.spec.groups[].rules[] | select(.alert=="WorkflowStuckOver24Hours") | .annotations.dashboard_url'
```

## Data Accuracy Requirements

### DA-1: Metric Calculation Accuracy
**Requirement**: All calculated metrics are mathematically correct

**Test Cases**:
- [ ] Success rate calculations use correct formulas
- [ ] Percentile calculations return accurate values
- [ ] Rate calculations over time windows are correct
- [ ] Aggregations by labels produce expected results
- [ ] Time-based calculations account for timezone correctly

**Verification**:
```bash
# Test success rate calculation accuracy
# Create known test data
for i in {1..10}; do
  # Create 8 successful and 2 failed operations
  if [ $i -le 8 ]; then
    curl -X POST http://victoria-metrics:8428/api/v1/import/prometheus \
      -d "agent_success_total{github_app=\"test-agent\"} $i $(date +%s)000"
  else
    curl -X POST http://victoria-metrics:8428/api/v1/import/prometheus \
      -d "agent_failure_total{github_app=\"test-agent\"} $((i-8)) $(date +%s)000"
  fi
done

# Query success rate and verify it's 80%
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=agent_success_total{github_app="test-agent"} / (agent_success_total{github_app="test-agent"} + agent_failure_total{github_app="test-agent"}) * 100' | \
  jq '.data.result[0].value[1]' | \
  grep -E '^80(.0+)?$'

# Test percentile calculation
curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=histogram_quantile(0.95, sum(rate(workflow_stage_duration_seconds_bucket[5m])) by (le))' | \
  jq '.data.result[0].value[1] // "no_data"'
```

### DA-2: Historical Data Consistency
**Requirement**: Historical data queries return consistent results

**Test Cases**:
- [ ] Same query returns identical results when repeated
- [ ] Historical aggregations match point-in-time queries
- [ ] Data retention policies don't affect accuracy
- [ ] Time zone handling is consistent
- [ ] Daylight saving time transitions handled correctly

**Verification**:
```bash
# Test query consistency
query='count(workflow_status_gauge{workflow_type="play-orchestration"})'
result1=$(curl -G http://victoria-metrics:8428/api/v1/query --data-urlencode "query=$query" | jq '.data.result[0].value[1]')
sleep 1
result2=$(curl -G http://victoria-metrics:8428/api/v1/query --data-urlencode "query=$query" | jq '.data.result[0].value[1]')
test "$result1" = "$result2"

# Test historical vs current aggregation
historical=$(curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=avg_over_time(workflow_duration_seconds[1h] offset 2h)' | \
  jq '.data.result[0].value[1] // "0"')

current=$(curl -G http://victoria-metrics:8428/api/v1/query \
  --data-urlencode 'query=avg(workflow_duration_seconds)' \
  --data-urlencode 'time='$(date -d '2 hours ago' +%s) | \
  jq '.data.result[0].value[1] // "0"')

# Values should be similar (within 10%)
python3 -c "import sys; exit(0 if abs(float('$historical') - float('$current')) / max(float('$historical'), float('$current'), 1) < 0.1 else 1)"
```

## Final Validation Checklist

Before considering Task 15 complete:

- [ ] All functional requirements (FR-1 through FR-4) pass
- [ ] All alert integration requirements (AI-1 through AI-2) pass
- [ ] All performance requirements (PR-1 through PR-2) pass
- [ ] All user experience requirements (UX-1 through UX-2) pass
- [ ] All integration requirements (IR-1 through IR-2) pass
- [ ] All data accuracy requirements (DA-1 through DA-2) pass
- [ ] Dashboard deployed and accessible to operations team
- [ ] Alert rules active and properly configured
- [ ] Documentation completed for dashboard usage
- [ ] Runbooks created for common alert scenarios
- [ ] Training provided to operations team
- [ ] Load testing validates performance under realistic conditions

## Success Metrics

1. **Dashboard Load Time <5 seconds** - Fast access to monitoring data
2. **Query Performance <10 seconds** - Responsive panel updates
3. **Alert Accuracy 100%** - No false positives or missed alerts
4. **Data Accuracy 99.9%** - Reliable metrics calculations
5. **User Adoption >90%** - Operations team actively uses dashboard
6. **Incident Response Time Reduction >50%** - Faster problem identification
7. **System Visibility Score 95%** - Comprehensive monitoring coverage

## Post-Deployment Monitoring

After Task 15 completion, monitor these key indicators:

- **Dashboard Usage Metrics**: Page views, session duration, user engagement
- **Alert Effectiveness**: Time to detection, false positive rate, resolution time
- **Query Performance**: Average query time, timeout rate, error rate
- **Data Freshness**: Metric update frequency, collection delays
- **User Feedback**: Satisfaction scores, feature requests, usability issues

## Operational Procedures

### Daily Health Checks
1. Verify all dashboard panels are loading data
2. Check alert status and resolve any firing alerts
3. Review workflow completion rates and identify bottlenecks
4. Monitor system resource usage trends

### Weekly Reviews
1. Analyze workflow performance trends
2. Compare agent effectiveness metrics
3. Review alert patterns and tune thresholds
4. Update dashboard based on operational feedback

### Monthly Optimization
1. Review query performance and optimize slow queries
2. Update alert rules based on operational experience
3. Add new metrics based on system evolution
4. Conduct user feedback sessions and implement improvements

When all acceptance criteria are met, Task 15 successfully delivers comprehensive workflow monitoring that enables proactive system management and rapid incident response.