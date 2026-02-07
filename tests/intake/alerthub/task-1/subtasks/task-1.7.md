# Subtask 1.7: Deploy Monitoring Stack

## Parent Task
Task 1

## Agent
prometheus-deployer

## Parallelizable
Yes

## Description
Deploy Prometheus, Grafana, and alerting infrastructure for observability.

## Details
- Install Prometheus Operator
- Configure ServiceMonitor for all services
- Set up AlertManager with notification channels
- Create Grafana dashboards for key metrics
- Configure recording rules for complex queries
- Set up Loki for log aggregation

## Deliverables
- `prometheus-operator.yaml` - Operator deployment
- `prometheus-rules.yaml` - Alert rules
- `grafana-dashboards.yaml` - Dashboard configs
- `alertmanager-config.yaml` - Notification routing
- `loki-config.yaml` - Log aggregation

## Acceptance Criteria
- [ ] Prometheus is scraping all service metrics
- [ ] AlertManager routes alerts correctly
- [ ] Grafana dashboards display metrics
- [ ] Loki is ingesting logs from all pods

## Testing Strategy
- Trigger a test alert and verify notification
- Query Prometheus for service metrics
- View logs in Loki
- Verify dashboard data freshness
