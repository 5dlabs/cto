Implement subtask 9005: Create NetworkPolicy allow rules for cto-pm ingress traffic

## Objective
Define NetworkPolicy resources allowing ingress traffic to cto-pm from the ingress controller on port 3000.

## Steps
Step-by-step:
1. Create an ingress allow policy for cto-pm:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: allow-ingress-to-cto-pm
     namespace: sigma-1
     labels:
       sigma-1-pipeline: production
   spec:
     podSelector:
       matchLabels:
         app: cto-pm
     policyTypes:
     - Ingress
     ingress:
     - from:
       - namespaceSelector:
           matchLabels:
             kubernetes.io/metadata.name: ingress-nginx
       ports:
       - protocol: TCP
         port: 3000
   ```
2. Adjust the `namespaceSelector` to match the actual ingress controller namespace label in the cluster.
3. Apply and verify: traffic from the ingress controller reaches cto-pm on port 3000.
4. Also allow DNS egress for all pods in sigma-1 (required for service discovery and external API calls):
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: allow-dns-egress
     namespace: sigma-1
   spec:
     podSelector: {}
     policyTypes:
     - Egress
     egress:
     - to:
       - namespaceSelector:
           matchLabels:
             kubernetes.io/metadata.name: kube-system
       ports:
       - protocol: UDP
         port: 53
       - protocol: TCP
         port: 53
   ```

## Validation
From the ingress controller pod (or via curl through the Ingress), cto-pm responds on port 3000. A pod in an unlabeled namespace cannot reach cto-pm (connection timeout). DNS resolution works from cto-pm pods (`nslookup kubernetes.default` succeeds).