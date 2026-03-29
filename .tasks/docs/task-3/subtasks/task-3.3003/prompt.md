Implement subtask 3003: Create Ingress resource with TLS via cert-manager and rate limiting

## Objective
Create an Ingress resource for the NotifyCore service with TLS termination via cert-manager ClusterIssuer (Let's Encrypt) and nginx rate limiting annotations.

## Steps
1. Create `infra/notifycore/templates/ingress.yaml`:
   - `apiVersion: networking.k8s.io/v1`, kind: Ingress.
   - metadata.annotations:
     - `cert-manager.io/cluster-issuer: letsencrypt-prod` (or parameterized from values).
     - `nginx.ingress.kubernetes.io/limit-rps: "100"`
     - `nginx.ingress.kubernetes.io/limit-burst-multiplier: "5"`
   - spec.ingressClassName: nginx (parameterized).
   - spec.tls: [{hosts: ["notifycore.{{ .Values.domain }}"], secretName: notifycore-tls}].
   - spec.rules: [{host: "notifycore.{{ .Values.domain }}", http: {paths: [{path: /, pathType: Prefix, backend: {service: {name: notifycore, port: {number: 8080}}}}]}}].
2. Conditionally render Ingress only when `ingress.enabled: true` in values.
3. In `values-prod.yaml`: `ingress.enabled: true`, `domain: example.com` (placeholder).
4. In `values-dev.yaml`: `ingress.enabled: false`.
5. Ensure the TLS secret name matches what cert-manager will create.

## Validation
`helm template` with values-prod.yaml renders an Ingress resource with TLS block referencing cert-manager ClusterIssuer, rate limiting annotations, and correct service backend. `helm template` with values-dev.yaml does NOT render an Ingress resource. Ingress spec contains both host and TLS configuration.