# Node Network Egress Issue: talos-evr-4zu

## Status: ⚠️ Workaround Applied, Root Cause Unknown

## Summary

The node `talos-evr-4zu` has broken network egress to external IPs (specifically 8.8.8.8:53 for DNS). This caused cluster-wide DNS failures because CoreDNS pods were scheduled on this node.

**Temporary Fix:** Deleted the CoreDNS pod on `talos-evr-4zu`, which rescheduled to a healthy node.

**Permanent Fix Needed:** Investigate and fix the node's network egress.

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
