# Healer: Desired Functionality

> **Vision**: Transform Healer from a reactive watchdog into a comprehensive ops agent that proactively maintains platform health 24/7.

## Executive Summary

Healer currently detects problems after they occur. We want to evolve it into an **always-on ops agent** that:
- Proactively monitors system health
- Predicts issues before they impact users
- Self-heals common problems autonomously
- Learns from past incidents to prevent recurrence
- Provides real-time visibility into platform status

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

## Implementation Priorities

### Must Have (MVP)
1. [ ] Continuous health check daemon mode
2. [ ] ArgoCD sync monitoring (H1)
3. [ ] Resource exhaustion alerts (H2, H8)
4. [ ] Auto-restart for simple failures
5. [ ] Basic health CLI (`healer status --all`)

### Should Have
1. [ ] Certificate expiry monitoring (H3)
2. [ ] Image pull failure detection (H4)
3. [ ] Slack notifications with updates
4. [ ] Remediation playbook framework
5. [ ] Health history and trends

### Nice to Have
1. [ ] TUI dashboard
2. [ ] Predictive alerts
3. [ ] Multi-cluster support
4. [ ] External dependency monitoring
5. [ ] Learning from remediations

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

---

## Next Steps

1. [ ] Review this document and prioritize features
2. [ ] Design Phase 1.1 (Continuous Health Checks)
3. [ ] Define CRDs for health state
4. [ ] Implement daemon mode
5. [ ] Add ArgoCD integration

---

*Last updated: December 2024*

