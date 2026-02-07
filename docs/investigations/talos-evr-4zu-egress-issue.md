# Node Network Egress Issue: talos-evr-4zu

## Status: ✅ RESOLVED (2026-01-04)

## Summary

The node `talos-evr-4zu` had broken network egress to external IPs (specifically 8.8.8.8:53 for DNS). This caused cluster-wide DNS failures because CoreDNS pods were scheduled on this node.

## Resolution

Two issues were identified and fixed:

### Root Cause 1: Wrong Default Gateway

The node had an incorrect default gateway configured (`192.168.1.254`) instead of the correct one (`192.168.1.1`). This was likely left over from a router change.

**Fix applied:**
```bash
talosctl -n 192.168.1.77 patch machineconfig --patch @/tmp/gateway-fix.yaml
# Patch content: Replace gateway 192.168.1.254 → 192.168.1.1
talosctl -n 192.168.1.77 reboot
```

### Root Cause 2: Flannel/Cilium CNI Conflict

Both Flannel and Cilium CNIs were running simultaneously. Talos was deploying Flannel by default, conflicting with the Cilium CNI.

**Fix applied:**
```bash
# 1. Disable Flannel in Talos cluster config on control plane
talosctl -n 192.168.1.77 patch machineconfig --patch '{"cluster":{"network":{"cni":{"name":"none"}}}}'

# 2. Remove Flannel DaemonSet and resources
kubectl delete daemonset kube-flannel -n kube-system
kubectl delete configmap kube-flannel-cfg -n kube-system
kubectl delete serviceaccount flannel -n kube-system
kubectl delete clusterrolebinding flannel
kubectl delete clusterrole flannel

# 3. Clean up stale flannel.1 interface and routes
kubectl exec -n kube-system <cilium-pod> -- ip link delete flannel.1
kubectl exec -n kube-system <cilium-pod> -- ip route del 10.4.0.1 via 10.244.3.0 dev flannel.1
kubectl exec -n kube-system <cilium-pod> -- ip route del 10.5.0.1 via 10.244.3.0 dev flannel.1

# 4. Restart Cilium to regenerate BPF programs
kubectl delete pod -n kube-system <cilium-pod-on-affected-node>
```

### Verification

After fixes, egress works correctly:
```
PING 8.8.8.8 (8.8.8.8): 56 data bytes
64 bytes from 8.8.8.8: seq=0 ttl=118 time=90.364 ms
64 bytes from 8.8.8.8: seq=1 ttl=118 time=62.001 ms
64 bytes from 8.8.8.8: seq=2 ttl=118 time=95.548 ms
--- 8.8.8.8 ping statistics ---
3 packets transmitted, 3 packets received, 0% packet loss
```

---

## Original Investigation (for reference)

## Evidence

### CoreDNS Logs (before fix)
```
[ERROR] plugin/errors: 2 ghcr.io. A: read udp 10.244.0.81:39727->8.8.8.8:53: i/o timeout
[ERROR] plugin/errors: 2 ghcr.io. AAAA: read udp 10.244.0.81:40920->8.8.8.8:53: i/o timeout
[ERROR] plugin/errors: 2 github.com. A: read udp 10.244.0.81:57391->8.8.8.8:53: i/o timeout
```

The CoreDNS pod IP `10.244.0.81` was on node `talos-evr-4zu`.

### Node Comparison Test

| Node | Ping 8.8.8.8 | Result |
|------|--------------|--------|
| `talos-irs-cis` | ✅ Success | 8.8ms RTT |
| `talos-tq1-f47` | ✅ Success | Working |
| `talos-evr-4zu` | ❌ **Timeout** | Cannot reach external IPs |

Test command used:
```bash
kubectl run test-evr -n kube-system --rm -it --image=busybox:1.36 --restart=Never \
  --overrides='{"spec":{"nodeSelector":{"kubernetes.io/hostname":"talos-evr-4zu"}}}' \
  -- sh -c "ping -c 2 8.8.8.8"
```

## Impact

- ~50% of cluster DNS queries failed (round-robin to broken CoreDNS pod)
- All pods attempting external network access from this node fail
- Agent pods (CodeRun) failed during git clone with "Could not resolve host"

## Temporary Workaround Applied

```bash
# Deleted the CoreDNS pod on the broken node
kubectl delete pod -n kube-system coredns-5bcd668f9b-tdlsl

# Pod rescheduled to talos-tq1-f47 (healthy)
```

## Investigation Steps for Permanent Fix

### 1. Check Node Network Configuration
```bash
# SSH or use talosctl to access the node
talosctl -n 192.168.1.77 get addresses
talosctl -n 192.168.1.77 get routes
talosctl -n 192.168.1.77 get links
```

### 2. Check Cilium Status on the Node
```bash
# Get Cilium pod on talos-evr-4zu
kubectl get pods -n kube-system -l app.kubernetes.io/name=cilium -o wide | grep talos-evr-4zu

# Check Cilium agent status
kubectl exec -n kube-system cilium-zcvd6 -- cilium status

# Check BPF routes
kubectl exec -n kube-system cilium-zcvd6 -- cilium bpf nat list
kubectl exec -n kube-system cilium-zcvd6 -- cilium bpf egress list
```

### 3. Test from the Node Directly
```bash
# Test from a pod on that specific node
kubectl run test-node --rm -it --image=alpine:latest --restart=Never \
  --overrides='{"spec":{"nodeSelector":{"kubernetes.io/hostname":"talos-evr-4zu"}}}' \
  -- sh -c "apk add --no-cache curl && curl -v --connect-timeout 10 https://github.com"
```

### 4. Check iptables/nftables Rules
```bash
talosctl -n 192.168.1.77 read /proc/net/nf_conntrack | head -50
talosctl -n 192.168.1.77 dmesg | grep -i "drop\|reject\|denied"
```

### 5. Check Physical Network
- Is the node on the same switch/VLAN as other nodes?
- Any firewall rules specific to this node's MAC/IP?
- Check upstream router/firewall logs

## Node Information

| Property | Value |
|----------|-------|
| Hostname | `talos-evr-4zu` |
| Node IP | `192.168.1.77` |
| Pod CIDR | `10.244.0.0/24` |
| Cilium Pod | `cilium-zcvd6` |
| Role | Control plane |

## Potential Causes

1. **Cilium BPF maps corrupted** - Restart Cilium agent on that node
2. **Node NAT/masquerade broken** - Check iptables MASQUERADE rules
3. **Physical network issue** - Cable, switch port, VLAN config
4. **Talos network config drift** - Compare machine config with other nodes
5. **Conntrack table full** - Check `/proc/sys/net/netfilter/nf_conntrack_max`

## Quick Fix Attempt

If investigation is taking too long, try restarting Cilium on the node:
```bash
kubectl delete pod -n kube-system cilium-zcvd6
```

Or cordon the node to prevent scheduling:
```bash
kubectl cordon talos-evr-4zu
```

## Related Issues

- DNS race condition fix: PR #3595 (merged)
- mcp-check partial fix: commit `3dbef85` (deployed)
