# Cilium-Only Invariant

This cluster profile is **Cilium-only**:

- No `kube-flannel` DaemonSet
- No `kube-proxy` DaemonSet
- Talos machine config includes:
  - `cluster.proxy.disabled: true`
  - `cluster.network.cni.name: none`
- Cilium is installed with kube-proxy replacement enabled

## Required Build-Time Enforcement

- `crates/metal/src/bin/metal.rs` bootstrap/provision/cluster flows call `with_cilium_cni()`.
- `crates/installer/src/orchestrator.rs` uses `with_cilium_cni()` as the canonical CNI path.
- `crates/metal/src/cilium.rs` provides `assert_cilium_only_network_plane()` guardrails.

## Required Runtime Gates

Run these commands and require success before cutover:

```bash
KUBECONFIG=/path/to/kubeconfig kubectl get ds -n kube-system
KUBECONFIG=/path/to/kubeconfig cilium status
```

Expected:

- `kube-system` daemonsets include `cilium` and `cilium-envoy`.
- No `kube-flannel` and no `kube-proxy`.
- `cilium status` shows Cilium/Operator/Envoy/Hubble as `OK`.

## Talos Patch for Existing Clusters

Use this JSON patch on every control-plane and worker node:

```json
[
  { "op": "add", "path": "/cluster/proxy", "value": { "disabled": true } },
  { "op": "add", "path": "/cluster/network/cni", "value": { "name": "none" } }
]
```

Example:

```bash
talosctl --talosconfig /path/to/talosconfig -e <control-plane-ip> -n <node-ip> \
  patch machineconfig --mode no-reboot --patch '[{"op":"add","path":"/cluster/proxy","value":{"disabled":true}},{"op":"add","path":"/cluster/network/cni","value":{"name":"none"}}]'
```

## Migration Safety Checks

- Snapshot current daemonsets and node readiness before migration.
- Remove legacy network resources after Talos patch:
  - `kube-flannel` daemonset and RBAC/service-account resources
  - `kube-proxy` daemonset/service-account/configmap resources
- Install Cilium immediately after removal.
- Re-check service and workload readiness (`solana`, `questdb`, `observability`) after network cutover.
