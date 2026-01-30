# Subtask 8.2: Configure Service Discovery and Network Infrastructure

**Parent Task:** Configure Kubernetes Deployments (Bolt - Kubernetes)
**Agent:** bolt | **Language:** yaml

## Description

Set up ClusterIP services, ingress controller, and network policies for secure service communication

## Details

Create ClusterIP service manifests for internal service discovery. Configure ingress controller (nginx/traefik) with proper routing rules for external access. Implement network policies to isolate services and control traffic flow between pods. Set up persistent volume claims for stateful components like databases.

## Dependencies

task-8.1

## Acceptance Criteria

- [ ] Subtask requirements implemented
- [ ] Parent task requirements still satisfied

## Resources

- Parent task: `.tasks/docs/task-8/prompt.md`
- PRD: `.tasks/docs/prd.md`
