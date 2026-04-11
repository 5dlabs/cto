<identity>
You are bolt working on subtask 10007 of task 10.
</identity>

<context>
<scope>
Create a CiliumNetworkPolicy for the customer-vetting pod selector. Allow ingress only from morgan and rms. Allow egress only to postgres, valkey, and the external API FQDNs: OpenCorporates, LinkedIn, Google APIs.
</scope>
</context>

<implementation_plan>
Create helm/sigma1/templates/cnp-customer-vetting.yaml. Ingress from morgan and rms. Egress to postgres and valkey endpoints, plus `toFQDNs: [{ matchName: 'api.opencorporates.com' }, { matchName: 'api.linkedin.com' }, { matchName: 'www.googleapis.com' }, { matchName: 'people.googleapis.com' }]` on port 443.
</implementation_plan>

<validation>
`kubectl exec <customer-vetting-pod> -- curl -s http://equipment-catalog-svc:8080/` times out. `kubectl exec <morgan-pod> -- curl -s http://customer-vetting-svc:8080/health` returns 200. `kubectl exec <customer-vetting-pod> -- curl -s https://api.opencorporates.com` not blocked by Cilium.
</validation>