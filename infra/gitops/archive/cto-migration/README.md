# CTO Migration Archive

This directory contains the original ArgoCD Application manifests that were replaced by the unified CTO umbrella chart.

## Migration Date

December 2, 2025

## Archived Applications

The following 13 applications were consolidated into a single `cto.yaml` application:

### CTO Core Components

| Original File | Component | New Location |
|---------------|-----------|--------------|
| `controller.yaml` | Agent Platform Controller | `charts/cto` (dependency) |
| `tools.yaml` | MCP Tool Management Server | `charts/cto` (dependency) |
| `heal.yaml` | Self-Healing Platform Monitor | `charts/cto` (dependency via universal-app) |
| `heal-resources.yaml` | Heal RBAC, PVC, ConfigMap | `charts/cto/templates/heal/` |
| `openmemory.yaml` | AI Agent Memory System | `charts/cto` (dependency) |
| `workflow-templates.yaml` | Argo WorkflowTemplates | `charts/cto` (dependency) |

### Sensors

| Original File | Component | New Location |
|---------------|-----------|--------------|
| `bolt-sensor.yaml` | Bolt Production Deployment | `charts/cto/templates/sensors/bolt-sensors.yaml` |
| `bolt-monitor-sensor.yaml` | Bolt Monitor Daemon | `charts/cto/templates/sensors/bolt-sensors.yaml` |
| `bolt-preview-sensor.yaml` | Bolt Preview Deployment | `charts/cto/templates/sensors/bolt-sensors.yaml` |
| `stitch-sensor.yaml` | Stitch PR Review | `charts/cto/templates/sensors/stitch-sensor.yaml` |
| `ci-remediation-sensor.yaml` | CI Failure Remediation | `charts/cto/templates/sensors/ci-remediation-sensor.yaml` |

### Webhooks

| Original File | Component | New Location |
|---------------|-----------|--------------|
| `github-webhooks.yaml` | EventBus, EventSource | `charts/cto/templates/webhooks/` |
| `github-webhooks-networking.yaml` | HTTPRoute, Service | `charts/cto/templates/webhooks/networking.yaml` |

## New Unified Application

All components are now managed by:

```
infra/gitops/applications/cto.yaml
```

Which deploys the umbrella chart at:

```
infra/charts/cto/
```

## Rollback Instructions

If you need to rollback to the individual applications:

1. Delete the unified CTO application:
   ```bash
   kubectl delete application cto -n argocd
   ```

2. Move archived files back:
   ```bash
   mv infra/gitops/archive/cto-migration/*.yaml infra/gitops/applications/
   mv infra/gitops/archive/cto-migration/bolt-*.yaml infra/gitops/apps/
   ```

3. Recreate the cto/ subdirectory:
   ```bash
   mkdir -p infra/gitops/applications/cto
   mv infra/gitops/applications/controller.yaml infra/gitops/applications/cto/
   mv infra/gitops/applications/tools.yaml infra/gitops/applications/cto/
   mv infra/gitops/applications/heal.yaml infra/gitops/applications/cto/
   mv infra/gitops/applications/heal-resources.yaml infra/gitops/applications/cto/
   ```

4. Commit and push to trigger ArgoCD sync.

## Notes

- PVCs have `helm.sh/resource-policy: keep` annotation to prevent data loss
- The unified chart deploys to both `cto` and `automation` namespaces
- Image updater annotations are consolidated in the single application

