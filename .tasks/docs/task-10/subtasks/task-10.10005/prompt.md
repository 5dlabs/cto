Implement subtask 10005: Author CiliumNetworkPolicy for RMS service

## Objective
Create a CiliumNetworkPolicy for the rms pod selector. Allow ingress only from morgan and equipment-catalog pods. Allow egress only to postgres, valkey, and Google Calendar API FQDN.

## Steps
Create helm/sigma1/templates/cnp-rms.yaml with `spec.endpointSelector: { matchLabels: { app: rms } }`. Ingress from morgan and equipment-catalog pods. Egress to postgres and valkey endpoints, plus `toFQDNs: [{ matchName: 'www.googleapis.com' }, { matchName: 'calendar.googleapis.com' }]` on port 443. Default deny all other traffic.

## Validation
`kubectl exec <rms-pod> -- curl -s http://finance-svc:8080/` times out (blocked). `kubectl exec <morgan-pod> -- curl -s http://rms-svc:8080/health` returns 200. `kubectl exec <rms-pod> -- curl -s https://calendar.googleapis.com` returns response (not blocked by Cilium).