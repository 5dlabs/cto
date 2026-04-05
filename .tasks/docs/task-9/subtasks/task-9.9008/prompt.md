Implement subtask 9008: Implement Cilium egress network policies for external API access

## Objective
Create CiliumNetworkPolicy egress rules to allow sigma1 services to reach external APIs (Stripe, OpenCorporates, etc.) while denying other outbound traffic.

## Steps
1. Identify all external API endpoints that services need to reach:
   - Stripe API: `api.stripe.com` (finance service)
   - OpenCorporates API: `api.opencorporates.com` (customer-vetting service)
   - Cloudflare R2 endpoint: `<account-id>.r2.cloudflarestorage.com` (equipment-catalog, social-engine)
   - Any AI/LLM API endpoints (Morgan)
2. Create egress CiliumNetworkPolicy:
   - Default deny egress (except DNS on port 53 to kube-dns)
   - Allow finance → `api.stripe.com:443`
   - Allow customer-vetting → `api.opencorporates.com:443`
   - Allow relevant services → R2 endpoint on port 443
   - Allow Morgan → AI API endpoint on port 443
   - Use `toFQDNs` selector for domain-based egress rules
3. Ensure DNS egress is allowed for all pods:
   - Allow egress to `kube-dns.kube-system` on port 53 (UDP and TCP)
4. Apply policies and verify services can still reach their required external APIs.

## Validation
Exec into finance pod, `curl -s -o /dev/null -w '%{http_code}' https://api.stripe.com` — verify 200 or 401 (reachable). Exec into finance pod, `curl -s --connect-timeout 5 https://example.com` — verify timeout (egress denied). Exec into customer-vetting pod, verify OpenCorporates API is reachable. Verify DNS resolution still works from all pods.