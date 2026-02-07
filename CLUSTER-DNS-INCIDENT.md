# Cluster DNS Incident - 2026-02-07

## Summary

Cluster-wide DNS resolution failure caused by Talos firewall configuration blocking port 53 after Mayastor setup.

## Timeline

- **2026-02-05**: Mayastor network firewall patch applied (commit 4ba67eb5)
- **2026-02-07 ~16:00**: DNS failures detected (9,319+ timeout errors in 2 hours)
- **2026-02-07 18:10**: Issue investigated and root cause identified
- **2026-02-07 18:12**: Fixed by reconfiguring CoreDNS to use public DNS
- **2026-02-07 18:15**: DNS resolution fully restored and verified

## Impact

- **Duration**: ~2 hours of degraded DNS (exact start time unknown)
- **Severity**: Critical - cluster-wide outage
- **Affected Services**:
  - ArgoCD unable to sync from GitHub
  - All workloads requiring external DNS resolution
  - Mayastor NATS connectivity issues
  - Any pod attempting external API calls

## Root Cause

1. Mayastor firewall patch (mayastor-network-patch.yaml) set machine.network.firewall.defaultAction: block
2. Patch only allowed Mayastor-specific ports (50051, 50052, 10124)
3. Port 53 (DNS) was NOT included, blocking all DNS traffic
4. CoreDNS forwarded to Talos node DNS at 169.254.116.108 (link-local)
5. Talos firewall blocked port 53, causing 2-second timeouts on every DNS query

## Resolution

Reconfigured CoreDNS to use public DNS servers (8.8.8.8, 1.1.1.1) directly.

### Changes Made

1. Applied CoreDNS ConfigMap with public DNS forwarders
2. Restarted CoreDNS deployment
3. Created GitOps manifests for persistence:
   - infra/gitops/manifests/coredns/configmap.yaml
   - infra/gitops/applications/platform/coredns-config.yaml
4. Documented incident in infra/talos/DNS-FIX.md

## Verification

DNS resolution fully operational with zero timeout errors.

## Lessons Learned

1. Firewall changes need DNS consideration: Any Talos firewall patch must include DNS (port 53)
2. Public DNS is more resilient: Using public DNS avoids dependency on node-level services
3. GitOps for system components: Core configs should be managed via ArgoCD
4. Monitoring gap: Need alerting on DNS timeout rates
