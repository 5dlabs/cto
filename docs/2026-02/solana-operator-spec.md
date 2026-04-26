# Solana Kubernetes Operator — Spec

> **Status:** Design / Future Work  
> **Target repo:** [5dlabs/kotal](https://github.com/5dlabs/kotal) (fork)  
> **Priority:** Implement after first manual deployment is stable

---

## Motivation

No production-quality Kubernetes operator exists for Solana validators or RPC nodes.
[Kotal](https://github.com/kotalco/kotal) supports Ethereum, NEAR, Polkadot, Aptos, Bitcoin
and others — but not Solana.

A Solana operator in the kotal fork would:
- Automate lifecycle (start, stop, upgrade, health checks)
- Enforce correct disk layout (accounts/ledger/snapshots on separate PVs)
- Manage identity key as a K8s Secret
- Handle snapshot bootstrapping on first start
- Integrate Yellowstone gRPC as a sidecar
- Enable one-line upgrades: change `spec.image` → operator handles drain + restart
- Make Solana a first-class citizen alongside NEAR, Ethereum, etc. in the CTO platform

---

## CRD Design

### `solana.kotal.io/v1alpha1` — `Node`

```yaml
apiVersion: solana.kotal.io/v1alpha1
kind: Node
metadata:
  name: mainnet-rpc
  namespace: solana
spec:
  # Network
  network: mainnet-beta               # or testnet, devnet
  
  # Client
  client: jito                        # jito | agave | firedancer
  image: ghcr.io/jito-foundation/jito-solana:v3.1.8
  
  # Identity
  identitySecret: solana-identity     # K8s Secret with identity.json keypair
  
  # Storage — each maps to a separate PersistentVolumeClaim
  storage:
    ledger:
      size: 500Gi
      storageClass: local-nvme-fast
    accounts:
      size: 2Ti
      storageClass: local-nvme-fast
    snapshots:
      size: 300Gi
      storageClass: local-nvme-fast
  
  # RPC
  rpc:
    enabled: true
    port: 8899
    fullApi: true
    transactionHistory: true
    maxMultipleAccounts: 5000
    threads: 16
  
  # Geyser plugin (Yellowstone gRPC)
  geyser:
    enabled: true
    config: yellowstone-grpc-config   # K8s ConfigMap name
    port: 10000
  
  # Performance
  resources:
    cpu: "24"                         # dedicated cores
    memory: 512Gi
    hugepages2Mi: 0                   # optional huge pages
  
  # Validator flags
  limitLedgerSize: 50000000
  snapshotIntervalSlots: 5000
  fullSnapshotIntervalSlots: 50000
  accountsDbCacheLimitMb: 8192
  accountsIndexBins: 128
  noVoting: true
  walRecoveryMode: skip_any_corrupted_record
  
  # Known validators (gossip trust anchors)
  knownValidators:
    - 7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2
    - GdnSyH3YtwcxFvQrVVJMm1JhTS4QVX7MFsX56uJLUfiZ
    - DE1bawNcRJB9rVm3buyMVfr8mBEoyyu73NBovf2oXJsJ
    - CakcnaRDHka2gXyfbEd2d3xsvkJkqsLw2akB3zsN1D2S
  
  # Networking — run in host network for best performance
  hostNetwork: true
```

---

## Controller Responsibilities

### Reconciliation Loop

```
Observe Node CR
  → Ensure PVCs exist (ledger, accounts, snapshots)
  → Ensure identity Secret exists and is valid keypair
  → Ensure StatefulSet/DaemonSet exists with correct spec
  → Ensure Service (ClusterIP :8899, :10000) exists
  → Check health via RPC getHealth endpoint
  → Update Node.status (synced slot, gap, replay rate)
  → Emit events on state changes
```

### Health Status

```go
type NodeStatus struct {
    Phase           NodePhase  // Pending, Syncing, Ready, Error
    CurrentSlot     int64
    NetworkSlot     int64
    SlotGap         int64
    ReplayRate      float64    // slots/sec
    SwapUsageMb     int64
    LastRestartTime metav1.Time
    Conditions      []NodeCondition
}
```

### Upgrade Path

```
spec.image changed
  → Operator sets Node.status.phase = Upgrading
  → Sends SIGTERM to validator (graceful shutdown)
  → Waits for clean stop (up to 120s)
  → Updates StatefulSet image
  → Watches for RPC to come back up
  → Sets phase = Syncing → Ready
```

---

## Snapshot Bootstrap

On first start (no local snapshot):
1. Operator detects empty snapshots PVC
2. Calls `solana-cluster fetch` (Blockdaemon sidecar pattern) if a cluster tracker is configured
3. Falls back to mainnet snapshot download
4. Sets `--no-snapshot-fetch=false` until snapshot is present, then flips

```yaml
spec:
  snapshot:
    bootstrapFrom: cluster-tracker    # cluster-tracker | mainnet | none
    trackerUrl: http://solana-tracker:8458
    minSlotAge: 500                   # refuse snapshots older than N slots behind tip
```

---

## Disk Layout Enforcement

The operator enforces correct disk layout as a **validating webhook**:

```go
// Reject Node if accounts and snapshots share the same storageClass AND
// that storageClass maps to the same physical device as ledger.
// Lesson: snapshot writes (20GB/33min) on accounts disk = I/O saturation.
func validateDiskLayout(node *Node) error {
    if node.Spec.Storage.Accounts.StorageClass == 
       node.Spec.Storage.Snapshots.StorageClass &&
       node.Spec.Storage.Accounts.StorageClass ==
       node.Spec.Storage.Ledger.StorageClass {
        return fmt.Errorf("accounts, snapshots, and ledger must use separate storage classes")
    }
    return nil
}
```

---

## Kotal Fork Integration

### New API Group
`solana.kotal.io/v1alpha1` — mirrors the pattern of `near.kotal.io/v1alpha1`

### Files to Add
```
apis/solana/v1alpha1/
  node_types.go          # Node CRD types
  node_webhook.go        # Validating + defaulting webhook
  groupversion_info.go

controllers/
  solana/
    node_controller.go   # Main reconciler
    node_statefulset.go  # StatefulSet builder
    node_service.go      # Service builder
    node_pvc.go          # PVC manager
    node_health.go       # Health checker (RPC polling)
```

### Reference Implementation
Model after `controllers/near/node_controller.go` — NEAR has similar requirements
(large state, multiple disk volumes, long sync times).

---

## Phased Delivery

### Phase 1 — Manual (now)
- Deploy validator on bare metal manually
- Document every step, flag, and decision
- Build institutional knowledge

### Phase 2 — Helm Chart
- Package the bare metal setup as a Helm chart
- `helm install solana-rpc ./charts/solana-validator`
- Handles: systemd service, startup script, sysctl tuning, swap config

### Phase 3 — Operator (future)
- Implement `solana.kotal.io/v1alpha1` in kotal fork
- K8s-native lifecycle management
- Works when validator eventually moves to K8s (after Talos migration matures)

---

## Open Questions

1. **hostPath vs local PV** — hostPath is simpler, local PV gives K8s visibility into disk usage
2. **DaemonSet vs StatefulSet** — DaemonSet with nodeSelector pins to specific hardware;
   StatefulSet with PVC affinity achieves the same differently
3. **Firedancer support** — Firedancer is a new Solana client in C; its container image
   story is different from agave/jito. Phase 3 could add `client: firedancer`.
4. **Metrics** — operator should export Prometheus metrics scraping the validator's
   internal metrics endpoint (`--rpc-port` has `/metrics` in newer builds)

---

## References

- [5dlabs/kotal](https://github.com/5dlabs/kotal) — our fork
- [kotalco/kotal NEAR controller](https://github.com/kotalco/kotal/tree/master/controllers/near) — reference impl
- [Blockdaemon/solana-cluster](https://github.com/Blockdaemon/solana-cluster) — snapshot management pattern
- [Agave validator flags](https://docs.anza.xyz/operations/setup-an-rpc-node)
