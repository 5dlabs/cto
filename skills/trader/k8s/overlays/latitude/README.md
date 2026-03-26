# Latitude Overlay

This overlay keeps the Solana trading manifests behaviorally aligned with the existing stack while replacing provider-coupled endpoint assumptions.

## What changes

- Base manifests include `agave-rpc-grpc` ClusterIP and `dex-indexer` `GRPC_URL`:
  `http://agave-rpc-grpc.solana.svc:10000` (no per-provider node IP in the overlay).

## Drift guardrails

- Keep this overlay limited to provider-coupled deltas only.
- Do not change base workload behavior here (resource envelopes, flags, ports, probes) unless a provider compatibility issue requires it.
- If behavior changes are needed, apply them in base manifests and roll through all environments.

## Apply

```bash
kubectl apply -k skills/trader/k8s/overlays/latitude
```
