Implement subtask 9003: Configure HorizontalPodAutoscaler for PM server

## Objective
Create an HPA manifest for the PM server with min 3, max 10 replicas targeting 70% average CPU utilization.

## Steps
1. Create `templates/pm-server-hpa.yaml` with `apiVersion: autoscaling/v2`.
2. Set `spec.scaleTargetRef` to the PM server Deployment name.
3. Set `spec.minReplicas: 3`, `spec.maxReplicas: 10`.
4. Add a metric of type Resource targeting CPU with `averageUtilization: 70`.
5. Wrap the template in a Helm conditional: `{{- if .Values.pmServer.autoscaling.enabled }}`.
6. In `values-sigma1-prod.yaml`, set `pmServer.autoscaling.enabled: true`, `pmServer.autoscaling.minReplicas: 3`, `pmServer.autoscaling.maxReplicas: 10`, `pmServer.autoscaling.targetCPUUtilizationPercentage: 70`.
7. Validate the rendered YAML with `helm template`.

## Validation
Run `helm template . -f values-sigma1-prod.yaml` and verify the HPA manifest is rendered with minReplicas=3, maxReplicas=10, and targetCPU=70. After deploy: `kubectl get hpa -n sigma1-prod` confirms correct values.