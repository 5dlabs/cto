# Task 8: Monitoring and Observability Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 8.

## Goal

Deploy Prometheus, Grafana, and logging stack for system observability

## Requirements

1. Install Prometheus operator and Prometheus instance\n2. Deploy Grafana with AlertHub dashboards\n3. Set up log aggregation with Loki or ELK\n4. Configure alert rules for SLA monitoring\n5. Create service monitors for all components

## Acceptance Criteria

Prometheus collecting metrics from all services, Grafana dashboards loading, alerts firing for test conditions

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-8): Monitoring and Observability Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
