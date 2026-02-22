# Trading Cluster Architecture & Security Plan

> **Status:** Draft — 2026-02-22 (updated: Cloudflare tunnels removed, Twingate for all private access)  
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
│  PhoenixNAP network-level firewall (BMC API)                        │
│  └── DDoS scrubbing + rate-limit rules at infra layer               │
│                                                                     │
│  Solana RPC node          10.2.0.11                                 │
│  ├── Gossip/TVU           public (8001/tcp, 8000-10000/udp)         │
│  └── RPC port 8899        INTERNAL ONLY → 10.2.0.0/24              │
│                                                                     │
│  Trading cluster          10.2.0.x  (new Talos node)               │
│  ├── Trading bots    ──────────────────────► 10.2.0.11:8899        │
│  ├── Paper trader    ──────────────────────► 10.2.0.11:8899        │
│  ├── Birdeye replacement ──────────────────► 10.2.0.11:8899        │
│  ├── Twingate connector (outbound dial-out)                         │
│  └── All internal services on cluster-local DNS                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
              │ Twingate (outbound connector, zero-trust overlay)
              ▼
        Twingate control plane
              │
              ▼
        Jon only — admin access to cluster services via Twingate client
        (Grafana, talosctl, kubectl — all private, no public ports)
```

### Key principles

**Colocation = zero-latency RPC**
The trader and the RPC node share a 50 Gbps bonded private link (`bond0.2`, 10.2.0.0/24).
Round-trip latency is microseconds. No proxy, no NAT, no extra hop on the hot path.

**Direct public IP = zero-latency exchange egress**
PhoenixNAP bare metal nodes are dual-homed: private VLAN (`bond0.2`) + real public IP
(`bond0.3`). Outbound to exchange APIs goes directly over the public interface — no NAT
device, no extra routing hop. This is the same setup already confirmed on the Solana node
(`10.2.0.11` private, `125.253.92.141` public).

```
bond0.2  10.2.0.x  ──────► Solana RPC node (10.2.0.11:8899)   microseconds, stays in DC
bond0.3  public IP ──────► Exchange REST/WS APIs               direct, no NAT
```

Inbound on `bond0.3`: default-deny (UFW + PhoenixNAP network firewall).
Outbound on `bond0.3`: unrestricted at OS level; optionally scoped per-bot via
Cilium egress NetworkPolicy (each bot namespace allowlisted to specific exchange IPs/ports).

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
| **Twingate connector** | Private admin access | — | Outbound dial-out, no open ports |
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
| Admin (kubectl, talosctl) | Twingate | Jon only |
| Grafana / dashboards | Twingate | Jon only |
| SSH to Talos nodes | N/A — Talos has no SSH | — |
| RPC (8899) | Internal 10.2.0.0/24 only | In-cluster services |
| Any other cluster service | Twingate | Jon only |

Twingate works via an outbound-only connector running in-cluster — no listening ports on
the host. The connector dials out to the Twingate control plane. Access is authenticated
via the Twingate client (device cert + IdP). Nothing is exposed on the public interface.

Talos has no SSH or shell on nodes by default — the API surface is `talosctl` only over
mTLS. Combined with Twingate this means the entire administrative surface is zero-trust
and has no open inbound ports whatsoever.

---

## Solana Node Security

### Gossip — the honest answer

Solana gossip is a peer-to-peer UDP protocol. The node **must** be reachable on the
public internet for gossip to work — you cannot proxy, WAF, or tunnel it.

**Can Cloudflare help here?** No, not practically:
- **Cloudflare Spectrum** — TCP only; gossip is UDP
- **Cloudflare Magic Transit** — can scrub UDP DDoS, but requires your own ASN and IP
  block announced via BGP. Enterprise-only, not applicable to a single rented server.
- **Cloudflare CDN / Tunnel** — HTTP/S only, irrelevant for gossip

The perimeter control that *does* work at this layer is the
**PhoenixNAP network-level firewall** (BMC API) — applied at the infrastructure layer
before packets hit the OS — combined with kernel-level UDP rate limiting on the host.

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

## Talos + Cilium — Responsibilities & Overlap

This is the most important thing to get right before deploying. Both touch networking and
security, and running them in conflict will break the cluster silently.

### Who owns what

| Concern | Talos | Cilium |
|---|---|---|
| OS / immutable filesystem | ✅ owns | — |
| Physical interfaces (bond0.x, VLANs, IPs) | ✅ owns | — |
| Kernel modules & sysctls at boot | ✅ owns | — |
| kubelet, etcd, control plane | ✅ owns | — |
| Node-level host firewall | ✅ owns | — |
| CNI / pod networking | ❌ set to `none` — hands off to Cilium | ✅ owns |
| Pod IP allocation (IPAM) | — | ✅ owns |
| kube-proxy (service VIPs) | ❌ must be **disabled** | ✅ replaces via eBPF |
| NetworkPolicy (pod/namespace) | — | ✅ owns |
| WireGuard mesh (KubeSpan) | ⚠️ available, see below | ⚠️ available, see below |
| Observable network flows | — | ✅ Hubble |
| Cluster mesh / multi-cluster | — | ✅ ClusterMesh |
| Load balancing / DSR | — | ✅ eBPF maglev |

### The three things you must get right

**1. Disable kube-proxy in Talos**

Talos deploys kube-proxy by default. Cilium replaces it entirely with an eBPF
implementation that is faster and more observable. Running both causes conflicts.

In your Talos machine config:
```yaml
cluster:
  proxy:
    disabled: true
```

And in the Cilium Helm values:
```yaml
kubeProxyReplacement: true
k8sServiceHost: <talos-api-ip>
k8sServicePort: 6443
```

**2. Set CNI to `none` in Talos**

Talos will try to install a default CNI (flannel) unless told not to. Let Cilium own it:

```yaml
cluster:
  network:
    cni:
      name: none
```

**3. Cilium needs specific cgroup config for Talos**

Talos mounts cgroups differently from standard Linux. Cilium's auto-mount will conflict:

```yaml
# Required Cilium Helm values for Talos
cgroup:
  autoMount:
    enabled: false
  hostRoot: /sys/fs/cgroup
securityContext:
  capabilities:
    ciliumAgent: [CHOWN, KILL, NET_ADMIN, NET_RAW, IPC_LOCK, SYS_ADMIN, SYS_RESOURCE, DAC_OVERRIDE, FOWNER, SETGID, SETUID]
    cleanCiliumState: [NET_ADMIN, SYS_ADMIN, SYS_RESOURCE]
```

### WireGuard — pick one, not both

This is the main overlap trap:

- **Talos KubeSpan** — WireGuard mesh at the **node level**. Connects Talos nodes across
  different networks (e.g., across datacenters). Requires both clusters to run Talos.
  Not applicable for single-node clusters.

- **Cilium WireGuard** — WireGuard encryption at the **pod level**. Encrypts pod-to-pod
  traffic within a cluster. Works with any k8s distro.

**For this cluster (single-node, same-datacenter as RPC node):**
- KubeSpan: irrelevant — there's only one Talos node, no mesh needed
- Cilium WireGuard: optional — pod traffic is already on a private datacenter network,
  encryption adds ~5–10% overhead. Reasonable to skip for the hot latency path, enable
  if regulatory/compliance requirements apply later.

**Do not enable both simultaneously.** The WireGuard interfaces will conflict.

### Talos host firewall + Cilium NetworkPolicy — complementary, not conflicting

These operate at different layers and should both be used:

```
Internet
    │
    ▼
Talos host firewall (machine.network level)
    │   • Blocks at OS/kernel before anything reaches pods
    │   • Rules: allow 8001/tcp, 8000-10000/udp; deny everything else inbound
    ▼
Cilium NetworkPolicy (pod/namespace level)
        • Default-deny all ingress + egress on bot namespaces
        • Explicit egress allow per bot (exchange API IPs only)
        • No pod-to-pod cross-namespace unless declared
```

The Talos firewall is your outer wall. Cilium is your inner wall. Defense in depth.

### Required kernel modules (Talos machine config)

Cilium's eBPF datapath needs these loaded at boot:

```yaml
machine:
  kernel:
    modules:
      - name: br_netfilter
      - name: xt_socket
  sysctls:
    net.core.bpf_jit_enable: "1"
    net.ipv4.conf.all.forwarding: "1"
    net.ipv6.conf.all.forwarding: "1"
```

---

## Multi-Cluster & Connectivity

### Current plan: standalone

The trading cluster is intentionally isolated from OVH/CTO for now. No cluster mesh.
Standalone is simpler, more secure, and easier to reason about when private keys are involved.

### ArgoCD

The existing ArgoCD on OVH manages the CTO platform. For the trading cluster:

**✅ Decision: Single ArgoCD (OVH) manages both clusters.**

- PhoenixNAP cluster registered as an external destination in the existing OVH ArgoCD
- One GitOps control plane, two kubeconfig destinations
- `argocd cluster add <trading-kubeconfig>` registers the cluster; ArgoCD then communicates
  with it via Twingate (trading cluster's talosctl/API endpoint is a Twingate resource)
- New GitOps path: `infra/trading/` in the `5dlabs/cto` repo

If the trading cluster ever needs to be fully air-gapped from OVH (e.g., stricter blast-radius
isolation when handling large positions), promote to its own ArgoCD at that point.

### Future: Cilium ClusterMesh (if needed)

If cross-cluster service discovery ever becomes necessary (e.g., CTO platform consuming
trading signals), Cilium ClusterMesh is the right answer — but requires:

1. Both clusters running Cilium as CNI
2. OVH cluster migrated from k3s+Kilo to something Cilium-primary
3. Non-overlapping PodCIDRs (plan these upfront)
4. WireGuard tunnel for node-level IP reachability

Not needed now. Design PodCIDRs to not overlap so the migration path stays open.

**CIDR allocation — non-overlapping, non-default:**

| Cluster | Node CIDR | Pod CIDR | Service CIDR |
|---|---|---|---|
| OVH (k3s, existing) | 10.0.0.0/24 | 10.42.0.0/16 | 10.43.0.0/16 |
| PhoenixNAP (Talos, new) | 10.2.0.0/24 | **10.44.0.0/16** | **10.45.0.0/16** |

These are intentionally non-default. The Talos/Cilium default pod CIDR is `10.244.0.0/16`
and default service CIDR is `10.96.0.0/12` — both conflict with common cloud VPC ranges and
each other if you ever peer networks. The OVH cluster uses k3s defaults (`10.42/43`).
The PhoenixNAP cluster uses `10.44/45` — non-overlapping with both.

**Set these explicitly at Talos install time** (cannot be changed later without rebuilding):
```yaml
# Talos machineconfig cluster section
cluster:
  network:
    podSubnets:
      - 10.44.0.0/16
    serviceSubnets:
      - 10.45.0.0/16
```

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

## Decisions

| Decision | Resolution |
|---|---|
| Private access | ✅ **Twingate** — connector deployed in-cluster, same account as OVH |
| ArgoCD | ✅ **Shared (OVH)** — PhoenixNAP cluster registered as external destination |
| Cloudflare tunnels | ✅ **No** — Twingate covers all private access |
| Bot private keys | ✅ **OpenBao Transit** — see implementation notes below |
| WireGuard | ✅ **Off** — single-node, same-DC, no overhead needed on hot path |
| PodCIDRs | ✅ **Non-default** — see CIDR table, avoids k3s + Talos defaults |

## Open Items

- [ ] **PhoenixNAP BMC credentials** — not in workspace, Henry offline. Jon to provide
      or ask Henry. Needed for: exact Singapore SKU availability, server pricing, and
      BMC network firewall configuration.
      > Current Solana node reference: `d2.m1.xlarge` = 16c / 512 GB / 2×4 TB NVMe /
      > 50 Gbps = **$1.10/hr ($435/mo at 36-month)**. Trading cluster will need less RAM
      > but similar NVMe for QuestDB — likely a mid-tier SKU in the $300–500/mo range.
- [ ] **Solana exporter** — deploy once RPC is back up; scrape via `10.2.0.11:9179`
- [ ] **Twingate connector** — provision for trading cluster; define resources
      (Grafana, talosctl endpoint, kubectl proxy)
- [ ] **BMC network firewall** — configure UDP rate-limit rules once credentials available

---

## OpenBao Transit — Private Key Implementation

> **Decision: use Transit, not KV.** The additional lift is small; the security gain is
> significant. A container escape or memory dump cannot exfiltrate a key that never exists
> in process memory.

### How it works

```
Bot process                          OpenBao (Transit engine)
    │                                        │
    │  1. Exchange SA token for Vault token  │
    │ ─────────────────────────────────────► │
    │                                        │  Key: ed25519, never exportable
    │  2. POST /v1/transit/sign/bot-key      │
    │     body: { input: base64(tx_bytes) }  │
    │ ─────────────────────────────────────► │
    │                                        │  Signs internally
    │  3. Returns: { signature: base64(...) }│
    │ ◄───────────────────────────────────── │
    │                                        │
    │  4. Attach signature to transaction    │
    │     and submit to RPC                  │
```

The key is created once in OpenBao with `exportable=false` and `allow_plaintext_backup=false`.
It cannot be extracted by anyone — not even with root access to OpenBao, without the unseal
keys. Every signing operation is logged in the OpenBao audit trail.

### OpenBao setup (per bot)

```bash
# Create ed25519 signing key for bot (non-exportable)
vault write transit/keys/bot-alpha type=ed25519

# Policy: bot-alpha can only sign with its own key
vault policy write bot-alpha-policy - <<EOF
path "transit/sign/bot-alpha" { capabilities = ["update"] }
path "transit/verify/bot-alpha" { capabilities = ["update"] }
EOF

# Kubernetes auth role: pod in bot-alpha namespace gets this policy
vault write auth/kubernetes/role/bot-alpha \
  bound_service_account_names=bot-alpha \
  bound_service_account_namespaces=bot-alpha \
  policies=bot-alpha-policy \
  ttl=1h
```

### Bot code change (Solana / Rust example)

Instead of:
```rust
let keypair = Keypair::read_from_file("~/.config/solana/id.json")?;
transaction.sign(&[&keypair], recent_blockhash);
```

The bot calls the Transit API and injects the signature:
```rust
// 1. Get vault token via k8s auth
// 2. POST to /v1/transit/sign/bot-alpha with base64(message_bytes)
// 3. Decode signature, attach to transaction manually
// Solana transactions accept pre-computed signatures
```

This is ~50-100 lines of code per bot, straightforward HTTP client calls. No special
libraries needed — OpenBao's API is plain JSON over HTTPS.
