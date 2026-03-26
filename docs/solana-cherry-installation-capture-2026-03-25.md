# Solana Cherry Installation Capture (2026-03-25)

This document captures what is still observable from the existing Cherry deployment so we can reproduce it on Latitude.

## Capture Metadata

- Capture time: 2026-03-25 (local)
- Capture host repo: `5dlabs/cto`
- Active kube context during capture: `ovh-cluster`
- Control plane reachable in active context: `https://141.94.213.36:6443`

## Critical Availability Finding

ArgoCD still has application definitions for Cherry, but the Cherry cluster API endpoint is timing out:

- Cherry destination server: `https://84.32.103.99:6443`
- Repeated condition from Argo apps: `dial tcp 84.32.103.99:6443: i/o timeout`

Implication: we still have strong configuration evidence in Git + Argo specs, but live object introspection directly against Cherry is currently unavailable from this control plane.

## Cherry ArgoCD Application Inventory

From `applications.argoproj.io` in namespace `argocd`:

- `cherry-solana` -> namespace `solana`, source `https://github.com/5dlabs/cto`, path `skills/trader/k8s`, revision `main`
- `cherry-dashboards` -> namespace `observability`, source `https://github.com/5dlabs/cto`, path `skills/trader/k8s/observability/dashboards`, revision `main`
- `cherry-fluent-bit` -> namespace `observability`, Helm chart `fluent-bit` `0.47.7` + values ref from `https://github.com/5dlabs/cto` (`ref: values`)
- `cherry-grafana` -> namespace `observability`, Helm chart `grafana` `9.2.9` + values ref from `https://github.com/5dlabs/cto` (`ref: values`)
- `cherry-loki` -> namespace `observability`, Helm chart `loki` `6.16.0` + values ref from `https://github.com/5dlabs/cto` (`ref: values`)
- `cherry-prometheus` -> namespace `observability`, Helm chart `prometheus` `25.27.0` + values ref from `https://github.com/5dlabs/cto` (`ref: values`)

Status observed for all Cherry apps at capture time:

- Sync: `Unknown`
- Health: mostly `Healthy` in cached Argo status
- Last condition: Cherry API timeout as above

## Solana Stack As-Built (GitOps Source of Truth)

The Solana deployment source is `skills/trader/k8s` and currently includes these key manifests:

- `namespace.yaml`
- `agave-rpc.yaml`
- `questdb.yaml`
- `dex-indexer.yaml`
- `dex-api.yaml`
- `solana-exporter-svc.yaml`
- observability value files under `skills/trader/k8s/observability/*`

Observed observability files backing Cherry apps:

- `skills/trader/k8s/observability/loki-values.yaml`
- `skills/trader/k8s/observability/fluent-bit-values.yaml`
- `skills/trader/k8s/observability/prometheus-values.yaml`
- `skills/trader/k8s/observability/grafana-values.yaml`
- `skills/trader/k8s/observability/dashboards/cluster-logs.yaml`
- `skills/trader/k8s/observability/dashboards/node-exporter.yaml`
- `skills/trader/k8s/observability/dashboards/cilium-agent.yaml`
- `skills/trader/k8s/observability/dashboards/cilium-operator.yaml`
- `skills/trader/k8s/observability/dashboards/hubble-network.yaml`
- `skills/trader/k8s/observability/dashboards/solana-validator.yaml`

### `agave-rpc.yaml` (Validator + Yellowstone + exporter)

- Namespace: `solana`
- Workload: `Deployment/agave-rpc`
- Node pinning: `kubernetes.io/hostname: solana-rpc-01`
- Networking: `hostNetwork: true`
- Key validator args include:
  - `--rpc-port 8899`, `--full-rpc-api`, `--enable-rpc-transaction-history`
  - `--limit-ledger-size 50000000`
  - `--geyser-plugin-config /etc/yellowstone/config.json`
  - mainnet entrypoints + known validators
- Yellowstone plugin path:
  - `/opt/yellowstone/lib/libyellowstone_grpc_geyser.so`
- Yellowstone gRPC config map:
  - `ConfigMap/yellowstone-grpc-config`
  - gRPC bind `0.0.0.0:10000`
  - Prometheus bind `0.0.0.0:8999`
- Exporter sidecar:
  - image `ghcr.io/asymmetric-research/solana-exporter:v3.0.2`
  - metrics endpoint `:9179`
- Storage paths (hostPath):
  - `/var/mnt/accounts`
  - `/var/mnt/ledger`
  - `/var/mnt/yellowstone`

### `questdb.yaml` (Time-series store)

- Namespace: `questdb`
- Workload: `StatefulSet/questdb` (1 replica)
- Image: `questdb/questdb:8.2.3`
- Node pinning: `kubernetes.io/hostname: db-01`
- Services:
  - `questdb` ClusterIP: ports `9000` (HTTP), `8812` (PG), `9009` (ILP), `9003` (health)
  - `questdb-nodeport`: `30009` (ILP), `30812` (PG), `30900` (HTTP)
- PVC template:
  - storageClass `local-path`
  - size `200Gi`

### `dex-indexer.yaml` (Yellowstone -> QuestDB)

- Namespace: `solana`
- Workload: `Deployment/dex-indexer`
- Image: `ghcr.io/5dlabs/dex-indexer:latest`
- Node pinning: `kubernetes.io/hostname: db-01`
- Env:
  - `GRPC_URL=http://10.172.144.41:10000`
  - `QUESTDB_URL=http://questdb.questdb.svc:9000`
  - `FLUSH_BATCH_SIZE=500`
  - `FLUSH_INTERVAL_MS=250`

### `dex-api.yaml` (gRPC query/stream API)

- Namespace: `solana`
- Workload: `Deployment/dex-api` + `Service/dex-api`
- Image: `ghcr.io/5dlabs/dex-api:latest`
- Port: `50051` (gRPC)
- Node pinning: `kubernetes.io/hostname: db-01`
- Env:
  - `LISTEN_ADDR=0.0.0.0:50051`
  - `QUESTDB_PG_HOST=questdb.questdb.svc`
  - `QUESTDB_PG_PORT=8812`
  - `QUESTDB_PG_USER=admin`
  - `QUESTDB_PG_PASSWORD=quest`
- Pull secret expected: `ghcr-pull-secret`

## Build/Artifact Pipeline Evidence

From repo workflow and Dockerfiles:

- Workflow present: `.github/workflows/dex-indexer-build.yaml`
  - Builds `dex-indexer` and pushes `ghcr.io/5dlabs/dex-indexer:{latest,sha}`
- Dockerfiles present:
  - `skills/trader/docker/Dockerfile.dex-indexer`
  - `skills/trader/docker/Dockerfile.dex-api`

Note: no dedicated `dex-api` GitHub workflow file was found at capture time; image publishing may currently rely on manual build or another pipeline path.

## Related Active Cluster State (Non-Cherry Context)

In active context (`ovh-cluster`), these namespaces are present and useful for migration reference:

- `trading` exists and contains OpenClaw trader statefulsets/pods/services
- `openclaw` exists with core platform and persistent volumes
- `solana` namespace is **not** present in this active context

This confirms the Cherry Solana stack is managed as a separate destination cluster via Argo, not as local objects in the currently reachable cluster.

## What We Could Not Capture Due to Current Reachability

Because `84.32.103.99:6443` is timing out, we could not fetch current live objects from Cherry directly (for example):

- live `kubectl get all -n solana` against Cherry
- live pod logs / events from Cherry solana workloads
- direct runtime drift checks between desired manifests and live objects

## Migration-Safe Snapshot Checklist (Already Captured)

- [x] Solana GitOps source path and revision lineage (`skills/trader/k8s`, `main`)
- [x] Cherry app inventory + destination endpoint + status
- [x] Validator/Yellowstone/QuestDB/Dex manifest-level runtime settings
- [x] Service ports and storage topology
- [x] Container image references
- [x] External chart versions for observability stack

## Recommended Next Step for Latitude Replication

Use this document and `skills/trader/k8s` as the baseline desired state, then produce a Latitude-specific overlay that only changes:

- node selectors / hostnames
- storage classes and disk sizing where provider differs
- network exposure and bridge endpoints
- secrets references (without changing secret values in plaintext)

Do not mutate app behavior (ports, flags, sync semantics) until parity verification passes.

