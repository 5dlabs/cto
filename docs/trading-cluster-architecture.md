# Trading Cluster Architecture & Security Plan

> **Status:** Draft — 2026-02-22  
> **Owner:** Jon Fritz / 5DLabs  
> **Location:** PhoenixNAP Singapore (collocated with Solana RPC node)

---

## Overview

A single-node Talos Linux cluster on PhoenixNAP Singapore, collocated on the same internal
network segment as the existing Solana RPC node. The hot path (trader → RPC) never leaves
the datacenter. No public ingress is exposed — all private access is via VPN, all
administrative/UI access is via Cloudflare tunnels.

This cluster is **strictly separate from the CTO platform** (OVH, Canada). It runs
trading infrastructure only.

---

## Network Topology

```
PhoenixNAP Singapore — internal 10.2.0.0/24 (bond0.2, 50 Gbps bonded 2×25GbE)
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Solana RPC node          10.2.0.11                                 │
│  ├── Gossip/TVU           public (8001/tcp, 8000-10000/udp)         │
│  └── RPC port 8899        INTERNAL ONLY → 10.2.0.0/24              │
│                                                                     │
│  Trading cluster          10.2.0.x  (new Talos node)               │
│  ├── Trading bots    ──────────────────────► 10.2.0.11:8899        │
│  ├── Paper trader    ──────────────────────► 10.2.0.11:8899        │
│  ├── Birdeye replacement ──────────────────► 10.2.0.11:8899        │
│  └── All internal services on cluster-local DNS                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
              │ Cloudflare tunnel (outbound-only, selected services)
              ▼
        Cloudflare edge (WAF, Access, Zero Trust)
              │ Twingate / VPN (private admin access)
              ▼
        Jon only — no public UI
```

### Key principle: colocation = zero-latency RPC

The trader and the RPC node share a 50 Gbps bonded private link. Round-trip latency is
microseconds. No Cloudflare, no public internet, no NAT on the hot path.

---

## Services

| Service | Purpose | Storage | Notes |
|---|---|---|---|
| **Trading bots** | Automated execution | — | Namespace-isolated, OpenBao secrets |
| **Paper trader** | Sim execution, strategy validation | Postgres | Uses same RPC path |
| **Birdeye replacement** | Market data aggregation / analytics | QuestDB + Redis | Tick data from RPC subscriptions |
| **QuestDB** | Time-series tick data | Large NVMe (see sizing) | Columnar, memory-mapped I/O |
| **Redis** | Pub/sub market data bus, caching | Memory | Persistence optional |
| **Postgres** | Positions, orders, account state | Moderate NVMe | CloudNative-PG operator |
| **OpenBao** | Secrets management | Small | Transit secrets engine for keys |
| **Prometheus + Alertmanager** | Metrics + alerting | Moderate | Scrapes all cluster services |
| **Loki** | Log aggregation | Moderate | All pod logs |
| **Grafana** | Dashboards | — | Private, access via Twingate |
| **Cloudflare operator** | Tunnel management | — | Outbound only |
| **Cilium** | CNI + NetworkPolicy enforcement | — | Default-deny egress |
| **Falco** | Runtime anomaly detection | — | Watches for unexpected syscalls |

### Storage sizing (QuestDB)

Solana mainnet at full throughput: ~50,000 TPS. Storing compressed OHLCV + orderbook snapshots
for all active markets:

- ~5–20 GB/day depending on what is subscribed to
- 1 TB NVMe → ~2–6 months of data at moderate subscription breadth
- QuestDB benefits heavily from NVMe (memory-mapped, sequential writes)

Recommend a server with **≥ 2 TB NVMe** and **≥ 64 GB RAM** for comfortable headroom.

---

## Access Model

**No public ingress. Period.**

| Access type | Method | Who |
|---|---|---|
| Admin (kubectl, talosctl) | Twingate or Headscale | Jon only |
| Grafana / dashboards | Cloudflare Access + tunnel | Jon only |
| SSH to Talos nodes | N/A — Talos has no SSH | — |
| RPC (8899) | Internal 10.2.0.0/24 only | In-cluster services |
| Any other UI | Cloudflare Access (mTLS or OTP) | Jon only |

Talos has no SSH or shell on nodes by default — the API surface is `talosctl` only over
mTLS. This dramatically reduces the attack surface compared to Ubuntu/k3s.

---

## Solana Node Security

### Gossip — the honest answer

Solana gossip is a peer-to-peer UDP protocol. The node **must** be reachable on the
public internet for gossip to work — there is no way to hide it behind a proxy or WAF
(Cloudflare Spectrum/Magic Transit are TCP/enterprise-grade, respectively, and don't help
UDP gossip in practice at this scale).

**What we can and should do:**

1. **Tighten the UFW rules** — remove the 3 unused open ports (`8888`, `8999`, `10000`).
   Lock `8899/tcp` to `10.2.0.0/24` only (already planned).

2. **PhoenixNAP built-in DDoS protection** — PhoenixNAP's Singapore datacenter has
   volumetric DDoS scrubbing at the network level before traffic reaches the host.
   No action needed — it's included. Can be supplemented via the BMC API with network
   firewall rules for rate-limiting per-source IP.

3. **Kernel-level rate limiting** — `iptables hashlimit` to rate-limit UDP per source IP:
   ```bash
   iptables -A INPUT -p udp --dport 8000:10000 \
     -m hashlimit --hashlimit-above 1000/sec \
     --hashlimit-burst 5000 \
     --hashlimit-mode srcip \
     --hashlimit-name udp-ratelimit \
     -j DROP
   ```
   Drops flood attacks from any single source without affecting normal gossip traffic.

4. **`--known-validator` entries** — already configured. These prioritise trusted peers in
   gossip without restricting connectivity to the wider network (unlike `--only-known-rpc`,
   which we removed).

5. **`--tpu-disable-quic`** — already applied. Reduces the QUIC attack surface.

6. **Monitoring** — Prometheus + solana-exporter will alert on `tvu_peers` dropping to 0
   (the canary for both network attacks and deadlocks).

### What's already fixed

| Item | Status |
|---|---|
| `netdev_max_backlog` 1000 → 1,000,000 | ✅ Applied |
| `--only-known-rpc` removed | ✅ Applied |
| `--tpu-disable-quic` added | ✅ Applied |
| `Restart=always` + watchdog service | ✅ Applied |
| `kernel.pid_max` 65k → 4.2M | ✅ Applied |
| `net.core.somaxconn` 4096 → 65535 | ✅ Applied |
| NUMA interleaving (`numactl --interleave=all`) | ✅ Applied |
| `--accounts-index-bins 128` | ✅ Applied |
| OOM guard service (score pinned to 0) | ✅ Running |
| 271 GB swap (NVMe priority) | ✅ Active |

### Still to do on Solana node

- [ ] Remove UFW rules for `8888/tcp`, `8999/tcp`, `10000/tcp` (not listening)
- [ ] Restrict `8899/tcp` to `10.2.0.0/24` only (once trading cluster has its IP)
- [ ] Add iptables UDP rate-limit rule + persist via `/etc/iptables/rules.v4`
- [ ] PhoenixNAP BMC API — configure network-level firewall (need API credentials)
- [ ] Deploy solana-exporter for Prometheus metrics

---

## Trading Cluster — Talos Single Node

### Why Talos

- **Immutable OS** — no package manager, no shell, no SSH on nodes
- **mTLS API only** — `talosctl` requires mutual TLS; no unauthenticated surface
- **Encrypted state** — disk encryption built-in
- **Minimal attack surface** — no systemd units beyond what Kubernetes needs
- **Strong CIS baseline** — `kube-bench` results are excellent out of the box

### Single-node tradeoffs

- etcd runs as a single instance — no quorum, but fine for stateless/replaceable workloads
- Node failure = cluster down until rescheduled — acceptable for trading if bots are
  designed to fail safe (cancel orders on shutdown signal, SIGTERM handlers)
- Can expand to 3-node for HA later without rebuilding (just `talosctl join`)

### Networking (Cilium)

Cilium as CNI gives us:
- **Default-deny egress** on all bot namespaces
- **Per-namespace allowlists** — each bot only egresses to its specific exchange endpoints
- **NetworkPolicy** — no pod-to-pod cross-namespace traffic
- **eBPF-based** — minimal overhead vs iptables, handles Solana-grade packet rates well
- **Hubble** — observable network flows for auditing

### Secret management (OpenBao)

```
OpenBao (Transit secrets engine)
├── Bot private keys → never leave OpenBao as plaintext
│   Bot calls: Sign(transaction_bytes) → returns signature
│   Bot NEVER holds the raw private key in memory
│
├── Exchange API keys → KV-v2, versioned, audited
│   ESO (External Secrets Operator) injects at pod start
│   Short-lived K8s secrets, not env vars where possible
│
└── Audit log → Loki
    Every secret access: who, what, when
```

**Kubernetes auth method** — pods exchange their ServiceAccount token for a short-lived
OpenBao token scoped to exactly the paths they need. No static credentials anywhere.

### Pod security baseline

Applied to all bot/trading namespaces:

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop: ["ALL"]
automountServiceAccountToken: false  # unless OpenBao auth required
```

Base images: distroless or scratch — no shell, no package manager in the container.

---

## Multi-Cluster & Connectivity

### Current plan: standalone

The trading cluster is intentionally isolated from OVH/CTO for now. No cluster mesh.
Standalone is simpler, more secure, and easier to reason about when private keys are involved.

### ArgoCD

The existing ArgoCD on OVH manages the CTO platform. For the trading cluster:

**Option A — Single ArgoCD (recommended for now):**
- Register the PhoenixNAP cluster as an external destination in the OVH ArgoCD
- One GitOps control plane, two cluster destinations
- Requires VPN tunnel (WireGuard/Twingate) between ArgoCD and PhoenixNAP API server

**Option B — Dedicated ArgoCD on trading cluster:**
- Fully isolated, no network dependency on OVH
- More operational overhead (two ArgoCD instances to maintain)

Option A is cleaner until you need strict isolation. If the trading cluster ever handles
production funds at scale, Option B gives you stronger blast-radius containment.

### Future: Cilium ClusterMesh (if needed)

If cross-cluster service discovery ever becomes necessary (e.g., CTO platform consuming
trading signals), Cilium ClusterMesh is the right answer — but requires:

1. Both clusters running Cilium as CNI
2. OVH cluster migrated from k3s+Kilo to something Cilium-primary
3. Non-overlapping PodCIDRs (plan these upfront)
4. WireGuard tunnel for node-level IP reachability

Not needed now. Design PodCIDRs to not overlap so the migration path stays open.

**Suggested CIDR allocation:**

| Cluster | Node CIDR | Pod CIDR | Service CIDR |
|---|---|---|---|
| OVH (k3s, existing) | 10.0.0.0/24 | 10.42.0.0/16 | 10.43.0.0/16 |
| PhoenixNAP (Talos, new) | 10.2.0.0/24 | 10.44.0.0/16 | 10.45.0.0/16 |

---

## Security Testing Pipeline

### Tools

| Tool | Layer | When |
|---|---|---|
| **nmap** | Port exposure | Before and after any firewall change |
| **Shannon** (KeygraphHQ) | Web app / API exploits | On every deploy of exposed services |
| **kubescape** | K8s posture (MITRE + NSA) | Weekly + after cluster changes |
| **kube-bench** | CIS benchmark | At cluster creation + major upgrades |
| **trivy** | Image + manifest CVEs | CI/CD — every image build |
| **Falco** | Runtime anomaly detection | Always-on in production |

### Shannon notes

Shannon is whitebox (source-code-aware) and web-app focused. Best used against:
- Any Cloudflare-tunneled UI (Grafana, Birdeye frontend, internal tools)
- REST/WebSocket APIs the bots or Birdeye replacement expose

Not suited for: Kubernetes infrastructure, UDP services, OS-level hardening.

### Baseline scan before going live

```bash
# 1. External port scan — confirm nothing unexpected is reachable
nmap -sS -sU -p- --open 125.253.92.141     # Solana node
nmap -sS -sU -p- --open <trading-node-ip>   # Trading cluster

# 2. k8s posture
kubescape scan --kubeconfig kubeconfig-trading.yaml

# 3. CIS benchmark
kubectl run kube-bench --image=aquasec/kube-bench -- --benchmark cis-1.9

# 4. Image scan (example)
trivy image your-trading-bot:latest

# 5. Shannon (against any exposed web surface)
shannon scan --target https://grafana.internal.5dlabs.ai --repo ./cto
```

---

## Monitoring & Alerting

### Stack

- **Prometheus** — scrapes all cluster services + solana-exporter (via internal network)
- **Alertmanager** — routes to Discord / PagerDuty
- **Loki** — all pod logs + OpenBao audit log
- **Grafana** — dashboards (private, Twingate access)
- **Falco** — runtime, alerts to Loki + Alertmanager

### Critical alerts to configure

| Alert | Condition | Severity |
|---|---|---|
| Solana TVU peers | `tvu_peers == 0` for > 2 min | Critical |
| Solana slot lag | local slot > network slot + 100 | Warning |
| RPC unresponsive | `getHealth` fails × 3 | Critical |
| Bot pod crash loop | `kube_pod_container_status_restarts_total` | Critical |
| OpenBao sealed | vault status = sealed | Critical |
| Secret access anomaly | Falco: unexpected OpenBao path access | Critical |
| OOM kill | `container_oom_events_total` > 0 | Warning |
| Disk > 80% | QuestDB / Postgres volumes | Warning |

---

## Open Items / Decisions Needed

- [ ] **PhoenixNAP BMC API credentials** — needed to query server SKUs (Singapore) and
      configure network-level firewall
- [ ] **Server SKU** — confirm budget/size; recommendation: ≥ 64 GB RAM, ≥ 2 TB NVMe,
      Singapore datacenter (same as Solana node for internal network colocation)
- [ ] **Twingate vs Headscale** for private admin access — Twingate already deployed on
      OVH; either works, just need to provision a connector on the trading cluster
- [ ] **ArgoCD approach** — Option A (single, OVH-hosted) vs Option B (dedicated on trading
      cluster). Recommendation: Option A to start
- [ ] **PodCIDR allocation** — confirm non-overlapping CIDRs now even if ClusterMesh is
      not immediate (avoids painful migration later)
- [ ] **Solana exporter deploy** — once RPC is back up; scrape via internal IP from
      trading cluster Prometheus
- [ ] **OpenBao Transit vs KV** for bot private keys — Transit is strongly preferred
      (key never leaves OpenBao), but requires the bot to support a sign API call
      rather than loading a keypair file
