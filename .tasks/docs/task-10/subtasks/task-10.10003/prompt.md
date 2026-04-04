Implement subtask 10003: Configure Ingress with TLS termination via cert-manager for PM server

## Objective
Create an Ingress resource for the PM server with TLS termination using cert-manager. Add annotations for rate limiting and request size limits.

## Steps
1. Ensure cert-manager is installed in the cluster. If not, install it via Helm: `helm install cert-manager jetstack/cert-manager --set installCRDs=true`.
2. Create a `ClusterIssuer` or `Issuer` resource for certificate provisioning (Let's Encrypt staging for validation, production for final deployment). Example:
   ```yaml
   apiVersion: cert-manager.io/v1
   kind: ClusterIssuer
   metadata:
     name: letsencrypt-prod
   spec:
     acme:
       server: https://acme-v02.api.letsencrypt.org/directory
       email: ops@sigma-1.dev
       privateKeySecretRef:
         name: letsencrypt-prod-key
       solvers:
       - http01:
           ingress:
             class: nginx
   ```
3. Create the Ingress resource for the PM server:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: Ingress
   metadata:
     name: sigma-1-pm-ingress
     namespace: sigma-1-dev
     annotations:
       cert-manager.io/cluster-issuer: letsencrypt-prod
       nginx.ingress.kubernetes.io/limit-rps: "50"
       nginx.ingress.kubernetes.io/proxy-body-size: "10m"
   spec:
     ingressClassName: nginx
     tls:
     - hosts:
       - pm.sigma-1.dev
       secretName: sigma-1-pm-tls
     rules:
     - host: pm.sigma-1.dev
       http:
         paths:
         - path: /
           pathType: Prefix
           backend:
             service:
               name: sigma-1-pm-server
               port:
                 number: 3000
   ```
4. Apply and wait for the Certificate to reach Ready state: `kubectl get certificate sigma-1-pm-tls -n sigma-1-dev`.
5. Verify TLS is working: `openssl s_client -connect <ingress-ip>:443 -servername pm.sigma-1.dev`.

## Validation
Run `kubectl get certificate sigma-1-pm-tls -n sigma-1-dev -o jsonpath='{.status.conditions[0].status}'` and assert `True`. Run `curl -k https://<ingress-host>/api/pipeline/status` and assert HTTP 200. Verify TLS certificate validity with `openssl s_client -connect <ingress-ip>:443 -servername pm.sigma-1.dev` and confirm the certificate subject matches.