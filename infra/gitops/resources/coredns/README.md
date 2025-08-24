# CoreDNS DNSSEC Configuration for Talos Linux

This directory contains the CoreDNS configuration with DNSSEC validation enabled to address RFC6840 compliance issues in Talos-managed CoreDNS deployments.

## Overview

The CoreDNS configuration has been updated to enable DNSSEC validation, which addresses the issue mentioned in [CoreDNS issue #5189](https://github.com/coredns/coredns/issues/5189) where CoreDNS cache was violating RFC6840.

**Note**: This configuration is designed for Talos Linux-managed CoreDNS deployments, not standalone Helm chart deployments.

## Configuration Changes

### DNSSEC Validation
- **Enabled DNSSEC validation** in the cache plugin
- **Multiple upstream resolvers** with health checks for reliability
- **Sequential policy** for upstream resolution to ensure consistent DNSSEC validation

### Upstream Resolvers
The configuration uses multiple DNSSEC-capable upstream resolvers instead of relying on `/etc/resolv.conf`:
- `8.8.8.8:53` (Google DNS)
- `8.8.4.4:53` (Google DNS secondary)
- `1.1.1.1:53` (Cloudflare DNS)
- `1.0.0.1:53` (Cloudflare DNS secondary)

### Cache Configuration
- **DNSSEC-aware caching** with `dnssec` option enabled
- **30-second TTL** for cached responses
- **Disabled caching** for cluster.local to avoid conflicts

## Benefits

1. **RFC6840 Compliance**: Proper DNSSEC validation prevents cache poisoning attacks
2. **Security**: Validates DNS responses against DNSSEC signatures
3. **Reliability**: Multiple upstream resolvers with health checks
4. **Performance**: Maintains caching while ensuring security

## Deployment

### Option 1: Direct Application (Recommended)
Use the provided script to apply the configuration directly:

```bash
./infra/scripts/apply-coredns-dnssec.sh
```

### Option 2: Manual Application
```bash
# Apply the ConfigMap
kubectl apply -f infra/gitops/resources/coredns/coredns-configmap.yaml

# Restart CoreDNS pods
kubectl rollout restart deployment/coredns -n kube-system

# Wait for rollout
kubectl rollout status deployment/coredns -n kube-system
```

## Verification

To verify DNSSEC validation is working:

```bash
# Check CoreDNS logs for DNSSEC validation
kubectl logs -n kube-system deployment/coredns

# Test DNSSEC validation from a pod
kubectl run test-dns --image=busybox --rm -it --restart=Never -- nslookup -type=ANY google.com

# Check if DNSSEC records are being validated
kubectl run test-dnssec --image=busybox --rm -it --restart=Never -- nslookup -type=DNSKEY google.com
```

## Troubleshooting

### Common Issues

1. **DNS Resolution Failures**: Check if upstream resolvers are reachable
2. **Performance Issues**: Monitor CoreDNS metrics for cache hit rates
3. **DNSSEC Validation Errors**: Check logs for DNSSEC-related errors

### Logs to Monitor

```bash
# CoreDNS logs
kubectl logs -n kube-system deployment/coredns

# CoreDNS metrics
kubectl port-forward -n kube-system service/coredns 9153:9153
curl http://localhost:9153/metrics | grep dns
```

## Talos Integration

This configuration is compatible with Talos Linux's CoreDNS deployment:
- **Version**: CoreDNS v1.12.1 (as deployed by Talos)
- **Namespace**: kube-system
- **Labels**: k8s-app=kube-dns, kubernetes.io/name=CoreDNS
- **ConfigMap**: coredns

## Related Issues

This configuration addresses:
- [CoreDNS issue #5189](https://github.com/coredns/coredns/issues/5189) - CoreDNS cache violates RFC6840
- [Mailu issue #2239](https://github.com/Mailu/Mailu/issues/2239) - DNSSEC not working properly with CoreDNS
