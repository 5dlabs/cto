Implement subtask 10007: Author CiliumNetworkPolicy for customer-vetting service

## Objective
Create a CiliumNetworkPolicy for the customer-vetting pod selector. Allow ingress only from morgan and rms. Allow egress only to postgres, valkey, and the external API FQDNs: OpenCorporates, LinkedIn, Google APIs.

## Steps
Create helm/sigma1/templates/cnp-customer-vetting.yaml. Ingress from morgan and rms. Egress to postgres and valkey endpoints, plus `toFQDNs: [{ matchName: 'api.opencorporates.com' }, { matchName: 'api.linkedin.com' }, { matchName: 'www.googleapis.com' }, { matchName: 'people.googleapis.com' }]` on port 443.

## Validation
`kubectl exec <customer-vetting-pod> -- curl -s http://equipment-catalog-svc:8080/` times out. `kubectl exec <morgan-pod> -- curl -s http://customer-vetting-svc:8080/health` returns 200. `kubectl exec <customer-vetting-pod> -- curl -s https://api.opencorporates.com` not blocked by Cilium.