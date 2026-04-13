<identity>
You are bolt working on subtask 10006 of task 10.
</identity>

<context>
<scope>
Create a CiliumNetworkPolicy for the finance pod selector. Allow ingress only from morgan and rms pods. Allow egress only to postgres, valkey, and Stripe API FQDN.
</scope>
</context>

<implementation_plan>
Create helm/sigma1/templates/cnp-finance.yaml with `spec.endpointSelector: { matchLabels: { app: finance } }`. Ingress from morgan and rms. Egress to postgres and valkey endpoints, plus `toFQDNs: [{ matchName: 'api.stripe.com' }]` on port 443. Default deny all else.
</implementation_plan>

<validation>
`kubectl exec <finance-pod> -- curl -s http://social-engine-svc:8080/` connection refused (blocked). `kubectl exec <rms-pod> -- curl -s http://finance-svc:8080/health` returns 200. `kubectl exec <finance-pod> -- curl -s https://api.stripe.com` returns response (not ETIMEDOUT).
</validation>