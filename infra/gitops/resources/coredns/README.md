# CoreDNS DNSSEC Configuration

This directory contains the CoreDNS configuration with DNSSEC validation enabled to address RFC6840 compliance issues.

## Overview

The CoreDNS configuration has been updated to enable DNSSEC validation, which addresses the issue mentioned in [CoreDNS issue #5189](https://github.com/coredns/coredns/issues/5189) where CoreDNS cache was violating RFC6840.

## Configuration Changes

### DNSSEC Validation
- **Enabled DNSSEC validation** in the cache plugin
- **Multiple upstream resolvers** with health checks for reliability
- **Sequential policy** for upstream resolution to ensure consistent DNSSEC validation

### Upstream Resolvers
The configuration uses multiple DNSSEC-capable upstream resolvers:
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

The configuration is deployed via ArgoCD as the `coredns-config` application. After deployment, CoreDNS pods will be automatically restarted to pick up the new configuration.

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

## Related Issues

This configuration addresses:
- [CoreDNS issue #5189](https://github.com/coredns/coredns/issues/5189) - CoreDNS cache violates RFC6840
- [Mailu issue #2239](https://github.com/Mailu/Mailu/issues/2239) - DNSSEC not working properly with CoreDNS
