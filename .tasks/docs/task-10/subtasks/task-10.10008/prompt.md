Implement subtask 10008: Author CiliumNetworkPolicy for social-engine service

## Objective
Create a CiliumNetworkPolicy for the social-engine pod selector. Allow ingress only from morgan. Allow egress to postgres, valkey, R2 endpoint, and the external social/AI API FQDNs: Instagram, LinkedIn, TikTok, Facebook, OpenAI.

## Steps
Create helm/sigma1/templates/cnp-social-engine.yaml. Ingress only from morgan. Egress to postgres and valkey, plus R2 FQDN, `graph.facebook.com`, `api.instagram.com`, `open.tiktokapis.com`, `api.linkedin.com`, `api.openai.com` on port 443.

## Validation
`kubectl exec <rms-pod> -- curl -s http://social-engine-svc:8080/` connection refused. `kubectl exec <morgan-pod> -- curl -s http://social-engine-svc:8080/health` returns 200. `kubectl exec <social-engine-pod> -- curl -s https://api.openai.com` not blocked.