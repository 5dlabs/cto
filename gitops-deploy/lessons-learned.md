# Lessons Learned - GitOps Deployment

This file is maintained by the **Hardener Agent (Droid)**. It captures what the Deployer Agent (Claude) had to do manually, and tracks the GitOps config fixes.

---

## Format

### [GITOPS-XXX] Short Description

**Date**: YYYY-MM-DD
**App**: ArgoCD application name
**Observation**: What Claude had to do manually
**Root Cause**: Why the config didn't handle it
**Codified Fix**: What was changed
**Files Modified**: Which files
**Status**: pending | fixed | wont-fix

---

## Issues

### [GITOPS-001] Workflow templates applied before Argo CRDs

**Date**: 2026-01-20
**App**: cto-workflows
**Observation**: WorkflowTemplates/EventSources/Sensors can fail on initial sync when Argo Workflows/Events CRDs are not yet established.
**Root Cause**: No explicit sync-wave ordering between Argo CRD apps and the cto-workflows manifests.
**Codified Fix**: Added sync-wave ordering for argo-workflows/argo-events and delayed cto-workflows; enabled CreateNamespace and SkipDryRunOnMissingResource for cto-workflows.
**Files Modified**: `infra/gitops/applications/platform/argo-workflows.yaml`, `infra/gitops/applications/platform/argo-events.yaml`, `infra/gitops/applications/platform/cto-workflows.yaml`
**Status**: fixed

### [GITOPS-002] Secrets-dependent apps sync before ExternalSecrets

**Date**: 2026-01-20
**App**: external-dns, alertmanager
**Observation**: external-dns and alertmanager can come up degraded when required secrets are not yet created by ExternalSecrets.
**Root Cause**: No sync-wave ordering to ensure ExternalSecrets config completes before these workloads.
**Codified Fix**: Added sync-wave annotations to delay external-dns and alertmanager until after external-secrets-config.
**Files Modified**: `infra/gitops/applications/secrets/external-dns.yaml`, `infra/gitops/applications/observability/alertmanager.yaml`
**Status**: fixed

### [GITOPS-003] Database CRs apply before operator CRDs

**Date**: 2026-01-20
**App**: database-instances
**Observation**: Test database resources can fail to sync when CloudNativePG/Redis operator CRDs are not yet established.
**Root Cause**: database-instances had no sync-wave ordering and no CRD dry-run bypass, so Argo validated CRs before operators were ready.
**Codified Fix**: Added sync-wave ordering to run after operators and enabled SkipDryRunOnMissingResource.
**Files Modified**: `infra/gitops/applications/workloads/database-instances.yaml`
**Status**: fixed

### [GITOPS-004] CTO core app applied before Argo Events/Workflows CRDs

**Date**: 2026-01-20
**App**: cto
**Observation**: CTO chart can go Degraded when Argo Events/Workflows CRDs are not yet established for Sensors/EventSources/WorkflowTemplates.
**Root Cause**: CTO application had no sync-wave ordering and did not skip CRD dry-run validation.
**Codified Fix**: Added sync-wave ordering after Argo Workflows/Events and enabled SkipDryRunOnMissingResource.
**Files Modified**: `infra/gitops/applications/workloads/cto.yaml`
**Status**: fixed

### [GITOPS-005] ExternalSecrets config dry-run fails before CRDs are ready

**Date**: 2026-01-20
**App**: external-secrets-config
**Observation**: ExternalSecret/ClusterSecretStore manifests can fail validation on first sync if CRDs are not yet established.
**Root Cause**: external-secrets-config did not skip dry-run for missing CRDs.
**Codified Fix**: Enabled SkipDryRunOnMissingResource to tolerate CRD establishment lag.
**Files Modified**: `infra/gitops/applications/secrets/external-secrets-config.yaml`
**Status**: fixed

### [GITOPS-006] Headscale workloads fail if namespace is missing

**Date**: 2026-01-20
**App**: headscale, tailscale-subnet-router
**Observation**: Headscale and subnet-router resources can fail to sync when the `headscale` namespace has not been created (namespace app disabled).
**Root Cause**: Applications relied on a separate namespace app and did not set `CreateNamespace=true`.
**Codified Fix**: Enabled `CreateNamespace=true` for headscale and tailscale-subnet-router applications.
**Files Modified**: `infra/gitops/applications/networking/headscale.yaml`, `infra/gitops/applications/networking/tailscale-subnet-router.yaml`
**Status**: fixed

### [GITOPS-007] Gateway API CRDs not ordered before dependent resources

**Date**: 2026-01-20
**App**: gateway-api-crds
**Observation**: Gateway API resources can fail validation if CRDs are not established before dependent apps sync.
**Root Cause**: gateway-api-crds application had no explicit sync-wave ordering for early CRD installation.
**Codified Fix**: Added sync-wave `-5` to deploy Gateway API CRDs before dependent resources.
**Files Modified**: `infra/gitops/applications/platform/gateway-api.yaml`
**Status**: fixed

### [GITOPS-008] CTO Cloudflare Tunnel CRs can sync before operator CRDs

**Date**: 2026-01-20
**App**: cto
**Observation**: CTO includes Cloudflare Tunnel CRs that can apply before the cloudflare-operator CRDs when both apps sync in the same wave, resulting in initial Degraded status.
**Root Cause**: cloudflare-operator and cto shared the same sync-wave, so CRDs were not guaranteed to exist before CTO resources applied.
**Codified Fix**: Moved cloudflare-operator to an earlier sync-wave to ensure CRDs are established before CTO sync.
**Files Modified**: `infra/gitops/applications/secrets/cloudflare-operator.yaml`
**Status**: fixed

### [GITOPS-009] Disabled optional apps show Missing/OutOfSync in ArgoCD

**Date**: 2026-01-20
**App**: buildkit, headscale, headscale-namespace, tailscale-subnet-router
**Observation**: Optional apps that are intentionally disabled still appear as Missing/OutOfSync because they are included in app-of-apps directories.
**Root Cause**: Disabled Application manifests were still included in directory-based app-of-apps, so ArgoCD continuously tracked them.
**Codified Fix**: Excluded disabled optional apps from the root app-of-apps and networking app-of-apps until explicitly enabled.
**Files Modified**: `infra/gitops/app-of-apps.yaml`, `infra/gitops/applications/networking/networking-apps.yaml`
**Status**: fixed

### [GITOPS-010] GitHub webhooks can sync before Argo Events CRDs/namespace

**Date**: 2026-01-20
**App**: github-webhooks
**Observation**: EventSource/Sensor resources can fail initial sync when the Argo Events CRDs or the `automation` namespace are not yet established, requiring a manual re-sync.
**Root Cause**: github-webhooks lacked explicit sync-wave ordering and did not enable `CreateNamespace` or `SkipDryRunOnMissingResource` for Argo Events CRDs.
**Codified Fix**: Added sync-wave ordering and enabled `CreateNamespace` + `SkipDryRunOnMissingResource` for github-webhooks.
**Files Modified**: `infra/gitops/applications/platform/github-webhooks.yaml`
**Status**: fixed

### [GITOPS-011] AI model CRs can fail before operator CRDs are established

**Date**: 2026-01-20
**App**: kubeai-models, ollama-models, llamastack-distributions
**Observation**: KubeAI/Ollama/LlamaStack custom resources can fail initial sync when their operator CRDs are not yet established, requiring a manual re-sync.
**Root Cause**: The model CR Applications did not skip dry-run validation, so ArgoCD validated custom resources before the operator CRDs were ready.
**Codified Fix**: Enabled `SkipDryRunOnMissingResource=true` on the model CR Applications to tolerate CRD establishment lag.
**Files Modified**: `infra/gitops/applications/ai-models/kubeai-models.yaml`, `infra/gitops/applications/ai-models/ollama-models.yaml`, `infra/gitops/applications/ai-models/llamastack-distributions.yaml`
**Status**: fixed

### [GITOPS-012] Fluent Bit namespace mismatch prevents namespace creation

**Date**: 2026-01-20
**App**: fluent-bit
**Observation**: Fluent Bit resources can fail to sync if the `telemetry` namespace does not exist, because the chart deploys into `telemetry` while the Application destination was `observability`.
**Root Cause**: `namespaceOverride` targets `telemetry`, but `CreateNamespace=true` only creates the destination namespace (`observability`).
**Codified Fix**: Aligned the Application destination namespace to `telemetry` so `CreateNamespace` creates the correct namespace.
**Files Modified**: `infra/gitops/applications/observability/fluent-bit.yaml`
**Status**: fixed

### [GITOPS-013] Cert-manager CRs can race CRD establishment

**Date**: 2026-01-20
**App**: cert-manager-config
**Observation**: ClusterIssuer/Certificate resources can fail dry-run validation on first sync if cert-manager CRDs are not yet fully established.
**Root Cause**: cert-manager-config did not skip dry-run validation for missing CRDs.
**Codified Fix**: Enabled `SkipDryRunOnMissingResource=true` to tolerate CRD establishment lag.
**Files Modified**: `infra/gitops/applications/platform/cert-manager-config.yaml`
**Status**: fixed

### [GITOPS-014] Tenant CRD can race dependent resources

**Date**: 2026-01-21
**App**: tenant-crd
**Observation**: Tenant custom resources can fail initial sync if the Tenant CRD is applied concurrently, requiring a manual re-sync once CRDs register.
**Root Cause**: tenant-crd lacked explicit sync-wave ordering and retry/backoff to ensure CRDs land early and tolerate registration lag.
**Codified Fix**: Added sync-wave `-4` and standard sync options/retry to install the Tenant CRD ahead of dependent resources.
**Files Modified**: `infra/gitops/applications/platform/tenant-crd.yaml`
**Status**: fixed

### [GITOPS-015] CloudNativePG operator lacked explicit sync-wave ordering

**Date**: 2026-01-21
**App**: cloudnative-pg-operator
**Observation**: Database CRs can sync before CloudNativePG CRDs/controllers on first reconcile, requiring a manual re-sync once the operator is ready.
**Root Cause**: cloudnative-pg-operator had no sync-wave annotation, so it could land in the same wave as dependent workloads.
**Codified Fix**: Added sync-wave `1` to align with other database operators and ensure ordering ahead of database-instances.
**Files Modified**: `infra/gitops/applications/operators/cloudnative-pg-operator.yaml`
**Status**: fixed

### [GITOPS-016] Blackbox exporter lacked retry/backoff for transient sync errors

**Date**: 2026-01-21
**App**: blackbox-exporter
**Observation**: Transient chart fetch/upgrade issues can leave blackbox-exporter OutOfSync and require a manual re-sync.
**Root Cause**: Application syncPolicy lacked retry/backoff and standard prune/ignore sync options for resilience.
**Codified Fix**: Added retry/backoff and standard syncOptions (prune propagation + respect ignore differences).
**Files Modified**: `infra/gitops/applications/observability/blackbox-exporter.yaml`
**Status**: fixed

### [GITOPS-017] Alertmanager lacked retry/backoff for transient sync errors

**Date**: 2026-01-21
**App**: alertmanager
**Observation**: Alertmanager can remain OutOfSync after transient Helm fetch/apply issues or secret timing, requiring a manual re-sync.
**Root Cause**: Application syncPolicy lacked retry/backoff and standard prune ordering options.
**Codified Fix**: Added retry/backoff plus prune ordering options to improve sync resilience.
**Files Modified**: `infra/gitops/applications/observability/alertmanager.yaml`
**Status**: fixed

### [GITOPS-018] Fluent Bit lacked retry/backoff and standard sync options

**Date**: 2026-01-21
**App**: fluent-bit
**Observation**: Fluent Bit can stay OutOfSync after transient chart errors or drift, requiring a manual re-sync.
**Root Cause**: Application syncPolicy did not include retry/backoff or standard prune/ignore sync options.
**Codified Fix**: Added retry/backoff plus `PruneLast` and `RespectIgnoreDifferences` to improve sync resiliency.
**Files Modified**: `infra/gitops/applications/observability/fluent-bit.yaml`
**Status**: fixed

### [GITOPS-019] ArgoCD Image Updater lacked ordering and resilient sync options

**Date**: 2026-01-21
**App**: argocd-image-updater
**Observation**: argocd-image-updater appeared OutOfSync and can require manual re-sync when it comes up before core ArgoCD components are fully ready.
**Root Cause**: No sync-wave ordering and minimal syncOptions, so ArgoCD had no ordering guarantee and fewer resiliency options for transient apply issues.
**Codified Fix**: Added sync-wave ordering and standard sync options (prune ordering, respect ignore differences, and skip dry-run on missing resources).
**Files Modified**: `infra/gitops/applications/platform/argocd-image-updater.yaml`
**Status**: fixed

### [GITOPS-020] CRD-heavy operators lacked dry-run bypass and prune ordering

**Date**: 2026-01-21
**App**: argo-workflows, arc-controller, clickhouse-operator, jaeger-operator
**Observation**: CRD-heavy charts can fail initial sync or remain OutOfSync when ArgoCD dry-runs custom resources before the CRDs are fully registered, requiring manual re-syncs.
**Root Cause**: Applications did not enable `SkipDryRunOnMissingResource` (and some lacked prune ordering), so ArgoCD validated CRDs/CRs too early.
**Codified Fix**: Added `SkipDryRunOnMissingResource=true` and `PruneLast=true` (plus prune propagation for ClickHouse) to improve CRD sync reliability.
**Files Modified**: `infra/gitops/applications/platform/argo-workflows.yaml`, `infra/gitops/applications/platform/arc-controller.yaml`, `infra/gitops/applications/operators/clickhouse-operator.yaml`, `infra/gitops/applications/operators/jaeger-operator.yaml`
**Status**: fixed

### [GITOPS-021] Cert-manager and ExternalSecrets ordering/CRD dry-run gaps

**Date**: 2026-01-21
**App**: cert-manager, cert-manager-config, external-secrets, external-secrets-config
**Observation**: Cert-manager and ExternalSecrets could still race CRD establishment and dependent resources when scheduled in later sync waves, increasing the chance of manual re-syncs on first install.
**Root Cause**: Sync-wave ordering was later than the common CRD/secret dependency chain and the operator charts did not skip dry-run validation during CRD registration.
**Codified Fix**: Moved cert-manager earlier to sync-wave `-5`, shifted cert-manager-config/external-secrets/external-secrets-config to `-4/-3/-2`, and enabled `SkipDryRunOnMissingResource=true` for cert-manager and external-secrets.
**Files Modified**: `infra/gitops/applications/platform/cert-manager.yaml`, `infra/gitops/applications/platform/cert-manager-config.yaml`, `infra/gitops/applications/secrets/external-secrets.yaml`, `infra/gitops/applications/secrets/external-secrets-config.yaml`
**Status**: fixed

### [GITOPS-022] Operator CRDs lacked dry-run bypass for AI/database stacks

**Date**: 2026-01-21
**App**: cloudnative-pg-operator, redis-operator, kubeai, ollama-operator, llamastack-operator
**Observation**: Operator charts that install CRDs can fail initial sync when ArgoCD dry-runs custom resources before CRDs are fully registered, leading to OutOfSync and manual re-syncs for dependent stacks.
**Root Cause**: These operator Applications did not enable `SkipDryRunOnMissingResource`, so ArgoCD validated CRDs too early during initial registration.
**Codified Fix**: Enabled `SkipDryRunOnMissingResource=true` for the AI/database operator Applications to tolerate CRD establishment lag.
**Files Modified**: `infra/gitops/applications/operators/cloudnative-pg-operator.yaml`, `infra/gitops/applications/operators/redis-operator.yaml`, `infra/gitops/applications/operators/kubeai.yaml`, `infra/gitops/applications/operators/ollama-operator.yaml`, `infra/gitops/applications/operators/llamastack-operator.yaml`
**Status**: fixed

### [GITOPS-023] ExternalSecrets config applied before core namespaces exist

**Date**: 2026-01-21
**App**: external-secrets-config
**Observation**: ExternalSecrets targeting `cto` and `external-dns` namespaces can fail initial sync when those namespaces do not exist yet, leaving external-secrets-config degraded and external-dns without credentials.
**Root Cause**: Core namespaces were created only by later apps (CTO chart / external-dns) and external-secrets-config synced before the observability namespace, causing namespace race conditions.
**Codified Fix**: Added a platform-namespaces app to create `cto` and `external-dns` namespaces early, made CTO chart namespace creation optional (disabled in GitOps), and moved external-secrets-config to sync-wave `0` to run after namespace apps.
**Files Modified**: `infra/gitops/applications/platform/platform-namespaces.yaml`, `infra/cluster-config/cto-namespace.yaml`, `infra/cluster-config/external-dns-namespace.yaml`, `infra/charts/cto/templates/namespace.yaml`, `infra/charts/cto/values.yaml`, `infra/gitops/applications/workloads/cto.yaml`, `infra/gitops/applications/secrets/external-secrets-config.yaml`
**Status**: fixed

### [GITOPS-024] GitHub webhooks can sync before workflow templates

**Date**: 2026-01-21
**App**: github-webhooks
**Observation**: Argo Events Sensors reference WorkflowTemplates that may not exist yet when github-webhooks and cto-workflows sync in the same wave, leading to failed triggers or manual re-syncs.
**Root Cause**: github-webhooks shared the same sync-wave as cto-workflows, so ordering between templates and sensors was not guaranteed.
**Codified Fix**: Moved github-webhooks to sync-wave `1` to ensure workflow templates land before sensors.
**Files Modified**: `infra/gitops/applications/platform/github-webhooks.yaml`
**Status**: fixed

### [GITOPS-025] AI ingress can race Gateway API CRD registration

**Date**: 2026-01-21
**App**: ai-ingress
**Observation**: HTTPRoute/GRPCRoute resources can fail initial sync if Gateway API CRDs are still registering, requiring a manual re-sync.
**Root Cause**: ai-ingress did not skip dry-run validation for missing CRDs.
**Codified Fix**: Enabled `SkipDryRunOnMissingResource=true` to tolerate CRD registration lag.
**Files Modified**: `infra/gitops/applications/ai-models/ai-ingress.yaml`
**Status**: fixed

### [GITOPS-026] Core apps lacked retry/backoff for transient sync errors

**Date**: 2026-01-21
**App**: cloudflare-operator, cto-workflows, prometheus, loki, kube-state-metrics
**Observation**: Transient chart fetch/apply errors can leave foundational apps OutOfSync and require a manual re-sync.
**Root Cause**: Applications did not configure syncPolicy retry/backoff.
**Codified Fix**: Added retry/backoff to improve GitOps resiliency for the operator, workflows, and observability stack apps.
**Files Modified**: `infra/gitops/applications/secrets/cloudflare-operator.yaml`, `infra/gitops/applications/platform/cto-workflows.yaml`, `infra/gitops/applications/observability/prometheus.yaml`, `infra/gitops/applications/observability/loki.yaml`, `infra/gitops/applications/observability/kube-state-metrics.yaml`
**Status**: fixed

### [GITOPS-027] Namespace creation disabled for config apps

**Date**: 2026-01-21
**App**: cert-manager-config, argocd-image-updater
**Observation**: Config apps can fail initial sync if their target namespaces are not present, forcing manual namespace creation or re-syncs.
**Root Cause**: `CreateNamespace=false` prevented ArgoCD from creating the target namespaces.
**Codified Fix**: Enabled `CreateNamespace=true` to avoid namespace race failures.
**Files Modified**: `infra/gitops/applications/platform/cert-manager-config.yaml`, `infra/gitops/applications/platform/argocd-image-updater.yaml`
**Status**: fixed

### [GITOPS-028] Platform ingress fails when headscale namespace is absent

**Date**: 2026-01-21
**App**: platform-ingress
**Observation**: platform-ingress includes headscale Ingress/ConfigMap resources, which fail to sync when the `headscale` namespace is missing (headscale apps are disabled).
**Root Cause**: The `headscale` namespace was only created by a disabled headscale-namespace app, so no GitOps-managed namespace existed when platform-ingress applied.
**Codified Fix**: Added `headscale` namespace to the platform-namespaces app to ensure it exists before platform-ingress syncs.
**Files Modified**: `infra/cluster-config/headscale-namespace.yaml`, `infra/gitops/applications/platform/platform-namespaces.yaml`
**Status**: fixed

### [GITOPS-029] ExternalSecrets config references namespaces not created early

**Date**: 2026-01-21
**App**: external-secrets-config
**Observation**: ExternalSecrets targeting `observability`, `mayastor`, and `openbao` can fail initial sync when those namespaces are created later by their respective apps, leaving external-secrets-config degraded and requiring manual namespace creation/re-sync.
**Root Cause**: platform-namespaces only created `cto`, `external-dns`, and `headscale`, so other namespaces referenced by ExternalSecrets were missing during the external-secrets-config sync wave.
**Codified Fix**: Added `observability`, `mayastor`, and `openbao` namespaces to the platform-namespaces app and introduced a managed `openbao` namespace manifest for early provisioning.
**Files Modified**: `infra/gitops/applications/platform/platform-namespaces.yaml`, `infra/cluster-config/openbao-namespace.yaml`
**Status**: fixed

### [GITOPS-030] Argo webhook CA bundle drift keeps apps OutOfSync

**Date**: 2026-01-21
**App**: argo-events, argo-workflows, arc-controller
**Observation**: Argo Events/Workflows and ARC controller can remain OutOfSync after initial sync because their admission webhooks rotate CA bundles.
**Root Cause**: Webhook `caBundle` fields are mutated by controllers, but the ArgoCD Applications did not ignore those fields.
**Codified Fix**: Ignored webhook CA bundle fields via `jqPathExpressions` for Mutating/ValidatingWebhookConfiguration resources.
**Files Modified**: `infra/gitops/applications/platform/argo-events.yaml`, `infra/gitops/applications/platform/argo-workflows.yaml`, `infra/gitops/applications/platform/arc-controller.yaml`
**Status**: fixed

---

## Common Patterns

### Sync Wave Best Practices

| Category | Sync Wave | Examples |
|----------|-----------|----------|
| Namespaces | -10 | gpu-operator-namespace |
| CRDs | -5 | operator CRDs |
| Operators | -3 | cert-manager, external-secrets |
| Secrets | -2 | ClusterSecretStore, secrets |
| Services | 0 | Application deployments |
| Workloads | 1+ | Database instances, runners |

### Dependency Chain

```
cert-manager (-5)
    └── ClusterIssuer (-4)
external-secrets-operator (-4)
    └── ClusterSecretStore (-3)
        └── ExternalSecrets (-2)
operators (-3)
    └── CRDs installed
        └── CR instances (0+)
```

---

## Adding New Lessons

When Claude or Droid encounters a GitOps issue:

1. **Document it here** with the format above
2. **Fix the GitOps manifest** (sync-wave, health check, etc.)
3. **Update status** once the fix is verified
4. **Test** by re-syncing the application
