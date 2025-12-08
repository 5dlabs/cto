# Healer: Desired Functionality

> **Vision**: Transform Healer from a reactive watchdog into a comprehensive ops agent that proactively maintains platform health 24/7, learning from every incident to become smarter over time.

## Executive Summary

Healer currently detects problems after they occur. We want to evolve it into an **always-on ops agent** that:
- Proactively monitors system health
- Predicts issues before they impact users
- Self-heals common problems autonomously
- **Learns from past incidents** to prevent recurrence (via OpenMemory)
- Provides real-time visibility into platform status
- **Remembers patterns** across workflows, agents, and time
- Uses **sleep-time compute** to analyze trends during idle periods

---

## Phase 1: Enhanced Monitoring (Foundation)

### 1.1 Continuous Health Checks

**Current Gap**: Healer only activates during workflow runs.

**Desired**:
```
┌────────────────────────────────────────────────────────────┐
│                  CONTINUOUS MONITORING                      │
│                                                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   ArgoCD     │  │  Prometheus  │  │    Loki      │     │
│  │   Health     │  │   Metrics    │  │    Logs      │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                 │                 │              │
│         ▼                 ▼                 ▼              │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              UNIFIED HEALTH ENGINE                   │  │
│  │   • App sync status    • Error rates                │  │
│  │   • Deployment drift   • Resource usage             │  │
│  │   • Git commit lag     • Response latencies         │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**New Alert Types to Add**:
| ID | Name | Description |
|----|------|-------------|
| **H1** | ArgoCD Sync Drift | Application out of sync for >X minutes |
| **H2** | Resource Exhaustion | Node/PVC approaching capacity (>80%) |
| **H3** | Certificate Expiry | TLS certs expiring within X days |
| **H4** | Image Pull Failures | Repeated ImagePullBackOff errors |
| **H5** | Network Connectivity | Service mesh / DNS issues detected |
| **H6** | Database Health | Connection pool exhaustion, slow queries |
| **H7** | Queue Backlog | Job queue growing beyond threshold |
| **H8** | Memory Pressure | OOMKilled containers or node memory warnings |

### 1.2 Resource Monitoring

**Desired Metrics to Track**:
- CPU/Memory utilization per namespace
- PVC usage and growth rate
- Node capacity and scheduling pressure
- Network throughput and error rates
- API server latency

```rust
// Proposed: Resource health check
pub struct ResourceHealth {
    pub namespace: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub pvc_usage: HashMap<String, PvcStatus>,
    pub pod_count: u32,
    pub restart_rate: f64,  // restarts/hour
}
```

### 1.3 GitOps Health

**Integration Points**:
- ArgoCD application sync status
- Helm release versions vs chart versions
- Git commit lag (main vs deployed)
- ConfigMap/Secret drift detection

---

## Phase 2: Proactive Intelligence

### 2.1 Predictive Alerts

**Current**: Alert when CPU hits 90%.
**Desired**: Alert when CPU *will* hit 90% in 30 minutes based on trend.

```rust
// Proposed: Trend-based prediction
pub struct TrendAnalysis {
    pub metric: String,
    pub current_value: f64,
    pub rate_of_change: f64,  // per minute
    pub predicted_threshold_time: Option<Duration>,
    pub confidence: f64,
}
```

**Prediction Categories**:
1. **Resource Exhaustion** — Disk filling up, memory leak patterns
2. **Capacity Planning** — Scale-up needed based on traffic patterns
3. **Failure Patterns** — Recurring failures at specific times/conditions
4. **Dependency Degradation** — Upstream service performance decay

### 2.2 Pattern Recognition

Learn from historical alerts to identify:
- Correlated failures (root cause analysis)
- Time-based patterns (daily/weekly cycles)
- Deployment-related issues (new version regressions)
- Environment-specific problems (staging vs prod differences)

```rust
// Proposed: Alert correlation
pub struct AlertCorrelation {
    pub primary_alert: AlertId,
    pub related_alerts: Vec<AlertId>,
    pub correlation_strength: f64,
    pub common_root_causes: Vec<String>,
    pub suggested_actions: Vec<RemediationAction>,
}
```

### 2.3 Anomaly Detection

**Baseline Learning**:
```
┌────────────────────────────────────────────────────────────┐
│                   ANOMALY DETECTION                         │
│                                                            │
│   1. Establish baselines (7-day rolling window)            │
│   2. Track standard deviations per metric                  │
│   3. Alert on 2σ+ deviations                               │
│   4. Auto-adjust baselines as system evolves               │
│                                                            │
│   Example Anomalies:                                       │
│   • Unusual API error rate spike                           │
│   • Unexpected traffic pattern                             │
│   • Memory usage creep without corresponding load          │
│   • Deployment frequency changes                           │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## Phase 3: Autonomous Remediation

### 3.1 Auto-Remediation Actions

**Safe, Pre-Approved Actions** (no human approval needed):
| Action | Trigger | Rollback |
|--------|---------|----------|
| Pod restart | OOMKilled, CrashLoop (<3) | N/A |
| Scale up | CPU/Memory >80% sustained | Scale down after 15m |
| Clear PVC | Temp files >80% capacity | N/A |
| Retry ArgoCD sync | Transient sync failure | None |
| Refresh secrets | Secret rotation detected | Restore previous |

**Approval-Required Actions**:
| Action | Trigger | Approval |
|--------|---------|----------|
| Rollback deployment | Health check failures | Slack notification + 5m timeout |
| Node drain | Hardware issues | Admin approval |
| Database failover | Primary unresponsive | PagerDuty escalation |
| Feature flag disable | Error spike correlated to flag | Product owner |

### 3.2 Remediation Playbooks

```yaml
# Proposed: Playbook definition
apiVersion: healer.platform/v1
kind: RemediationPlaybook
metadata:
  name: high-memory-pod
spec:
  trigger:
    alert: H8
    conditions:
      - metric: container_memory_usage_bytes
        threshold: 0.9
        duration: 5m
  actions:
    - type: restart_pod
      timeout: 60s
      retry: 3
    - type: scale_up
      condition: restart_failed
      replicas: +1
    - type: alert_escalate
      condition: scale_failed
      channel: oncall
  rollback:
    enabled: true
    window: 15m
```

### 3.3 Learning from Remediations

After each remediation:
1. **Record outcome** — Did it work? How long until stable?
2. **Update confidence scores** — Increase/decrease based on success
3. **Refine playbooks** — Add conditions that led to failure
4. **Share learnings** — Update OpenMemory with new patterns

---

## Phase 4: Platform Observability

### 4.1 Health Dashboard

**Real-time Platform Status**:
```
┌────────────────────────────────────────────────────────────┐
│                    HEALER DASHBOARD                         │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Overall Health: 🟢 HEALTHY                                │
│                                                            │
│  ┌─────────────┬─────────────┬─────────────┬────────────┐ │
│  │ Workflows   │ Agents      │ Infra       │ GitOps     │ │
│  │ 🟢 12/12    │ 🟡 5/6      │ 🟢 OK       │ 🟢 Synced  │ │
│  └─────────────┴─────────────┴─────────────┴────────────┘ │
│                                                            │
│  Recent Alerts:                                            │
│  ├─ [5m ago]  A2 Silent Failure: play-task-4-abc          │
│  │            └─ Auto-remediated: Pod restarted            │
│  └─ [2h ago]  H2 Resource Warning: cto namespace 78%      │
│               └─ Pending: Requires attention               │
│                                                            │
│  Active Workflows: 3                                       │
│  │ play-task-12  Rex (implementation) ▓▓▓▓▓▓░░░░ 60%     │
│  │ play-task-14  Cleo (quality)       ▓▓▓░░░░░░░ 30%     │
│  │ play-task-8   Tess (testing)       ▓▓▓▓▓▓▓▓▓░ 90%     │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 4.2 CLI Enhancements

```bash
# Platform health overview
healer status --all

# Interactive dashboard (TUI)
healer dashboard

# Health history
healer history --since 24h --alerts-only

# Silence alerts for maintenance
healer silence --duration 2h --reason "scheduled maintenance"

# View and manage playbooks
healer playbooks list
healer playbooks run high-memory-pod --dry-run
healer playbooks edit high-memory-pod
```

### 4.3 Notifications & Escalation

**Notification Channels**:
- Slack (with thread updates as remediation progresses)
- PagerDuty (for critical alerts)
- GitHub Issues (for workflow failures)
- Email digest (daily summary)

**Escalation Paths**:
```yaml
escalation:
  levels:
    - delay: 0m
      channels: [slack-alerts]
    - delay: 15m
      channels: [slack-oncall, pagerduty]
    - delay: 30m
      channels: [pagerduty-critical]
```

---

## Phase 5: Integration & Coordination

### 5.1 Agent Coordination

**Healer as Orchestrator**:
- Track all active CodeRuns across the platform
- Detect resource contention between agents
- Prioritize critical remediations over routine work
- Pause low-priority work when issues detected

```rust
// Proposed: Platform-wide agent tracking
pub struct AgentCoordinator {
    active_coderuns: HashMap<String, CodeRunStatus>,
    resource_usage: ClusterResources,
    pending_remediations: PriorityQueue<Remediation>,
}
```

### 5.2 External Integrations

**Upstream Dependencies**:
- Monitor external API health (GitHub, OpenAI, etc.)
- Track rate limits and quota usage
- Failover to backup providers when needed

**Downstream Consumers**:
- Webhook notifications for status changes
- API for external monitoring systems
- Prometheus metrics export

### 5.3 Multi-Cluster Support

```rust
// Proposed: Multi-cluster monitoring
pub struct ClusterHealth {
    pub name: String,
    pub context: String,
    pub status: ClusterStatus,
    pub alerts: Vec<Alert>,
    pub last_check: DateTime<Utc>,
}

pub enum ClusterStatus {
    Healthy,
    Degraded { reason: String },
    Unreachable { since: DateTime<Utc> },
}
```

---

## Phase 6: Long-Running Historical Context (Memory)

> **Core Insight**: Memory is the differentiator. An ops agent that learns from past incidents will outperform one that treats every issue as new.

### 6.1 Memory Architecture

**Current Gap**: Healer is stateless—it doesn't learn from past remediations or remember patterns.

**Desired Architecture**:
```
┌─────────────────────────────────────────────────────────────────────┐
│                    HEALER MEMORY ARCHITECTURE                       │
│                                                                     │
│  ┌────────────────────────┐    ┌────────────────────────────────┐  │
│  │   WORKING MEMORY       │    │   LONG-TERM MEMORY (OpenMemory)│  │
│  │   (Current Context)    │    │                                 │  │
│  │   • Active alerts      │◀───│   FACTUAL: "Pod X crashes      │  │
│  │   • Current workflows  │    │   when memory >2Gi"            │  │
│  │   • Recent events      │    │                                 │  │
│  └────────────────────────┘    │   TEMPORAL: "Deploys fail on   │  │
│                                │   Fridays 4PM (high traffic)"  │  │
│  ┌────────────────────────┐    │                                 │  │
│  │   SLEEP-TIME COMPUTE   │───▶│   RELATIONAL: "Alert A7 in     │  │
│  │   (Between Incidents)  │    │   cto → usually Redis issue"   │  │
│  │   • Pattern analysis   │    │                                 │  │
│  │   • Correlation study  │    │   BEHAVIORAL: "Rex succeeds    │  │
│  │   • Playbook refinement│    │   when given explicit paths"   │  │
│  └────────────────────────┘    └────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Concepts**:

| Concept | Definition | Healer Application |
|---------|------------|-------------------|
| **Working Memory** | Immediate context during inference | Current alert, recent events, active workflows |
| **Long-Term Memory** | Persistent knowledge across sessions | Past remediations, patterns, playbook outcomes |
| **Sleep-Time Compute** | Background processing during idle | Correlation analysis, pattern extraction, predictions |
| **Memory Decay** | Gradual forgetting of stale info | Old patterns fade unless reinforced by success |

### 6.2 Memory Types for Ops

Different types of memories serve different purposes:

| Memory Type | Examples | Decay Rate | Use Case |
|-------------|----------|------------|----------|
| **Factual** | "Service X needs 4Gi memory limit" | Slow | Resource configuration |
| **Episodic** | "Task-45 failed due to Git auth on Nov 24" | Medium | Debugging similar issues |
| **Semantic** | "OOMKilled usually means leak, not limit" | Very slow | Root cause reasoning |
| **Procedural** | "To fix ArgoCD sync: 1) Check git 2) Refresh" | None (playbook) | Automated remediation |
| **Temporal** | "Traffic spikes Mondays 9AM" | Weekly update | Proactive scaling |

```rust
// Proposed: Memory entry for Healer
pub struct HealerMemory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub metadata: HealerMemoryMetadata,
    pub salience: f64,           // 0.0-1.0, higher = more important
    pub reinforcements: u32,     // Times this memory proved useful
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

pub struct HealerMemoryMetadata {
    pub alert_type: Option<AlertId>,
    pub namespace: String,
    pub service: Option<String>,
    pub success: bool,
    pub duration_mins: Option<u32>,
    pub method: Option<String>,
    pub pr_url: Option<String>,
}

pub enum MemoryType {
    Factual,
    Episodic,
    Semantic,
    Procedural,
    Temporal,
}
```

### 6.3 Memory-Augmented Remediation

**Before Spawning Remediation**:
```rust
impl Healer {
    async fn remediate_with_memory(&self, alert: &Alert) -> Result<()> {
        // 1. Query relevant memories
        let memories = self.memory.query(
            &format!("{} failure pattern in {}", alert.id, alert.namespace),
            15,  // limit
            true // include waypoints for related memories
        ).await?;

        // 2. Check for known high-confidence solutions
        let known_fixes: Vec<_> = memories.iter()
            .filter(|m| m.metadata.success && m.salience > 0.7)
            .collect();

        if let Some(best_fix) = known_fixes.first() {
            tracing::info!(
                "Found known fix for {} with {:.0}% confidence",
                alert.id,
                best_fix.salience * 100.0
            );
            // Consider applying directly if confidence > 90%
            if best_fix.salience > 0.9 {
                return self.apply_known_fix(best_fix).await;
            }
        }

        // 3. Enrich context for remediation agent
        let context = EnrichedAlertContext {
            alert: alert.clone(),
            past_failures: memories.iter()
                .filter(|m| !m.metadata.success)
                .cloned()
                .collect(),
            past_successes: memories.iter()
                .filter(|m| m.metadata.success)
                .cloned()
                .collect(),
            related_patterns: self.find_correlated_patterns(alert).await?,
        };

        self.spawn_remediation_coderun(context).await
    }
}
```

**After Remediation Completes**:
```rust
async fn learn_from_outcome(&self, run: &CodeRun, outcome: &Outcome) -> Result<()> {
    // Store the experience
    self.memory.add(HealerMemory {
        content: format!(
            "Alert {} on {}/{} resolved by: {}. Duration: {}min",
            run.alert_id,
            run.namespace,
            run.target,
            outcome.summary,
            outcome.duration_mins
        ),
        memory_type: MemoryType::Episodic,
        metadata: HealerMemoryMetadata {
            alert_type: Some(run.alert_id.clone()),
            namespace: run.namespace.clone(),
            service: run.service.clone(),
            success: outcome.resolved,
            duration_mins: Some(outcome.duration_mins),
            method: Some(outcome.method.clone()),
            pr_url: outcome.pr_url.clone(),
        },
        salience: if outcome.resolved { 0.7 } else { 0.4 },
        ..Default::default()
    }).await?;

    // Reinforce memories that contributed to success
    if outcome.resolved {
        for memory_id in &outcome.relevant_memory_ids {
            self.memory.reinforce(memory_id, 2).await?;
        }
    }

    // Update playbook confidence scores
    if let Some(playbook) = self.playbooks.get_mut(&run.alert_id) {
        playbook.update_confidence(outcome.resolved);
    }

    Ok(())
}
```

### 6.4 Sleep-Time Compute

**Concept**: Use idle periods to analyze patterns and pre-compute insights.

> "By anticipating what queries users might ask and pre-computing useful quantities, we can significantly reduce compute requirements at test-time." — [Letta Research](https://arxiv.org/abs/2504.13171)

**Implementation**:
```rust
// Run as CronJob every 2 hours during low-activity periods
pub struct SleepTimeAnalyzer {
    memory: OpenMemoryClient,
    prometheus: PrometheusClient,
    loki: LokiClient,
}

impl SleepTimeAnalyzer {
    pub async fn analyze(&self) -> Result<SleepTimeReport> {
        // 1. Find alert correlations from past 24h
        let correlations = self.find_alert_correlations(Duration::hours(24)).await?;
        
        // 2. Extract failure patterns from logs
        let patterns = self.extract_failure_patterns().await?;
        
        // 3. Predict resource issues from trends
        let predictions = self.predict_resource_exhaustion().await?;
        
        // 4. Identify time-based patterns
        let temporal = self.analyze_temporal_patterns().await?;
        
        // 5. Store insights as memories
        for correlation in &correlations {
            self.memory.add(HealerMemory {
                content: format!(
                    "Correlation: {} often followed by {} (strength: {:.0}%)",
                    correlation.primary,
                    correlation.secondary,
                    correlation.strength * 100.0
                ),
                memory_type: MemoryType::Semantic,
                salience: correlation.strength,
                ..Default::default()
            }).await?;
        }
        
        // 6. Generate pre-computed responses
        let precomputed = self.generate_response_templates(&patterns).await?;
        
        Ok(SleepTimeReport {
            correlations,
            patterns,
            predictions,
            temporal,
            precomputed,
        })
    }
}
```

**Sleep-Time Activities**:

| Activity | Frequency | Output |
|----------|-----------|--------|
| Alert Correlation | Every 2h | "A7 in cto namespace → 80% caused by Redis" |
| Pattern Extraction | Every 4h | Common failure signatures with solutions |
| Resource Prediction | Every 1h | "PVC will fill in 6 hours at current rate" |
| Temporal Analysis | Daily | "Monday 9AM: expect 3x normal load" |
| Playbook Tuning | Weekly | Confidence score adjustments |

### 6.5 Context Engineering for Ops

**Avoid Context Rot**: Large context windows degrade performance.

```rust
// BAD: Accumulating all history (context rot)
struct AlertContext {
    all_alerts: Vec<Alert>,      // Grows unbounded → degrades LLM performance
    all_logs: Vec<String>,       // Token explosion
    full_history: Vec<Event>,    // Context window overflow
}

// GOOD: Selective, summarized context
struct AlertContext {
    current_alert: AlertDetail,           // Full detail for current issue
    recent_alerts: RingBuffer<Alert, 10>, // Last 10 only
    relevant_memories: Vec<MemorySummary>,// Summarized, high-salience
    suggested_actions: Vec<String>,       // Pre-computed from memory
}
```

**Memory Selection Strategy**:
```rust
impl MemorySelector {
    /// Select most relevant memories, avoiding context pollution
    fn select_relevant(&self, query: &str, limit: usize) -> Vec<Memory> {
        let candidates = self.memory.query(query, limit * 3)?;
        
        candidates.into_iter()
            .filter(|m| m.salience > 0.3)              // Above threshold
            .filter(|m| !self.is_stale(m))             // Not outdated
            .sorted_by(|a, b| {
                // Score: 40% relevance + 30% recency + 30% success rate
                let score = |m: &Memory| {
                    m.relevance * 0.4 +
                    m.recency_score() * 0.3 +
                    m.success_rate() * 0.3
                };
                score(b).partial_cmp(&score(a)).unwrap()
            })
            .take(limit)
            .collect()
    }
    
    fn is_stale(&self, memory: &Memory) -> bool {
        let age = Utc::now() - memory.last_accessed;
        // Procedural memories never stale; others decay
        match memory.memory_type {
            MemoryType::Procedural => false,
            MemoryType::Factual => age > Duration::days(90),
            MemoryType::Episodic => age > Duration::days(30),
            MemoryType::Temporal => age > Duration::days(7),
            _ => age > Duration::days(60),
        }
    }
}
```

### 6.6 Integration with OpenMemory

Healer should integrate with the existing OpenMemory deployment:

```
┌─────────────────────────────────┐
│    OpenMemory Server            │
│    Namespace: cto-system        │
│    Service: openmemory:8080     │
└─────────────────────────────────┘
              ▲
              │ HTTP API
              │
    ┌─────────┴─────────┐
    │                   │
  Healer            Other Agents
  (namespace:       (Rex, Cleo, etc.)
   agent/healer)
```

**CLI Commands** (already partially implemented):
```bash
# List memories for healer
healer memory list --limit 20

# Query for relevant patterns
healer memory query --text "ArgoCD sync failures" --limit 10

# Get memory statistics
healer memory stats

# Manually store a pattern (for debugging/testing)
healer memory add --content "Pod X requires 4Gi" --type factual

# Trigger sleep-time analysis manually
healer analyze --since 24h --output report.json
```

### 6.7 Memory-Driven Predictive Alerts

New alert type that uses accumulated memory:

```rust
// H10: Predicted Issue (memory-driven)
impl AlertHandler for PredictedIssueAlert {
    async fn evaluate(&self, ctx: &MonitorContext) -> Option<Alert> {
        // Query memories for patterns matching current conditions
        let patterns = self.memory.query(
            &format!("failure pattern {} {}", ctx.namespace, ctx.service),
            20,
            true // include waypoints
        ).await.ok()?;
        
        let current = CurrentConditions::from_context(ctx);
        
        for pattern in &patterns {
            if pattern.matches(&current) && pattern.salience > 0.8 {
                return Some(Alert {
                    id: AlertId::H10,
                    severity: Severity::Warning,
                    message: format!(
                        "Predicted: {} likely based on pattern '{}'",
                        pattern.failure_type,
                        pattern.name
                    ),
                    suggested_action: pattern.prevention_action.clone(),
                    confidence: pattern.salience,
                });
            }
        }
        
        None
    }
}
```

### 6.8 Solution Comparison

| Feature | OpenMemory (Current) | Mem0 | Letta |
|---------|---------------------|------|-------|
| **Deployment** | ✅ Already deployed | New infra needed | Complex setup |
| **MCP Integration** | ✅ Native | ✅ Has MCP | ✅ Has MCP |
| **Multi-sector Memory** | ✅ 5 sectors | ✅ Intelligent filtering | ✅ Memory blocks |
| **Decay/Aging** | ✅ Configurable | ✅ Dynamic forgetting | ✅ Self-pruning |
| **Sleep-time Compute** | ⚠️ Implement ourselves | ❌ No | ✅ Native |
| **Self-editing Memory** | ❌ External only | ❌ No | ✅ Agent can edit |
| **Kubernetes Native** | ✅ Helm chart | Needs adaptation | Needs adaptation |
| **Cost** | Free (self-hosted) | Paid cloud option | Free (self-hosted) |

**Recommendation**: Leverage existing OpenMemory deployment, implement sleep-time compute ourselves.

---

## Implementation Priorities

### Must Have (MVP)
1. [ ] Continuous health check daemon mode
2. [ ] ArgoCD sync monitoring (H1)
3. [ ] Resource exhaustion alerts (H2, H8)
4. [ ] Auto-restart for simple failures
5. [ ] Basic health CLI (`healer status --all`)
6. [ ] **Memory-augmented remediation** — Query OpenMemory before spawning fixes

### Should Have
1. [ ] Certificate expiry monitoring (H3)
2. [ ] Image pull failure detection (H4)
3. [ ] Slack notifications with updates
4. [ ] Remediation playbook framework
5. [ ] Health history and trends
6. [ ] **Outcome learning** — Store remediation results in memory
7. [ ] **Memory CLI enhancements** — `healer memory query/add/stats`

### Nice to Have
1. [ ] TUI dashboard
2. [ ] Predictive alerts (H10 memory-driven)
3. [ ] Multi-cluster support
4. [ ] External dependency monitoring
5. [ ] **Sleep-time compute** — Background pattern analysis
6. [ ] **Temporal pattern detection** — Time-based failure prediction
7. [ ] **Cross-agent memory sharing** — Learn from Rex/Cleo/Tess patterns

---

## Technical Considerations

### Deployment Model

**Option A: Long-Running Daemon**
```yaml
# Runs continuously as a Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: healer
spec:
  replicas: 1  # Leader election for HA
```

**Option B: CronJob + Event-Driven**
```yaml
# Periodic checks + webhook triggers
apiVersion: batch/v1
kind: CronJob
metadata:
  name: healer-check
spec:
  schedule: "*/5 * * * *"  # Every 5 minutes
```

**Recommendation**: Hybrid — daemon for critical monitoring, CronJob for intensive checks.

### State Management

Where should Healer store state?
- **ConfigMap**: Simple, K8s-native, limited size
- **CRD**: Structured, queryable, versioned
- **Redis/PostgreSQL**: Scalable, complex queries, external dependency

**Recommendation**: Start with CRDs (`HealerState`, `RemediationRecord`), migrate to database if needed.

### Resource Requirements

```yaml
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

---

## Open Questions

1. **Scope**: Should Healer monitor only CTO workloads or entire cluster?
2. **Permissions**: What RBAC permissions does Healer need? Cluster-admin?
3. **Blast Radius**: How do we prevent Healer from making things worse?
4. **Testing**: How do we test auto-remediation safely?
5. **Audit Trail**: What compliance requirements exist for autonomous actions?
6. **Memory Isolation**: Should Healer have its own memory namespace or share with agents?
7. **Memory Retention**: How long should different memory types persist before decay?
8. **Confidence Thresholds**: At what confidence level should Healer auto-apply known fixes?
9. **Sleep-Time Scheduling**: When should background analysis run to avoid resource contention?
10. **Memory Size Limits**: How much memory storage is acceptable before pruning?

---

## Next Steps

1. [ ] Review this document and prioritize features
2. [ ] Design Phase 1.1 (Continuous Health Checks)
3. [ ] Define CRDs for health state
4. [ ] Implement daemon mode
5. [ ] Add ArgoCD integration
6. [ ] **Integrate OpenMemory client** — Add memory queries to remediation flow
7. [ ] **Implement outcome learning** — Store results after each remediation
8. [ ] **Design sleep-time analyzer** — CronJob for background pattern analysis
9. [ ] **Add H10 predictive alert** — Memory-driven prediction handler
10. [ ] **Create memory dashboard** — Grafana panels for memory health/hit rates

---

## Appendix: Memory Integration Checklist

### Phase 6.A: Memory-Augmented Remediation (2-3 days)
- [ ] Add OpenMemory client to Healer
- [ ] Query memories before spawning remediation CodeRun
- [ ] Pass relevant memories to remediation agent context
- [ ] Add `--use-memory` flag to remediation commands

### Phase 6.B: Outcome Learning (1 week)
- [ ] Detect CodeRun completion (success/failure)
- [ ] Extract outcome summary from PR/logs
- [ ] Store experience as new memory
- [ ] Reinforce memories that contributed to success
- [ ] Update playbook confidence scores

### Phase 6.C: Sleep-Time Compute (2 weeks)
- [ ] Create `SleepTimeAnalyzer` module
- [ ] Implement alert correlation detection
- [ ] Implement failure pattern extraction
- [ ] Implement resource trend prediction
- [ ] Add CronJob for periodic analysis
- [ ] Store insights as memories

### Phase 6.D: Predictive Alerts (2 weeks)
- [ ] Implement H10 `PredictedIssueAlert` handler
- [ ] Query memories for matching patterns
- [ ] Add confidence scoring to predictions
- [ ] Integrate with notification system
- [ ] Add `healer predict --namespace cto` command

---

*Last updated: December 2024*

