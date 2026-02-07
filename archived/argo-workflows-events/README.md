# Archived: Argo Workflows & Argo Events

**Archived on:** 2026-02-07
**Reason:** Replaced with different workflow orchestration functionality

## What's Here

This directory contains the complete Argo Workflows and Argo Events infrastructure that was previously deployed via ArgoCD. Files are preserved in their original directory structure for reference.

### ArgoCD Applications (`applications/platform/`)
- `argo-workflows.yaml` - Argo Workflows Helm chart (v0.45.21) deployed to `automation` namespace
- `argo-events.yaml` - Argo Events Helm chart (v2.4.16) deployed to `automation` namespace
- `cto-workflows.yaml` - CTO WorkflowTemplates and CronWorkflows deployed to `cto` namespace
- `github-webhooks.yaml` - EventSources and Sensors deployed to `automation` namespace

### Workflow Manifests (`manifests/argo-workflows/`)
- `play-workflow-template.yaml` - Multi-agent Play Workflow (Rex -> Cleo -> Tess)
- `play-project-workflow-template.yaml` - Full Project Play Workflow (sequential task execution)
- `project-intake-template.yaml` - Project intake workflow (was already deprecated)
- `log-scanner-cronworkflow.yaml` - Hourly log scanning via Healer
- `controller-templates-pm-configmap.yaml` - Controller templates ConfigMap for PM
- `morgan-mcp-config.yaml` - Morgan MCP tool configuration

### Event Sources & Sensors (`manifests/argo-workflows/eventsources/` and `sensors/`)
- `eventbus.yaml` - NATS JetStream EventBus (3 replicas)
- `github-eventsource.yaml` - GitHub organization webhook receiver (port 12000)
- `play-workflow-pr-merged-sensor.yaml` - PR merge handler (intake + task completion)
- `stitch-pr-review-sensor.yaml` - Stitch code review trigger
- `agent-mention-sensor.yaml` - @agent mention handler
- `ci-failure-button-sensor.yaml` - CI failure remediation button creator
- `remediation-button-sensor.yaml` - Remediation button click handler

### RBAC (`manifests/rbac/`)
- `argo-workflow-permissions.yaml` - Roles/RoleBindings for argo-workflow SA
- `argo-events-cto.yaml` - Cross-namespace access for argo-events-sa
- `github-webhooks-networking.yaml` - Service + HTTPRoute RBAC for eventsource

### Helm Chart Templates (`charts/cto-app/templates/argo-workflows/`)
- `configmap.yaml`, `controller.yaml`, `crd.yaml`, `server.yaml` - Local dev Argo Workflows deployment
