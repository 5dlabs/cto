<identity>
You are bolt working on subtask 10004 of task 10.
</identity>

<context>
<scope>
Create a CiliumNetworkPolicy for the equipment-catalog namespace/pod selector. Allow ingress only from morgan, blaze-website, and mobile-app pods. Allow egress only to postgres, valkey, and the R2 endpoint CIDR/FQDN. Deny all other traffic by default.
</scope>
</context>

<implementation_plan>
Create helm/sigma1/templates/cnp-equipment-catalog.yaml: `apiVersion: cilium.io/v2, kind: CiliumNetworkPolicy, metadata.name: equipment-catalog-policy, spec.endpointSelector: { matchLabels: { app: equipment-catalog } }, spec.ingress: [{ fromEndpoints: [{ matchLabels: { app: morgan } }, { matchLabels: { app: blaze-website } }, { matchLabels: { app: mobile-ingress } }] }], spec.egress: [{ toEndpoints: [{ matchLabels: { app: postgres } }, { matchLabels: { app: valkey } }] }, { toFQDNs: [{ matchName: '<account>.r2.cloudflarestorage.com' }], toPorts: [{ ports: [{ port: '443', protocol: TCP }] }] }]`. Apply with helm upgrade.
</implementation_plan>

<validation>
`kubectl exec -n sigma1 <equipment-catalog-pod> -- curl -s http://finance-svc:8080/health` returns connection timeout or refused (blocked). `kubectl exec -n sigma1 <morgan-pod> -- curl -s http://equipment-catalog-svc:8080/health` returns 200 (allowed). `kubectl exec` from finance pod to equipment-catalog must be blocked.
</validation>