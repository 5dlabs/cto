# Latitude Solana Capacity Map (2026-03-25)

This document maps the Cherry Solana stack to currently in-stock Latitude capacity with parity-first sizing and explicit headroom.

## Inputs

- Cherry runtime capture: `docs/solana-cherry-installation-capture-2026-03-25.md`
- Solana manifests: `skills/trader/k8s/*.yaml`
- Live Latitude inventory (via `metal plans --provider latitude --in-stock`, 2026-03-25)

## Capacity Targets

Headroom policy used for selection:

- CPU sustained target: `< 70%`
- RAM sustained target: `< 75%`
- Disk sustained target: `< 70%`

Workload classes to support:

- **Solana validator + Yellowstone** (`agave-rpc`) with high-memory profile
- **Data plane** (`questdb`, `dex-indexer`, `dex-api`) with NVMe and 10G
- **Kubernetes control plane / platform ops**

## Region/Site Selection

Selected site: **NYC** (same site for all nodes to preserve private networking assumptions and avoid cross-site VLAN issues).

Reason:

- Required plans are simultaneously in stock in NYC.
- Keeps all nodes in one Latitude site, matching `metal` same-site networking constraints.

## Cherry -> Latitude Mapping

| Role | Cherry intent | Latitude plan | Key specs | Price |
|---|---|---|---|---|
| `control-plane-01` | Cluster control plane + GitOps/ops | `m4-metal-small` | 6 cores, 64 GB RAM, 2x960GB NVMe, 2x10Gbps | `$0.81/hr` |
| `solana-rpc-01` | Validator + Yellowstone (high memory) | `m3-large-x86` | 32 cores, 1024 GB RAM, 2x3.8TB NVMe, 2x10Gbps | `$2.57/hr` |
| `db-01` | QuestDB + dex-indexer + dex-api | `c3-large-x86` | 24 cores, 256 GB RAM, 2x1.9TB NVMe, 2x10Gbps | `$1.36/hr` |

Total estimated cost:

- **Hourly:** `$4.74/hr`
- **Monthly (730h):** `$3460.20/mo`

## Parity Checks Against Current Solana Manifests

- `agave-rpc.yaml` requests `16 CPU / 740Gi` for validator sidecar pod.
  - `m3-large-x86` (1024 GB RAM) supports this request with memory headroom.
- `questdb.yaml` requests `2 CPU / 16Gi`, limits `4 CPU / 32Gi`.
  - `c3-large-x86` leaves substantial headroom for query/ingest spikes.
- `dex-indexer.yaml` and `dex-api.yaml` resource requests are small relative to `c3-large-x86`.
- Node pinning in manifests expects hostnames `solana-rpc-01` and `db-01`.
  - Hostnames above are chosen to match this with minimal manifest drift.

## Risk Notes

- `m4-metal-xlarge` is available but materially increases cost; not required for parity with current configured requests.
- If validator memory pressure appears during shadow run, first fallback is `m4-metal-xlarge` in same site where stock allows.

## Billing Guardrail

Given early billing credits, run this environment in shadow mode and stop non-critical nodes when idle if compatible with validation timeline.

## Execution Parameters (for next phases)

- Provider: `latitude`
- Site: `NYC`
- Hostnames:
  - `control-plane-01`
  - `solana-rpc-01`
  - `db-01`
- Talos version baseline: `v1.9.0` (current `metal` defaults)
- Install disk for Talos: `/dev/nvme0n1` (NVMe-based plans)
