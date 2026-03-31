Implement subtask 9009: Configure HorizontalPodAutoscalers for backend and frontend services

## Objective
Create HPA resources for the Bun/Elysia backend (min 2, max 10, target 70% CPU) and Next.js frontend (min 2, max 5, target 70% CPU), with appropriate resource requests and limits on the Deployment specs.

## Steps
1. Update the backend Deployment spec with resource requests/limits: `requests: {cpu: 500m, memory: 512Mi}`, `limits: {cpu: 1, memory: 1Gi}`.
2. Create HPA for backend: `minReplicas: 2`, `maxReplicas: 10`, `targetCPUUtilizationPercentage: 70`.
3. Update the frontend Deployment spec with resource requests/limits: `requests: {cpu: 250m, memory: 256Mi}`, `limits: {cpu: 500m, memory: 512Mi}`.
4. Create HPA for frontend: `minReplicas: 2`, `maxReplicas: 5`, `targetCPUUtilizationPercentage: 70`.
5. Verify metrics-server is available in the cluster (HPA depends on it).
6. Set appropriate scale-down stabilization window (default 300s is fine for production).

## Validation
Verify `kubectl get hpa -n hermes-production` shows both HPAs with current metrics. Under load (50 concurrent requests), verify backend HPA scales beyond 2 replicas within 3 minutes. After load subsides, verify replicas scale back to 2 within 10 minutes. Verify resource requests are set: `kubectl get deploy -n hermes-production -o jsonpath='{.items[*].spec.template.spec.containers[*].resources}'`.