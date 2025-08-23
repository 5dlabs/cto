# Autonomous Agent Prompt: Comprehensive Monitoring and Alerting for Long-Running Multi-Agent Workflows



## Objective

Implement a comprehensive monitoring and alerting system specifically designed for long-running multi-agent workflows that operate for days or weeks. Build robust health checks, stuck workflow detection, resource tracking, and automated cleanup strategies to ensure operational excellence and prevent system degradation.

## Context

You are implementing monitoring infrastructure for the Task Master platform where workflows involving Rex, Cleo, and Tess agents can run for extended periods. These long-running workflows require specialized monitoring to detect stuck states, resource exhaustion, and operational issues that may develop over time. The system must be proactive in identifying problems and automated in maintaining system health.

## Implementation Requirements

### Health Check Infrastructure

Implement comprehensive health monitoring:



1. **Agent Health Endpoints**


   - Create custom health check endpoints for each agent type (Rex, Cleo, Tess)


   - Include agent-specific health validations (MCP connectivity, tool availability, resource access)


   - Return structured health status with detailed check results


   - Implement proper HTTP status codes and JSON response formats



2. **Kubernetes Integration**


   - Configure liveness and readiness probes for all agent pods


   - Set appropriate probe parameters for long-running workflows


   - Implement health check containers with minimal resource footprint


   - Enable health status reporting to Kubernetes API



3. **Synthetic Health Monitoring**


   - Create synthetic workflows that execute every 6 hours


   - Test end-to-end workflow creation, execution, and completion


   - Validate agent connectivity and basic functionality


   - Implement dead letter queue pattern for failed health checks

### Stuck Workflow Detection System

Build intelligent stuck workflow detection:



1. **Prometheus Alert Rules**


   - Create alerts for workflows stuck at same stage >6 hours (configurable)


   - Implement graduated workflow age thresholds (7 days warning, 10 days critical)


   - Monitor workflow phase transitions and detect anomalies


   - Alert on multiple stuck workflows indicating system-wide issues



2. **Automated Stuck Analysis**


   - Analyze workflow status and node states to determine stuck reasons


   - Check resource availability, scheduling constraints, and dependencies


   - Examine controller logs for error patterns and failure indicators


   - Generate automated recommendations for resolution



3. **AlertManager Integration**


   - Configure alert routing to appropriate notification channels


   - Implement escalation policies for different severity levels


   - Set up PagerDuty and Slack integrations for incident response


   - Enable alert correlation and suppression to reduce noise

### Resource Consumption Monitoring

Implement comprehensive resource tracking:



1. **Pod-Level Metrics Collection**


   - Collect CPU, memory, and disk I/O metrics for all agent pods


   - Track resource usage trends and identify resource exhaustion


   - Monitor network usage and egress costs


   - Implement resource forecasting based on historical data



2. **Storage Monitoring**


   - Monitor PVC usage with alerts at 80% capacity threshold


   - Track storage growth patterns and predict capacity needs


   - Identify orphaned volumes and implement cleanup procedures


   - Monitor backup and archival storage usage



3. **External Service Monitoring**


   - Track GitHub API rate limits per agent with remaining quota alerts


   - Monitor external service dependencies and response times


   - Implement circuit breaker patterns for external service failures


   - Track API cost allocation across different agents

### Automated Cleanup Strategies

Design intelligent cleanup systems:



1. **Graduated Retention Policies**
   - Implement different retention periods based on workflow state:
     - Active workflows: no cleanup
     - Completed workflows: 7 days retention
     - Failed workflows: 3 days retention
     - Stuck workflows: 14 days retention


   - Allow cleanup exemption via workflow annotations



2. **Workflow Archival System**


   - Archive workflows to S3/MinIO before deletion with compression


   - Include workflow metadata, logs, and execution history


   - Implement retrieval mechanisms for archived workflows


   - Maintain compliance with retention requirements (30 days minimum)



3. **Resource Cleanup Automation**


   - Clean up orphaned PVCs and associated storage


   - Remove completed workflow artifacts and temporary files


   - Implement log aggregation before resource cleanup


   - Create cleanup reporting and audit trails

### Monitoring Infrastructure

Build scalable monitoring platform:



1. **Metrics Collection and Storage**


   - Extend Prometheus configuration for workflow-specific metrics


   - Implement custom metric collectors for specialized monitoring


   - Configure appropriate retention policies for different metric types


   - Use Pushgateway for short-lived job metrics



2. **Visualization and Dashboards**


   - Create Grafana dashboards for long-running workflow monitoring


   - Include workflow age distribution, stuck workflow detection, and resource usage


   - Implement agent performance comparison visualizations


   - Create operational runbook integration within dashboards



3. **Distributed Tracing**


   - Implement OpenTelemetry for cross-agent workflow correlation


   - Trace workflow execution across multiple agents and stages


   - Correlate performance issues with specific workflow steps


   - Enable root cause analysis for complex workflow failures



## Expected Deliverables



1. **Health Check System**


   - Custom health endpoints for Rex, Cleo, and Tess agents


   - Kubernetes probe configurations and health monitoring


   - Synthetic health check workflows with automated execution


   - Dead letter queue implementation for failed health checks



2. **Stuck Workflow Detection**


   - Prometheus rules for stuck workflow detection


   - Automated analysis scripts for stuck reason identification


   - AlertManager configuration with escalation policies


   - Integration with PagerDuty, Slack, and incident response tools



3. **Resource Monitoring Platform**


   - Pod-level resource metric collection system


   - PVC usage monitoring with capacity planning


   - GitHub API rate limit tracking and quota management


   - Custom metrics for workflow-specific resource consumption



4. **Automated Cleanup System**


   - CronJob-based cleanup with graduated retention policies


   - S3/MinIO archival system with compression and metadata


   - PVC and resource cleanup automation


   - Compliance-aware retention and audit logging



5. **Monitoring Dashboards**


   - Grafana dashboards for operational monitoring


   - SLO/SLI definitions and tracking


   - Performance comparison and trend analysis


   - Operational runbook integration



6. **Operational Tools**


   - kubectl plugin for workflow diagnostics


   - Workflow pause/resume CLI tools


   - Bulk operation APIs for workflow management


   - Migration tools for controller upgrades

## Acceptance Criteria



- Health checks detect agent failures within 5 minutes


- Stuck workflow alerts fire correctly for workflows >6 hours in same phase


- Resource monitoring captures all pod-level metrics accurately


- PVC usage alerts trigger at 80% capacity threshold


- GitHub API rate limit tracking works for all agents


- Automated cleanup respects retention policies and exemptions


- Workflow archival to S3/MinIO completes before deletion


- Grafana dashboards display real-time operational metrics


- Alert routing reaches appropriate notification channels


- Synthetic health checks validate system functionality every 6 hours

## Quality Standards



- Follow Prometheus and Grafana best practices for metric naming and dashboards


- Implement proper error handling and retry logic for all monitoring components


- Use structured logging with correlation IDs for all monitoring activities


- Ensure monitoring system performance doesn't impact workflow execution


- Maintain backward compatibility with existing monitoring infrastructure


- Document all metrics, alerts, and operational procedures


- Test all failure scenarios and recovery procedures

## Performance Requirements



- Health checks complete within 30 seconds for normal cases


- Stuck workflow detection processes all workflows within 1 minute


- Resource metrics collection completes within 2 minutes


- Cleanup operations don't impact running workflows


- Dashboard queries respond within 5 seconds


- Alert firing latency under 1 minute for critical alerts

## Security Considerations



- Secure access to monitoring dashboards with proper authentication


- Protect sensitive workflow data in archives and logs


- Implement audit logging for all cleanup and administrative operations


- Use least-privilege access for monitoring service accounts


- Encrypt archived workflow data in S3/MinIO


- Sanitize sensitive information before logging or alerting



## Resources



- Prometheus operator documentation for custom rules and metrics


- Grafana dashboard development best practices


- Argo Workflows metrics and monitoring guides


- Kubernetes resource monitoring with metrics-server


- OpenTelemetry implementation patterns


- AWS S3 and MinIO archival configurations


- AlertManager routing and escalation policies

Focus on building a robust, scalable monitoring system that proactively identifies issues in long-running workflows while maintaining excellent performance and operational visibility.