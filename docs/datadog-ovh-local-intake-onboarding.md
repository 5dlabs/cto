# Datadog OVH + Local Intake Onboarding

This runbook implements phase 1 onboarding for:
- OVH Kubernetes cluster (primary)
- Local intake implementation (kind/local validation)

It keeps the existing OpenTelemetry + Loki/Grafana path intact while enabling Datadog infrastructure metrics, logs, and APM for intake-critical services.

## Trial Feature Matrix (OVH + Local Intake)

| Datadog capability | Phase 1 status | Why |
| --- | --- | --- |
| Kubernetes infrastructure monitoring | Enable now | Covered by Datadog Agent + Cluster Agent features in `DatadogAgent` CR. |
| Kubernetes logs | Enable now | Enabled with Agent log collection and workload Autodiscovery annotations for intake-critical pods. |
| APM (Single Step Instrumentation) | Enable now | Enabled with scoped targets for known intake workloads to avoid broad blast radius. |
| Continuous profiling | Enable now (APM dependent) | Added through APM tracer config (`DD_PROFILING_ENABLED=auto`) on scoped targets. |
| LLM Observability | Phase 2 candidate | Relevant for intake workflows, but needs explicit runtime SDK-level validation per service before enabling broadly. |
| Browser RUM / Product analytics | Not in this phase | Requires browser app IDs and frontend product decisions; separate from Kubernetes intake core. |
| Security / ASM / AAP | Phase 2 candidate | Valuable, but requires language/runtime-specific rollout and policy decisions. |
| Synthetic monitoring | Phase 2 candidate | Not required for first Kubernetes observability baseline. |

## GitOps Layout

- Argo app for operator install: `infra/gitops/applications/observability/datadog-operator.yaml`
- Argo app for DatadogAgent CR: `infra/gitops/applications/observability/datadog-agent.yaml`
- DatadogAgent manifest source: `infra/gitops/manifests/datadog/datadog-agent.yaml`
- Kustomize root for Datadog manifests: `infra/gitops/manifests/datadog/kustomization.yaml`

## Intake-Critical Workloads Covered in Phase 1

- `controller` (`cto-controller`)
- `pm` (`pm-server`)
- `tools` (`tools-mcp`)
- `linear-bridge`
- `discord-bridge`

These are selected because they represent intake orchestration, intake decisioning, tool execution, and intake-channel bridge traffic.

### APM scope statement (required)

Single Step Instrumentation in this phase applies only to:
- `pm-server` (`app.kubernetes.io/name=pm`, namespace `cto`)
- `linear-bridge` (`app=linear-bridge`, namespace `bots`)
- `discord-bridge` (`app=discord-bridge`, namespace `bots`)

Detected intake workloads not instrumented with SSI yet:
- `cto-controller` (Rust runtime; SSI support is currently oriented to Node/Python/Java/.NET/Ruby/PHP)
- `tools-mcp` (Rust runtime; covered for logs/metrics/tags in this phase, not SSI)

## OTEL Coexistence Strategy

Phase 1 intentionally does not remove or replace:
- `otel-collector` Argo app
- Loki/Grafana log flow
- Existing `OTEL_*` env-based emission from intake/controller templates

Datadog is added in parallel for validation and correlation. Migration decisions can happen later, based on signal quality and cost.

## Datadog Agent Configuration Notes

- Datadog site: `us5.datadoghq.com`
- Namespace: `datadog-agent`
- Cluster name in phase 1 manifest: `cto-ovh`
- Log collection starts with targeted annotations (not collect-all)
- APM instrumentation uses explicit targets with selectors

## Local Intake Validation Mode

Use the same Helm chart values plus Datadog annotations/tags in local clusters by setting:
- `datadog.enabled=true`
- `datadog.tags.env=local`

Keep `DatadogAgent.spec.global.clusterName` distinct per local cluster context.

### What you need to do in a terminal
```bash
# 1) Confirm datadog-operator chart version for reproducible installs
helm repo add datadog https://helm.datadoghq.com
helm repo update
helm search repo datadog/datadog-operator --versions | head -n 20

# 2) Install the operator (if not managed via Argo yet)
helm upgrade --install datadog-operator datadog/datadog-operator \
  --namespace datadog-agent \
  --create-namespace \
  --version 2.20.0

# 3) Export API key from Datadog (do not commit it)
export DD_API_KEY=REPLACE_ME

# 4) Create the cluster secret in datadog-agent namespace
kubectl -n datadog-agent create secret generic datadog-secret \
  --from-literal api-key="$DD_API_KEY"

# 5) Apply DatadogAgent CR from this repo
kubectl apply -f infra/gitops/manifests/datadog/datadog-agent.yaml

# 6) Restart scoped workloads to activate SSI/APM injection
kubectl rollout restart deployment/cto-controller -n cto
kubectl rollout restart deployment/pm-server -n cto
kubectl rollout restart deployment/tools-mcp -n cto
kubectl rollout restart deployment/linear-bridge -n bots
kubectl rollout restart deployment/discord-bridge -n bots
```

### What you need to do in a terminal
```bash
# Local intake validation overlay (kind/local)
helm upgrade --install cto ./infra/charts/cto \
  --namespace cto \
  --create-namespace \
  -f ./infra/charts/cto/values.yaml \
  -f ./infra/charts/cto/values.datadog-local-intake.yaml
```

## Verification Checkpoints

### What you need to do in a terminal
```bash
# Agent and cluster-agent health
kubectl get pods -n datadog-agent

# Check intake service pods after restart
kubectl get pods -n cto
kubectl get pods -n bots

# Spot check Datadog Agent auth/status logs
kubectl logs -n datadog-agent -l app.kubernetes.io/component=agent --tail=100

# Confirm no breakage in existing OTEL collector path
kubectl get pods -n observability
kubectl logs -n observability deploy/otel-collector --tail=100
```

## Rollback

### What you need to do in a terminal
```bash
# Disable instrumentation pressure by scaling back to previous workload state
kubectl rollout undo deployment/cto-controller -n cto
kubectl rollout undo deployment/pm-server -n cto
kubectl rollout undo deployment/tools-mcp -n cto
kubectl rollout undo deployment/linear-bridge -n bots
kubectl rollout undo deployment/discord-bridge -n bots

# Remove DatadogAgent CR if needed
kubectl delete -f infra/gitops/manifests/datadog/datadog-agent.yaml
```

## Troubleshooting Reference

- Datadog onboarding setup guide: https://docs.datadoghq.com/agentic_onboarding/setup
