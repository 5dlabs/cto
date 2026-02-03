# Subtask 10.1: Deploy Prometheus

## Parent Task
Task 10

## Agent
prometheus-deployer

## Parallelizable
Yes

## Description
Deploy Prometheus for metrics collection.

## Details
- Install Prometheus Operator
- Configure ServiceMonitors for all services
- Set up recording rules
- Configure alerting rules
- Set up remote write (if using Cortex/Thanos)

## Deliverables
- `prometheus-operator.yaml`
- `service-monitors/`

## Acceptance Criteria
- [ ] Prometheus scraping all services
- [ ] Alerts firing correctly
