Implement subtask 9007: End-to-end verification of HA, CDN, TLS, and ingress

## Objective
Perform comprehensive end-to-end testing to verify all production hardening changes work together: HA failover, CDN caching, TLS termination, ingress routing, and network policy enforcement.

## Steps
1. Run a full connectivity matrix test: from external → Cloudflare → Tunnel → services → databases.
2. Simulate PostgreSQL primary failure and verify application continues serving requests.
3. Simulate Redis master failure and verify application continues with brief interruption.
4. Kill one replica of each backend service and verify continued availability.
5. Verify CDN cache hit ratios for static assets.
6. Verify TLS certificates are valid for all public-facing endpoints.
7. Verify network policies block unauthorized cross-namespace traffic using a test pod.
8. Document any issues found and remediate.

## Validation
All 6 backend services remain available during single-pod failures. PostgreSQL and Redis failover completes within 30 seconds. CDN serves cached assets with HIT status. All public URLs respond with valid TLS. Unauthorized network connections are blocked. Create a test report documenting each scenario's pass/fail status.