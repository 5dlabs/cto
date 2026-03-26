# Latitude Parity Gates - 2026-03-26

## Scope

Validation snapshot for Cherry-to-Latitude Solana parity closure across:

- Data plane
- Telemetry plane
- Service plane

## Gate Results

| Gate | Check | Status | Evidence |
|---|---|---|---|
| Network plane | Cilium-only invariant (no Flannel/kube-proxy DS) | PASS | `kubectl get ds -n kube-system` now returns only `cilium` and `cilium-envoy`; no `kube-flannel`, no `kube-proxy`. |
| Network plane | Cilium control-plane health | PASS | `cilium status` reports Cilium/Operator/Envoy/Hubble all `OK` after Latitude migration and Talos CNI/proxy patching. |
| Data plane | `agave-rpc` pod readiness | PASS | Pod converged to `2/2 Running` on `solana-rpc-01` after account index generation completed. |
| Data plane | Yellowstone plugin artifact present | PASS | `/var/mnt/yellowstone/lib/libyellowstone_grpc_geyser.so` exists and `check-yellowstone` reports "found". |
| Data plane | `dex-indexer` Yellowstone stream | PASS | Indexer now subscribes successfully with sustained ingest logs after GRPC endpoint override to `http://152.236.0.21:10000`. |
| Data plane | `dex-api` rollout from GHCR image | PASS | Deployment uses `ghcr.io/5dlabs/dex-api:latest` and reaches `1/1 Running` after probe fix. |
| Telemetry plane | Observability stack pod health | PASS | Loki, Fluent Bit, Prometheus, Grafana, Alertmanager, node-exporter all `Running`. |
| Telemetry plane | Prometheus target health | PARTIAL | Core targets `up`; `solana-exporter` remains `down` until `agave-rpc` reaches RPC readiness. |
| Service plane | QuestDB read query | PASS | `select 1 as ok` succeeds via `http://questdb.questdb.svc:9000/exec`. |
| Service plane | `dex-api` service connectivity | PASS | In-cluster TCP smoke to `dex-api.solana.svc:50051` succeeds. |
| Service plane | Restart resilience (`dex-api`) | PASS | `kubectl rollout restart deployment/dex-api -n solana` converges successfully. |
| Service plane | Post-Cilium workload stability (`questdb`, `dex-api`, `dex-indexer`) | PASS | All three are `Running` and converged after network cutover and rollout restarts. |
| Telemetry plane | Post-Cilium observability readiness | PASS | `prometheus-server`, `grafana`, `loki`, `alertmanager`, `fluent-bit`, `kube-state-metrics` all `Running` after rollout remediation. |
| API parity | BirdEye suggestion surface in proto/service | PASS | `dex_feed.proto` now includes `GetPriceSuggestion`, `GetMultiPriceSuggestion`, `StreamSuggestion`, plus quote/signal/quality messages. |
| API parity | Suggestion handlers compile and wire through API | PASS | `cargo check -p dex-indexer` green after wiring `api/suggestions.rs`, `db.rs` suggestion model, and stream support. |
| API parity | BirdEye/local parity harness ready | PARTIAL | Harness now executes grpcurl with proto imports; full BirdEye-vs-Latitude run still needs reference endpoint and updated dex-api image deployment. |
| Release gate | Shadow validation window | PARTIAL | 5-sample shadow window shows sustained `dex-indexer` ingest growth (`swaps` counter increasing with no reconnect errors), but cutover remains blocked by parity API deployment + BirdEye reference comparison evidence. |
| Telemetry plane | Suggestion/API availability alerts | PASS | Added Prometheus rules for `dex-api`/`dex-indexer` availability and restart bursts in `skills/trader/k8s/observability/prometheus-values.yaml`. |

## Key Command Snippets

```bash
# Agave host-port readiness on solana-rpc-01
kubectl run rpc-port-check --rm -i --restart=Never -n observability \
  --overrides='{"spec":{"hostNetwork":true,"nodeSelector":{"kubernetes.io/hostname":"solana-rpc-01"}}}' \
  --image=busybox:1.36 --command -- \
  sh -c 'nc -zvw2 127.0.0.1 8001; nc -zvw2 127.0.0.1 8899; nc -zvw2 127.0.0.1 10000'
# Result: 8001 open, 8899 closed, 10000 closed
```

```bash
# Prometheus target health snapshot
kubectl exec -n observability prometheus-server-66f6bf67c4-64qnn -c prometheus-server -- \
  sh -c 'wget -qO- http://127.0.0.1:9090/api/v1/targets'
# Result: solana-exporter target down, core observability targets up
```

```bash
# QuestDB service-plane read check
kubectl run questdb-query --rm -i --restart=Never -n questdb \
  --image=curlimages/curl:8.8.0 --command -- \
  sh -c 'curl -fsS "http://questdb.questdb.svc:9000/exec?query=select%201%20as%20ok"'
# Result: {"dataset":[[1]]}
```

## Promotion Decision

Go/No-Go: **NO-GO** right now.

Blocking items before cutover:

- Deploy updated `dex-api` image containing BirdEye parity proto/service additions and tolerant QuestDB timestamp parsing.
- Execute full BirdEye reference parity harness run and capture median/p95 error evidence.
