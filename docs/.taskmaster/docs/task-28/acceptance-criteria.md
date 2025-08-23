# Acceptance Criteria: Comprehensive Monitoring and Alerting for Long-Running Multi-Agent Workflows

## Health Check System Requirements

### ✅ Agent Health Endpoints
- [ ] Rex agent health endpoint responds at `/health` with agent-specific checks
- [ ] Cleo agent health endpoint validates cargo tools and workspace access
- [ ] Tess agent health endpoint verifies test tools and database connectivity
- [ ] Health endpoints return proper HTTP status codes: 200 (healthy), 503 (unhealthy)
- [ ] JSON response includes timestamp, agent type, and detailed check results
- [ ] Health checks complete within 30 seconds under normal conditions

### ✅ Health Check Validation Tests
**Rex Agent Health Checks:**
- [ ] MCP server connectivity verification
- [ ] Documentation query functionality test
- [ ] Git repository access validation
- [ ] Workspace file system access check

**Cleo Agent Health Checks:**
- [ ] `cargo fmt` availability and functionality
- [ ] `cargo clippy` availability and configuration
- [ ] Git access and commit capabilities
- [ ] Workspace read/write permissions

**Tess Agent Health Checks:**
- [ ] `cargo test` execution capability
- [ ] Coverage tool availability (llvm-cov/tarpaulin)
- [ ] Database connectivity (PostgreSQL/Redis admin access)
- [ ] Deployment validation tools availability

### ✅ Kubernetes Integration
- [ ] Liveness probes configured with appropriate failure threshold (3 failures)
- [ ] Readiness probes configured with shorter intervals (5s period)
- [ ] Initial delay settings account for agent startup time (30s liveness, 5s readiness)
- [ ] Probe timeout configured for reasonable response times (5s timeout)
- [ ] Failed health checks trigger pod restarts through Kubernetes
- [ ] Health status reported to Kubernetes API and visible via kubectl

### ✅ Synthetic Health Check Workflows
- [ ] Synthetic workflows execute automatically every 6 hours
- [ ] Test workflow creation, execution, and completion cycle
- [ ] Validate each agent type responds to synthetic requests
- [ ] Failed synthetic checks trigger immediate alerts
- [ ] Dead letter queue captures failed health check attempts
- [ ] Synthetic workflows clean up after execution

## Stuck Workflow Detection

### ✅ Prometheus Alert Rules
- [ ] `WorkflowStuckAtSameStage` alert fires for workflows stuck >6 hours
- [ ] `WorkflowExcessiveAge` alert fires for workflows running >7 days
- [ ] `WorkflowCriticalAge` alert fires for workflows running >10 days
- [ ] `MultipleWorkflowsStuck` alert fires when ≥3 workflows stuck in same namespace
- [ ] Alert labels include workflow name, namespace, phase, and severity
- [ ] Alert annotations provide clear descriptions and recommended actions

### ✅ Stuck Workflow Detection Tests
- [ ] Create test workflow stuck in 'Running' phase for >6 hours → Alert fires
- [ ] Create test workflow stuck in 'Pending' phase for >6 hours → Alert fires
- [ ] Create workflow running normally for 5 hours → No alert fires
- [ ] Create 3 stuck workflows in same namespace → MultipleWorkflowsStuck alert fires
- [ ] Fix stuck workflow → Alert auto-resolves within alert evaluation interval

### ✅ Automated Stuck Analysis
- [ ] Analysis script identifies stuck workflows and determines root causes
- [ ] Detects resource constraint issues (CPU, memory, storage)
- [ ] Identifies scheduling problems and node capacity issues
- [ ] Analyzes PVC binding failures and storage issues
- [ ] Checks for workflow dependency deadlocks
- [ ] Generates structured analysis reports with recommendations

### ✅ AlertManager Integration
- [ ] Alert routing configured for different severity levels
- [ ] P1 alerts (critical) route to PagerDuty for immediate response
- [ ] P2 alerts route to Slack channels for team awareness
- [ ] P3 alerts route to email for tracking and review
- [ ] Alert suppression configured during maintenance windows
- [ ] Alert correlation reduces noise from cascading failures

## Resource Consumption Monitoring

### ✅ Pod-Level Metrics Collection
- [ ] CPU usage percentage tracked for all agent pods
- [ ] Memory usage in bytes monitored with trend analysis
- [ ] Disk I/O metrics collected (read/write bytes per second)
- [ ] Network I/O tracked for ingress/egress monitoring
- [ ] Metrics collection interval configured at 1-minute intervals
- [ ] Historical data retention supports 30-day trend analysis

### ✅ Resource Usage Tests
- [ ] High CPU usage (>90%) triggers resource exhaustion alerts
- [ ] Memory usage approaching limits (>95%) generates warnings
- [ ] Disk I/O saturation (>90% utilization) creates alerts
- [ ] Network egress costs tracked and alerted on budget thresholds
- [ ] Resource forecasting predicts capacity needs based on trends
- [ ] Cost allocation tags properly assigned to multi-tenant scenarios

### ✅ PVC Usage Monitoring
- [ ] PVC usage percentage monitored for all persistent volumes
- [ ] Alerts fire when PVC usage exceeds 80% capacity
- [ ] Storage growth rate calculated and projected
- [ ] Orphaned PVC detection identifies cleanup candidates
- [ ] PVC performance metrics track I/O performance degradation
- [ ] Storage class optimization recommendations generated

### ✅ External Service Monitoring
- [ ] GitHub API rate limits tracked per agent with remaining quota
- [ ] Rate limit alerts fire when <1000 requests remaining per hour
- [ ] API response times monitored with SLA tracking
- [ ] External service circuit breaker patterns implemented
- [ ] API cost tracking enabled for budget monitoring
- [ ] Service dependency health checks validate external connectivity

## Automated Cleanup System

### ✅ Retention Policy Implementation
- [ ] Completed workflows retain for 7 days before cleanup
- [ ] Failed workflows retain for 3 days before cleanup
- [ ] Error workflows retain for 3 days before cleanup
- [ ] Running workflows retain for 14 days before forced cleanup
- [ ] Cleanup exemption annotation `taskmaster.io/cleanup-exempt=true` respected
- [ ] Compliance retention policies enforce 30-day minimum for audit

### ✅ Workflow Archival System
- [ ] Workflows archived to S3/MinIO before deletion
- [ ] Archive format includes workflow definition, logs, and metadata
- [ ] Compression applied to reduce storage costs (gzip format)
- [ ] Archive retrieval mechanism allows workflow restoration
- [ ] Archive metadata includes workflow name, phase, and archive date
- [ ] Archive integrity validation ensures successful storage

### ✅ Cleanup Automation Tests
- [ ] Create completed workflow >7 days old → Automatically archived and deleted
- [ ] Create failed workflow >3 days old → Automatically cleaned up
- [ ] Create workflow with cleanup exemption → Not cleaned up regardless of age
- [ ] Verify archived workflow can be retrieved and restored
- [ ] Validate associated PVCs cleaned up after workflow deletion
- [ ] Confirm compliance data retained for required audit period

### ✅ PVC and Resource Cleanup
- [ ] Orphaned PVCs identified and marked for cleanup
- [ ] PVC cleanup CronJob removes unused persistent volumes
- [ ] Temporary files and workspace data cleaned up post-workflow
- [ ] Log aggregation completes before resource cleanup
- [ ] Cleanup operations logged for audit and debugging
- [ ] Resource cleanup doesn't impact running workflows

## Monitoring Infrastructure

### ✅ Metrics Collection and Storage
- [ ] Custom workflow metrics collected via Prometheus
- [ ] Pushgateway receives metrics from short-lived cleanup jobs
- [ ] Metrics retention policies configured appropriately (7 days high-res, 30 days low-res)
- [ ] Metric cardinality managed to prevent performance issues
- [ ] Storage capacity monitored for Prometheus data retention
- [ ] Backup and recovery procedures implemented for metrics data

### ✅ Grafana Dashboard Implementation
- [ ] "Long-Running Workflow Monitoring" dashboard created with required panels
- [ ] Workflow age distribution histogram shows workflow lifecycle
- [ ] Stuck workflows panel displays count by phase and status
- [ ] Resource usage graphs show CPU/memory trends by agent
- [ ] PVC usage table highlights storage issues requiring attention
- [ ] GitHub API rate limit gauges show remaining quota per agent
- [ ] Dashboard auto-refresh configured at 1-minute intervals

### ✅ Dashboard Functionality Tests
- [ ] Workflow age histogram updates in real-time with new workflows
- [ ] Stuck workflow counts reflect actual stuck workflows in cluster
- [ ] Resource usage graphs show meaningful trends and spikes
- [ ] PVC usage alerts properly highlighted in dashboard
- [ ] GitHub rate limits update accurately based on API usage
- [ ] All panels load within 5 seconds for operational use

### ✅ Distributed Tracing Implementation
- [ ] OpenTelemetry configured for cross-agent workflow correlation
- [ ] Trace spans created for each workflow stage and agent transition
- [ ] Distributed traces correlate activities across Rex, Cleo, and Tess
- [ ] Trace data enables root cause analysis for performance issues
- [ ] Trace retention configured for debugging and analysis needs
- [ ] Jaeger or similar tool configured for trace visualization

## Operational Tools and Integration

### ✅ kubectl Plugin Implementation
- [ ] `kubectl workflow-health` command shows workflow health status
- [ ] Plugin identifies stuck workflows with analysis and recommendations
- [ ] Resource usage summary available through plugin interface
- [ ] Workflow cleanup status and exemptions displayed
- [ ] Plugin installation and usage documented for operators

### ✅ Workflow Management APIs
- [ ] Workflow pause API suspends specified workflows
- [ ] Workflow resume API continues paused workflows
- [ ] Bulk operations API handles multiple workflows simultaneously
- [ ] Workflow migration tool handles controller upgrades safely
- [ ] APIs include proper authentication and authorization
- [ ] Bulk operations don't impact cluster performance

### ✅ Alert and Notification Validation
- [ ] PagerDuty integration creates incidents for P1 alerts
- [ ] Slack notifications post to correct channels with proper formatting
- [ ] Email notifications include relevant workflow details and links
- [ ] Alert escalation occurs when initial alerts not acknowledged
- [ ] Alert suppression works during scheduled maintenance
- [ ] Notification templates provide actionable information

## Performance and Scalability

### ✅ Performance Requirements
- [ ] Health checks complete within 30 seconds under normal load
- [ ] Stuck workflow detection processes all workflows within 1 minute
- [ ] Resource metrics collection completes within 2 minutes
- [ ] Dashboard queries respond within 5 seconds
- [ ] Alert firing latency under 1 minute for critical alerts
- [ ] Cleanup operations complete without impacting running workflows

### ✅ Load Testing Results
- [ ] System handles 100+ concurrent long-running workflows
- [ ] Monitoring overhead <5% of cluster resources
- [ ] Dashboard remains responsive with 1000+ workflows
- [ ] Alert processing scales linearly with workflow count
- [ ] Metric storage growth remains predictable and manageable
- [ ] Cleanup operations scale with cluster size

### ✅ Scalability Validation
- [ ] Monitoring components scale with cluster auto-scaling
- [ ] Resource limits configured to prevent resource exhaustion
- [ ] Horizontal pod autoscaling configured for monitoring components
- [ ] Storage auto-scaling enabled for metrics and archive data
- [ ] Performance degrades gracefully under extreme load
- [ ] Recovery mechanisms restore normal operation after load spikes

## Security and Compliance

### ✅ Security Implementation
- [ ] Monitoring dashboards require proper authentication
- [ ] Service accounts use least-privilege RBAC permissions
- [ ] Sensitive workflow data encrypted in archives
- [ ] Audit logging captures all administrative operations
- [ ] Secrets properly managed for external service access
- [ ] Network policies restrict monitoring component communication

### ✅ Compliance and Audit
- [ ] Workflow archives retained for required compliance period (30 days minimum)
- [ ] Audit logs include timestamps, user identity, and actions performed
- [ ] Data retention policies documented and enforced
- [ ] Archive integrity verification procedures implemented
- [ ] Data privacy requirements met for workflow content
- [ ] Compliance reporting capabilities available

## Integration and Compatibility

### ✅ Platform Integration
- [ ] Monitoring integrates with existing Prometheus/Grafana stack
- [ ] AlertManager configuration extends existing alert routing
- [ ] Archive storage integrates with organizational backup policies
- [ ] Monitoring metrics compatible with existing SLO/SLI definitions
- [ ] Dashboard integrates with existing operational runbooks
- [ ] APIs compatible with existing automation tools

### ✅ Backward Compatibility
- [ ] Existing workflow monitoring continues to function
- [ ] Legacy alert rules remain active during transition
- [ ] Historical metrics data preserved during upgrade
- [ ] Existing dashboard configurations remain functional
- [ ] Current notification channels continue to work
- [ ] Gradual migration path available for existing workflows

## Deployment and Operations

### ✅ Production Readiness
- [ ] All monitoring components deployed via GitOps
- [ ] Monitoring infrastructure monitored (monitoring the monitors)
- [ ] Backup and recovery procedures tested and documented
- [ ] Disaster recovery plan includes monitoring system restoration
- [ ] Performance tuning completed for production workloads
- [ ] Security scanning passed for all monitoring components

### ✅ Operational Procedures
- [ ] Runbooks created for common monitoring scenarios
- [ ] On-call procedures documented for alert response
- [ ] Escalation procedures defined for unresolved issues
- [ ] Maintenance procedures documented for monitoring updates
- [ ] Training materials created for operations team
- [ ] Knowledge transfer completed for platform team
