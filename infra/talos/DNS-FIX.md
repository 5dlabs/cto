# DNS Resolution Fix for Talos Cluster

## Problem

After applying the Mayastor network firewall patch (`mayastor-network-patch.yaml`), cluster-wide DNS resolution failed with timeout errors:

```
[ERROR] plugin/errors: 2 github.com. A: read udp 10.244.0.156:48060->169.254.116.108:53: i/o timeout
```

### Root Cause

1. The Mayastor firewall patch sets `machine.network.firewall.defaultAction: block` on all Talos nodes
2. The patch only allows specific Mayastor ports (50051, 50052, 10124) through the firewall
3. **Port 53 (DNS) was NOT allowed**, breaking upstream DNS resolution
4. CoreDNS was configured to forward queries to `/etc/resolv.conf`, which points to Talos node DNS at `169.254.116.108` (link-local address)
5. When CoreDNS tried to reach the node's DNS service, the Talos firewall blocked port 53, causing all external DNS lookups to timeout

## Solution

Changed CoreDNS to use public DNS servers (Google DNS 8.8.8.8 and Cloudflare DNS 1.1.1.1) directly instead of relying on the Talos node DNS.

### Files Changed

- `infra/gitops/manifests/coredns/configmap.yaml` - CoreDNS config with public DNS forwarders
- `infra/gitops/applications/platform/coredns-config.yaml` - ArgoCD app to manage CoreDNS config

### Configuration Change

**Before:**
```yaml
forward . /etc/resolv.conf {
   max_concurrent 1000
}
```

**After:**
```yaml
# Use public DNS servers instead of /etc/resolv.conf
# This avoids issues with Talos node DNS when firewall rules block port 53
forward . 8.8.8.8 1.1.1.1 {
   max_concurrent 1000
}
```

## Alternative Solutions Considered

### Option 1: Add DNS port to Talos firewall (NOT CHOSEN)

Could modify `mayastor-network-patch.yaml` to allow port 53:

```yaml
- portSelector:
    ports:
      - 53
    protocol: udp
  ingress:
    - subnet: 10.0.0.0/8
```

**Why NOT chosen:**
- More complex (requires Talos node patching and reboot)
- Adds unnecessary dependency on Talos node DNS
- Public DNS is more reliable and doesn't require maintaining node-level DNS

### Option 2: Disable Talos firewall defaultAction: block (NOT CHOSEN)

Could change firewall to allowlist mode instead of blocklist.

**Why NOT chosen:**
- Reduces security posture
- Mayastor hostNetwork pods need firewall protection

## Verification

DNS resolution working correctly:

```bash
# Test DNS from within cluster
kubectl run dns-test --image=busybox:1.36 --rm -it --restart=Never --command -- nslookup github.com

# Check CoreDNS logs for errors
kubectl logs -n kube-system -l k8s-app=kube-dns --tail=50 | grep -i error

# Verify no timeout errors
kubectl logs -n kube-system -l k8s-app=kube-dns --since=5m | grep -i timeout | wc -l
# Should return 0
```

## Impact Timeline

- **2026-02-05**: Mayastor network patch applied (commit 4ba67eb5)
- **2026-02-07**: DNS failures detected (9,319+ timeout errors)
- **2026-02-07**: Fixed by switching CoreDNS to public DNS

## Related Issues

- ArgoCD unable to sync apps from GitHub
- Any workload requiring external DNS resolution affected
- Mayastor components may have had DNS issues (mayastor-nats lookups timing out)

## References

- Commit: `4ba67eb5` - Infrastructure: Complete Mayastor storage setup with Talos networking fixes
- File: `infra/talos/config/simple/mayastor-network-patch.yaml`
- File: `infra/talos/config/simple/MAYASTOR-SETUP.md`
