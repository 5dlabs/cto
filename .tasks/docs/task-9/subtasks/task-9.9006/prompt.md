Implement subtask 9006: Create NetworkPolicy allow rules for cto-pm egress traffic

## Objective
Define NetworkPolicy resources allowing egress from cto-pm to in-cluster services (discord-bridge-http, linear-bridge, Hermes) and external APIs (Linear, GitHub, NOUS on port 443).

## Steps
Step-by-step:
1. Create egress allow for in-cluster services:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: allow-cto-pm-egress-in-cluster
     namespace: sigma-1
     labels:
       sigma-1-pipeline: production
   spec:
     podSelector:
       matchLabels:
         app: cto-pm
     policyTypes:
     - Egress
     egress:
     - to:
       - namespaceSelector:
           matchLabels:
             kubernetes.io/metadata.name: bots
         podSelector:
           matchLabels:
             app: discord-bridge-http
       ports:
       - protocol: TCP
         port: <discord-bridge-port>
     - to:
       - namespaceSelector:
           matchLabels:
             kubernetes.io/metadata.name: bots
         podSelector:
           matchLabels:
             app: linear-bridge
       ports:
       - protocol: TCP
         port: <linear-bridge-port>
     - to:
       - namespaceSelector: {}
         podSelector:
           matchLabels:
             app: hermes
       ports:
       - protocol: TCP
         port: <hermes-port>
   ```
2. Create egress allow for external HTTPS APIs:
   ```yaml
   apiVersion: networking.k8s.io/v1
   kind: NetworkPolicy
   metadata:
     name: allow-cto-pm-egress-external
     namespace: sigma-1
   spec:
     podSelector:
       matchLabels:
         app: cto-pm
     policyTypes:
     - Egress
     egress:
     - to:
       - ipBlock:
           cidr: 0.0.0.0/0
           except:
           - 10.0.0.0/8
           - 172.16.0.0/12
           - 192.168.0.0/16
       ports:
       - protocol: TCP
         port: 443
   ```
3. Create egress allow for external-secrets-operator if it runs in sigma-1:
   Similar pattern targeting the ESO pods and the backing secret store endpoint on 443.
4. Look up actual service ports from the `sigma-1-infra-endpoints` ConfigMap or service definitions.
5. Apply all policies and validate each traffic flow.

## Validation
From a cto-pm pod: `curl discord-bridge-http.bots:<port>` succeeds, `curl linear-bridge.bots:<port>` succeeds, `curl hermes.<ns>:<port>` succeeds, `curl https://api.linear.app` succeeds, `curl https://api.github.com` succeeds. From a cto-pm pod: `curl <random-internal-service>` on a non-allowed port times out. From a non-cto-pm pod in sigma-1: external egress is blocked.