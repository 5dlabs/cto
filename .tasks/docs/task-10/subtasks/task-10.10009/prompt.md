Implement subtask 10009: Author CiliumNetworkPolicy for Morgan service

## Objective
Create a CiliumNetworkPolicy for the Morgan pod selector. Allow ingress from cloudflared pods (Cloudflare Tunnel) and any web clients arriving through the tunnel. Allow egress to all 5 backend services, signal-cli pod, and the external FQDNs: ElevenLabs, Twilio.

## Steps
Create helm/sigma1/templates/cnp-morgan.yaml. Ingress from pods with label `app: cloudflared` and from within the sigma1 namespace on the Morgan service port. Egress to all 5 service pods (equipment-catalog, rms, finance, customer-vetting, social-engine) and `app: signal-cli`, plus `toFQDNs: [{ matchName: 'api.elevenlabs.io' }, { matchName: 'api.twilio.com' }]` on port 443. Deny ingress from arbitrary internet IPs (only cloudflared terminates external traffic).

## Validation
`kubectl exec <finance-pod> -- curl -s http://morgan-svc:3000/health` connection refused. `kubectl exec <cloudflared-pod> -- curl -s http://morgan-svc:3000/health` returns 200. `kubectl exec <morgan-pod> -- curl -s http://equipment-catalog-svc:8080/health` returns 200.