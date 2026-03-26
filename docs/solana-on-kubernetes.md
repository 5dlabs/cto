# Solana RPC on Kubernetes — Architecture & Rationale

> **Status:** Planning + active deployment notes (updated 2026-03-21)  
> **Decision:** Keep architecture guidance here; use the implementation section below for the live trader manifests  
> **Key insight:** Cilium eBPF gives co-located pods sub-microsecond RPC latency — faster than bare-metal TCP loopback

---

## Overview

The Solana RPC node runs directly on bare metal (no containerization). A small Kubernetes
cluster (Talos Linux, 3 nodes) runs on the same provider network handling:

- Trading bots
- TimescaleDB (price feed / tick data)
- Redis (market data bus)
- Postgres (positions, orders)
- Prometheus + Grafana
- Twingate connector (private admin access)

The hot path — trading pod → RPC — stays on the provider's private network.
With Cilium's socket-level load balancing, same-node pod communication bypasses
the TCP stack entirely.

---

## Current Implementation (Trader Cluster)

The live trader deployment currently uses:

- `skills/trader/k8s/agave-rpc.yaml` for the Agave RPC `Deployment` plus Yellowstone config `ConfigMap`
- `skills/trader/k8s/observability/prometheus-values.yaml` for `yellowstone-grpc` scrape target (`solana-rpc-01:8999`)
- `skills/trader/docker/Dockerfile.yellowstone-grpc` to build `libyellowstone_grpc_geyser.so` from source

### Why source-build Yellowstone

The prebuilt Yellowstone release artifact for `v7.0.0+solana.2.2.20` previously crashed at startup
(`dlopen` during `GeyserPluginService`) in `ghcr.io/dysnix/docker-agave:v2.2.20`.

The current fix is to compile the plugin with the matching Rust toolchain (`rust:1.84.1`) and
the matching Yellowstone tag (`v7.0.0+solana.2.2.20`), then mount the resulting `.so` into the
Agave pod from host path `/var/mnt/yellowstone/lib`.

### Build and deploy flow (verified from repo config)

```bash
# 1) Build plugin image (from skills/trader/docker/)
docker build -t yellowstone-grpc-builder:v2.2.20 \
  -f Dockerfile.yellowstone-grpc .

# 2) Extract the shared object
docker create --name ys-extract yellowstone-grpc-builder:v2.2.20
docker cp ys-extract:/output/libyellowstone_grpc_geyser.so .
docker rm ys-extract

# 3) Copy plugin to Solana host volume expected by agave-rpc.yaml
scp libyellowstone_grpc_geyser.so solana-rpc-01:/var/mnt/yellowstone/lib/

# 4) Apply/update deployment
kubectl apply -f skills/trader/k8s/agave-rpc.yaml
```

### Runtime expectations

- `agave-validator` is configured with `--geyser-plugin-config /etc/yellowstone/config.json`
- plugin config points to `/opt/yellowstone/lib/libyellowstone_grpc_geyser.so`
- gRPC endpoint listens on `:10000`
- Yellowstone Prometheus endpoint listens on `:8999`
- if the `.so` is missing, the `check-yellowstone` init container logs a warning before startup

### Troubleshooting quick checks

```bash
# Plugin exists on host path
ls -lah /var/mnt/yellowstone/lib/libyellowstone_grpc_geyser.so

# Agave pod logs (watch for geyser load errors)
kubectl -n solana logs deploy/agave-rpc -c agave-validator --tail=200

# Yellowstone metrics reachable from Prometheus target
curl -fsS http://solana-rpc-01:8999/metrics | head
```

---

## Why Not K8s for the Validator?

Running the Solana validator itself inside Kubernetes adds complexity without benefit:
- Accounts I/O is ~150-177 MB/s sustained random reads — needs raw NVMe, not a PVC abstraction
- The validator manages its own process lifecycle and signal handling
- No enterprise operator runs Solana validators in K8s today (Sol Strategies, Blockdaemon, etc. all use bare metal)
- Talos has no SSH — all management is via `talosctl`. Managing the validator separately (via SSH) is simpler and more debuggable.

The service cluster K8s nodes run on the same physical network as the Solana node,
giving microsecond-level access to port 8899.

---

## Cilium eBPF Performance Stack

We run Cilium with `kubeProxyReplacement=true` already. To unlock maximum performance
for the trading workload, enable the full performance profile:

```yaml
# cilium helm values (add to existing CTO cluster values or trading cluster)
routingMode: native              # no VXLAN overlay
bpf:
  datapathMode: netkit           # kernel 6.8+ — zero namespace overhead
  masquerade: true
  distributedLRU:
    enabled: true
  mapDynamicSizeRatio: 0.08
kubeProxyReplacement: true
bandwidthManager:
  enabled: true
  bbr: true                      # BBR congestion control for pods
enableIPv4BIGTCP: true           # larger GSO/GRO, better NIC throughput
bpfClockProbe: true
```

### What This Unlocks

| Feature | Effect |
|---|---|
| `netkit` device mode | Pod network namespace overhead → **zero** (as-if in host namespace) |
| eBPF host routing | Bypasses iptables entirely on the forwarding path |
| Socket-level LB (sockmap) | Same-host pod→pod traffic redirected at socket layer, not packet layer |
| BIG TCP | Fewer stack traversals at high throughput; enables 100Gbps+ |
| BBR | Better TCP congestion response; lower latency under load |

**Result:** Trading pod → RPC pod on same K8s node = **sub-microsecond** socket redirect.
Bare metal TCP loopback is ~2-5μs. Cilium sockmap is ~0.5μs.

### Requirements
- Kernel >= 6.8 for `netkit` (Talos ships this ✅)
- `routingMode: native` (not tunnel/VXLAN)
- kube-proxy disabled (already done on CTO cluster ✅)

---

## 3-Node Service Cluster

```
┌─────────────────────────────── Same Provider Network ───────────────────────────────┐
│                                                                                       │
│  Solana RPC (bare metal)          K8s node-1 (control plane)                        │
│  ├── agave-validator (Jito)       ├── talos control plane                           │
│  ├── yellowstone-grpc :10000      └── small: 64GB, 400GB NVMe                      │
│  └── RPC :8899 (internal only)                                                       │
│                                   K8s node-2 (DB worker)                            │
│  Trading bots ──────────────────► ├── TimescaleDB (tick data)                      │
│  (K8s pods on node-2 or node-3)   ├── Postgres (positions/orders)                  │
│                                   ├── Redis (market data pub/sub)                   │
│                                   └── 256GB RAM, 2x500GB NVMe                      │
│                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

### Node Sizing (Cherry Servers Singapore, hourly billing)

| Role | Plan | CPU | RAM | Storage | Cost |
|---|---|---|---|---|---|
| Control plane | `G1-16-64gb-400nv-ded` | 8x Gold 6230R | 64GB | 400GB NVMe | €0.221/hr |
| DB + trading | `2x-amd-epyc-7443` | 48c/96t | 256GB | 2x500GB NVMe | €0.803/hr |
| Solana RPC | See [Solana hardware](#solana-hardware) | | | | |

**Total service cluster: ~€1.02/hr** while active. Stop nodes when not running.

### Pod Affinity Strategy

Trading bots scheduled to node-2 (same node as databases):
```yaml
affinity:
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
      - matchExpressions:
        - key: node-role
          operator: In
          values: ["trading"]
```

---

## Solana Hardware

### Singapore Option (co-location with trading cluster)
- Best available: `2x-amd-epyc-7443` — 256GB RAM, 2x500GB NVMe, €0.803/hr
- **Concern:** 256GB is tight for Solana mainnet (needs 512GB+ for full accounts index in RAM)
- Can run with `--accounts-db-cache-limit-mb=65536` to cap memory use, but may swap under load
- Same physical network as K8s cluster = microsecond RPC latency ✅

### EU/Lithuania Option (solana-server-gen5)
- `solana-server-gen5` — 768GB RAM, 2x1TB NVMe + 2x4TB NVMe, €2.053/hr
- Purpose-built by Cherry specifically for Solana ✅
- Ideal 4-drive layout: OS + ledger + accounts + snapshots each on separate NVMes ✅
- Frankfurt/Amsterdam has ~35% of all staked Solana validators → lower peer latency ✅
- Tradeoff: ~150ms from Singapore, so trading cluster needs to be EU too, OR accept RPC over WAN

### Recommendation
If trading cluster stays Singapore → use `2x-amd-epyc-7443` and accept the 256GB constraint.
If you're willing to move everything EU → `solana-server-gen5` in Lithuania is the best Solana hardware available anywhere.

---

## Disk Layout (Bare Metal Solana)

```
Drive 1 (1TB NVMe) — OS + Ledger + Snapshots
├── /          (OS, ~50GB)
├── /ledger    (RocksDB, ~130GB at --limit-ledger-size 50000000)
└── /snapshots (full 108GB + incrementals ~20GB)

Drive 2 (4TB NVMe) — Accounts only
└── /accounts  (1.1TB accounts index + run_to_be_deleted)

(solana-server-gen5 has 2x1TB + 2x4TB — even better separation)
```

Lesson learned from PhoenixNAP: snapshot writes (20GB every ~33 min) on the same disk
as accounts caused 96% I/O saturation and 60+ slots/min drift. **Accounts and snapshots
must be on separate NVMe drives.**

---

## Key Startup Flags

```bash
--ledger /ledger
--accounts /accounts
--snapshots /snapshots
--limit-ledger-size 50000000          # ~130GB RocksDB, no further growth
--snapshot-interval-slots 5000
--full-snapshot-interval-slots 50000
--accounts-db-cache-limit-mb 8192     # tune based on available RAM
--accounts-index-bins 128
--no-voting
--full-rpc-api
--enable-rpc-transaction-history
--rpc-threads 16
--geyser-plugin-config /home/ubuntu/yellowstone-grpc-config.json  # Yellowstone gRPC
```

---

## Solana MEV Equivalent

On Solana, the equivalent of Ethereum MEV is:

1. **Jito tips/bundles** — we run Jito v3.1.8 which participates in the Jito block engine.
   Searchers submit bundles with tips to guarantee transaction ordering.
2. **Priority fees** — higher `computeUnitPrice` = faster inclusion
3. **Yellowstone gRPC** — streams every incoming transaction before confirmation.
   Trading pods on the same node see new transactions in ~0.5μs via Cilium sockmap,
   then can submit competing transactions through the same RPC in the same round-trip.
4. **Front-running window:** `gRPC stream → detect opportunity → submit tx` is entirely
   within the same host when trading pods are co-located with the RPC node.

---

## Blockdaemon Cluster Manager (Future)

For multi-node Solana fleets, [Blockdaemon/solana-cluster](https://github.com/Blockdaemon/solana-cluster)
provides:
- **Sidecar** — lightweight agent on each node serving snapshot metadata
- **Tracker** — aggregates all sidecars, serves "best snapshot source" API
- **Fetch** — nodes pull snapshots from internal peers instead of mainnet (saves bandwidth, ~10x faster)

With Cilium ClusterMesh between K8s clusters, snapshot distribution can happen entirely
over the provider's internal network. This is the architecture Blockdaemon uses for their
private backbone at enterprise scale.

---

## References

- [Blockdaemon solana-cluster](https://github.com/Blockdaemon/solana-cluster)
- [Sol Strategies: Managing Multiple Validators](https://blog.solstrategies.io/managing-multiple-solana-validators-in-an-enterprise-environment-7d921b92864f)
- [Cilium Performance Tuning Guide](https://docs.cilium.io/en/stable/operations/performance/tuning/)
- [Cilium Bandwidth Manager](https://docs.cilium.io/en/stable/network/kubernetes/bandwidth-manager/)
- [Cilium ClusterMesh](https://docs.cilium.io/en/stable/network/clustermesh/clustermesh/)
- [solana-fm/timescaledb-kubernetes](https://github.com/solana-fm/timescaledb-kubernetes)
- [Agave Hardware Requirements](https://docs.anza.xyz/operations/requirements)
