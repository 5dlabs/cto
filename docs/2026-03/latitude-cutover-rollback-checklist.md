# Latitude Cutover + Rollback Checklist

## Preconditions (must be green first)

- `agave-rpc` pod is fully ready (`2/2`) and stable for a sustained window.
- `agave-rpc` host ports `8899` and `10000` are reachable.
- `dex-indexer` stops gRPC reconnect-loop errors and shows non-zero ingest activity.
- `dex-api` remains `1/1` after at least one controlled restart.
- Suggestion endpoints (`GetPriceSuggestion`, `GetMultiPriceSuggestion`, `StreamSuggestion`) return valid payloads for canary tokens.
- BirdEye parity harness report shows gate-compliant median/p95 price error.
- Prometheus `solana-exporter` target is `up`.
- Grafana Solana dashboards show live slot progression.

## Cutover Steps

1. Freeze non-essential manifest churn on both clusters.
2. Confirm latest image tags are pinned and pulled on Latitude nodes.
3. Route read/stream clients to Latitude endpoints in staged batches.
4. Watch SLOs (latency, error rate, ingest freshness) for each batch.
5. Complete cutover only after full-batch SLO hold window.

## Live Verification Commands

```bash
# Solana workload readiness
kubectl get pods -n solana -o wide

# Agave port readiness on node
kubectl run rpc-port-check --rm -i --restart=Never -n observability \
  --overrides='{"spec":{"hostNetwork":true,"nodeSelector":{"kubernetes.io/hostname":"solana-rpc-01"}}}' \
  --image=busybox:1.36 --command -- \
  sh -c 'nc -zvw2 127.0.0.1 8899; nc -zvw2 127.0.0.1 10000'

# Indexer stream health
kubectl logs -n solana deploy/dex-indexer --tail=100

# API health and restart resilience
kubectl rollout restart deployment/dex-api -n solana
kubectl rollout status deployment/dex-api -n solana --timeout=180s

# BirdEye parity harness
python3 scripts/2026-03/birdeye_parity_harness.py \
  --local dex-api.solana.svc.cluster.local:50051 \
  --reference <birdeye-grpc-endpoint:port> \
  --token So11111111111111111111111111111111111111112
```

## Rollback Triggers

Trigger rollback immediately if any of these occur during cutover:

- `dex-indexer` reconnect-loop error rate spikes and persists.
- `dex-api` availability drops below SLO for two consecutive windows.
- Suggestion confidence/fallback behavior regresses (confidence collapse or persistent fallback-active state).
- `solana-exporter` or core telemetry targets flap/down persistently.
- QuestDB read/write checks fail or freshness falls outside threshold.

## Rollback Actions

```bash
# 1) Repoint client routing back to Cherry endpoints.
# (Apply your existing traffic policy / DNS rollback change.)

# 2) Keep Latitude workloads running for forensics.
kubectl get pods -n solana -o wide

# 3) Capture incident evidence
kubectl logs -n solana deploy/dex-indexer --tail=200
kubectl logs -n solana deploy/dex-api --tail=200
kubectl get events -n solana --sort-by=.metadata.creationTimestamp | tail -n 100
```

## Current State Note (2026-03-26)

`agave-rpc` and Yellowstone stream are now live and indexer ingest is active. A short shadow validation sample window completed with steady ingest. Cutover remains blocked on deploying the updated `dex-api` parity build and completing BirdEye-vs-Latitude harness evidence.
