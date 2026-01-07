# Task 14: Setup Monitoring and Observability

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 14.

## Goal

Deploy Prometheus, Grafana, and configure dashboards for monitoring AlertHub services

## Requirements

1. Deploy Prometheus using kube-prometheus-stack:
   helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
   helm install prometheus prometheus-community/kube-prometheus-stack -n monitoring --create-namespace

2. Configure ServiceMonitors for AlertHub services:
   - Create ServiceMonitor for notification-router (scrape /metrics on port 8080)
   - Create ServiceMonitor for integration-service (scrape /metrics on port 3000)
   - Create ServiceMonitor for admin-api (scrape /metrics on port 8080)

3. Create Grafana dashboards:
   - Dashboard 1: Service Health Overview
     - Panels: Service uptime, request rate, error rate, latency (p50, p95, p99)
     - Filters: Service, time range
   - Dashboard 2: Notification Metrics
     - Panels: Notifications submitted, delivered, failed, delivery time, channel breakdown
   - Dashboard 3: Infrastructure Metrics
     - Panels: PostgreSQL connections, Redis memory, Kafka lag, RabbitMQ queue depth
   - Dashboard 4: Resource Usage
     - Panels: CPU usage, memory usage, network I/O per service

4. Configure Prometheus alerts:
   - HighErrorRate: Alert when error rate > 5% for 5 minutes
   - HighLatency: Alert when p95 latency > 500ms for 5 minutes
   - ServiceDown: Alert when service is down for 1 minute
   - HighQueueDepth: Alert when RabbitMQ queue depth > 10000
   - DatabaseConnectionPoolExhausted: Alert when PostgreSQL connections > 90%

5. Setup Alertmanager:
   - Configure Slack integration for alerts
   - Configure email integration for critical alerts
   - Set up alert routing (critical -> Slack + email, warning -> Slack)

6. Configure log aggregation:
   - Deploy Loki for log aggregation
   - Configure Promtail to scrape logs from all pods
   - Add Loki data source to Grafana
   - Create log dashboard with filters (service, level, trace ID)

7. Setup distributed tracing (optional):
   - Deploy Jaeger or Tempo
   - Configure OpenTelemetry in services
   - Add tracing data source to Grafana
   - Create trace dashboard

## Acceptance Criteria

1. Verify Prometheus is scraping metrics from all services
2. Verify Grafana dashboards display data correctly
3. Test alert rules by triggering conditions (e.g., stop a service)
4. Verify alerts are sent to Slack and email
5. Test log aggregation (search for specific log entries)
6. Verify trace data is collected (if enabled)
7. Test dashboard filters and time range selection
8. Verify metrics are retained for configured period (default 15 days)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-14): Setup Monitoring and Observability`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 8, 9, 10
