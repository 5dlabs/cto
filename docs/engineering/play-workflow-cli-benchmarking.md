# Play Workflow CLI Performance Benchmarking

**Date**: 2025-01-29  
**Status**: 📋 PLANNED (Implement after optimization testing)  
**Purpose**: Compare cursor, factory, and claude CLI performance in multi-agent workflows

---

## 🎯 Objectives

1. **Identify fastest CLI** for each agent type (Rex, Cleo, Tess)
2. **Track optimization impact** over time
3. **Detect performance regressions** in future changes
4. **Guide CLI selection** for different workload types

---

## 📊 Three-Phase Approach

### Phase 1: Immediate Visibility (JSON Logs)
**Timeline**: Implement after optimization testing  
**Effort**: ~10 minutes  
**Benefit**: Instant results in pod logs

Add timing instrumentation to all container scripts:

```bash
# At start of container script (after shebang)
WORKFLOW_START_TIME=$(date +%s)
BENCHMARK_DATA="/tmp/benchmark-${WORKFLOW_STAGE:-agent}-${ATTEMPT}.json"

# Track phase timings
SETUP_START=$(date +%s)
# ... setup work (git clone, dependencies, auth) ...
SETUP_END=$(date +%s)
SETUP_DURATION=$((SETUP_END - SETUP_START))

CLI_EXEC_START=$(date +%s)
# ... CLI execution (factory/cursor/claude) ...
CLI_EXEC_END=$(date +%s)
CLI_EXEC_DURATION=$((CLI_EXEC_END - CLI_EXEC_START))

COMPLETION_CHECK_START=$(date +%s)
# ... completion checking logic ...
COMPLETION_CHECK_END=$(date +%s)
COMPLETION_CHECK_DURATION=$((COMPLETION_CHECK_END - COMPLETION_CHECK_START))

# At end of script, output structured metrics
WORKFLOW_TOTAL_DURATION=$(($(date +%s) - WORKFLOW_START_TIME))

cat > "$BENCHMARK_DATA" <<EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "workflow_id": "${WORKFLOW_ID:-unknown}",
  "task_id": "${TASK_ID:-unknown}",
  "cli": "factory|cursor|claude",
  "agent": "rex|cleo|tess",
  "workflow_stage": "${WORKFLOW_STAGE:-unknown}",
  "pod_name": "${HOSTNAME:-unknown}",
  "attempt": ${ATTEMPT:-1},
  "max_retries": ${MAX_RETRIES:-10},
  "model": "${CURRENT_MODEL:-unknown}",
  "timings": {
    "total_seconds": ${WORKFLOW_TOTAL_DURATION},
    "setup_seconds": ${SETUP_DURATION},
    "cli_execution_seconds": ${CLI_EXEC_DURATION},
    "completion_check_seconds": ${COMPLETION_CHECK_DURATION}
  },
  "optimizations": {
    "fast_path_activated": ${FAST_PATH_ACTIVATED:-false},
    "fast_path_time_saved": ${FAST_PATH_TIME_SAVED:-0},
    "context_loaded": ${CONTEXT_LOADED:-false},
    "context_load_success": ${CONTEXT_LOAD_SUCCESS:-false}
  },
  "outcome": {
    "success": ${SUCCESS:-0},
    "exit_code": ${CLI_EXIT_CODE:-999},
    "completion_response": "${COMPLETION_RESPONSE:-unknown}"
  },
  "resource_usage": {
    "memory_mb": $(awk '/^VmRSS:/ {print $2/1024}' /proc/self/status 2>/dev/null || echo "0"),
    "cpu_time_seconds": $(awk '{print $14+$15}' /proc/self/stat 2>/dev/null || echo "0")
  }
}
EOF

# Echo to stdout with clear marker for log parsing
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║                    BENCHMARK RESULTS                         ║"
echo "════════════════════════════════════════════════════════════════"
cat "$BENCHMARK_DATA"
echo "════════════════════════════════════════════════════════════════"
```

**Usage**: After workflow runs, extract metrics:
```bash
# Get benchmark data from pod logs
kubectl logs -n agent-platform <pod-name> | sed -n '/BENCHMARK RESULTS/,/════/p'

# Aggregate across all pods in workflow
kubectl logs -n agent-platform -l workflow-id=<id> | grep -A30 "BENCHMARK RESULTS" | jq -s '.'
```

### Phase 2: Prometheus Metrics (Production Monitoring)
**Timeline**: After Phase 1 validation  
**Effort**: ~30 minutes  
**Benefit**: Historical tracking, alerting, dashboards

Add Prometheus pushgateway integration:

```bash
# At end of container script, push metrics
PUSHGATEWAY_URL="http://pushgateway.monitoring.svc.cluster.local:9091"

cat <<EOF | curl -s --data-binary @- "${PUSHGATEWAY_URL}/metrics/job/play_workflow/instance/${HOSTNAME}/cli/${CLI_NAME}/agent/${AGENT_NAME}/stage/${WORKFLOW_STAGE}"
# HELP play_workflow_duration_seconds Total workflow execution time
# TYPE play_workflow_duration_seconds gauge
play_workflow_duration_seconds{cli="${CLI_NAME}",agent="${AGENT_NAME}",stage="${WORKFLOW_STAGE}",success="${SUCCESS}"} ${WORKFLOW_TOTAL_DURATION}

# HELP play_cli_execution_duration_seconds CLI-specific execution time
# TYPE play_cli_execution_duration_seconds gauge
play_cli_execution_duration_seconds{cli="${CLI_NAME}",agent="${AGENT_NAME}",model="${CURRENT_MODEL}"} ${CLI_EXEC_DURATION}

# HELP play_workflow_attempts_total Number of attempts made
# TYPE play_workflow_attempts_total counter
play_workflow_attempts_total{cli="${CLI_NAME}",agent="${AGENT_NAME}",stage="${WORKFLOW_STAGE}"} ${ATTEMPT}

# HELP play_fast_path_activations_total Fast-path optimization activations
# TYPE play_fast_path_activations_total counter
play_fast_path_activations_total{cli="${CLI_NAME}",agent="${AGENT_NAME}"} ${FAST_PATH_ACTIVATED:-0}

# HELP play_context_loads_total Context persistence loads
# TYPE play_context_loads_total counter
play_context_loads_total{cli="${CLI_NAME}",agent="${AGENT_NAME}",success="${CONTEXT_LOAD_SUCCESS}"} ${CONTEXT_LOADED:-0}

# HELP play_workflow_success_total Successful completions
# TYPE play_workflow_success_total counter
play_workflow_success_total{cli="${CLI_NAME}",agent="${AGENT_NAME}",stage="${WORKFLOW_STAGE}"} ${SUCCESS}
EOF
```

**PromQL Queries**:
```promql
# Average duration by CLI
avg by (cli) (play_workflow_duration_seconds)

# P95 latency by CLI and agent
histogram_quantile(0.95, 
  rate(play_cli_execution_duration_seconds_bucket[1h])
)

# Success rate by CLI
sum by (cli) (rate(play_workflow_success_total[1h])) 
/ 
sum by (cli) (rate(play_workflow_attempts_total[1h]))

# Fast-path activation rate
sum by (cli, agent) (rate(play_fast_path_activations_total[1h]))
```

### Phase 3: Grafana Dashboard (Visualization)
**Timeline**: After Phase 2 data collection  
**Effort**: ~20 minutes  
**Benefit**: Real-time comparison, trend analysis

**Dashboard Panels**:

1. **CLI Performance Comparison** (Bar Chart)
   - X-axis: CLI (factory, cursor, claude)
   - Y-axis: Average duration (seconds)
   - Breakdown by agent type

2. **Latency Percentiles** (Line Graph)
   - P50, P95, P99 over time
   - Separate lines for each CLI
   - Filterable by agent

3. **Success Rate Heatmap**
   - Rows: CLI type
   - Columns: Agent type
   - Color: Success percentage

4. **Optimization Impact** (Stacked Bar)
   - Fast-path time saved
   - Context load time saved
   - Per CLI comparison

5. **Resource Efficiency** (Scatter Plot)
   - X-axis: Duration
   - Y-axis: Memory usage
   - Size: CPU time
   - Color: CLI type

6. **Retry Frequency** (Heatmap)
   - Rows: Time of day
   - Columns: Day of week
   - Color: Average retries
   - Filterable by CLI

**Dashboard JSON**: (Create in Grafana UI, export to git)

---

## 📈 Key Metrics to Track

### Primary Metrics
- **Total Duration**: End-to-end workflow time
- **CLI Execution Time**: Pure CLI runtime (excluding setup)
- **Success Rate**: Percentage of successful completions
- **Retry Count**: Average attempts needed

### Optimization Metrics
- **Fast-Path Hit Rate**: Percentage activating fast-path
- **Fast-Path Time Saved**: Actual minutes saved
- **Context Load Success**: Percentage successful loads
- **Context Load Impact**: Time difference with/without context

### Resource Metrics
- **Memory Usage**: Peak RSS memory
- **CPU Time**: Total CPU seconds consumed
- **I/O Wait**: Time blocked on I/O

### Model Performance
- **Tokens Consumed**: Total input + output tokens
- **API Latency**: Time waiting for model responses
- **Token Rate**: Tokens per second

---

## 🔬 Benchmark Test Scenarios

### Test 1: Simple Implementation Task
**Goal**: Baseline performance comparison  
**Task**: Add a simple REST endpoint  
**Expected Duration**: 5-10 minutes  
**Run Count**: 5 runs per CLI

### Test 2: Complex Refactoring
**Goal**: Stress test CLI reasoning  
**Task**: Refactor module with dependencies  
**Expected Duration**: 15-25 minutes  
**Run Count**: 3 runs per CLI

### Test 3: Quality Review (Cleo)
**Goal**: Compare code analysis speed  
**Task**: Review PR with lint/format issues  
**Expected Duration**: 5-10 minutes  
**Run Count**: 5 runs per CLI

### Test 4: E2E Testing (Tess)
**Goal**: Compare testing workflow  
**Task**: Deploy and test service  
**Expected Duration**: 10-20 minutes  
**Run Count**: 3 runs per CLI

### Test 5: Retry Scenario
**Goal**: Test incremental context  
**Task**: Task requiring 2-3 retries  
**Expected Duration**: 20-30 minutes  
**Run Count**: 3 runs per CLI

### Test 6: Fast-Path Scenario
**Goal**: Validate optimization impact  
**Task**: Run Cleo/Tess on pre-approved PR  
**Expected Duration**: 1-2 minutes  
**Run Count**: 5 runs per CLI

---

## 📊 Sample Results Table

| Metric | Factory | Cursor | Claude | Winner |
|--------|---------|--------|--------|--------|
| **Rex (Implementation)** |
| Avg Duration | 18m 23s | 22m 41s | 19m 15s | Factory |
| Success Rate | 92% | 88% | 90% | Factory |
| Avg Retries | 1.2 | 1.8 | 1.4 | Factory |
| Memory Peak | 850 MB | 1.1 GB | 920 MB | Factory |
| **Cleo (Quality)** |
| Avg Duration | 8m 12s | 6m 45s | 9m 03s | Cursor |
| Success Rate | 95% | 97% | 93% | Cursor |
| Avg Retries | 1.1 | 1.0 | 1.3 | Cursor |
| Fast-Path Hit | 85% | 88% | 82% | Cursor |
| **Tess (Testing)** |
| Avg Duration | 14m 30s | 16m 18s | 12m 55s | Claude |
| Success Rate | 89% | 85% | 91% | Claude |
| Avg Retries | 1.5 | 1.9 | 1.3 | Claude |
| Deploy Success | 94% | 91% | 96% | Claude |

**Overall Recommendation**: 
- Rex: Factory (fastest, most reliable)
- Cleo: Cursor (best analysis speed)
- Tess: Claude (best E2E testing)

---

## 🚀 Implementation Plan

### Step 1: Add Timing to One CLI (Pilot)
- Choose factory (most commonly used)
- Add timing instrumentation to `factory/container-base.sh.hbs`
- Test with single workflow run
- Validate JSON output format

### Step 2: Roll Out to All CLIs
- Copy timing code to cursor and claude scripts
- Adjust CLI-specific variables
- Test each CLI independently

### Step 3: Create Log Parser Script
```bash
#!/bin/bash
# scripts/parse-benchmark-logs.sh

kubectl logs -n agent-platform -l "workflow-id=$1" \
  | grep -A30 "BENCHMARK RESULTS" \
  | jq -s 'group_by(.cli) | map({
      cli: .[0].cli,
      avg_duration: (map(.timings.total_seconds) | add / length),
      success_rate: (map(select(.outcome.success == 1)) | length / length * 100),
      fast_path_rate: (map(select(.optimizations.fast_path_activated == true)) | length / length * 100)
    })'
```

### Step 4: Deploy Prometheus Integration
- Ensure pushgateway is running
- Add curl commands to scripts
- Validate metrics appear in Prometheus UI

### Step 5: Create Grafana Dashboard
- Import base dashboard template
- Configure data sources
- Add custom panels for CLI comparison
- Set up alerts for regressions

### Step 6: Document Findings
- Run benchmark suite (6 test scenarios)
- Collect results in table format
- Update cto-config.json with optimal CLI choices
- Document recommendations in CLAUDE.md

---

## 🔄 Continuous Benchmarking

### Weekly Automated Tests
Run benchmark suite every week:
```yaml
# .github/workflows/cli-benchmark.yaml
name: CLI Performance Benchmark
on:
  schedule:
    - cron: '0 2 * * 1'  # Monday 2 AM
  workflow_dispatch:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmark suite
        run: ./scripts/run-benchmark-suite.sh
      - name: Generate report
        run: ./scripts/generate-benchmark-report.sh
      - name: Post to Slack
        run: ./scripts/post-benchmark-slack.sh
```

### Regression Detection
Alert if any CLI performance degrades >20%:
```promql
# Prometheus alert rule
- alert: CLIPerformanceRegression
  expr: |
    (avg_over_time(play_workflow_duration_seconds[7d]) 
     - avg_over_time(play_workflow_duration_seconds[7d] offset 7d))
    / avg_over_time(play_workflow_duration_seconds[7d] offset 7d) > 0.2
  for: 1h
  labels:
    severity: warning
  annotations:
    summary: "CLI {{ $labels.cli }} performance degraded >20%"
```

---

## 📝 Next Steps

- [ ] Review and approve benchmarking approach
- [ ] Complete optimization testing first
- [ ] Implement Phase 1 (JSON logging) after optimization validation
- [ ] Run initial benchmark suite with all 3 CLIs
- [ ] Analyze results and document findings
- [ ] Implement Phase 2 (Prometheus) for ongoing monitoring
- [ ] Create Phase 3 (Grafana) dashboard
- [ ] Update cto-config.json with optimal CLI selections
- [ ] Set up automated weekly benchmarks

---

**Note**: This benchmarking system will be implemented AFTER the current optimization changes are tested and validated. Initial focus is on measuring the optimization impact, then expanding to compare CLIs.
