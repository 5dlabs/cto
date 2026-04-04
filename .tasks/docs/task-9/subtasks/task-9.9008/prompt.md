Implement subtask 9008: Configure HorizontalPodAutoscaler for cto-pm

## Objective
Create an HPA resource targeting the cto-pm Deployment with min 2, max 5 replicas, and target CPU utilization of 70%.

## Steps
Step-by-step:
1. Ensure metrics-server is running in the cluster: `kubectl get deployment metrics-server -n kube-system`.
2. Create the HPA:
   ```yaml
   apiVersion: autoscaling/v2
   kind: HorizontalPodAutoscaler
   metadata:
     name: cto-pm-hpa
     namespace: sigma-1
     labels:
       sigma-1-pipeline: production
   spec:
     scaleTargetRef:
       apiVersion: apps/v1
       kind: Deployment
       name: cto-pm
     minReplicas: 2
     maxReplicas: 5
     metrics:
     - type: Resource
       resource:
         name: cpu
         target:
           type: Utilization
           averageUtilization: 70
   ```
3. Apply: `kubectl apply -f cto-pm-hpa.yaml`.
4. Remove the static `spec.replicas` from the Deployment if present (HPA should manage replica count).
5. Verify: `kubectl get hpa -n sigma-1` shows the HPA with TARGETS showing current CPU utilization and MINPODS=2, MAXPODS=5.

## Validation
`kubectl get hpa cto-pm-hpa -n sigma-1` shows minReplicas=2, maxReplicas=5, targetCPUUtilization=70%. Current replicas >= 2. Under load (e.g., `hey -n 1000 -c 50 https://<host>/health`), the HPA scales up within 2-3 minutes. When load drops, it scales back down to minReplicas.