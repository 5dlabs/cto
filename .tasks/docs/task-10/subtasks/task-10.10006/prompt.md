Implement subtask 10006: Network policies: allow sigma1 social-engine to NATS in openclaw namespace

## Objective
Create NetworkPolicy allowing only the social-engine pod in sigma1 to reach NATS on port 4222 in the openclaw namespace.

## Steps
Step-by-step:
1. Create `netpol-allow-nats.yaml`:
   - Egress policy in sigma1 namespace:
     - `podSelector.matchLabels: {app: social-engine}` (only social-engine)
     - `egress[0].to[0].namespaceSelector.matchLabels: {name: openclaw}`
     - `egress[0].ports: [{protocol: TCP, port: 4222}]`
2. Create ingress policy in openclaw namespace:
   - Allow ingress from sigma1 namespace, pod label `app: social-engine`, on port 4222 for NATS pods.
3. Ensure `openclaw` namespace has label `name: openclaw`.

## Validation
From the social-engine pod, run `nc -zv <nats-service>.openclaw.svc.cluster.local 4222` and verify success. From a different sigma1 pod (e.g., equipment-catalog), run the same command and verify it is denied.