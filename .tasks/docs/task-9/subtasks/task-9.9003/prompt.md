Implement subtask 9003: Configure Ingress resource with TLS termination via cert-manager

## Objective
Create an Ingress resource for the cto-pm service with TLS termination using cert-manager, including a ClusterIssuer or Issuer for certificate provisioning.

## Steps
Step-by-step:
1. Ensure cert-manager is installed in the cluster (check `kubectl get pods -n cert-manager`). If not, this is a prerequisite.
2. Create a ClusterIssuer (or Issuer in sigma-1 namespace) for the chosen CA (Let's Encrypt staging first, then production):
   ```yaml
   apiVersion: cert-manager.io/v1
   kind: ClusterIssuer
   metadata:
     name: letsencrypt-prod
   spec:
     acme:
       server: https://acme-v02.api.letsencrypt.org/directory
       email: ops@<org>.com
       privateKeySecretRef:
         name: letsencrypt-prod-key
       solvers:
       - http01:
           ingress:
             class: nginx
   ```
3. Create the Ingress resource:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: Ingress
   metadata:
     name: cto-pm-ingress
     namespace: sigma-1
     labels:
       sigma-1-pipeline: production
     annotations:
       cert-manager.io/cluster-issuer: letsencrypt-prod
       nginx.ingress.kubernetes.io/rate-limit: "100"
       nginx.ingress.kubernetes.io/rate-limit-window: "1m"
   spec:
     ingressClassName: nginx
     tls:
     - hosts:
       - <cto-pm-hostname>
       secretName: cto-pm-tls
     rules:
     - host: <cto-pm-hostname>
       http:
         paths:
         - path: /
           pathType: Prefix
           backend:
             service:
               name: cto-pm
               port:
                 number: 3000
   ```
4. Apply and wait for cert-manager to issue the certificate: `kubectl get certificate -n sigma-1`.
5. Verify TLS: `curl -v https://<cto-pm-hostname>` shows valid TLS handshake.

## Validation
`kubectl get ingress cto-pm-ingress -n sigma-1` shows the Ingress with TLS configured. `kubectl get certificate -n sigma-1` shows the certificate in `Ready: True` state. `curl -v https://<cto-pm-hostname>` completes TLS handshake with a valid certificate and returns a response from cto-pm.